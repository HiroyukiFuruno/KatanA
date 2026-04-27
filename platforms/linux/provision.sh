#!/usr/bin/env bash
set -euo pipefail

BREW_PREFIX="/home/linuxbrew/.linuxbrew"
BREW_BIN="${BREW_PREFIX}/bin/brew"
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
printf 'java: '
command -v java || true
java -version 2>&1 | sed -n '1,2p' || true
