./target/release/node-template benchmark pallet \
    --execution=wasm \
    --wasm-execution=compiled \
    --pallet "pallet_kitty" \
    --extrinsic "*" \
    --repeat 5 \
    --output weight.rs
