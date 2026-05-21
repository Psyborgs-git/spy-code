#!/usr/bin/env python3
import sys
import os
import re
import json

def bump_version(version):
    # Strip leading 'v' if present
    if version.startswith('v'):
        version = version[1:]
    
    print(f"Bumping version to: {version}")
    
    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    
    # 1. Update Cargo.toml
    cargo_path = os.path.join(root_dir, "Cargo.toml")
    if os.path.exists(cargo_path):
        with open(cargo_path, "r", encoding="utf-8") as f:
            content = f.read()
        
        # We want to replace the version line under [workspace.package]
        new_content = re.sub(
            r'(?<=^version = ")[^"]+',
            version,
            content,
            flags=re.MULTILINE
        )
        with open(cargo_path, "w", encoding="utf-8") as f:
            f.write(new_content)
        print("Updated Cargo.toml")

    # 2. Update npm/package.json
    npm_json_path = os.path.join(root_dir, "npm", "package.json")
    if os.path.exists(npm_json_path):
        with open(npm_json_path, "r", encoding="utf-8") as f:
            data = json.load(f)
        data["version"] = version
        with open(npm_json_path, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2)
            f.write("\n")
        print("Updated npm/package.json")

    # 3. Update npm/install.js (if any hardcoded VERSION left, though we made it dynamic, we can keep it as safety or skip)
    npm_install_path = os.path.join(root_dir, "npm", "install.js")
    if os.path.exists(npm_install_path):
        with open(npm_install_path, "r", encoding="utf-8") as f:
            content = f.read()
        # replace const VERSION = '...'; if it exists
        content = re.sub(
            r"const VERSION = '[^']+';",
            f"const VERSION = '{version}';",
            content
        )
        with open(npm_install_path, "w", encoding="utf-8") as f:
            f.write(content)
        print("Checked/Updated npm/install.js")

    # 4. Update python-package/pyproject.toml
    pyproject_path = os.path.join(root_dir, "python-package", "pyproject.toml")
    if os.path.exists(pyproject_path):
        with open(pyproject_path, "r", encoding="utf-8") as f:
            content = f.read()
        content = re.sub(
            r'(?<=^version = ")[^"]+',
            version,
            content,
            flags=re.MULTILINE
        )
        with open(pyproject_path, "w", encoding="utf-8") as f:
            f.write(content)
        print("Updated python-package/pyproject.toml")

    # 5. Update python-package/spy_code/__init__.py
    py_init_path = os.path.join(root_dir, "python-package", "spy_code", "__init__.py")
    if os.path.exists(py_init_path):
        with open(py_init_path, "r", encoding="utf-8") as f:
            content = f.read()
        content = re.sub(
            r'VERSION = "[^"]+"',
            f'VERSION = "{version}"',
            content
        )
        with open(py_init_path, "w", encoding="utf-8") as f:
            f.write(content)
        print("Updated python-package/spy_code/__init__.py")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 bump_version.py <version>")
        sys.exit(1)
    bump_version(sys.argv[1])
