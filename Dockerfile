# Build UI
FROM node:20-alpine3.20 as web

# Copy web folder
COPY ./web ./web
WORKDIR web

# Install requirements
RUN npm i

# Build
RUN npm run build

# Build Cup
FROM rust:1.80.1-alpine3.20 AS build

# Requirements
RUN apk add musl-dev

# Copy rust
WORKDIR /cup

COPY Cargo.toml .
COPY Cargo.lock .
COPY ./src ./src

# Copy UI from web builder
COPY --from=web /web/dist src/static

# Build
RUN cargo build --release

# Runner
FROM scratch

# Copy binary
COPY --from=build /cup/target/release/cup /cup

ENTRYPOINT ["/cup"]