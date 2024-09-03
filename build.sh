#!/bin/sh

rm -rf src/static
cd web/
bun run build
cp -r dist/ ../src/static
cargo build $@