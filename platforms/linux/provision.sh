#!/usr/bin/env bash
set -euo pipefail

BREW_PREFIX="/home/linuxbrew/.linuxbrew"
BREW_BIN="${BREW_PREFIX}/bin/brew"
JAVA_HOME_CANDIDATE="/usr/lib/jvm/java-25-openjdk-amd64"
WORKSPACE_DIR="/config/workspace/katana"
CARGO_HOME="${CARGO_HOME:-${HOME}/.cargo}"
RUSTUP_HOME="${RUSTUP_HOME:-${HOME}/.rustup}"

log() {
  printf '[katana-linux] %s\n' "$*"
}

ensure_file() {
  local file="$1"
  if [ ! -f "$file" ]; then
    touch "$file"
  fi
}

upsert_env_block() {
  local file="$1"
  local tmp

  ensure_file "$file"
  tmp="$(mktemp)"

  awk '
    BEGIN { skip = 0 }
    /# >>> katana dev env >>>/ { skip = 1; next }
    /# <<< katana dev env <<</ { skip = 0; next }
    skip == 0 &&
    $0 !~ /brew shellenv bash/ &&
    $0 !~ /^export JAVA_HOME=\/usr\/lib\/jvm\/java-25-openjdk-amd64/ &&
    $0 !~ /^\s*if \[ -d \/usr\/lib\/jvm\/java-25-openjdk-amd64 \]/ &&
    $0 !~ /^export PATH="\$HOME\/\.local\/bin:\$PATH"$/ {
      print
    }
  ' "$file" > "$tmp"

  mv "$tmp" "$file"

  cat >> "$file" <<'EOF'

# >>> katana dev env >>>
export CARGO_HOME="${HOME}/.cargo"
export CARGO_TARGET_DIR="/config/workspace/katana/target"
export PATH="${CARGO_HOME}/bin:$HOME/.local/bin:$PATH"
if [ -x /home/linuxbrew/.linuxbrew/bin/brew ]; then
  eval "$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)"
fi
if [ -d /usr/lib/jvm/java-25-openjdk-amd64 ]; then
  export JAVA_HOME="/usr/lib/jvm/java-25-openjdk-amd64"
  export PATH="$JAVA_HOME/bin:$PATH"
fi
# <<< katana dev env <<<
EOF
}

ensure_bash_profile() {
  local file="${HOME}/.bash_profile"

  ensure_file "$file"

  if ! grep -qxF '[ -f "$HOME/.profile" ] && . "$HOME/.profile"' "$file"; then
    printf '%s\n' '[ -f "$HOME/.profile" ] && . "$HOME/.profile"' >> "$file"
  fi

  if ! grep -qxF '[ -f "$HOME/.bashrc" ] && . "$HOME/.bashrc"' "$file"; then
    printf '%s\n' '[ -f "$HOME/.bashrc" ] && . "$HOME/.bashrc"' >> "$file"
  fi
}

source_cargo_env() {
  if [ -f "${CARGO_HOME}/env" ]; then
    # shellcheck disable=SC1091
    . "${CARGO_HOME}/env"
  fi
}

ensure_rustup() {
  export CARGO_HOME
  export RUSTUP_HOME
  export PATH="${CARGO_HOME}/bin:${PATH}"

  if ! command -v rustup >/dev/null 2>&1; then
    log "Installing rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
      | sh -s -- -y --no-modify-path
  else
    log "rustup already installed. Skipping install."
  fi

  source_cargo_env
  rustup toolchain install stable --no-self-update
  rustup default stable
}

ensure_cargo_sweep() {
  source_cargo_env
  if cargo sweep --version >/dev/null 2>&1; then
    log "cargo-sweep already installed. Skipping install."
    return
  fi

  log "Installing cargo-sweep..."
  cargo install cargo-sweep
}


install_brew_wrapper() {
  sudo tee /usr/local/bin/brew >/dev/null <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
exec /home/linuxbrew/.linuxbrew/bin/brew "$@"
EOF
  sudo chmod +x /usr/local/bin/brew
}

link_global_command() {
  local source_path="$1"
  local target_name="$2"

  if [ -x "$source_path" ]; then
    sudo ln -sf "$source_path" "/usr/local/bin/${target_name}"
  fi
}

log "Installing apt prerequisites..."
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  ca-certificates \
  curl \
  file \
  git \
  lld \
  make \
  openjdk-25-jdk \
  dbus-x11 \
  libdbus-1-dev \
  libssl-dev \
  libxcb-render0-dev \
  libxcb-shape0-dev \
  libxcb-xfixes0-dev \
  libxkbcommon-dev \
  pkg-config \
  procps \
  python3 \
  xdg-desktop-portal \
  xdg-desktop-portal-gtk \
  xdg-utils \
  xz-utils \
  zenity \
  zip \
  unzip

log "Fixing owned cache and build directories..."
mkdir -p "${HOME}/.cache" "${HOME}/.local" "${HOME}/.config" "${CARGO_HOME}" "${RUSTUP_HOME}"
if [ -d "${WORKSPACE_DIR}/target" ]; then
  sudo chown -R "$(id -u):$(id -g)" "${WORKSPACE_DIR}/target"
fi
sudo chown -R "$(id -u):$(id -g)" \
  "${HOME}/.cache" \
  "${HOME}/.local" \
  "${HOME}/.config" \
  "${CARGO_HOME}" \
  "${RUSTUP_HOME}"

if [ ! -x "$BREW_BIN" ]; then
  log "Installing Homebrew..."
  NONINTERACTIVE=1 CI=1 bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
else
  log "Homebrew already installed. Skipping install."
fi

upsert_env_block "${HOME}/.bashrc"
upsert_env_block "${HOME}/.profile"
ensure_bash_profile

if [ -x "$BREW_BIN" ]; then
  eval "$("$BREW_BIN" shellenv)"
  install_brew_wrapper
fi


if [ -d "${JAVA_HOME_CANDIDATE}" ]; then
  export JAVA_HOME="${JAVA_HOME_CANDIDATE}"
  export PATH="${JAVA_HOME}/bin:${PATH}"
  link_global_command "${JAVA_HOME}/bin/java" java
  link_global_command "${JAVA_HOME}/bin/javac" javac
fi

ensure_rustup
ensure_cargo_sweep

log "Provisioning complete."
printf 'brew: '
command -v brew || true
brew --version | head -n 1 || true
printf 'java: '
command -v java || true
java -version 2>&1 | sed -n '1,2p' || true
printf 'rustc: '
command -v rustc || true
rustc --version || true
printf 'cargo-sweep: '
cargo sweep --version || true
