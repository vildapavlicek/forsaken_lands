# Start with the official Rust slim image
FROM rust:slim

# Install system dependencies and Node.js (for the CLI)
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
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
