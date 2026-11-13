# autofuzz harness: WebAssembly

The harness plugin the autofuzz engine loads for WebAssembly targets. It instantiates a `.wasm` module with `wasmtime`, discovers its exports, lowers generated values to wasm parameters, invokes each export under a tick budget, and answers with coverage digests and fault outcomes over the plugin protocol (`afz/1`).

Host runtime: none. The plugin is one host binary; the target is a `.wasm` file. Nothing here needs a `wasm32-unknown-unknown` target installed — except the scan example below, which builds one.

## Install

```sh
afz install-harness https://github.com/DustinEwan/autofuzz-harness-wasm.git
```

Install runs `cargo build --release`, then drives the conformance suite against the built plugin; a plugin that violates any rule is not recorded. `afz harnesses` lists what the registry holds.

A scan project names the harness and no command, because the module is the scan target, not a plugin property:

```json
{ "codebase": { "harness": "wasm", "include": ["**/*.wasm"] } }
```

The plugin takes the module from the first non-flag argument or, failing that, the first `.wasm` under the scan root. To try it on the fixture that ships here:

```sh
cargo build --release --target wasm32-unknown-unknown --manifest-path test-targets/wasm-probe/Cargo.toml
afz probe-plugin --path test-targets/wasm-probe
```

## Author

Edit, then validate — there are no dependencies to fetch beyond cargo's registry cache:

```sh
afz validate-harness
```

`afz validate-harness` with no argument validates the repository it is run in: it runs `cargo build --release` (plugin, emitter, and validator in one step), renders the conformance project with this repository's own emitter, drives the suite, and reports rule by rule. It never touches an install or a registry. Conformance needs no wasm toolchain, because the emitter synthesizes its module with `wasm-encoder` rather than compiling a fixture crate.

Run the unit tests:

```sh
cargo test
```

The wire types come from `afz-protocol`, a git dependency on the engine repository, so this repository needs no `docs/` checkout to compile.

## afz-harness.json

| key | meaning |
|---|---|
| `install` | argv that fetch dependencies; run only by `afz install-harness`, never by `afz validate-harness`. This harness has none: cargo resolves dependencies during `build` |
| `build` | argv that compile the plugin, the emitter, and the validator; run by both commands, after `install`, in order, in this root; a failure names the stage, the program, and the status |
| `command` | the plugin argv; an element that names an existing file is resolved against this root. Only the host binary is named: the module is the scan target |
| `conformance.emitter` | a program run as `<argv…> --out <dir>` that renders the whole conformance scan project — module, `autofuzz.json`, and `afz-conformance-manifest.json` — into the directory it is given; that rendered project is the only project validation ever drives |
| `conformance.deadlineMs` | the per-invocation deadline the suite passes, default 1000 |

A harness with no `conformance` block installs only with `--no-validate`, and `afz validate-harness` refuses it by name, because there is nothing to validate against.

## The conformance project

`src/bin/emit.rs` (`afz-wasm-emit`) renders a four-export module with `wasm-encoder` and the manifest that names the export behind each template property. `src/bin/check.rs` (`afz-wasm-check`) is the project's own fixture validator: it loads the rendered module with `wasmtime` and reads it with `wasmparser` — no protocol, no plugin in the path — and the suite requires its verdicts to agree with the plugin's.

## The contract

`docs/harness-authoring.md` in the engine repository, <https://github.com/DustinEwan/autofuzz>, states the plugin contract this harness implements.
