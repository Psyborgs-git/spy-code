#!/bin/bash

# Spy-Code Auto-Installer Script
# Detects AI coding environment and configures spy-code MCP integration

set -e

# Script version
VERSION="1.0.0"

# Default options
DRY_RUN=false
SKIP_INDEX=false
FORCE_CONFIG=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1" >&2
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" >&2
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1" >&2
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

print_usage() {
    cat << EOF
Spy-Code Auto-Installer v${VERSION}

Usage: $0 [OPTIONS]

Options:
  --dry-run          Show what would be done without making changes
  --skip-index       Skip codebase indexing
  --force-config     Force re-creation of spy.config.json
  --help             Show this help message
  --version          Show version information

Examples:
  $0                    # Run full installation
  $0 --dry-run          # Preview changes without applying
  $0 --skip-index       # Skip indexing (faster setup)
  $0 --force-config     # Force config recreation

EOF
}

# Detect current directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Check if spy-code is installed
check_spy_code() {
    print_info "Checking if spy-code is installed..."
    if command -v spy-code &> /dev/null; then
        VERSION=$(spy-code --version 2>/dev/null || echo "unknown")
        print_success "spy-code is installed (version: $VERSION)"
        return 0
    else
        print_warning "spy-code is not installed"
        return 1
    fi
}

# Install spy-code if not present
install_spy_code() {
    print_info "Installing spy-code..."

    # Try npm installation
    if command -v npm &> /dev/null; then
        print_info "Installing via npm..."
        npm install -g spy-code
        if [ $? -eq 0 ]; then
            print_success "spy-code installed via npm"
            return 0
        fi
    fi

    # Try pip installation
    if command -v pip &> /dev/null; then
        print_info "Installing via pip..."
        pip install spy-code
        if [ $? -eq 0 ]; then
            print_success "spy-code installed via pip"
            return 0
        fi
    fi

    print_error "Could not install spy-code. Please install manually from https://github.com/Psyborgs-git/spy-code"
    return 1
}

# Detect AI coding environment
detect_environment() {
    print_info "Detecting AI coding environment..." >&2

    ENVIRONMENT="unknown"

    # Check for Cursor
    if [ -d "$HOME/.cursor" ] || [ -f "$HOME/.cursor/mcp_config.json" ]; then
        ENVIRONMENT="cursor"
        print_success "Detected: Cursor" >&2
        echo "cursor"
        return 0
    fi

    # Check for Windsurf/Cascade
    if [ -d "$HOME/.windsurf" ] || [ -f "$HOME/.windsurf/mcp_config.json" ]; then
        ENVIRONMENT="windsurf"
        print_success "Detected: Windsurf/Cascade" >&2
        echo "windsurf"
        return 0
    fi

    # Check for Claude Desktop
    if [ -f "$HOME/Library/Application Support/Claude/claude_desktop_config.json" ] || \
       [ -f "$HOME/.config/Claude/claude_desktop_config.json" ]; then
        ENVIRONMENT="claude"
        print_success "Detected: Claude Desktop" >&2
        echo "claude"
        return 0
    fi

    # Check for VS Code with Copilot
    if [ -d "$HOME/.vscode" ] || [ -d "$HOME/.vscode-insiders" ]; then
        ENVIRONMENT="vscode"
        print_success "Detected: VS Code (with Copilot)" >&2
        echo "vscode"
        return 0
    fi

    # Check for Gemini
    if [ -d "$HOME/.gemini" ] || [ -f "$HOME/.gemini/config.json" ]; then
        ENVIRONMENT="gemini"
        print_success "Detected: Gemini" >&2
        echo "gemini"
        return 0
    fi

    # Check for Codex
    if [ -d "$HOME/.codex" ] || [ -f "$HOME/.codex/config.json" ]; then
        ENVIRONMENT="codex"
        print_success "Detected: Codex" >&2
        echo "codex"
        return 0
    fi

    # Check for OpenCode
    if [ -d "$HOME/.opencode" ] || [ -f "$HOME/.opencode/config.json" ]; then
        ENVIRONMENT="opencode"
        print_success "Detected: OpenCode" >&2
        echo "opencode"
        return 0
    fi

    print_warning "Could not detect specific AI coding environment" >&2
    print_info "Will use generic MCP configuration" >&2
    ENVIRONMENT="generic"
    echo "generic"
    return 0
}

# Initialize spy-code config
init_spy_code_config() {
    print_info "Initializing spy-code configuration..."

    if [ -f "$REPO_ROOT/spy.config.json" ] && [ "$FORCE_CONFIG" != "true" ]; then
        print_warning "spy.config.json already exists, skipping initialization"
        return 0
    fi

    if [ "$DRY_RUN" = "true" ]; then
        print_info "[DRY RUN] Would run: cd \"$REPO_ROOT\" && spy-code init"
        return 0
    fi

    cd "$REPO_ROOT"
    spy-code init
    if [ $? -eq 0 ]; then
        print_success "spy.config.json created"
        return 0
    else
        print_error "Failed to initialize spy-code configuration"
        return 1
    fi
}

# Index the codebase
index_codebase() {
    if [ "$SKIP_INDEX" = "true" ]; then
        print_info "Skipping codebase indexing (skip-index flag set)"
        return 0
    fi

    print_info "Indexing codebase (this may take a while)..."

    if [ "$DRY_RUN" = "true" ]; then
        print_info "[DRY RUN] Would run: cd \"$REPO_ROOT\" && spy-code index"
        return 0
    fi

    cd "$REPO_ROOT"
    spy-code index
    if [ $? -eq 0 ]; then
        print_success "Codebase indexed successfully"
        return 0
    else
        print_error "Failed to index codebase"
        return 1
    fi
}

# Configure MCP for detected environment
configure_mcp() {
    local env=$1
    print_info "Configuring MCP for $env..."

    local config_dir=""
    local config_file=""
    local source_config="$REPO_ROOT/configs/mcp/${env}.json"
    if [ ! -f "$source_config" ]; then
        source_config="$REPO_ROOT/configs/mcp/generic.json"
    fi

    # Set config paths based on environment
    case $env in
        cursor)
            config_dir="$HOME/.cursor"
            config_file="$config_dir/mcp_config.json"
            ;;
        windsurf)
            config_dir="$HOME/.windsurf"
            config_file="$config_dir/mcp_config.json"
            ;;
        claude)
            if [ -d "$HOME/Library/Application Support/Claude" ]; then
                config_dir="$HOME/Library/Application Support/Claude"
            else
                config_dir="$HOME/.config/Claude"
            fi
            config_file="$config_dir/claude_desktop_config.json"
            ;;
        gemini)
            config_dir="$HOME/.gemini"
            config_file="$config_dir/config.json"
            ;;
        codex)
            config_dir="$HOME/.codex"
            config_file="$config_dir/config.json"
            ;;
        opencode)
            config_dir="$HOME/.opencode"
            config_file="$config_dir/config.json"
            ;;
        vscode)
            config_dir="$HOME/.vscode"
            config_file="$config_dir/mcp_config.json"
            ;;
        generic)
            print_info "Generic configuration - copy manually to your MCP config"
            print_info "Config template: $source_config"
            return 0
            ;;
        *)
            print_error "Unknown environment: $env"
            return 1
            ;;
    esac

    # Create config directory if it doesn't exist
    if [ ! -d "$config_dir" ]; then
        if [ "$DRY_RUN" = "true" ]; then
            print_info "[DRY RUN] Would create config directory: $config_dir"
        else
            print_info "Creating config directory: $config_dir"
            mkdir -p "$config_dir"
        fi
    fi

    # Check if source config exists
    if [ ! -f "$source_config" ]; then
        print_error "MCP config template not found: $source_config"
        return 1
    fi

    # Backup existing config
    if [ -f "$config_file" ]; then
        if [ "$DRY_RUN" = "true" ]; then
            print_info "[DRY RUN] Would backup existing config to ${config_file}.backup"
        else
            print_info "Backing up existing config to ${config_file}.backup"
            cp "$config_file" "${config_file}.backup"
        fi
    fi

    # Copy or merge config
    if [ "$DRY_RUN" = "true" ]; then
        if [ -f "$config_file" ]; then
            print_info "[DRY RUN] Would merge spy-code MCP configuration into existing config"
        else
            print_info "[DRY RUN] Would create new MCP config file: $config_file"
        fi
    else
        if [ -f "$config_file" ]; then
            print_info "Merging spy-code MCP configuration into existing config..."
            # Simple merge - in production, use jq or similar
            if command -v jq &> /dev/null; then
                jq --argfile spy "$source_config" '.mcpServers += $spy.mcpServers' "$config_file" > "${config_file}.tmp"
                mv "${config_file}.tmp" "$config_file"
            else
                print_warning "jq not found, appending config (may need manual merge)"
                cat "$source_config" >> "$config_file"
            fi
        else
            print_info "Creating new MCP config file..."
            cp "$source_config" "$config_file"
        fi
    fi

    print_success "MCP configuration completed for $env"
    print_info "Config file: $config_file"
    return 0
}

# Verify setup
verify_setup() {
    print_info "Verifying setup..."

    # Check spy-code
    if ! command -v spy-code &> /dev/null; then
        print_error "spy-code not found in PATH"
        return 1
    fi

    # Check config
    if [ ! -f "$REPO_ROOT/spy.config.json" ]; then
        print_error "spy.config.json not found"
        return 1
    fi

    # Check database
    if [ ! -f "$REPO_ROOT/.spy-code/graph.db" ]; then
        print_warning "Graph database not found (may need to run index)"
    fi

    print_success "Setup verification passed"
    return 0
}

# Print next steps
print_next_steps() {
    local env=$1
    echo ""
    echo -e "${GREEN}=== Setup Complete ===${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Restart your AI coding environment ($env)"
    echo "2. Test spy-code integration:"
    echo "   - Try: spy-code search \"main\""
    echo "   - Try: spy-code stats"
    echo "3. For graph visualization: spy-code graph --open"
    echo "4. For semantic search: spy-code embed && spy-code ask \"how do I...\""
    echo ""
    echo "Documentation:"
    echo "- Universal skills: $REPO_ROOT/skills/universal/"
    echo "- Environment-specific skills: $REPO_ROOT/skills/environments/"
    echo ""
    echo "For issues, see: https://github.com/Psyborgs-git/spy-code"
}

# Main execution
main() {
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --skip-index)
                SKIP_INDEX=true
                shift
                ;;
            --force-config)
                FORCE_CONFIG=true
                shift
                ;;
            --help)
                print_usage
                exit 0
                ;;
            --version)
                echo "Spy-Code Auto-Installer v${VERSION}"
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                print_usage
                exit 1
                ;;
        esac
    done

    echo -e "${BLUE}"
    echo "================================"
    echo "Spy-Code Auto-Installer v${VERSION}"
    echo "================================"
    echo -e "${NC}"
    echo ""

    if [ "$DRY_RUN" = "true" ]; then
        print_warning "DRY RUN MODE - No changes will be made"
        echo ""
    fi

    # Check/install spy-code
    if ! check_spy_code; then
        if ! install_spy_code; then
            print_error "Installation failed. Please install spy-code manually."
            exit 1
        fi
    fi

    # Detect environment
    ENV_NAME=$(detect_environment)

    # Initialize config
    if ! init_spy_code_config; then
        print_error "Failed to initialize spy-code config"
        exit 1
    fi

    # Index codebase
    if ! index_codebase; then
        print_warning "Indexing failed, you may need to run 'spy-code index' manually"
    fi

    # Configure MCP
    configure_mcp "$ENV_NAME"

    # Verify
    verify_setup

    # Print next steps
    print_next_steps "$ENV_NAME"

    echo ""
    print_success "Installation complete!"
}

# Run main function
main "$@"
