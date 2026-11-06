#!/usr/bin/env bash
# Runs once after the devcontainer is created.
set -euo pipefail

cd "$(dirname "$0")/.."

# Named volumes are created owned by root, hand them to the dev user
sudo chown -R "$(id -u):$(id -g)" target web/node_modules docs/node_modules /usr/local/cargo/registry

# Frontend dependencies, then build it once: src/server.rs embeds src/static/* with include_str!,
# so `cargo build` fails until the frontend has been built
(cd web && bun install --frozen-lockfile)
./build.sh

# Docs dependencies
(cd docs && bun install --frozen-lockfile)

# Warm up the crate cache
cargo fetch

cat <<'EOF'

Cup devcontainer is ready.

The Docker daemon inside this container starts empty. Pull some images to test against, e.g.:
  docker pull nginx:1.25 alpine:3.18 ghcr.io/linuxserver/syncthing:latest
Then:
  cargo run -- check                    # CLI
  cargo run -- serve                    # server on http://localhost:8000
  cd web && bun dev                     # frontend dev server on http://localhost:5173
  cd docs && bun dev                    # docs on http://localhost:3000
  ./build.sh cargo build                # rebuild after frontend changes
EOF
