use super::*;
use super::util::{upload_slice, upload_rgba, make_depth};

impl Renderer {
    pub fn new(context: &gpu::Context, screen: gpu::Extent, format: gpu::TextureFormat) -> Self {
        let hunt_shader = context.create_shader(gpu::ShaderDesc {
            source: &shader_source("hunt.wgsl"),
            naga_module: None,
        });
        let flat_shader = context.create_shader(gpu::ShaderDesc {
            source: &shader_source("flat.wgsl"),
            naga_module: None,
        });
        let hunt_layout = <HuntFrame as gpu::ShaderData>::layout();
        let hunt_draw_layout = <HuntDraw as gpu::ShaderData>::layout();
        let hunt = context.create_render_pipeline(gpu::RenderPipelineDesc {
            name: "hunt",
            data_layouts: &[&hunt_layout, &hunt_draw_layout],
            vertex: hunt_shader.at("vs_main"),
            vertex_fetches: &[gpu::VertexFetchState {
                layout: &<MeshVertex as gpu::Vertex>::layout(),
                instanced: false,
            }],
            primitive: gpu::PrimitiveState {
                topology: gpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: Some(gpu::DepthStencilState {
                format: gpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: gpu::CompareFunction::Less,
                stencil: gpu::StencilState::default(),
                bias: gpu::DepthBiasState::default(),
            }),
            fragment: Some(hunt_shader.at("fs_main")),
            color_targets: &[format.into()],
            multisample_state: gpu::MultisampleState::default(),
        });
        let flat_layout = <FlatFrame as gpu::ShaderData>::layout();
        let flat_draw_layout = <FlatDraw as gpu::ShaderData>::layout();
        let flat = context.create_render_pipeline(gpu::RenderPipelineDesc {
            name: "flat",
            data_layouts: &[&flat_layout, &flat_draw_layout],
            vertex: flat_shader.at("vs_main"),
            vertex_fetches: &[gpu::VertexFetchState {
                layout: &<QuadVertex as gpu::Vertex>::layout(),
                instanced: false,
            }],
            primitive: gpu::PrimitiveState {
                topology: gpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            fragment: Some(flat_shader.at("fs_main")),
            color_targets: &[gpu::ColorTargetState {
                format,
                blend: Some(gpu::BlendState::ALPHA_BLENDING),
                write_mask: gpu::ColorWrites::default(),
            }],
            multisample_state: gpu::MultisampleState::default(),
        });

        let mesh = unit_hex_prism([1.0; 4]);
        let mut verts: Vec<MeshVertex> = Vec::new();
        for tri in mesh.indices.chunks(3) {
            for &i in tri {
                let v = mesh.vertices[i as usize];
                verts.push(MeshVertex {
                    pos: v.position,
                    normal: v.normal,
                });
            }
        }
        let prism = upload_slice(context, "prism", &verts);
        let quad_data = [
            QuadVertex { pos: [0.0, 0.0] },
            QuadVertex { pos: [1.0, 0.0] },
            QuadVertex { pos: [0.0, 1.0] },
            QuadVertex { pos: [1.0, 1.0] },
        ];
        let quad = upload_slice(context, "quad", &quad_data);
        let sampler = context.create_sampler(gpu::SamplerDesc {
            name: "linear",
            mag_filter: gpu::FilterMode::Linear,
            min_filter: gpu::FilterMode::Linear,
            ..Default::default()
        });
        let pixel = context.create_sampler(gpu::SamplerDesc {
            name: "nearest",
            mag_filter: gpu::FilterMode::Nearest,
            min_filter: gpu::FilterMode::Nearest,
            ..Default::default()
        });
        let white = upload_rgba(context, "white", 1, 1, &[255, 255, 255, 255]);
        let font_px = font::atlas_rgba();
        let font = upload_rgba(context, "font", font::ATLAS_W, font::ATLAS_H, &font_px);
        let (depth, depth_view) = make_depth(context, screen);

        let mut images = HashMap::new();
        let mut paths: Vec<&str> = vec!["art/title.jpg", "art/world-map.jpg"];
        for u in WARRIORS.iter().chain(ENEMIES.iter()) {
            paths.push(u.portrait);
            paths.push(u.sprite);
        }
        for loc in LOCATIONS {
            paths.push(loc.art);
        }
        for enc in ENCOUNTERS {
            paths.push(enc.art);
        }
        paths.extend_from_slice(KENNEY);
        paths.sort_unstable();
        paths.dedup();
        for rel in paths {
            match load_rgba(rel) {
                Ok((w, h, px)) => {
                    images.insert(rel.into(), upload_rgba(context, rel, w, h, &px));
                }
                Err(e) => log::warn!("asset {rel}: {e}"),
            }
        }

        Self {
            hunt,
            flat,
            prism,
            prism_count: verts.len() as u32,
            quad,
            sampler,
            pixel,
            white,
            font,
            images,
            depth,
            depth_view,
            screen,
            format,
        }
    }

}
