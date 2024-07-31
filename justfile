legacy-parsing *args:
  just -f legacy/parsing/justfile {{args}}

website *args:
  just -f website/justfile {{args}}

rust *args:
  just -f rust/justfile {{args}}

build-rotext:
  just rust build-rotext-wasm-bindings
  cd packages/wasm-bindings-adapter && pnpm run build

build-rotext-dev:
  just rust build-rotext-wasm-bindings-dev
  cd packages/wasm-bindings-adapter && pnpm run build