use llvm_ir::Module;
use std::path::{Path, PathBuf};

fn init_logging() {
    let _ = env_logger::builder().is_test(true).try_init();
}

const BC_DIR: &str = "tests/basic_bc/";

fn llvm_bc_dir() -> PathBuf {
    if cfg!(feature = "llvm-9") {
        Path::new(BC_DIR).join("llvm9")
    } else if cfg!(feature = "llvm-10") {
        Path::new(BC_DIR).join("llvm10")
    } else if cfg!(feature = "llvm-11") {
        Path::new(BC_DIR).join("llvm11")
    } else if cfg!(feature = "llvm-12") {
        Path::new(BC_DIR).join("llvm12")
    } else if cfg!(feature = "llvm-13") {
        Path::new(BC_DIR).join("llvm13")
    } else if cfg!(feature = "llvm-14") {
        Path::new(BC_DIR).join("llvm14")
    } else if cfg!(feature = "llvm-15") {
        Path::new(BC_DIR).join("llvm15")
    } else if cfg!(feature = "llvm-16") {
        Path::new(BC_DIR).join("llvm16")
    } else if cfg!(feature = "llvm-17") {
        Path::new(BC_DIR).join("llvm17")
    } else if cfg!(feature = "llvm-18") {
        Path::new(BC_DIR).join("llvm18")
    } else if cfg!(feature = "llvm-19") {
        Path::new(BC_DIR).join("llvm19")
    } else {
        unimplemented!("new llvm version?")
    }
}

/// Parse a `.bc` file, display it as `.ll` text, then re-parse the generated
/// text with LLVM to verify it produces valid IR.
fn roundtrip_bc(filename: &str) {
    init_logging();
    let path = llvm_bc_dir().join(filename);
    let module = Module::from_bc_path(&path).expect("Failed to parse bc");
    let ll_text = module.to_string();

    // Re-parse the generated text through LLVM's IR parser to ensure validity
    let reparsed = Module::from_ir_str(&ll_text);
    match reparsed {
        Ok(_) => {},
        Err(e) => {
            // Print the generated IR for debugging
            eprintln!("Generated IR:\n{}", ll_text);
            panic!(
                "Roundtrip failed for {}: generated IR did not re-parse: {}",
                filename, e
            );
        },
    }
}

#[test]
fn display_hello() {
    roundtrip_bc("hello.bc");
}

#[test]
fn display_loop() {
    roundtrip_bc("loop.bc");
}

#[test]
fn display_switch() {
    roundtrip_bc("switch.bc");
}

#[test]
fn display_variables() {
    roundtrip_bc("variables.bc");
}

#[test]
fn display_linkedlist() {
    roundtrip_bc("linkedlist.bc");
}

#[test]
fn display_module_to_string() {
    init_logging();
    let path = llvm_bc_dir().join("hello.bc");
    let module = Module::from_bc_path(&path).expect("Failed to parse bc");
    let text = module.to_string();

    // Basic structural checks
    assert!(text.contains("source_filename"));
    assert!(text.contains("target datalayout"));
    assert!(text.contains("define"));
    assert!(text.contains("ret i32"));
}
