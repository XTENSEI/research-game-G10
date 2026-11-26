//! Build glue for desktop portable asset bundling and browser wasm linking.

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut list: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    list.sort();
    for p in list {
        if p.is_dir() {
            println!("cargo:rerun-if-changed={}", p.display());
            collect_files(root, &p, out);
        } else if p.is_file() {
            let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
            if matches!(ext, "png" | "ogg" | "ttf") {
                if let Ok(rel) = p.strip_prefix(root) {
                    let key = rel.to_string_lossy().replace('\\', "/");
                    println!("cargo:rerun-if-changed={}", p.display());
                    out.push((key, p));
                }
            }
        }
    }
}

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if arch == "wasm32" {
        println!("cargo:rustc-link-arg=--allow-undefined");
        return;
    }

    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut items = Vec::new();
    for sub in ["assets/fonts", "assets/game"] {
        let dir = manifest.join(sub);
        println!("cargo:rerun-if-changed={}", dir.display());
        collect_files(&manifest, &dir, &mut items);
    }
    items.sort_by(|a, b| a.0.cmp(&b.0));

    let dest = out_dir.join("embedded_assets.rs");
    let mut f = fs::File::create(&dest).expect("create embedded_assets.rs");
    writeln!(f, "pub fn get_asset(path: &str) -> Option<&'static [u8]> {{").unwrap();
    writeln!(f, "    match path {{").unwrap();
    for (key, abs_path) in &items {
        let abs = abs_path.to_string_lossy().replace('\\', "/");
        writeln!(f, "        {key:?} => Some(include_bytes!(r#\"{abs}\"#)),").unwrap();
    }
    writeln!(f, "        _ => None,").unwrap();
    writeln!(f, "    }}").unwrap();
    writeln!(f, "}}").unwrap();
}
