#!/bin/sh

echo "Starting entrypoint"

echo "*** Waiting for redis to arrive"

while ! redis-cli -h redis PING >/dev/null; do
    echo "   *** Waiting..."
    sleep 1
done

echo "*** Redis is up"

echo "Performing sanity check"

if ! test -d /build; then
    echo "*** /build not found, exiting"
    exit 1
fi

if ! test -r /code/Cargo.toml; then
    echo "*** Source code not found, exiting"
fi

cd /code

cargo watch -i target -i frontend/dist $(ls *.md | xargs -n 1 echo -i) -- /code/dev-build.sh
