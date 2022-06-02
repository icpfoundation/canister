#!/usr/bin/env bash

cargo build --target wasm32-unknown-unknown --release
ic-cdk-optimizer ./target/wasm32-unknown-unknown/release/manage.wasm -o ./target/wasm32-unknown-unknown/release/manage.wasm && \ 

ic-cdk-optimizer ./target/wasm32-unknown-unknown/release/test_canister.wasm -o ./target/wasm32-unknown-unknown/release/test_canister.wasm && \

ic-cdk-optimizer ./target/wasm32-unknown-unknown/release/image_store.wasm -o ./target/wasm32-unknown-unknown/release/image_store.wasm && \

ic-cdk-optimizer ./target/wasm32-unknown-unknown/release/canister_log.wasm -o ./target/wasm32-unknown-unknown/release/canister_log.wasm

#  cargo build --target wasm32-unknown-unknown --package chain_cloud --release