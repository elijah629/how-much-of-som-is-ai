cd        training-bin
cargo     r            -r
cd        ../sonai
wasm-pack build        --release -d ../inference-wasm-web/src/pkg
