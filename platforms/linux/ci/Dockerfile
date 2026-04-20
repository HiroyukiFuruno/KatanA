FROM rust:1.95.0-slim

# Install required dependencies for Katana Linux
RUN apt-get update && apt-get install -y \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libxkbcommon-dev \
    libssl-dev \
    gcc \
    lld \
    pkg-config \
    libdbus-1-dev \
    curl \
    python3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
CMD ["cargo", "test", "--workspace"]
