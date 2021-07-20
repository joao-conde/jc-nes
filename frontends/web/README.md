# Web NES emulator

## Building

At `Cargo.toml` level, use the following to build and output to the `web` directory using [wasm-pack](https://rustwasm.github.io/wasm-pack/):

```bash
wasm-pack build --release --target=web --no-typescript -d web/ -- --features web
```

Then just serve the `web` folder and open `index.html` in a browser.
