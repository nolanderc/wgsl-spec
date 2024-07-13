#!/usr/bin/env bash

set -eu -o pipefail

function list {
    echo "commands:"
    declare -F | awk '{ print "  " $3 }'
}

function update-upstream {
    curl https://www.w3.org/TR/WGSL/ > spec/WGSL.html
}

function build {
    cargo build -F scrape "$@"
}

function test {
    cargo test -F scrape "$@"
}

function run {
    cargo run -F scrape "$@"
}

function check {
    cargo clippy -F scrape "$@"
}

function watch {
    watchexec -e rs,toml -c --on-busy-update=do-nothing -- ./x "$@"
}

if [[ $# == 0 ]]; then
    list
    exit 1
elif declare -f "$1" > /dev/null; then
    "$@"
    exit $?
else
    echo "'$1' is not a known command"
    list
    exit 1
fi
