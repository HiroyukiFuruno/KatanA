FROM rust:slim

# Install dependencies required to download and build cross-compilation toolchains
RUN apt-get update && apt-get install -y \
    curl \
    pkg-config \
    libssl-dev \
    gcc \
    clang \
    lld \
    && ln -s /usr/bin/clang /usr/bin/clang-cl \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-xwin using pre-built binary to save compilation time
RUN curl -L https://github.com/rust-cross/cargo-xwin/releases/download/v0.21.4/cargo-xwin-v0.21.4.x86_64-unknown-linux-musl.tar.gz | tar xz -C /usr/local/cargo/bin

# Add windows target
RUN rustup target add x86_64-pc-windows-msvc

WORKDIR /app
CMD ["cargo", "xwin", "check", "--workspace", "--target", "x86_64-pc-windows-msvc", "--tests"]
