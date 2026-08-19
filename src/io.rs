//! Load files from `assets/` on desktop and HTTP next to the page on web.

pub fn read_bytes(rel: &str) -> Result<Vec<u8>, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = format!("assets/{rel}");
        std::fs::read(&path).map_err(|e| format!("{path}: {e}"))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let xhr = web_sys::XmlHttpRequest::new().map_err(|e| format!("{e:?}"))?;
        // Window documents forbid `responseType` on synchronous XHR.
        xhr.override_mime_type("text/plain; charset=x-user-defined")
            .map_err(|e| format!("{e:?}"))?;
        xhr.open_with_async("GET", rel, false)
            .map_err(|e| format!("{e:?}"))?;
        xhr.send().map_err(|e| format!("{e:?}"))?;
        let status = xhr.status().unwrap_or(0);
        if status != 200 {
            return Err(format!("GET {rel} -> {status}"));
        }
        let text = xhr
            .response_text()
            .map_err(|e| format!("{e:?}"))?
            .unwrap_or_default();
        let mut v = Vec::with_capacity(text.len());
        for c in text.chars() {
            let u = c as u32;
            v.push(if u < 256 { u as u8 } else { (u & 0xff) as u8 });
        }
        Ok(v)
    }
}

pub fn load_rgba(rel: &str) -> Result<(u32, u32, Vec<u8>), String> {
    let bytes = read_bytes(rel)?;
    let img = image::load_from_memory(&bytes).map_err(|e| format!("{rel}: {e}"))?;
    let max = if rel.ends_with(".png") { 256 } else { 1024 };
    let img = if img.width() > max || img.height() > max {
        img.resize(max, max, image::imageops::FilterType::Triangle)
    } else {
        img
    };
    let rgba = img.to_rgba8();
    Ok((rgba.width(), rgba.height(), rgba.into_raw()))
}

pub fn shader_source(name: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        match name {
            "hunt.wgsl" => include_str!("../assets/hunt.wgsl").into(),
            "flat.wgsl" => include_str!("../assets/flat.wgsl").into(),
            other => panic!("unknown shader {other}"),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = format!("assets/{name}");
        std::fs::read_to_string(&path).unwrap_or_else(|_| match name {
            "hunt.wgsl" => include_str!("../assets/hunt.wgsl").into(),
            "flat.wgsl" => include_str!("../assets/flat.wgsl").into(),
            other => panic!("unknown shader {other}"),
        })
    }
}
