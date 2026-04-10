#!/usr/bin/env bash
set -euo pipefail

BREW_PREFIX="/home/linuxbrew/.linuxbrew"
BREW_BIN="${BREW_PREFIX}/bin/brew"
NVM_VERSION="v0.40.4"
NVM_DIR="${HOME}/.nvm"
JAVA_HOME_CANDIDATE="/usr/lib/jvm/java-25-openjdk-amd64"

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
    $0 !~ /^export NVM_DIR=.*\.nvm/ &&
    $0 !~ /nvm\.sh/ &&
    $0 !~ /nvm\/bash_completion/ &&
    $0 !~ /^export JAVA_HOME=\/usr\/lib\/jvm\/java-25-openjdk-amd64/ &&
    $0 !~ /^\s*if \[ -d \/usr\/lib\/jvm\/java-25-openjdk-amd64 \]/ &&
    $0 !~ /^export PATH="\$HOME\/\.local\/bin:\$PATH"$/ {
      print
    }
  ' "$file" > "$tmp"

  mv "$tmp" "$file"

  cat >> "$file" <<'EOF'

# >>> katana dev env >>>
export PATH="$HOME/.local/bin:$PATH"
if [ -x /home/linuxbrew/.linuxbrew/bin/brew ]; then
  eval "$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)"
fi
export NVM_DIR="$HOME/.nvm"
if [ -s "$NVM_DIR/nvm.sh" ]; then
  . "$NVM_DIR/nvm.sh"
fi
if [ -s "$NVM_DIR/bash_completion" ]; then
  . "$NVM_DIR/bash_completion"
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

install_nvm_wrapper() {
  sudo tee /usr/local/bin/nvm >/dev/null <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
export NVM_DIR="${HOME}/.nvm"
if [ ! -s "${NVM_DIR}/nvm.sh" ]; then
  echo "nvm is not installed at ${NVM_DIR}" >&2
  exit 1
fi
# shellcheck disable=SC1090
. "${NVM_DIR}/nvm.sh"
nvm "$@"
EOF
  sudo chmod +x /usr/local/bin/nvm
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
  openjdk-25-jdk \
  procps \
  xz-utils \
  zip \
  unzip

log "Fixing home directory ownership for user cache dirs..."
mkdir -p "${HOME}/.cache" "${HOME}/.local" "${HOME}/.config"
sudo chown -R "$(id -u):$(id -g)" "${HOME}/.cache" "${HOME}/.local" "${HOME}/.config"

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

if [ ! -s "${NVM_DIR}/nvm.sh" ]; then
  log "Installing nvm ${NVM_VERSION}..."
  PROFILE=/dev/null NVM_DIR="${NVM_DIR}" bash -c "$(curl -fsSL "https://raw.githubusercontent.com/nvm-sh/nvm/${NVM_VERSION}/install.sh")"
else
  log "nvm already installed. Skipping install."
fi

if [ -s "${NVM_DIR}/nvm.sh" ]; then
  # shellcheck disable=SC1090
  . "${NVM_DIR}/nvm.sh"
  log "Installing Node.js v24 via nvm..."
  nvm install 24
  nvm alias default 24
  nvm use default >/dev/null
  install_nvm_wrapper
  NODE_BIN_DIR="$(dirname "$(nvm which default)")"
  link_global_command "${NODE_BIN_DIR}/node" node
  link_global_command "${NODE_BIN_DIR}/npm" npm
  link_global_command "${NODE_BIN_DIR}/npx" npx
  link_global_command "${NODE_BIN_DIR}/corepack" corepack
fi

if [ -d "${JAVA_HOME_CANDIDATE}" ]; then
  export JAVA_HOME="${JAVA_HOME_CANDIDATE}"
  export PATH="${JAVA_HOME}/bin:${PATH}"
  link_global_command "${JAVA_HOME}/bin/java" java
  link_global_command "${JAVA_HOME}/bin/javac" javac
fi

log "Provisioning complete."
printf 'brew: '
command -v brew || true
brew --version | head -n 1 || true
printf 'node: '
command -v node || true
node -v || true
printf 'java: '
command -v java || true
java -version 2>&1 | sed -n '1,2p' || true
