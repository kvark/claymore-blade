//! Persist / load hunt state.

use super::Persist;

pub(super) fn load_save() -> Option<Persist> {
    let raw = read_save()?;
    serde_json::from_str(&raw).ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn save_path() -> std::path::PathBuf {
    #[cfg(test)]
    {
        thread_local! {
            static PATH: std::path::PathBuf = {
                let id = std::thread::current().id();
                std::env::temp_dir().join(format!("claymore-test-save-{id:?}.json"))
            };
        }
        PATH.with(|p| p.clone())
    }
    #[cfg(not(test))]
    {
        std::path::PathBuf::from("claymore.save.json")
    }
}

pub(super) fn read_save() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window()?;
        let storage = window.local_storage().ok()??;
        storage.get_item("claymore.save.v1").ok()?
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read_to_string(save_path()).ok()
    }
}

pub(super) fn write_save(s: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item("claymore.save.v1", s);
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = std::fs::write(save_path(), s);
    }
}
