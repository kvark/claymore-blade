use super::*;

pub(super) fn loc_prop(id: &str, kind: &str) -> &'static str {
    match id {
        "doga" => "kenney/prop/well.png",
        "stora" => "kenney/prop/house.png",
        "hanel" => "kenney/prop/church.png",
        "shire" => "kenney/prop/tower.png",
        "paburo" => "kenney/prop/pine.png",
        "lacroa" => "kenney/prop/farm.png",
        "gonal" => "kenney/prop/tent.png",
        "pieta" => "kenney/prop/castle.png",
        "maw" => "kenney/prop/ruins.png",
        "sutafu" => "kenney/prop/tower.png",
        _ => match kind {
            "village" => "kenney/prop/house.png",
            "city" => "kenney/prop/church.png",
            "shrine" => "kenney/prop/tower.png",
            "wild" => "kenney/prop/pine.png",
            "keep" => "kenney/prop/castle.png",
            _ => "kenney/prop/house.png",
        },
    }
}

pub(super) fn prop_size(kind: &str) -> (f32, f32) {
    match kind {
        "city" | "keep" | "office" => (0.055, 0.07),
        "wild" => (0.04, 0.06),
        _ => (0.045, 0.055),
    }
}

pub(super) fn upload_slice<T: Copy>(context: &gpu::Context, name: &str, data: &[T]) -> gpu::Buffer {
    let bytes = (data.len() * mem::size_of::<T>()) as u64;
    let buf = context.create_buffer(gpu::BufferDesc {
        name,
        size: bytes,
        memory: gpu::Memory::Shared,
    });
    unsafe {
        ptr::copy_nonoverlapping(data.as_ptr(), buf.data() as *mut T, data.len());
    }
    context.sync_buffer(buf, gpu::BufferTarget::Data);
    buf
}

pub(super) fn upload_rgba(context: &gpu::Context, name: &str, width: u32, height: u32, px: &[u8]) -> GpuTex {
    let extent = gpu::Extent {
        width,
        height,
        depth: 1,
    };
    let texture = context.create_texture(gpu::TextureDesc {
        name,
        format: gpu::TextureFormat::Rgba8Unorm,
        size: extent,
        dimension: gpu::TextureDimension::D2,
        array_layer_count: 1,
        mip_level_count: 1,
        usage: gpu::TextureUsage::RESOURCE | gpu::TextureUsage::COPY,
        sample_count: 1,
        external: None,
    });
    let view = context.create_texture_view(
        texture,
        gpu::TextureViewDesc {
            name,
            format: gpu::TextureFormat::Rgba8Unorm,
            dimension: gpu::ViewDimension::D2,
            subresources: &Default::default(),
        },
    );
    let upload = context.create_buffer(gpu::BufferDesc {
        name: "staging",
        size: px.len() as u64,
        memory: gpu::Memory::Upload,
    });
    unsafe {
        ptr::copy_nonoverlapping(px.as_ptr(), upload.data(), px.len());
    }
    let mut encoder = context.create_command_encoder(gpu::CommandEncoderDesc {
        name: "upload",
        buffer: None,
        texture: None,
    });
    encoder.start();
    encoder.init_texture(texture);
    {
        let mut transfer = encoder.transfer();
        transfer.copy_buffer_to_texture(
            upload.into(),
            width * 4,
            texture.into(),
            extent,
        );
    }
    let sp = context.submit(&mut encoder);
    #[cfg(target_arch = "wasm32")]
    let _ = sp;
    #[cfg(not(target_arch = "wasm32"))]
    let _ = context.wait_for(&sp, !0);
    context.destroy_command_encoder(&mut encoder);
    context.destroy_buffer(upload);
    GpuTex { texture, view }
}

pub(super) fn make_depth(context: &gpu::Context, size: gpu::Extent) -> (gpu::Texture, gpu::TextureView) {
    let texture = context.create_texture(gpu::TextureDesc {
        name: "depth",
        size,
        format: gpu::TextureFormat::Depth32Float,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: gpu::TextureDimension::D2,
        usage: gpu::TextureUsage::TARGET,
        external: None,
    });
    let view = context.create_texture_view(
        texture,
        gpu::TextureViewDesc {
            name: "depth",
            format: gpu::TextureFormat::Depth32Float,
            dimension: gpu::ViewDimension::D2,
            subresources: &gpu::TextureSubresources::default(),
        },
    );
    (texture, view)
}
