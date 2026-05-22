import os
import sys
import subprocess
import urllib.request
import platform

VERSION = "0.2.2"
REPO = "Psyborgs-git/spy-code"

def get_binary_info():
    system = platform.system().lower()
    arch = platform.machine().lower()

    target = ""
    exe = ""

    if system == "darwin":
        if arch in ["arm64", "aarch64"]:
            target = "darwin-arm64"
        else:
            target = "darwin-x64"
    elif system == "linux":
        if arch in ["x86_64", "amd64"]:
            target = "linux-x64"
        else:
            raise RuntimeError(f"Unsupported architecture for Linux: {arch}")
    elif system == "windows":
        if arch in ["x86_64", "amd64"]:
            target = "win32-x64"
            exe = ".exe"
        else:
            raise RuntimeError(f"Unsupported architecture for Windows: {arch}")
    else:
        raise RuntimeError(f"Unsupported platform: {system}")

    binary_name = f"spy-code-{target}{exe}"
    local_name = f"spy-code{exe}"
    url = f"https://github.com/{REPO}/releases/download/v{VERSION}/{binary_name}"

    return url, local_name

def main():
    pkg_dir = os.path.dirname(os.path.abspath(__file__))
    
    # Check platform and get binary details
    try:
        url, local_name = get_binary_info()
    except RuntimeError as err:
        print(f"Error: {err}", file=sys.stderr)
        sys.exit(1)

    binary_path = os.path.join(pkg_dir, local_name)

    # If the binary is not bundled, download it dynamically
    if not os.path.exists(binary_path):
        print(f"Downloading spy-code binary v{VERSION} for platform {platform.system()}-{platform.machine()}...")
        print(f"URL: {url}")
        
        try:
            # Setup request header to prevent user-agent block (though GitHub doesn't block python's UA generally)
            req = urllib.request.Request(
                url, 
                headers={'User-Agent': 'spy-code-pip-downloader'}
            )
            with urllib.request.urlopen(req) as response:
                # Follow redirect if necessary (urllib handles 302 automatically)
                with open(binary_path, 'wb') as out_file:
                    out_file.write(response.read())
            
            # Set executable permissions on Unix platforms
            if platform.system() != "Windows":
                os.chmod(binary_path, 0o755)
            print(f"Successfully downloaded binary to {binary_path}")
        except Exception as e:
            print(f"Error downloading binary: {e}", file=sys.stderr)
            print("Please check your internet connection or manually place the binary in "
                  f"'{binary_path}'.", file=sys.stderr)
            sys.exit(1)
    else:
        # Make sure it's executable if it was copied/bundled
        if platform.system() != "Windows":
            try:
                os.chmod(binary_path, 0o755)
            except Exception:
                pass

    # Run the native binary with arguments
    try:
        # Use subprocess.run to execute the binary and forward exit code
        result = subprocess.run([binary_path] + sys.argv[1:])
        sys.exit(result.returncode)
    except KeyboardInterrupt:
        # Handle ctrl+c gracefully
        sys.exit(130)
    except Exception as e:
        print(f"Error executing binary: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
