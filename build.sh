#!/bin/bash

# Arguments
METHOD=$1
TARGET=$2

# Frontend
cd web/

# Install requirements
npm i

# Build
npm run build

# Copy UI to src folder
cp -r dist/ ../src/static

# Check selected method and build

if [[ $METHOD == 'cargo' ]]; then
    echo "Building with cargo..."
    cargo build --release
elif [[ $METHOD == 'cargo-dev' ]]; then
    echo "Building with cargo using --verbose (dev)..."
elif [[ $METHOD == 'cross' ]]; then
    echo "Building with cross for $TARGET..."
    cross build --target $TARGET --release
else
    echo "Unkown method!"
    exit 1
fi
