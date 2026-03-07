#!/usr/bin/env bash
# AgenticAegis Universal Installer
# Usage: curl -fsSL https://agentralabs.com/install/aegis | bash
#
# Environment variables:
#   AGENTIC_AEGIS_VERSION    - Version to install (default: latest)
#   AGENTIC_INSTALL_DIR      - Installation directory (default: ~/.agentic)
#   AGENTIC_BIN_DIR          - Binary directory (default: $INSTALL_DIR/bin)
#   AGENTIC_PROFILE          - Profile: desktop, terminal, server (default: desktop)

set -euo pipefail

VERSION="${AGENTIC_AEGIS_VERSION:-latest}"
INSTALL_DIR="${AGENTIC_INSTALL_DIR:-$HOME/.agentic}"
BIN_DIR="${AGENTIC_BIN_DIR:-$INSTALL_DIR/bin}"
PROFILE="${AGENTIC_PROFILE:-desktop}"

# Colors (disabled if not a terminal)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    BOLD='\033[1m'
    NC='\033[0m'
else
    RED='' GREEN='' YELLOW='' BLUE='' BOLD='' NC=''
fi

info()    { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warn()    { echo -e "${YELLOW}[WARN]${NC} $1"; }
error()   { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

detect_platform() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)                    OS="linux" ;;
        Darwin)                   OS="macos" ;;
        MINGW*|MSYS*|CYGWIN*)     OS="windows" ;;
        *)                        error "Unsupported OS: $os" ;;
    esac

    case "$arch" in
        x86_64|amd64)   ARCH="x86_64" ;;
        arm64|aarch64)  ARCH="aarch64" ;;
        *)              error "Unsupported architecture: $arch" ;;
    esac

    PLATFORM="${OS}-${ARCH}"
    info "Detected platform: $PLATFORM"
}

check_prerequisites() {
    if ! command -v curl &>/dev/null && ! command -v wget &>/dev/null; then
        error "Neither curl nor wget found. Please install one of them."
    fi

    if ! command -v tar &>/dev/null; then
        error "tar not found. Please install tar."
    fi
}

resolve_version() {
    if [ "$VERSION" = "latest" ]; then
        info "Resolving latest version..."
        if command -v curl &>/dev/null; then
            VERSION=$(curl -fsSL "https://api.github.com/repos/agentralabs/agentic-aegis/releases/latest" 2>/dev/null | grep -o '"tag_name": "[^"]*"' | head -1 | cut -d'"' -f4) || true
        fi

        if [ -z "$VERSION" ] || [ "$VERSION" = "latest" ]; then
            VERSION="v0.1.0"
            warn "Could not resolve latest version, using $VERSION"
        fi
        info "Version: $VERSION"
    fi
}

download_binary() {
    local url="https://github.com/agentralabs/agentic-aegis/releases/download/${VERSION}/agentic-aegis-${VERSION}-${PLATFORM}.tar.gz"
    local tmp_dir
    tmp_dir=$(mktemp -d)

    info "Downloading from $url"

    if command -v curl &>/dev/null; then
        curl -fsSL "$url" -o "$tmp_dir/agentic-aegis.tar.gz" || error "Download failed. Check the version and try again."
    else
        wget -q "$url" -O "$tmp_dir/agentic-aegis.tar.gz" || error "Download failed. Check the version and try again."
    fi

    info "Extracting..."
    tar -xzf "$tmp_dir/agentic-aegis.tar.gz" -C "$tmp_dir"

    mkdir -p "$BIN_DIR"

    if [ -f "$tmp_dir/aegis" ]; then
        mv "$tmp_dir/aegis" "$BIN_DIR/aegis"
    elif [ -f "$tmp_dir/agentic-aegis-${VERSION}-${PLATFORM}/aegis" ]; then
        mv "$tmp_dir/agentic-aegis-${VERSION}-${PLATFORM}/aegis" "$BIN_DIR/aegis"
    else
        error "Binary not found in archive"
    fi

    chmod +x "$BIN_DIR/aegis"
    rm -rf "$tmp_dir"
    success "Installed aegis to $BIN_DIR/aegis"
}

configure_path() {
    local shell_rc=""
    local current_shell
    current_shell="$(basename "${SHELL:-/bin/bash}")"

    case "$current_shell" in
        zsh)  shell_rc="$HOME/.zshrc" ;;
        bash) shell_rc="$HOME/.bashrc" ;;
        fish) shell_rc="$HOME/.config/fish/config.fish" ;;
        *)    shell_rc="$HOME/.profile" ;;
    esac

    if echo "$PATH" | tr ':' '\n' | grep -qF "$BIN_DIR"; then
        info "PATH already includes $BIN_DIR"
        return
    fi

    if [ -n "$shell_rc" ] && [ -f "$shell_rc" ]; then
        if ! grep -qF "AGENTIC" "$shell_rc" 2>/dev/null; then
            {
                echo ""
                echo "# AgenticOS"
                if [ "$current_shell" = "fish" ]; then
                    echo "set -gx PATH \$PATH $BIN_DIR"
                else
                    echo "export PATH=\"\$PATH:$BIN_DIR\""
                fi
            } >> "$shell_rc"
            info "Added $BIN_DIR to PATH in $shell_rc"
        fi
    elif [ -n "$shell_rc" ]; then
        {
            echo "# AgenticOS"
            echo "export PATH=\"\$PATH:$BIN_DIR\""
        } > "$shell_rc"
        info "Created $shell_rc with PATH configuration"
    fi
}

configure_mcp_desktop() {
    info "Configuring MCP for Claude Desktop..."
    local config_dir config_file
    if [ "$OS" = "macos" ]; then
        config_dir="$HOME/Library/Application Support/Claude"
    elif [ "$OS" = "linux" ]; then
        config_dir="${XDG_CONFIG_HOME:-$HOME/.config}/claude"
    else
        config_dir="$APPDATA/Claude"
    fi

    config_file="$config_dir/claude_desktop_config.json"
    mkdir -p "$config_dir"

    if [ -f "$config_file" ]; then
        cp "$config_file" "$config_file.backup.$(date +%s)"
        if command -v jq &>/dev/null; then
            local tmp_file
            tmp_file=$(mktemp)
            jq --arg bin "$BIN_DIR/aegis" '
                .mcpServers["agentic-aegis"] = {
                    "command": $bin,
                    "args": ["serve"]
                }
            ' "$config_file" > "$tmp_file"
            mv "$tmp_file" "$config_file"
            success "MCP configured for Claude Desktop (merged into existing config)"
        else
            warn "jq not found. Please manually add agentic-aegis to $config_file"
        fi
    else
        cat > "$config_file" << CONF
{
  "mcpServers": {
    "agentic-aegis": {
      "command": "$BIN_DIR/aegis",
      "args": ["serve"]
    }
  }
}
CONF
        success "MCP configured for Claude Desktop (new config created)"
    fi
}

generate_auth_token() {
    local token_file="$INSTALL_DIR/auth_token"
    if [ ! -f "$token_file" ]; then
        mkdir -p "$INSTALL_DIR"
        if command -v openssl &>/dev/null; then
            openssl rand -hex 32 > "$token_file"
        else
            head -c 32 /dev/urandom | xxd -p -c 64 > "$token_file"
        fi
        chmod 600 "$token_file"
        info "Generated auth token: $token_file"
        warn "Set AGENTIC_TOKEN to the contents of this file"
    else
        info "Auth token already exists at $token_file"
    fi
}

install_service() {
    info "Installing system service..."
    if [ "$OS" = "macos" ]; then
        local plist="$HOME/Library/LaunchAgents/com.agentralabs.aegis.plist"
        mkdir -p "$HOME/Library/LaunchAgents"
        mkdir -p "$INSTALL_DIR/logs"
        cat > "$plist" << PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.agentralabs.aegis</string>
    <key>ProgramArguments</key>
    <array>
        <string>$BIN_DIR/aegis</string>
        <string>serve</string>
        <string>--mode</string>
        <string>stdio</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>$INSTALL_DIR/logs/aegis.log</string>
    <key>StandardErrorPath</key>
    <string>$INSTALL_DIR/logs/aegis.err</string>
</dict>
</plist>
PLIST
        launchctl load "$plist" 2>/dev/null || true
        success "Installed launchd service"
    elif [ "$OS" = "linux" ]; then
        local service_dir="$HOME/.config/systemd/user"
        mkdir -p "$service_dir"
        cat > "$service_dir/agentic-aegis.service" << UNIT
[Unit]
Description=AgenticAegis MCP Server
After=network.target

[Service]
Type=simple
ExecStart=$BIN_DIR/aegis serve --mode stdio
Restart=always
RestartSec=10

[Install]
WantedBy=default.target
UNIT
        systemctl --user daemon-reload 2>/dev/null || true
        systemctl --user enable agentic-aegis 2>/dev/null || true
        success "Installed systemd service"
    else
        warn "Service installation not supported on $OS"
    fi
}

print_client_help() {
    local BIN_PATH="$BIN_DIR/aegis"
    local SERVER_ARGS_TEXT='["serve"]'

    echo ""
    echo "MCP client summary:"
    echo "  - Auto-configured: Claude Desktop (if detected)"
    echo "  - Auto-configured: VS Code / Cursor (if detected)"
    echo ""
    echo "Universal MCP entry (works in any MCP client):"
    echo "  command: ${BIN_PATH}"
    echo "  args: ${SERVER_ARGS_TEXT}"
    echo ""
    echo "Quick terminal check:"
    echo "  ${BIN_PATH} --version"
    echo "  (Run aegis info to verify installation)"
}

print_next_steps() {
    local SERVER_KEY="agentic-aegis"
    echo ""
    echo "What happens after installation:"
    echo "  1. ${SERVER_KEY} was installed as MCP server command: ${BIN_DIR}/aegis"
    if [ "$PROFILE" = "server" ]; then
        echo "  2. Generate a token (openssl rand -hex 32) and set AGENTIC_TOKEN on the server."
        echo "  3. Start MCP with auth, connect clients, then restart clients."
        echo "  4. Optional feedback: open https://github.com/agentralabs/agentic-aegis/issues"
    else
        echo "  2. Restart your MCP client so it reloads MCP config."
        echo "  3. After restart, confirm '${SERVER_KEY}' appears in your MCP server list."
        echo "  4. Run: aegis info"
        echo "  5. Optional feedback: open https://github.com/agentralabs/agentic-aegis/issues"
    fi
}

main() {
    if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
        echo "AgenticAegis Universal Installer"
        echo "Usage: curl -fsSL https://agentralabs.com/install/aegis | bash"
        echo "Profiles: desktop, terminal, server"
        exit 0
    fi

    echo ""
    echo "============================================================"
    echo "         AgenticAegis Universal Installer"
    echo "           Sister #11: The Shield"
    echo "============================================================"
    echo ""

    check_prerequisites
    detect_platform
    resolve_version
    download_binary
    configure_path

    case "$PROFILE" in
        desktop)
            configure_mcp_desktop
            ;;
        terminal)
            info "Terminal profile: No MCP configuration"
            ;;
        server)
            generate_auth_token
            install_service
            ;;
        *)
            error "Unknown profile: $PROFILE (use: desktop, terminal, or server)"
            ;;
    esac

    echo ""
    success "Installation complete!"
    echo ""

    if [ "$PROFILE" = "server" ]; then
        echo "Server mode: no MCP client config was modified."
        echo "Set AGENTIC_TOKEN before starting the service:"
        echo "  export AGENTIC_TOKEN=\$(openssl rand -hex 32)"
        echo ""
    else
        print_client_help
    fi

    print_next_steps

    echo ""
    echo "Restart your terminal (or run: source ~/.bashrc) to use the aegis command."
    echo ""
}

main "$@"
