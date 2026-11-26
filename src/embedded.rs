//! Unified asset loader supporting zero-dependency embedded desktop binaries
//! (with automatic external `assets/` disk override beside the exe) and async
//! HTTP asset loading on wasm32.

use std::borrow::Cow;
use macroquad::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
include!(concat!(env!("OUT_DIR"), "/embedded_assets.rs"));

#[cfg(not(target_arch = "wasm32"))]
fn read_disk_override(rel: &str) -> Option<Vec<u8>> {
    if let Ok(bytes) = std::fs::read(rel) {
        return Some(bytes);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if let Ok(bytes) = std::fs::read(dir.join(rel)) {
                return Some(bytes);
            }
        }
    }
    None
}

pub fn desktop_bytes(path: &str) -> Option<Cow<'static, [u8]>> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(disk) = read_disk_override(path) {
            return Some(Cow::Owned(disk));
        }
        if let Some(emb) = get_asset(path) {
            return Some(Cow::Borrowed(emb));
        }
    }
    let _ = path;
    None
}

pub async fn load_bytes(path: &str) -> Result<Vec<u8>, String> {
    if let Some(bytes) = desktop_bytes(path) {
        return Ok(bytes.into_owned());
    }
    macroquad::file::load_file(path)
        .await
        .map_err(|e| format!("{:?}", e))
}

pub async fn load_tex(path: &str, filter: FilterMode) -> Texture2D {
    let t = if let Some(bytes) = desktop_bytes(path) {
        Texture2D::from_file_with_format(&bytes, Some(ImageFormat::Png))
    } else {
        macroquad::texture::load_texture(path)
            .await
            .unwrap_or_else(|e| panic!("missing art asset {}: {:?}", path, e))
    };
    t.set_filter(filter);
    t
}
