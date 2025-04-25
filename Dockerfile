### Build UI ###
FROM oven/bun:alpine AS web

# Copy package.json and lockfile from web
WORKDIR /web
COPY ./web/package.json ./web/bun.lockb ./

# Install requirements
RUN bun install

# Copy web folder
COPY ./web .

# Build frontend
RUN bun run build

### Build Cup ###
FROM rust:1-alpine AS build

# Requirements
RUN apk add musl-dev

# Copy files
WORKDIR /cup

COPY Cargo.toml .
COPY Cargo.lock .
COPY ./src ./src

# Copy UI from web builder
COPY --from=web /web/dist src/static

# Build
RUN cargo build --release

### Main ###
FROM scratch

# Copy binary
COPY --from=build /cup/target/release/cup /cup

EXPOSE 8000
ENTRYPOINT ["/cup"]
