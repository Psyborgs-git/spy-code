"""
MCP Setup Module for Python package
Automatically configures MCP for the detected AI coding environment
"""

import os
import sys
import json
import platform
from pathlib import Path


def detect_environment():
    """Detect the current AI coding environment"""
    home = Path.home()
    
    # Check for Cursor
    if (home / ".cursor").exists() or (home / ".cursor" / "mcp_config.json").exists():
        return "cursor"
    
    # Check for Windsurf/Cascade
    if (home / ".windsurf").exists() or (home / ".windsurf" / "mcp_config.json").exists():
        return "windsurf"
    
    # Check for Claude Desktop
    claude_paths = [
        home / "Library" / "Application Support" / "Claude" / "claude_desktop_config.json",
        home / ".config" / "Claude" / "claude_desktop_config.json"
    ]
    if any(p.exists() for p in claude_paths):
        return "claude"
    
    # Check for VS Code
    if (home / ".vscode").exists() or (home / ".vscode-insiders").exists():
        return "vscode"
    
    return "generic"


def get_config_path(environment):
    """Get the MCP config path for the detected environment"""
    home = Path.home()
    
    config_paths = {
        "cursor": home / ".cursor" / "mcp_config.json",
        "windsurf": home / ".windsurf" / "mcp_config.json",
        "claude": home / "Library" / "Application Support" / "Claude" / "claude_desktop_config.json",
        "vscode": home / ".vscode" / "mcp_config.json",
        "generic": None
    }
    
    # Fallback for Linux Claude config
    if environment == "claude" and not config_paths["claude"].exists():
        config_paths["claude"] = home / ".config" / "Claude" / "claude_desktop_config.json"
    
    return config_paths.get(environment)


def get_mcp_config_template(environment):
    """Get the MCP config template for the environment"""
    # Try to find the template relative to this module
    module_dir = Path(__file__).parent.parent.parent
    template_path = module_dir / "configs" / "mcp" / f"{environment}.json"
    
    if template_path.exists():
        return template_path.read_text()
    
    return None


def merge_mcp_config(existing_config, new_config):
    """Merge new MCP config into existing config"""
    try:
        existing = json.loads(existing_config)
        new_spy_config = json.loads(new_config)
        
        if "mcpServers" not in existing:
            existing["mcpServers"] = {}
        
        # Merge spy-code server config
        existing["mcpServers"].update(new_spy_config["mcpServers"])
        
        return json.dumps(existing, indent=2)
    except json.JSONDecodeError as e:
        print(f"Error merging configs: {e}", file=sys.stderr)
        return existing_config


def setup_mcp():
    """Setup MCP configuration for the detected environment"""
    print("Setting up spy-code MCP integration...")
    
    # Detect environment
    env = detect_environment()
    print(f"Detected environment: {env}")
    
    # Get config path
    config_path = get_config_path(env)
    if not config_path:
        print("No MCP config path for this environment. Manual setup required.")
        print("Config template available at: configs/mcp/generic.json")
        return
    
    print(f"Config path: {config_path}")
    
    # Get template
    template = get_mcp_config_template(env)
    if not template:
        print(f"MCP config template not found for environment: {env}")
        return
    
    # Create config directory if needed
    config_dir = config_path.parent
    if not config_dir.exists():
        print(f"Creating config directory: {config_dir}")
        config_dir.mkdir(parents=True, exist_ok=True)
    
    # Backup existing config
    if config_path.exists():
        backup_path = config_path.with_suffix(".json.backup")
        print(f"Backing up existing config to: {backup_path}")
        config_path.rename(backup_path)
    
    # Write or merge config
    if config_path.exists():
        print("Merging spy-code MCP configuration...")
        existing_config = config_path.read_text()
        merged_config = merge_mcp_config(existing_config, template)
        config_path.write_text(merged_config)
    else:
        print("Creating new MCP configuration...")
        config_path.write_text(template)
    
    print("✓ MCP configuration completed")
    print(f"Config file: {config_path}")
    print()
    print("Next steps:")
    print("1. Restart your AI coding environment")
    print("2. Test with: spy-code search \"main\"")


def main():
    """Main entry point for the setup-mcp command"""
    setup_mcp()


if __name__ == "__main__":
    main()
