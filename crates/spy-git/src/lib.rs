use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Status of a file change between two git refs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileChangeStatus {
    Added,
    Modified,
    Deleted,
    Renamed { old_path: PathBuf },
}

/// A file that changed between two git refs.
#[derive(Debug, Clone)]
pub struct FileDiff {
    pub path: PathBuf,
    pub status: FileChangeStatus,
}

/// Wrapper around a git repository for spy-code operations.
///
/// All git operations are performed by shelling out to `git`, which is always
/// present in any environment that has a repository.
pub struct GitRepo {
    workdir: PathBuf,
}

impl GitRepo {
    /// Discover the nearest git repository at or above `path`.
    ///
    /// Returns `Ok(None)` when `path` is not inside a git repository.
    pub fn discover(path: &Path) -> Result<Option<Self>> {
        let out = Command::new("git")
            .arg("-C")
            .arg(path)
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .context("Failed to spawn git")?;

        if out.status.success() {
            let raw = String::from_utf8_lossy(&out.stdout);
            let workdir = PathBuf::from(raw.trim());
            Ok(Some(GitRepo { workdir }))
        } else {
            Ok(None)
        }
    }

    /// Return the current HEAD SHA, or `None` when HEAD is unborn.
    pub fn current_sha(&self) -> Option<String> {
        let out = Command::new("git")
            .arg("-C")
            .arg(&self.workdir)
            .args(["rev-parse", "HEAD"])
            .output()
            .ok()?;

        if out.status.success() {
            let sha = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if sha.is_empty() { None } else { Some(sha) }
        } else {
            None
        }
    }

    /// Return `true` if the working tree has uncommitted changes.
    pub fn is_dirty(&self) -> bool {
        Command::new("git")
            .arg("-C")
            .arg(&self.workdir)
            .args(["status", "--porcelain"])
            .output()
            .map(|out| !out.stdout.is_empty())
            .unwrap_or(false)
    }

    /// Return the list of files that differ between `old_sha` and HEAD.
    ///
    /// Returns an error when `old_sha` is not reachable (e.g. force-pushed
    /// history or shallow clone boundary).  Callers should fall back to a full
    /// re-index in that case.
    pub fn diff_files_since(&self, old_sha: &str) -> Result<Vec<FileDiff>> {
        let range = format!("{}..HEAD", old_sha);
        let out = Command::new("git")
            .arg("-C")
            .arg(&self.workdir)
            .args(["diff", "--name-status", &range])
            .output()
            .context("Failed to spawn git diff")?;

        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            anyhow::bail!("git diff failed: {}", stderr.trim());
        }

        parse_name_status(&String::from_utf8_lossy(&out.stdout))
    }

    /// Return the set of file paths (absolute) changed since `git_ref`.
    ///
    /// Used by the `changedSince` GraphQL / MCP query.
    pub fn files_changed_since_ref(&self, git_ref: &str) -> Result<Vec<PathBuf>> {
        let range = format!("{}..HEAD", git_ref);
        let out = Command::new("git")
            .arg("-C")
            .arg(&self.workdir)
            .args(["diff", "--name-only", &range])
            .output()
            .context("Failed to spawn git diff")?;

        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            anyhow::bail!("git diff failed: {}", stderr.trim());
        }

        Ok(String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| self.workdir.join(l.trim()))
            .collect())
    }

    /// The working-tree root of this repository.
    pub fn workdir(&self) -> &Path {
        &self.workdir
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Parse `git diff --name-status` output into `FileDiff` entries.
fn parse_name_status(output: &str) -> Result<Vec<FileDiff>> {
    let mut diffs = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Fields are tab-separated: <status_code>\t<path> or for renames
        // <R|C><score>\t<old_path>\t<new_path>
        let mut fields = line.splitn(3, '\t');
        let status_code = match fields.next() {
            Some(s) => s,
            None => continue,
        };

        if status_code.starts_with('R') || status_code.starts_with('C') {
            let old = fields.next().unwrap_or("").trim();
            let new = fields.next().unwrap_or("").trim();
            if !old.is_empty() && !new.is_empty() {
                diffs.push(FileDiff {
                    path: PathBuf::from(new),
                    status: FileChangeStatus::Renamed {
                        old_path: PathBuf::from(old),
                    },
                });
            }
        } else {
            let path_str = match fields.next() {
                Some(p) => p.trim(),
                None => continue,
            };
            let status = match status_code.chars().next().unwrap_or(' ') {
                'A' => FileChangeStatus::Added,
                'M' => FileChangeStatus::Modified,
                'D' => FileChangeStatus::Deleted,
                _ => continue,
            };
            diffs.push(FileDiff {
                path: PathBuf::from(path_str),
                status,
            });
        }
    }

    Ok(diffs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_name_status_basic() {
        let input = "M\tsrc/main.rs\nA\tsrc/new.rs\nD\tsrc/old.rs\n";
        let diffs = parse_name_status(input).unwrap();
        assert_eq!(diffs.len(), 3);
        assert_eq!(diffs[0].status, FileChangeStatus::Modified);
        assert_eq!(diffs[1].status, FileChangeStatus::Added);
        assert_eq!(diffs[2].status, FileChangeStatus::Deleted);
    }

    #[test]
    fn test_parse_name_status_rename() {
        let input = "R100\told/file.rs\tnew/file.rs\n";
        let diffs = parse_name_status(input).unwrap();
        assert_eq!(diffs.len(), 1);
        assert!(matches!(
            &diffs[0].status,
            FileChangeStatus::Renamed { old_path } if old_path == &PathBuf::from("old/file.rs")
        ));
        assert_eq!(diffs[0].path, PathBuf::from("new/file.rs"));
    }

    #[test]
    fn test_parse_name_status_empty() {
        let diffs = parse_name_status("").unwrap();
        assert!(diffs.is_empty());
    }
}

