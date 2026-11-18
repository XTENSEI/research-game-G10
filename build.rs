//! Build glue for the browser target.
//!
//! The `audio` feature pulls in `quad-snd`, whose wasm backend declares its
//! `audio_*` entry points in a bare `extern "C"` block with no
//! `#[link(wasm_import_module = ...)]`. Those are satisfied at runtime by
//! `web/macroquad_audio.js`, but rust-lld refuses to link a wasm module with
//! undefined symbols, so the import list has to be declared permissive.
//!
//! `cargo:rustc-link-arg` (no target suffix) is scoped to this package's own
//! binaries, so it cannot leak into dependencies.

fn main() {
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if arch == "wasm32" {
        println!("cargo:rustc-link-arg=--allow-undefined");
    }
}
