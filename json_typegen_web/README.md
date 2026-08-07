# `json_typegen_web`

Web interface for `json_typegen` using WebAssembly.

See the [main project repository](https://github.com/evestera/json_typegen) for
project documentation.

## Development

Install [wasm-pack](https://github.com/rustwasm/wasm-pack) and Bun 1.3.13. Build
the local wasm package before installing the web dependencies:

```sh
bun run build:wasm
bun ci
bun run dev
```

When changing Rust sources, rebuild the wasm package in another terminal:

```sh
watchexec -w ../json_typegen_wasm -e rs,toml -- bun run build:wasm
```

Run `bun run build` to create the production site in `dist/`.
