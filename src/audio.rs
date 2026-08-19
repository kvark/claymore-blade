//! Kenney SFX. Web plays `<audio>` after the first click; native stays quiet.

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
