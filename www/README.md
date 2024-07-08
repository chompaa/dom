# www

This directory contains the files for the playground. Following instructions will assume you have [Bun](https://bun.sh/), but feel free to use your preferred JavaScript runtime.

## Development

> [!IMPORTANT]
> This is assumes you have the `dom_wasm` crate in the path `../dom_wasm/`. If you don't, modify `package.json`.

First, use `wasm-pack` to build `dom_wasm` and install all dependencies using

```sh
bun run install:wasm
```

then use

```sh
bun run dev
```

to start the local development server.
