FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# Bun from the official image (same major version as the main Dockerfile and CI)
COPY --from=oven/bun:1-debian /usr/local/bin/bun /usr/local/bin/bun
RUN ln -s /usr/local/bin/bun /usr/local/bin/bunx
