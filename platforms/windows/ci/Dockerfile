FROM rust:1.95.0-slim

# Install dependencies required to download and build cross-compilation toolchains
RUN apt-get update && apt-get install -y \
    curl \
    pkg-config \
    libssl-dev \
    gcc \
    clang \
    lld \
    llvm \
    && ln -s /usr/bin/clang /usr/bin/clang-cl \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-xwin using pre-built binary dynamically based on architecture to save compilation time
RUN XWIN_VERSION=0.23.1 && \
    ARCH=$(uname -m) && \
    case "$ARCH" in \
      x86_64) XWIN_ARCH="x86_64"; XWIN_SHA256="c492c6dfb7e5ac0eee586b796e0fc950ba13077e7f0bbb046f445d71790d5360" ;; \
      aarch64|arm64) XWIN_ARCH="aarch64"; XWIN_SHA256="dc0a0fb28fdd90405f69e7365bce91d2504ea7491eb4d79d5655ffa106358925" ;; \
      *) echo "Unsupported architecture: $ARCH" && exit 1 ;; \
    esac && \
    XWIN_ASSET="cargo-xwin-v${XWIN_VERSION}.${XWIN_ARCH}-unknown-linux-musl.tar.gz" && \
    curl -fsSL --retry 2 "https://github.com/rust-cross/cargo-xwin/releases/download/v${XWIN_VERSION}/${XWIN_ASSET}" -o /tmp/cargo-xwin.tar.gz && \
    printf '%s  %s\n' "$XWIN_SHA256" /tmp/cargo-xwin.tar.gz | sha256sum -c - && \
    tar xzf /tmp/cargo-xwin.tar.gz -C /usr/local/cargo/bin && \
    rm /tmp/cargo-xwin.tar.gz

# Add windows target
RUN rustup target add x86_64-pc-windows-msvc

# Install cargo-sweep
RUN cargo install cargo-sweep

WORKDIR /app
CMD cargo sweep --time 7 --dry-run && cargo xwin check --workspace --target x86_64-pc-windows-msvc --tests
