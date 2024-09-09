#!/bin/sh

# Exit on error
set -e

# This is kind of like a shim that makes sure the frontend is rebuilt when running a build. For example you can run `./build.sh cargo build --release`

# Remove old files
rm -rf src/static

# Frontend
cd web/

# Build
bun run build

# Copy UI to src folder
cp -r dist/ ../src/static

# Run command from argv

$@