#!/bin/sh

# This is kind of like a shim that makes sure the frontend is rebuilt when running a build. For example you can run `./build.sh cargo build --release`

# Frontend
cd web/

# Build
bun run build

# Copy UI to src folder
cp -r dist/ ../src/static

# Run command from argv

$@