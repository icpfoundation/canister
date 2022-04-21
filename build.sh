#!/usr/bin/env bash

cargo build --target wasm32-unknown-unknown --release
ic-cdk-optimizer ./target/wasm32-unknown-unknown/release/manage.wasm -o ./target/wasm32-unknown-unknown/release/manage.wasm && \ 


ic-cdk-optimizer ./target/wasm32-unknown-unknown/release/test_canister.wasm -o ./target/wasm32-unknown-unknown/release/test_canister.wasm

#  cargo build --target wasm32-unknown-unknown --package chain_cloud --release