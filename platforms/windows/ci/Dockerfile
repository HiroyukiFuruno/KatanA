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
RUN ARCH=$(uname -m) && \
    case "$ARCH" in \
      x86_64) XWIN_ARCH="x86_64" ;; \
      aarch64|arm64) XWIN_ARCH="aarch64" ;; \
      *) echo "Unsupported architecture: $ARCH" && exit 1 ;; \
    esac && \
    curl -L "https://github.com/rust-cross/cargo-xwin/releases/download/v0.21.4/cargo-xwin-v0.21.4.${XWIN_ARCH}-unknown-linux-musl.tar.gz" | tar xz -C /usr/local/cargo/bin

# Add windows target
RUN rustup target add x86_64-pc-windows-msvc

WORKDIR /app
CMD ["cargo", "xwin", "check", "--workspace", "--target", "x86_64-pc-windows-msvc", "--tests"]
