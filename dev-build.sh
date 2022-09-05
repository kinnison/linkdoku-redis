#!/bin/sh

set -e

# This script is used by `dev-entrypoint.sh` and is run by cargo watch
# whenever the inputs change.

# We expect to be running in the docker, so don't use this on a host.

CARGO_TARGET_DIR=/build/target
export CARGO_TARGET_DIR


echo "*** Constructing frontend with trunk..."
cd /code/frontend
trunk build --public-url /- -d /build/dist

echo "*** Content of dist tree"
find /build/dist -type f

echo "*** Constructing backend..."
cd /code/backend
cargo build

echo "*** Running backend with resources from trunk and development settings..."

RUST_LOG=linkdoku_backend=info,tower_http=info
export RUST_LOG

LINKDOKU_RESOURCES=/build/dist
export LINKDOKU_RESOURCES
LINKDOKU_PORT=3000
export LINKDOKU_PORT

exec /build/target/debug/linkdoku-backend
