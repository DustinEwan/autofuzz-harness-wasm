//! Render the conformance fixture project for the wasm harness.
//!
//! `afz validate-harness` and `afz install-harness` validate a plugin against the template
//! (`crates/afz-conformance/template/v1.json` in the engine repository): ten semantic
//! properties stated as wire-level expectations. This emitter synthesizes a bare module with
//! `wasm-encoder` - no crate compile, no `wasm32-unknown-unknown` target - writes the
//! `autofuzz.json` that points the plugin at it, and writes
//! `afz-conformance-manifest.json` naming the symbol behind each property. The suite, not
//! this file, decides pass/fail.
//!
//! The module carries exactly four exports, one per property a bare module can realize, and
//! nothing else: no `__`-prefixed plumbing and no counters memory are exported, so discovery
//! sees only the fixture.
//!
//! Usage:
//!   target/release/afz-wasm-emit --out /tmp/conformance-wasm

use std::path::{Path, PathBuf};

use wasm_encoder::{
    BlockType, CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction, Module, TypeSection,
    ValType,
};

/// The rendered module's file name inside the project.
const MODULE: &str = "probe.wasm";

fn render_module() -> Vec<u8> {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    types.ty().function(vec![ValType::I32], vec![ValType::I32]);
    types.ty().function(vec![ValType::I32], vec![ValType::I32, ValType::I32]);
    types.ty().function(vec![ValType::FUNCREF], vec![ValType::I32]);
    module.section(&types);

    let mut funcs = FunctionSection::new();
    for type_index in [0, 0, 1, 2] {
        funcs.function(type_index);
    }
    module.section(&funcs);

    let mut exports = ExportSection::new();
    exports.export("crash", ExportKind::Func, 0);
    exports.export("branch", ExportKind::Func, 1);
    exports.export("pair", ExportKind::Func, 2);
    exports.export("apply", ExportKind::Func, 3);
    module.section(&exports);

    let mut code = CodeSection::new();
    // crash-unguarded: the trap is inside the exported function itself, so the fault frame
    // names this module file.
    let mut crash = Function::new(vec![]);
    crash.instruction(&Instruction::LocalGet(0));
    crash.instruction(&Instruction::Unreachable);
    crash.instruction(&Instruction::End);
    code.function(&crash);
    // coverage-digest-live: input 0 and input 2 take different arms of the `if`, so two
    // invocations must produce different digests. The arms differ in length (one instruction
    // versus three), so the executed-instruction count differs too and the project's own
    // validator can see the branch without a protocol in the path.
    let mut branch = Function::new(vec![]);
    branch.instruction(&Instruction::LocalGet(0));
    branch.instruction(&Instruction::I32Const(2));
    branch.instruction(&Instruction::I32LtS);
    branch.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
    branch.instruction(&Instruction::I32Const(1));
    branch.instruction(&Instruction::Else);
    branch.instruction(&Instruction::I32Const(2));
    branch.instruction(&Instruction::I32Const(0));
    branch.instruction(&Instruction::I32Add);
    branch.instruction(&Instruction::End);
    branch.instruction(&Instruction::End);
    code.function(&branch);
    // unrepresentable-symbol: two results, which one call result cannot carry, so discovery
    // skips the export `unsupported-signature`.
    let mut pair = Function::new(vec![]);
    pair.instruction(&Instruction::LocalGet(0));
    pair.instruction(&Instruction::LocalGet(0));
    pair.instruction(&Instruction::End);
    code.function(&pair);
    // unconstructible-receiver: the parameter is a funcref the host would have to hand the
    // module, so discovery skips the export `unconstructible-parameter` naming `(ref null func)`.
    let mut apply = Function::new(vec![]);
    apply.instruction(&Instruction::I32Const(0));
    apply.instruction(&Instruction::End);
    code.function(&apply);
    module.section(&code);

    module.finish()
}

fn autofuzz_json() -> String {
    serde_json::json!({
        "project": "conformance-wasm",
        "harness": {
            // No `command` and no module argument: the plugin the suite launches is the one
            // `afz-harness.json` names, and the module is the first `.wasm` under `AFZ_ROOT`,
            // which the driver points at this rendered project.
            "name": "wasm",
            "include": ["**/*.wasm"],
        },
        "oracles": {},
        "budget": { "invocations": 2000, "wallClockMs": 60000, "workers": 0, "maxCorpus": 256, "strategy": "guided" },
        "output": { "dir": ".autofuzz", "renderers": ["json"] },
    })
    .to_string()
}

fn number(n: i64) -> serde_json::Value {
    serde_json::json!({ "node": "number", "value": n, "provenance": "user" })
}

fn manifest(check: &Path, module: &Path) -> String {
    serde_json::json!({
        "templateVersion": 1,
        "properties": {
            "crash-unguarded": { "symbol": "crash", "input": [number(0)] },
            "guard-refusal": {
                "notApplicable": "a bare module has no guard that can refuse: the plugin's only refusal channel is `via: \"unmaterializable\"` for a value wasm cannot carry, and every trap is a fault, not a rejection"
            },
            "private-symbol": {
                "notApplicable": "a module's public API is its export section, so a non-exported function can never appear in `skipped` to be named at all"
            },
            "unrepresentable-symbol": { "symbol": "pair" },
            "unconstructible-receiver": { "symbol": "apply", "blockedByType": "(ref null func)" },
            "method-receiver": {
                "notApplicable": "an export is a free function: there is no receiver object to build, so the property has no fixture to realize"
            },
            // 0 takes the `if` arm and 2 takes the `else`: two invocations that answer the
            // same digest mean a dead or never-reset counter.
            "coverage-digest-live": { "symbol": "branch", "inputs": [[number(0)], [number(2)]] },
            "stdout-purity": {
                "notApplicable": "the runner links no host imports and no WASI, so a rendered module cannot write to stdout at all"
            },
            "reproducer-script": { "notApplicable": "the handshake does not declare `reproducer`: a bare module carries no source to render a script from" },
            "deadline-sleep": {
                "notApplicable": "the runner stops an endless export with ticks and answers `ok`, which is not the `deadline-cut` channel this rule names"
            },
        },
        // The project judges its own fixture: the check binary loads the rendered module with
        // wasmtime and reads it with wasmparser, with no protocol and no plugin in the path.
        "validator": [check.display().to_string(), module.display().to_string()],
    })
    .to_string()
}

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut out: Option<PathBuf> = None;
    while let Some(arg) = args.next() {
        if arg == "--out" {
            out = Some(PathBuf::from(args.next().ok_or("usage: afz-wasm-emit --out DIR")?));
        }
    }
    let out = out.ok_or("usage: afz-wasm-emit --out DIR")?;
    std::fs::create_dir_all(&out).map_err(|e| format!("cannot create {}: {e}", out.display()))?;
    let module = out.join(MODULE);
    std::fs::write(&module, render_module()).map_err(|e| format!("cannot write {}: {e}", module.display()))?;
    std::fs::write(out.join("autofuzz.json"), autofuzz_json() + "\n").map_err(|e| e.to_string())?;
    // The fixture validator sits beside this binary in the build directory the install built.
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let check = exe.parent().ok_or("the emitter is not running from a directory")?.join("afz-wasm-check");
    std::fs::write(out.join("afz-conformance-manifest.json"), manifest(&check, &module) + "\n").map_err(|e| e.to_string())?;
    println!("{}", out.display());
    Ok(())
}
