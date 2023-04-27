#!/bin/bash

# workspace-optimizer only optimizes contracts in the `contracts/*`
# folder, and needs to be invoked differently depending on the chip
# archetecture, so this script exists to abstract that away.

# this function is called when the script exits,
# regardless of the reason (error, success, etc)
function cleanup {
  for folder in accessories/*
  do
    folder=${folder/'accessories/'}
    rm -R contracts/$folder
  done

  # do cargo check to update (revert) lock file
  cargo check
}

# register the cleanup function to be called on the EXIT signal
trap cleanup EXIT

# copy all the accessories contracts to contracts folder
# we need this because workspace-optimizer is only compiling
# contracts that are in the contracts folder.
for folder in accessories/*
do
  folder=${folder/'accessories/'}
  cp -R accessories/$folder contracts/$folder
done

# do cargo check to update lock file
cargo check

# Run workspace-optimizer
if [[ $(uname -m) =~ "arm64" ]]; then \
    docker run --rm -v "$(pwd)":/code \
        --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
        --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
        --platform linux/arm64 \
        cosmwasm/workspace-optimizer-arm64:0.12.13
else
    docker run --rm -v "$(pwd)":/code \
        --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
        --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
        --platform linux/amd64 \
        cosmwasm/workspace-optimizer:0.12.13
fi

mkdir -p tests/wasms

if [[ $(uname -m) =~ "arm64" ]]; then \
    cp artifacts/polytone_note-aarch64.wasm tests/wasms/polytone_note.wasm
    cp artifacts/polytone_voice-aarch64.wasm tests/wasms/polytone_voice.wasm
    cp artifacts/polytone_tester-aarch64.wasm tests/wasms/polytone_tester.wasm
    cp artifacts/polytone_proxy-aarch64.wasm tests/wasms/polytone_proxy.wasm
else
    cp artifacts/polytone_note.wasm tests/wasms/
    cp artifacts/polytone_voice.wasm tests/wasms/
    cp artifacts/polytone_tester.wasm tests/wasms/
    cp artifacts/polytone_proxy.wasm tests/wasms/
fi
