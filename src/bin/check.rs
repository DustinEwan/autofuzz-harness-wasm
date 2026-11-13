//! The conformance project's own fixture validator, for the wasm harness.
//!
//! The protocol judge only ever sees the plugin's statements about the rendered module. This
//! binary is the second judgment: it loads the module with `wasmtime` and reads it with
//! `wasmparser` - no protocol, no plugin, no emitter in the path - and reports the same
//! property ids. The suite requires the two verdict lists to agree.
//!
//! Contract (docs/harness-authoring.md, section 12): stdout is exactly one JSON
//! `TemplateReport` and nothing else; exit 0 even when a property fails, non-zero only when
//! this validator itself could not run; the seven fixture-shaped ids only.
//!
//! Usage: afz-wasm-check <module.wasm>

use wasmparser::{CompositeInnerType, ExternalKind, Parser, Payload, ValType};
use wasmtime::{Config, Engine, Linker, Module, Store, Val};

#[derive(Default)]
struct CheckReport {
    properties: Vec<serde_json::Value>,
}

impl CheckReport {
    fn record(&mut self, id: &str, verdict: &str, detail: impl Into<String>) {
        self.properties.push(serde_json::json!({ "id": id, "verdict": verdict, "detail": detail.into() }));
    }

    fn to_json(&self) -> String {
        serde_json::json!({ "version": 1, "properties": self.properties }).to_string()
    }
}

fn main() {
    let mut args = std::env::args().skip(1).filter(|arg| !arg.starts_with('-'));
    let Some(module_path) = args.next() else {
        eprintln!("usage: afz-wasm-check <module.wasm>");
        std::process::exit(1);
    };
    match judge(&module_path) {
        Ok(report) => println!("{}", report.to_json()),
        // The validator itself could not run: that is the one non-zero exit, and the driver
        // names the stderr tail it reads here.
        Err(reason) => {
            eprintln!("afz-wasm-check: {reason}");
            std::process::exit(1);
        }
    }
}

fn judge(module_path: &str) -> Result<CheckReport, String> {
    let bytes = std::fs::read(module_path).map_err(|e| format!("cannot read {module_path}: {e}"))?;
    let signature = read_signature(&bytes)?;
    let runtime = run_module(module_path, &bytes)?;

    let mut report = CheckReport::default();
    // Template order, the seven fixture-shaped ids.
    if runtime.crash_trapped {
        report.record("crash-unguarded", "pass", "calling the crash export traps inside the module, so the fault frame names the module file");
    } else {
        report.record("crash-unguarded", "fail", "the crash export returned instead of trapping: the crash fixture stopped crashing");
    }
    report.record(
        "guard-refusal",
        "not-applicable",
        "a bare module carries no source-level guard that can refuse: there is nothing here to inspect besides a trap, which is a fault, not a rejection",
    );
    report.record(
        "private-symbol",
        "not-applicable",
        "a module's visibility is its export section: there is no source-level private name in the file for a validator to point at",
    );
    match signature.pair_results {
        Some(2) => report.record("unrepresentable-symbol", "pass", "the pair function type has two results, which one call result cannot carry"),
        Some(n) => report.record("unrepresentable-symbol", "fail", format!("the pair function type has {n} results, not the declared two")),
        None => report.record("unrepresentable-symbol", "fail", "the module exports no `pair` function to inspect"),
    }
    match signature.apply_parameter.as_deref() {
        Some("(ref null func)") => {
            report.record("unconstructible-receiver", "pass", "the apply function takes a funcref parameter, which the host cannot build from a ValueTree")
        }
        Some(other) => {
            report.record("unconstructible-receiver", "fail", format!("the apply parameter is {other}, not the declared (ref null func)"))
        }
        None => report.record("unconstructible-receiver", "fail", "the module exports no `apply` function to inspect"),
    }
    report.record(
        "method-receiver",
        "not-applicable",
        "an export is a free function: the module carries no receiver object whose construction could be checked",
    );
    match (runtime.fuel_if, runtime.fuel_else) {
        (Some(if_executed), Some(else_executed)) if if_executed != else_executed => report.record(
            "coverage-digest-live",
            "pass",
            format!("branch(0) executed {if_executed} instructions and branch(2) executed {else_executed}: the two inputs take different arms of the `if`"),
        ),
        (Some(if_executed), Some(else_executed)) => report.record(
            "coverage-digest-live",
            "fail",
            format!("branch(0) executed {if_executed} instructions and branch(2) executed {else_executed}, the same count: the fixture does not branch on the input"),
        ),
        _ => report.record("coverage-digest-live", "fail", "the branch export could not be executed to compare the two inputs"),
    }
    Ok(report)
}

/// What `wasmparser` reads off the module: the two function types the manifest names.
struct Signature {
    pair_results: Option<usize>,
    apply_parameter: Option<String>,
}

fn read_signature(bytes: &[u8]) -> Result<Signature, String> {
    let mut types: Vec<wasmparser::SubType> = Vec::new();
    let mut function_types: Vec<u32> = Vec::new();
    let mut pair_results = None;
    let mut apply_parameter = None;
    for payload in Parser::new(0).parse_all(bytes) {
        match payload.map_err(|e| format!("wasmparser: {e}"))? {
            Payload::TypeSection(section) => {
                for group in section {
                    let group = group.map_err(|e| format!("wasmparser: {e}"))?;
                    types.extend(group.types().cloned());
                }
            }
            Payload::FunctionSection(section) => {
                for type_index in section {
                    function_types.push(type_index.map_err(|e| format!("wasmparser: {e}"))?);
                }
            }
            Payload::ExportSection(section) => {
                for export in section {
                    let export = export.map_err(|e| format!("wasmparser: {e}"))?;
                    if !matches!(export.kind, ExternalKind::Func) {
                        continue;
                    }
                    let func_type = function_types
                        .get(export.index as usize)
                        .and_then(|ty| types.get(*ty as usize))
                        .map(|sub| &sub.composite_type.inner);
                    let Some(CompositeInnerType::Func(func)) = func_type else { continue };
                    if export.name == "pair" {
                        pair_results = Some(func.results().len());
                    }
                    if export.name == "apply" {
                        apply_parameter = func.params().first().map(display_val_type);
                    }
                }
            }
            _ => {}
        }
    }
    Ok(Signature { pair_results, apply_parameter })
}

/// The wasm text spelling of a value type, in the long form `(ref [null] <heap>)` that the
/// plugin's `format!("{parameter:?}")` (wasmtime's `Display`) produces, so the two strings
/// compare verbatim.
fn display_val_type(ty: &ValType) -> String {
    match ty {
        ValType::I32 => "i32".to_string(),
        ValType::I64 => "i64".to_string(),
        ValType::F32 => "f32".to_string(),
        ValType::F64 => "f64".to_string(),
        ValType::V128 => "v128".to_string(),
        ValType::Ref(reference) => match reference.heap_type() {
            wasmparser::HeapType::Abstract { shared: false, ty: heap } => {
                let null = if reference.is_nullable() { "null " } else { "" };
                let name = match heap {
                    wasmparser::AbstractHeapType::Func => "func",
                    wasmparser::AbstractHeapType::NoFunc => "nofunc",
                    wasmparser::AbstractHeapType::Extern => "extern",
                    wasmparser::AbstractHeapType::NoExtern => "noextern",
                    // No other heap type appears in a rendered fixture; fall back to the
                    // debug spelling so a hand-edited module still gets a name in the detail.
                    _ => return format!("(ref {reference:?})"),
                };
                format!("(ref {null}{name})")
            }
            _ => format!("(ref {reference:?})"),
        },
    }
}

/// What running the module shows: the trap and the two executed-instruction counts.
struct Runtime {
    crash_trapped: bool,
    fuel_if: Option<u64>,
    fuel_else: Option<u64>,
}

fn run_module(path: &str, bytes: &[u8]) -> Result<Runtime, String> {
    let mut config = Config::new();
    config.consume_fuel(true);
    let engine = Engine::new(&config).map_err(|e| format!("wasmtime: {e}"))?;
    let module = Module::new(&engine, bytes).map_err(|e| format!("{path}: {e}"))?;
    let linker = Linker::new(&engine);
    let mut store = Store::new(&engine, ());
    const START_FUEL: u64 = 100_000;

    store.set_fuel(START_FUEL).map_err(|e| format!("wasmtime: {e}"))?;
    let instance = linker.instantiate(&mut store, &module).map_err(|e| format!("{path}: {e}"))?;
    let crash = instance.get_func(&mut store, "crash").ok_or("the rendered module exports no `crash`")?;
    let mut results = [Val::I32(0)];
    let crash_trapped = crash.call(&mut store, &[Val::I32(0)], &mut results).is_err();

    let branch = instance.get_func(&mut store, "branch").ok_or("the rendered module exports no `branch`")?;
    let executed = |store: &mut Store<()>, input: i32| -> Result<Option<u64>, String> {
        store.set_fuel(START_FUEL).map_err(|e| format!("wasmtime: {e}"))?;
        let mut out = [Val::I32(0)];
        branch.call(&mut *store, &[Val::I32(input)], &mut out).map_err(|e| format!("branch({input}): {e}"))?;
        Ok(Some(START_FUEL - store.get_fuel().map_err(|e| format!("wasmtime: {e}"))?))
    };
    let fuel_if = executed(&mut store, 0)?;
    let fuel_else = executed(&mut store, 2)?;
    Ok(Runtime { crash_trapped, fuel_if, fuel_else })
}
