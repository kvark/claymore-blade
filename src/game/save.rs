//! Persist / load hunt state.

use super::Persist;

pub(super) fn load_save() -> Option<Persist> {
    let raw = read_save()?;
    serde_json::from_str(&raw).ok()
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
        std::fs::read_to_string("claymore.save.json").ok()
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
        let _ = std::fs::write("claymore.save.json", s);
    }
}
