# Build Stage
FROM rust as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
  pkg-config \
  libssl-dev \
  nodejs \
  npm

# Install Dioxus CLI
RUN cargo install dioxus-cli

# Create app directory
WORKDIR /app

# Copy dependency files first for caching
COPY Cargo.toml Cargo.lock ./
COPY api/Cargo.toml api/
COPY desktop/Cargo.toml desktop/
COPY mobile/Cargo.toml mobile/
COPY ui/Cargo.toml ui/
COPY web/Cargo.toml web/
COPY lib/shared/Cargo.toml lib/shared/
COPY lib/soulful/Cargo.toml lib/soulful/

# Copy source code
COPY . .

# Install Tailwind dependencies
RUN npm install

# Build the Tailwind CSS
RUN npx @tailwindcss/cli -i ./web/assets/input.css -o ./web/assets/tailwind.css

# Build the application
RUN dx build --platform web --release

# Runtime Stage
FROM debian:bullseye-slim

# Install runtime dependencies: openssl, python/pip for beets
RUN apt-get update && apt-get install -y \
  libssl1.1 \
  ca-certificates \
  python3 \
  python3-pip \
  ffmpeg \
  && rm -rf /var/lib/apt/lists/*

# Install beets
RUN pip3 install beets

# Working directory
WORKDIR /app

# Copy artifacts from builder
COPY --from=builder /app/target/dx/web/release/web /app/server
COPY --from=builder /app/target/dx/web/release/assets /app/assets
COPY beets_config.yaml /app/beets_config.yaml

# Create data directory for SQLite
RUN mkdir -p /data

# Set environment variables
ENV DATABASE_URL=sqlite:/data/soulful.db
ENV PORT=9765
ENV IP=0.0.0.0

# Expose the port
EXPOSE 9765

# Run the server
CMD ["/app/server"]