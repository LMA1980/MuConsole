______________________________________________________________________

## name: muconsole-wasm-ui description: "Build and test the MuConsole unified WASM observability, CI/CD, and hardware telemetry console across muconsole-protocol, muconsole-theme, muconsole-layout, and muconsole-ui."

# `MuConsole` Unified WASM Frontend & UX Gate Invariants

## 1. Dual-Layer Static Asset Caching & Mobile Viewport Compliance

- Serve HTML entry points with `Cache-Control: no-cache, no-store, must-revalidate` and WASM/JS/CSS assets with content-hash URL busting (`?v=<hash>`) and `Cache-Control: no-cache, must-revalidate`.
- Enforce mobile viewport constraints across all `muconsole-ui` components: contained segmented controls with horizontal scrolling, responsive labels, and `font-size: 16px` on form inputs to prevent iOS Safari auto-zoom.
