//! Blade-graphics hunt view. Same path on Vulkan/Metal/GLES and WebGL2.

#![allow(irrefutable_let_patterns)]

use crate::catalog::{ENCOUNTERS, ENEMIES, LOCATIONS, WARRIORS};
use crate::combat::{core_hex, current_unit, live_cells, CombatState, Side, Terrain};
use crate::font;
use crate::game::{Game, Mode};
use crate::hex::axial_to_world;
use crate::io::{load_rgba, shader_source};
use crate::iso::{board_size, camera_origin, terrain_height, world_to_iso};
use crate::prism::unit_hex_prism;
use crate::world::clock_label;
use blade_graphics as gpu;
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use std::{mem, ptr};

#[cfg(target_arch = "wasm32")]
type PipeEnc<'a> = gpu::PipelineEncoder<'a>;
#[cfg(not(target_arch = "wasm32"))]
type PipeEnc<'a> = gpu::PipelineEncoder<'a, 'a>;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct HuntGlobals {
    origin_zoom: [f32; 4],
    screen: [f32; 4],
    light_dir: [f32; 4],
}

#[derive(blade_macros::ShaderData)]
struct HuntFrame {
    globals: HuntGlobals,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct HuntLocal {
    world: [f32; 4],
    color: [f32; 4],
}

#[derive(blade_macros::ShaderData)]
struct HuntDraw {
    locals: HuntLocal,
}

#[derive(Clone, Copy, blade_macros::Vertex)]
struct MeshVertex {
    pos: [f32; 3],
    normal: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct FlatGlobals {
    pad: [f32; 4],
}

#[derive(blade_macros::ShaderData)]
struct FlatFrame {
    globals: FlatGlobals,
    sprite_texture: gpu::TextureView,
    sprite_sampler: gpu::Sampler,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct FlatLocal {
    pos_size: [f32; 4],
    uv_rect: [f32; 4],
    tint: [f32; 4],
}

#[derive(blade_macros::ShaderData)]
struct FlatDraw {
    locals: FlatLocal,
}

#[derive(Clone, Copy, blade_macros::Vertex)]
struct QuadVertex {
    pos: [f32; 2],
}

struct GpuTex {
    texture: gpu::Texture,
    view: gpu::TextureView,
}

pub struct Renderer {
    hunt: gpu::RenderPipeline,
    flat: gpu::RenderPipeline,
    prism: gpu::Buffer,
    prism_count: u32,
    quad: gpu::Buffer,
    sampler: gpu::Sampler,
    white: GpuTex,
    font: GpuTex,
    images: HashMap<String, GpuTex>,
    depth: gpu::Texture,
    depth_view: gpu::TextureView,
    screen: gpu::Extent,
    format: gpu::TextureFormat,
}

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
            white,
            font,
            images,
            depth,
            depth_view,
            screen,
            format,
        }
    }

    pub fn resize(&mut self, context: &gpu::Context, screen: gpu::Extent) {
        if screen == self.screen {
            return;
        }
        context.destroy_texture_view(self.depth_view);
        context.destroy_texture(self.depth);
        let (d, v) = make_depth(context, screen);
        self.depth = d;
        self.depth_view = v;
        self.screen = screen;
        let _ = self.format;
    }

    pub fn depth_texture(&self) -> gpu::Texture {
        self.depth
    }

    pub fn is_zero_screen(&self) -> bool {
        self.screen.width == 0 || self.screen.height == 0
    }

    pub fn render(
        &mut self,
        encoder: &mut gpu::CommandEncoder,
        target: gpu::TextureView,
        game: &Game,
    ) {
        let w = self.screen.width.max(1) as f32;
        let h = self.screen.height.max(1) as f32;
        if let mut pass = encoder.render(
            "hunt",
            gpu::RenderTargetSet {
                colors: &[gpu::RenderTarget {
                    view: target,
                    init_op: gpu::InitOp::Clear(gpu::TextureColor::OpaqueBlack),
                    finish_op: gpu::FinishOp::Store,
                }],
                depth_stencil: Some(gpu::RenderTarget {
                    view: self.depth_view,
                    init_op: gpu::InitOp::Clear(gpu::TextureColor::White),
                    finish_op: gpu::FinishOp::Discard,
                }),
            },
        ) {
            if game.mode == Mode::Combat {
                if let Some(combat) = game.combat.as_ref() {
                    self.draw_board(&mut pass, combat, game, w, h);
                }
            }
        }
        if let mut pass = encoder.render(
            "flat",
            gpu::RenderTargetSet {
                colors: &[gpu::RenderTarget {
                    view: target,
                    init_op: if game.mode == Mode::Combat {
                        gpu::InitOp::Load
                    } else {
                        gpu::InitOp::Clear(gpu::TextureColor::OpaqueBlack)
                    },
                    finish_op: gpu::FinishOp::Store,
                }],
                depth_stencil: None,
            },
        ) {
            match game.mode {
                Mode::Title => self.draw_title(&mut pass, game, w, h),
                Mode::Intro => self.draw_intro(&mut pass, game, w, h),
                Mode::World => self.draw_world(&mut pass, game, w, h),
                Mode::Town => self.draw_town(&mut pass, game, w, h),
                Mode::Combat => self.draw_combat_overlay(&mut pass, game, w, h),
                Mode::Result => self.draw_result(&mut pass, game, w, h),
                Mode::Codex => self.draw_codex(&mut pass, game, w, h),
            }
        }
    }

    pub fn destroy(&mut self, context: &gpu::Context) {
        context.destroy_texture_view(self.depth_view);
        context.destroy_texture(self.depth);
        context.destroy_texture_view(self.white.view);
        context.destroy_texture(self.white.texture);
        context.destroy_texture_view(self.font.view);
        context.destroy_texture(self.font.texture);
        for t in self.images.values() {
            context.destroy_texture_view(t.view);
            context.destroy_texture(t.texture);
        }
        context.destroy_sampler(self.sampler);
        context.destroy_buffer(self.prism);
        context.destroy_buffer(self.quad);
        context.destroy_render_pipeline(&mut self.hunt);
        context.destroy_render_pipeline(&mut self.flat);
    }

    fn draw_board(
        &self,
        pass: &mut gpu::RenderCommandEncoder,
        combat: &CombatState,
        game: &Game,
        w: f32,
        h: f32,
    ) {
        let size = board_size(combat.cols, combat.rows, w, h);
        let (ox, oy) = camera_origin(
            combat.cols,
            combat.rows,
            size,
            w,
            h,
            game.ui.pan,
            game.ui.zoom,
        );
        let mut rc = pass.with(&self.hunt);
        rc.bind(
            0,
            &HuntFrame {
                globals: HuntGlobals {
                    origin_zoom: [ox, oy, game.ui.zoom, 0.0],
                    screen: [w, h, 0.0, 0.0],
                    light_dir: [0.35, 0.82, 0.42, 0.0],
                },
            },
        );
        rc.bind_vertex(0, self.prism.into());
        let preview = game.preview_zone();
        for (hex, terrain) in &combat.terrain {
            let height = terrain_height(*terrain, size);
            let (x, z) = axial_to_world(*hex, size);
            let mut color = match terrain {
                Terrain::Water => [0.18, 0.24, 0.27],
                Terrain::Mud => [0.30, 0.22, 0.16],
                Terrain::Ruin => [0.30, 0.29, 0.28],
                Terrain::Grass => [0.19, 0.20, 0.16],
            };
            if preview.iter().any(|h| h.q == hex.q && h.r == hex.r) {
                color = [0.42, 0.36, 0.22];
            }
            if game.ui.hover == Some(*hex) {
                color = [0.55, 0.48, 0.28];
            }
            rc.bind(
                1,
                &HuntDraw {
                    locals: HuntLocal {
                        world: [x, 0.0, z, size * 0.92],
                        color: [color[0], color[1], color[2], height.max(0.04)],
                    },
                },
            );
            rc.draw(0, self.prism_count, 0, 1);
        }
        for u in &combat.units {
            if u.dead {
                continue;
            }
            for cell in live_cells(u) {
                let (x, z) = axial_to_world(cell, size);
                let hgt = size * if u.side == Side::Enemy { 0.55 } else { 0.38 };
                rc.bind(
                    1,
                    &HuntDraw {
                        locals: HuntLocal {
                            world: [x, 0.0, z, size * 0.55],
                            color: [u.color[0], u.color[1], u.color[2], hgt],
                        },
                    },
                );
                rc.draw(0, self.prism_count, 0, 1);
            }
        }
    }

    fn tex(&self, rel: &str) -> gpu::TextureView {
        self.images
            .get(rel)
            .map(|t| t.view)
            .unwrap_or(self.white.view)
    }

    fn draw_title(&self, pass: &mut gpu::RenderCommandEncoder, game: &Game, _w: f32, _h: f32) {
        let mut rc = pass.with(&self.flat);
        self.blit(
            &mut rc,
            self.tex("art/title.jpg"),
            [0.0, 0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0, 1.0],
        );
        self.rect(&mut rc, [0.0, 0.0, 1.0, 1.0], [0.04, 0.03, 0.02, 0.45]);
        self.text(&mut rc, "CLAYMORE", 0.08, 0.12, 0.028);
        self.text(&mut rc, "NO. 47", 0.08, 0.20, 0.016);
        self.rect(&mut rc, [0.08, 0.72, 0.34, 0.09], [0.12, 0.10, 0.08, 0.92]);
        self.text(&mut rc, "NEW HUNT", 0.12, 0.745, 0.018);
        if game.has_save {
            self.rect(&mut rc, [0.08, 0.84, 0.34, 0.09], [0.10, 0.09, 0.07, 0.92]);
            self.text(&mut rc, "CONTINUE", 0.12, 0.865, 0.018);
        }
    }

    fn draw_intro(&self, pass: &mut gpu::RenderCommandEncoder, game: &Game, _w: f32, _h: f32) {
        let mut rc = pass.with(&self.flat);
        self.blit(
            &mut rc,
            self.tex("art/title.jpg"),
            [0.0, 0.0, 1.0, 1.0],
            [0.4, 0.4, 0.4, 1.0],
        );
        self.rect(&mut rc, [0.0, 0.0, 1.0, 1.0], [0.03, 0.02, 0.02, 0.55]);
        let mut y = 0.22;
        for line in game.intro_text().lines() {
            self.text(&mut rc, line, 0.10, y, 0.014);
            y += 0.045;
        }
        self.text(&mut rc, "CLICK TO WALK THE ISLAND", 0.10, 0.86, 0.014);
    }

    fn draw_world(&self, pass: &mut gpu::RenderCommandEncoder, game: &Game, _w: f32, _h: f32) {
        let mut rc = pass.with(&self.flat);
        self.blit(
            &mut rc,
            self.tex("art/world-map.jpg"),
            [0.0, 0.0, 1.0, 1.0],
            [1.0; 4],
        );
        for loc in crate::catalog::LOCATIONS {
            let st = game.world.locations.get(loc.id).map(|s| s.status);
            let tint = match st {
                Some(crate::world::WorldStatus::Beacon) => [0.92, 0.55, 0.18, 1.0],
                Some(crate::world::WorldStatus::Cleared) => [0.35, 0.55, 0.32, 1.0],
                Some(crate::world::WorldStatus::Locked) => [0.2, 0.2, 0.2, 0.8],
                Some(crate::world::WorldStatus::Dead) => [0.5, 0.12, 0.1, 1.0],
                _ => [0.75, 0.7, 0.55, 1.0],
            };
            self.rect(&mut rc, [loc.x - 0.008, loc.y - 0.012, 0.016, 0.024], tint);
        }
        self.rect(
            &mut rc,
            [
                game.world.party_x - 0.01,
                game.world.party_y - 0.016,
                0.02,
                0.032,
            ],
            [0.92, 0.9, 0.82, 1.0],
        );
        self.rect(&mut rc, [0.0, 0.0, 1.0, 0.07], [0.05, 0.04, 0.03, 0.85]);
        self.text(
            &mut rc,
            &format!(
                "CLARE  NO.{}   {}   KARMA {}",
                game.world.rank,
                clock_label(game.world.hours),
                game.world.karma
            ),
            0.03,
            0.02,
            0.014,
        );
        self.text(
            &mut rc,
            "WASD MOVE   CLICK A TOWN   CODEX >",
            0.03,
            0.94,
            0.012,
        );
    }

    fn draw_town(&self, pass: &mut gpu::RenderCommandEncoder, game: &Game, _w: f32, _h: f32) {
        let mut rc = pass.with(&self.flat);
        let id = game
            .world
            .last_town
            .clone()
            .unwrap_or_else(|| "doga".into());
        let loc = crate::catalog::location(&id);
        let art = loc.map(|l| l.art).unwrap_or("art/tavern.jpg");
        self.blit(&mut rc, self.tex(art), [0.0, 0.0, 1.0, 1.0], [1.0; 4]);
        self.rect(&mut rc, [0.0, 0.0, 1.0, 1.0], [0.04, 0.03, 0.02, 0.45]);
        if let Some(loc) = loc {
            self.text(&mut rc, loc.name, 0.08, 0.10, 0.024);
            self.text(&mut rc, loc.region, 0.08, 0.16, 0.014);
            self.text(&mut rc, loc.blurb, 0.08, 0.24, 0.012);
        }
        self.rect(&mut rc, [0.08, 0.78, 0.22, 0.09], [0.14, 0.10, 0.06, 0.92]);
        self.text(&mut rc, "HUNT", 0.12, 0.805, 0.016);
        self.rect(&mut rc, [0.36, 0.78, 0.20, 0.09], [0.10, 0.10, 0.10, 0.92]);
        self.text(&mut rc, "REST", 0.40, 0.805, 0.016);
        self.rect(&mut rc, [0.60, 0.78, 0.20, 0.09], [0.10, 0.10, 0.10, 0.92]);
        self.text(&mut rc, "LEAVE", 0.64, 0.805, 0.016);
    }

    fn draw_combat_overlay(
        &self,
        pass: &mut gpu::RenderCommandEncoder,
        game: &Game,
        w: f32,
        h: f32,
    ) {
        let Some(combat) = game.combat.as_ref() else {
            return;
        };
        let mut rc = pass.with(&self.flat);
        let size = board_size(combat.cols, combat.rows, w, h);
        let (ox, oy) = camera_origin(
            combat.cols,
            combat.rows,
            size,
            w,
            h,
            game.ui.pan,
            game.ui.zoom,
        );
        for u in &combat.units {
            if u.dead {
                continue;
            }
            let c = core_hex(u);
            let (wx, wz) = axial_to_world(c, size);
            let (sx, sy) = world_to_iso(wx, size * 0.4, wz);
            let px = (ox + sx * game.ui.zoom) / w - 0.04;
            let py = (oy + sy * game.ui.zoom) / h - 0.12;
            let spr = if self.images.contains_key(&u.sprite) {
                self.tex(&u.sprite)
            } else {
                self.white.view
            };
            self.blit(&mut rc, spr, [px, py, 0.08, 0.16], [1.0, 1.0, 1.0, 1.0]);
        }
        self.rect(&mut rc, [0.0, 0.86, 1.0, 0.14], [0.05, 0.04, 0.03, 0.88]);
        if let Some(u) = current_unit(combat) {
            self.blit(
                &mut rc,
                self.tex(&u.portrait),
                [0.01, 0.87, 0.07, 0.12],
                [1.0; 4],
            );
            self.text(
                &mut rc,
                &format!("{}  AP {}  TRANS {}", u.name, u.ap, u.trans),
                0.10,
                0.88,
                0.012,
            );
            self.text(
                &mut rc,
                &format!("HP {}  YOKI {}", u.hp, u.yoki),
                0.10,
                0.92,
                0.012,
            );
        }
        self.text(&mut rc, "WAIT", 0.02, 0.96, 0.012);
        self.text(&mut rc, "RAISE", 0.16, 0.96, 0.012);
        self.text(&mut rc, "GUARD", 0.30, 0.96, 0.012);
        self.text(&mut rc, "CUT", 0.46, 0.96, 0.012);
        if let Some(line) = combat.log.first() {
            self.text(&mut rc, &line.text, 0.58, 0.88, 0.011);
        }
        self.text(&mut rc, combat.title.as_str(), 0.02, 0.02, 0.014);
    }

    fn draw_result(&self, pass: &mut gpu::RenderCommandEncoder, game: &Game, _w: f32, _h: f32) {
        let mut rc = pass.with(&self.flat);
        self.rect(&mut rc, [0.0, 0.0, 1.0, 1.0], [0.05, 0.04, 0.03, 1.0]);
        self.text(&mut rc, &game.result_title, 0.12, 0.36, 0.022);
        self.text(&mut rc, &game.result_body, 0.12, 0.46, 0.014);
        self.text(&mut rc, "CLICK TO RETURN", 0.12, 0.78, 0.014);
    }

    fn draw_codex(&self, pass: &mut gpu::RenderCommandEncoder, _game: &Game, _w: f32, _h: f32) {
        let mut rc = pass.with(&self.flat);
        self.rect(&mut rc, [0.0, 0.0, 1.0, 1.0], [0.05, 0.04, 0.03, 1.0]);
        self.text(&mut rc, "CODEX", 0.08, 0.08, 0.022);
        self.text(
            &mut rc,
            "CLARE  MIRIA  HELEN  DENEVE  OPHELIA",
            0.08,
            0.20,
            0.014,
        );
        self.text(&mut rc, "CLICK TO CLOSE", 0.08, 0.88, 0.014);
    }

    fn blit(&self, rc: &mut PipeEnc<'_>, view: gpu::TextureView, pos: [f32; 4], tint: [f32; 4]) {
        rc.bind(
            0,
            &FlatFrame {
                globals: FlatGlobals { pad: [0.0; 4] },
                sprite_texture: view,
                sprite_sampler: self.sampler,
            },
        );
        rc.bind(
            1,
            &FlatDraw {
                locals: FlatLocal {
                    pos_size: pos,
                    uv_rect: [0.0, 0.0, 1.0, 1.0],
                    tint,
                },
            },
        );
        rc.bind_vertex(0, self.quad.into());
        rc.draw(0, 4, 0, 1);
    }

    fn rect(&self, rc: &mut PipeEnc<'_>, pos: [f32; 4], tint: [f32; 4]) {
        self.blit(rc, self.white.view, pos, tint);
    }

    fn text(&self, rc: &mut PipeEnc<'_>, s: &str, x: f32, y: f32, glyph_h: f32) {
        let glyph_w = glyph_h * 0.7;
        rc.bind(
            0,
            &FlatFrame {
                globals: FlatGlobals { pad: [0.0; 4] },
                sprite_texture: self.font.view,
                sprite_sampler: self.sampler,
            },
        );
        let mut cx = x;
        for ch in s.chars() {
            if ch == '\n' {
                continue;
            }
            let (u, v, du, dv) = font::glyph_uv(if ch == '·' {
                '-'
            } else {
                ch.to_ascii_uppercase()
            });
            rc.bind(
                1,
                &FlatDraw {
                    locals: FlatLocal {
                        pos_size: [cx, y, glyph_w, glyph_h],
                        uv_rect: [u, v, du, dv],
                        tint: [0.92, 0.88, 0.78, 1.0],
                    },
                },
            );
            rc.bind_vertex(0, self.quad.into());
            rc.draw(0, 4, 0, 1);
            cx += glyph_w * 0.85;
        }
    }
}

fn upload_slice<T: Copy>(context: &gpu::Context, name: &str, data: &[T]) -> gpu::Buffer {
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

fn upload_rgba(context: &gpu::Context, name: &str, width: u32, height: u32, px: &[u8]) -> GpuTex {
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
    context.sync_buffer(upload, gpu::BufferTarget::Data);
    let mut encoder = context.create_command_encoder(gpu::CommandEncoderDesc {
        name: "tex-upload",
        buffer_count: 1,
        manual_barriers: false,
    });
    encoder.start();
    encoder.init_texture(texture);
    if let mut transfer = encoder.transfer("tex") {
        transfer.copy_buffer_to_texture(upload.into(), width * 4, texture.into(), extent);
    }
    let sp = context.submit(&mut encoder);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = context.wait_for(&sp, !0);
    #[cfg(target_arch = "wasm32")]
    let _ = sp;
    context.destroy_command_encoder(&mut encoder);
    context.destroy_buffer(upload);
    GpuTex { texture, view }
}

fn make_depth(context: &gpu::Context, size: gpu::Extent) -> (gpu::Texture, gpu::TextureView) {
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
