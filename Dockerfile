### Build UI ###
FROM node:20 AS web

# Install bun
RUN curl -fsSL https://bun.sh/install | bash

# Copy web folder
COPY ./web /web
WORKDIR /web

# Install requirements
RUN ~/.bun/bin/bun install

# Build frontend
RUN ~/.bun/bin/bun run build

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
