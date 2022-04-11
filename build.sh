#!/usr/bin/env bash

cargo build --target wasm32-unknown-unknown --release --package manage && \
 ic-cdk-optimizer ./target/wasm32-unknown-unknown/release/manage.wasm -o ./target/wasm32-unknown-unknown/release/manage.wasm

#  cargo build --target wasm32-unknown-unknown --package chain_cloud --release