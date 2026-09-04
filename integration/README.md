# Live Integration

Executable Rust and Deno tests are the integration catalog. The standard runner
discovers them directly and provides one shared real NATS and Trellis runtime:

```sh
deno run -A -c ts/deno.json integration/live_runner.ts
```

Rust live tests are ordinary `libtest` cases under
`rust/crates/trellis/tests/integration/`. Deno tests are discovered at the
boundary that invokes them; for example, the Rust Field Ops case runs the Deno
out-of-tree consumer proof. Add new coverage at the smallest real boundary that
proves the invariant. Hidden skips and separate test inventories are forbidden.
