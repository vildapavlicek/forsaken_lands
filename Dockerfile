# Start with the official Rust slim image
FROM rust:slim

# Install core tools, Node.js, and ALL of Bevy's native Linux dependencies
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libudev-dev \
    libasound2-dev \
    libwayland-dev \
    libxkbcommon-dev \
    libx11-dev \
    libxrandr-dev \
    libxi-dev \
    libxcursor-dev \
    && curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Install the Gemini CLI globally
RUN npm install -g @google/gemini-cli

# Set the working directory
WORKDIR /workspace

# Set the entrypoint
ENTRYPOINT ["gemini"]