# autofuzz scan M75XVGESJD

`/home/dusti/repos/autofuzz/targets/wasm-probe` at git:a533d7f
harness wasm 0.1.0 (design-time; coverage, concurrent)

Started 2026-11-11T18:27:50Z, finished 2026-11-11T18:28:15Z — stopped: budget.

Engine 0.1.0 with seed 0x5e3a91c4, config sha256:7905625a3f18b8f360fedc369f99aeb32acdc1fbfd40042d121f5f2090bba326.

## Summary

| | |
|---|---|
| targets discovered | 2 |
| fuzzable | 1 |
| skipped | 1 |
| materialization: constructible | 1 |
| invocations | 670 |
| corpus entries | 2 |
| coverage | 9 edges (100.0%) |
| deadline exceeded | 0 |
| plugin restarts | 0 |
| unattempted invocations | 2330 |
| findings | 3 |

## Skipped targets

| target | reason | blocked by | unlocks |
|---|---|---|---|
| `afz:t:bbfe2d988ffd` | unsupported-signature | - | - |

## Findings (3)

### afz:f:65ea2703 — failed (high)

`afz:t:f65fd124eb3b`: error while executing at wasm backtrace:
    0:   0x1c14 - wasm_probe.wasm!__rustc[3d2f79520d6fa809]::__rust_abort
    1:    0x23b - wasm_probe.wasm!__rustc[3d2f79520d6fa809]::__rust_start_panic
    2:    0x636 - wasm_probe.wasm!__rustc[3d2f79520d6fa809]::rust_panic
    3:    0x61a - wasm_probe.wasm!std[d775520e4a0009e6]::panicking::panic_with_hook
    4:    0x3d9 - wasm_probe.wasm!std[d775520e4a0009e6]::panicking::panic_handler::{closure#0}
    5:    0x373 - wasm_probe.wasm!std[d775520e4a0009e6]::sys::backtrace::__rust_end_short_backtrace::<std[d775520e4a0009e6]::panicking::panic_handler::{closure#0}, !>
    6:   0x2800 - wasm_probe.wasm!__rustc[3d2f79520d6fa809]::rust_begin_unwind
    7:   0x3232 - wasm_probe.wasm!core[37ee0ce0c4cd57d5]::panicking::panic_fmt
    8:   0x3e2d - wasm_probe.wasm!core[37ee0ce0c4cd57d5]::panicking::panic_const::panic_const_mul_overflow
    9:    0x1e5 - wasm_probe.wasm!probe via `wasm-trap`

- Fault class: `unreachable-code-reached`
  - target/wasm32-unknown-unknown/release/wasm_probe.wasm:485 in `probe`
- Input: `inputs/afz-f-65ea2703.json` (value-tree+json)
- Minimized input: not shrunk again, because an earlier scan reported this identity
- Reproducer: none emitted
- Reproduced: no; first seen at invocation 1
- Seen before: this project reported this identity in an earlier scan
- Identity: `afz:i:1977631a649f5a85`

### afz:f:383cd71d — failed (high)

`afz:t:f65fd124eb3b`: error while executing at wasm backtrace:
    0:   0x1c14 - wasm_probe.wasm!__rustc[3d2f79520d6fa809]::__rust_abort
    1:    0x23b - wasm_probe.wasm!__rustc[3d2f79520d6fa809]::__rust_start_panic
    2:    0x636 - wasm_probe.wasm!__rustc[3d2f79520d6fa809]::rust_panic
    3:    0x61a - wasm_probe.wasm!std[d775520e4a0009e6]::panicking::panic_with_hook
    4:    0x3d9 - wasm_probe.wasm!std[d775520e4a0009e6]::panicking::panic_handler::{closure#0}
    5:    0x373 - wasm_probe.wasm!std[d775520e4a0009e6]::sys::backtrace::__rust_end_short_backtrace::<std[d775520e4a0009e6]::panicking::panic_handler::{closure#0}, !>
    6:   0x2800 - wasm_probe.wasm!__rustc[3d2f79520d6fa809]::rust_begin_unwind
    7:   0x3232 - wasm_probe.wasm!core[37ee0ce0c4cd57d5]::panicking::panic_fmt
    8:   0x3e2d - wasm_probe.wasm!core[37ee0ce0c4cd57d5]::panicking::panic_const::panic_const_mul_overflow
    9:    0x1f7 - wasm_probe.wasm!probe via `wasm-trap`

- Fault class: `unreachable-code-reached`
  - target/wasm32-unknown-unknown/release/wasm_probe.wasm:503 in `probe`
- Input: `inputs/afz-f-383cd71d.json` (value-tree+json)
- Minimized input: not shrunk again, because an earlier scan reported this identity
- Reproducer: none emitted
- Reproduced: no; first seen at invocation 6
- Seen before: this project reported this identity in an earlier scan
- Identity: `afz:i:b0b9e1cc4b9154c3`

### afz:f:c87c7a2a — deadline (medium)

`afz:t:f65fd124eb3b`: took 2033 ms against a 2000 ms deadline via ``

- Fault class: `deadline`
- Input: `inputs/afz-f-c87c7a2a.json` (value-tree+json)
- Minimized input: not shrunk again, because an earlier scan reported this identity
- Reproducer: none emitted
- Reproduced: no; first seen at invocation 104
- Seen before: this project reported this identity in an earlier scan
- Identity: `afz:i:70df6bf17355c5f4`

## Artifacts

- Corpus: `corpus/`
- Coverage: `coverage.json`
- Engine log: `engine.log`

