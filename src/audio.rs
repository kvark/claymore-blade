//! Kenney SFX + music loops.
//! Web plays `<audio>` after the first user gesture; native stays quiet for now.

#[cfg(target_arch = "wasm32")]
thread_local! {
    static MUSIC: std::cell::RefCell<Option<web_sys::HtmlAudioElement>> = const { std::cell::RefCell::new(None) };
}

/// One-shot SFX. Names map to `assets/kenney/audio/{name}.ogg`.
pub fn play(name: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        let src = format!("kenney/audio/{name}.ogg");
        if let Ok(el) = web_sys::HtmlAudioElement::new_with_src(&src) {
            el.set_volume(0.52);
            el.set_playback_rate(0.90 + js_sys::Math::random() * 0.20);
            play_quietly(&el);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = name;
    }
}

pub fn click() {
    play("ui-click");
}

pub fn confirm() {
    play("confirm");
}

pub fn error() {
    play("error");
}

/// `play()` returns a promise that rejects on autoplay blocks or a missing /
/// undecodable source; swallow it so the console stays clean.
#[cfg(target_arch = "wasm32")]
fn play_quietly(el: &web_sys::HtmlAudioElement) {
    use wasm_bindgen::closure::Closure;
    if let Ok(promise) = el.play() {
        thread_local! {
            static IGNORE: Closure<dyn FnMut(wasm_bindgen::JsValue)> =
                Closure::new(|_err: wasm_bindgen::JsValue| {});
        }
        IGNORE.with(|ignore| {
            let _ = promise.catch(ignore);
        });
    }
}

/// Bed tracks actually shipped under `assets/kenney/music/` (copied to the
/// web bundle with the rest of `assets/kenney`). None are checked in yet, so
/// every bed is silent; add a name here when its `.ogg` lands.
pub const MUSIC_TRACKS: &[&str] = &[];

/// Start a looping bed track. Names map to `assets/kenney/music/{name}.ogg`.
/// Tracks not in [`MUSIC_TRACKS`] are a silent no-op (no request, no 404).
/// Wanted beds: `sad-descent`, `infinite-descent`, `sad-town`, `game-over`.
pub fn music(name: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        MUSIC.with(|slot| {
            // Stop previous bed if any.
            if let Some(prev) = slot.borrow_mut().take() {
                let _ = prev.pause();
            }
            if !MUSIC_TRACKS.contains(&name) {
                return;
            }
            let src = format!("kenney/music/{name}.ogg");
            if let Ok(el) = web_sys::HtmlAudioElement::new_with_src(&src) {
                el.set_loop(true);
                el.set_volume(0.28);
                play_quietly(&el);
                *slot.borrow_mut() = Some(el);
            }
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = name;
    }
}

pub fn music_stop() {
    #[cfg(target_arch = "wasm32")]
    {
        MUSIC.with(|slot| {
            if let Some(el) = slot.borrow_mut().take() {
                let _ = el.pause();
            }
        });
    }
}

/// Island / world-map bed.
pub fn music_island() {
    music("sad-descent");
}

/// Hunt / combat bed (darker).
pub fn music_hunt() {
    music("infinite-descent");
}

/// Town bed.
pub fn music_town() {
    music("sad-town");
}

/// Defeat / late-beacon sting (loops until stopped).
pub fn music_defeat() {
    music("game-over");
}

#[cfg(test)]
mod tests {
    #[test]
    fn listed_music_tracks_exist() {
        for name in super::MUSIC_TRACKS {
            let path = format!("assets/kenney/music/{name}.ogg");
            assert!(std::path::Path::new(&path).exists(), "{path} missing");
        }
    }

    #[test]
    fn sfx_names_exist() {
        for name in [
            "ui-click", "confirm", "error", "block", "chop", "close", "cloth", "draw", "hit",
            "miss", "raise", "slash", "step",
        ] {
            let path = format!("assets/kenney/audio/{name}.ogg");
            assert!(std::path::Path::new(&path).exists(), "{path} missing");
        }
    }
}
