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
            let _ = el.play();
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

/// Start a looping bed track. Names map to `assets/kenney/music/{name}.ogg`.
/// Preferred island beds: `sad-descent`, `infinite-descent`, `flowing-rocks`, `retro-mystic`, `sad-town`.
pub fn music(name: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        MUSIC.with(|slot| {
            // Stop previous bed if any.
            if let Some(prev) = slot.borrow_mut().take() {
                let _ = prev.pause();
            }
            let src = format!("kenney/music/{name}.ogg");
            if let Ok(el) = web_sys::HtmlAudioElement::new_with_src(&src) {
                el.set_loop(true);
                el.set_volume(0.28);
                let _ = el.play();
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
