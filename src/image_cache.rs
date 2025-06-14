use crate::paths::PATH_IMG_CACHE;
use std::path::{Path, PathBuf};

/// Download an image for the given appid. Tries a local path first and caches
/// the result under `res/images_cache/{appid}.jpg`.
pub fn get_image(appid: &str, local: Option<&Path>) -> Option<PathBuf> {
    let cache_dir = PATH_IMG_CACHE.clone();
    if std::fs::create_dir_all(&cache_dir).is_err() {
        return None;
    }
    let cached = cache_dir.join(format!("{appid}.jpg"));
    if cached.exists() {
        return Some(cached);
    }
    if let Some(path) = local {
        if path.exists() {
            let _ = std::fs::copy(path, &cached);
            return Some(cached);
        }
    }
    let url = format!("https://cdn.cloudflare.steamstatic.com/steam/apps/{appid}/library_hero.jpg");
    if let Ok(resp) = reqwest::blocking::get(url) {
        if resp.status().is_success() {
            if let Ok(bytes) = resp.bytes() {
                if std::fs::write(&cached, &bytes).is_ok() {
                    return Some(cached);
                }
            }
        }
    }
    None
}
