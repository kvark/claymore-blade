//! Blade-graphics hunt view. Same path on Vulkan/Metal/GLES and WebGL2.

#![allow(irrefutable_let_patterns)]

use crate::catalog::{ENCOUNTERS, ENEMIES, LOCATIONS, WARRIORS};
use crate::combat::{core_hex, current_unit, live_cells, CombatState, Side, Terrain};
use crate::font;
use crate::game::{Game, Mode};
use crate::hex::axial_to_world;
use crate::hud;
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

const ASH: [f32; 4] = [0.72, 0.64, 0.52, 1.0];
const BLOOD: [f32; 4] = [0.72, 0.18, 0.14, 1.0];

const KENNEY: &[&str] = &[
    "kenney/ui/button.png",
    "kenney/ui/button-line.png",
    "kenney/ui/button-red.png",
    "kenney/ui/button-brown.png",
    "kenney/ui/button-grey.png",
    "kenney/ui/bar.png",
    "kenney/ui/bar-fill.png",
    "kenney/ui/panel.png",
    "kenney/ui/panel-brown.png",
    "kenney/ui/banner.png",
    "kenney/ui/hex.png",
    "kenney/ui/hex-move.png",
    "kenney/ui/hex-hit.png",
    "kenney/ui/divider.png",
    "kenney/fx/slash.png",
    "kenney/fx/slash2.png",
    "kenney/fx/spark.png",
    "kenney/fx/smoke.png",
    "kenney/fx/magic.png",
    "kenney/fx/circle.png",
    "kenney/fx/star.png",
    "kenney/fx/dust.png",
    "kenney/fx/flame.png",
    "kenney/fx/light.png",
    "kenney/fx/twirl.png",
    "kenney/fx/scorch.png",
    "kenney/prop/house.png",
    "kenney/prop/ruins.png",
    "kenney/prop/church.png",
    "kenney/prop/castle.png",
    "kenney/prop/tower.png",
    "kenney/prop/farm.png",
    "kenney/prop/pine.png",
    "kenney/prop/pine-small.png",
    "kenney/prop/banner.png",
    "kenney/prop/well.png",
    "kenney/prop/brick.png",
    "kenney/prop/rock.png",
    "kenney/prop/tent.png",
    "kenney/prop/roof.png",
    "kenney/iso/chest.png",
    "kenney/iso/barrel.png",
    "kenney/iso/ruin-floor.png",
    "kenney/iso/column.png",
    "kenney/iso/planks.png",
    "kenney/prompt/w.png",
    "kenney/prompt/a.png",
    "kenney/prompt/s.png",
    "kenney/prompt/d.png",
    "kenney/prompt/space.png",
    "kenney/prompt/esc.png",
    "kenney/prompt/1.png",
    "kenney/prompt/2.png",
    "kenney/prompt/3.png",
    "kenney/prompt/mouse.png",
    "kenney/rune/mark.png",
    "kenney/rune/brand.png",
    "kenney/cursor/target.png",
];

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
    pixel: gpu::Sampler,
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
                Mode::Scene => self.draw_scene(&mut pass, game, w, h),
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
        context.destroy_sampler(self.pixel);
        context.destroy_buffer(self.prism);
        context.destroy_buffer(self.quad);
        context.destroy_render_pipeline(&mut self.hunt);
        context.destroy_render_pipeline(&mut self.flat);
    }

    fn pan(game: &Game, w: f32, h: f32) -> [f32; 2] {
        let s = game.fx.shake(w, h);
        [game.ui.pan[0] + s[0], game.ui.pan[1] + s[1]]
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
        let pan = Self::pan(game, w, h);
        let (ox, oy) = camera_origin(combat.cols, combat.rows, size, w, h, pan, game.ui.zoom);
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
        let skill_on = game.ui.selected_skill.is_some();
        for (hex, terrain) in &combat.terrain {
            let height = terrain_height(*terrain, size);
            let (x, z) = axial_to_world(*hex, size);
            let mut color = match terrain {
                Terrain::Water => [0.16, 0.20, 0.22],
                Terrain::Mud => [0.28, 0.20, 0.14],
                Terrain::Ruin => [0.32, 0.30, 0.28],
                Terrain::Grass => [0.18, 0.19, 0.15],
            };
            if preview.iter().any(|h| h.q == hex.q && h.r == hex.r) {
                color = if skill_on {
                    [0.48, 0.18, 0.14]
                } else {
                    [0.36, 0.42, 0.22]
                };
            }
            if game.ui.hover == Some(*hex) {
                color = [0.55, 0.46, 0.26];
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
        self.rect(&mut rc, [0.0, 0.0, 1.0, 1.0], [0.04, 0.03, 0.02, 0.48]);
        self.blit_px(
            &mut rc,
            self.tex("kenney/rune/brand.png"),
            [0.78, 0.08, 0.16, 0.22],
            [0.55, 0.18, 0.16, 0.85],
        );
        self.text(&mut rc, "CLAYMORE", 0.08, 0.12, 0.028);
        self.text(&mut rc, "NO. 47", 0.08, 0.20, 0.016);
        self.text(&mut rc, game.title_flavor(), 0.08, 0.28, 0.012);
        self.kenney_btn(&mut rc, "kenney/ui/button.png", hud::title_new(), "NEW HUNT", false);
        if game.has_save {
            self.kenney_btn(
                &mut rc,
                "kenney/ui/button-grey.png",
                hud::title_continue(),
                "CONTINUE",
                false,
            );
        }
        self.draw_fx(&mut rc, game);
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
        self.blit_px(
            &mut rc,
            self.tex("kenney/ui/panel-brown.png"),
            [0.07, 0.16, 0.72, 0.58],
            [0.55, 0.46, 0.36, 0.55],
        );
        let mut y = 0.22;
        for line in game.intro_text().lines() {
            self.text(&mut rc, line, 0.10, y, 0.014);
            y += 0.045;
        }
        self.prompt(&mut rc, "kenney/prompt/space.png", 0.10, 0.84, 0.05);
        self.text(&mut rc, "WALK THE ISLAND", 0.17, 0.855, 0.014);
        self.draw_fx(&mut rc, game);
    }

    fn draw_world(&self, pass: &mut gpu::RenderCommandEncoder, game: &Game, _w: f32, _h: f32) {
        let mut rc = pass.with(&self.flat);
        self.blit(
            &mut rc,
            self.tex("art/world-map.jpg"),
            [0.0, 0.0, 1.0, 1.0],
            [0.92, 0.88, 0.82, 1.0],
        );
        for loc in crate::catalog::LOCATIONS {
            let st = game.world.locations.get(loc.id).map(|s| s.status);
            let (tint, prop) = match st {
                Some(crate::world::WorldStatus::Beacon) => (
                    [0.92, 0.55, 0.18, 1.0],
                    loc_prop(loc.id, loc.kind),
                ),
                Some(crate::world::WorldStatus::Cleared) => {
                    ([0.38, 0.52, 0.32, 1.0], loc_prop(loc.id, loc.kind))
                }
                Some(crate::world::WorldStatus::Locked) => ([0.22, 0.20, 0.18, 0.75], loc_prop(loc.id, loc.kind)),
                Some(crate::world::WorldStatus::Dead) => ([0.55, 0.12, 0.10, 1.0], "kenney/prop/ruins.png"),
                _ => ([0.70, 0.62, 0.48, 1.0], loc_prop(loc.id, loc.kind)),
            };
            let (pw, ph) = prop_size(loc.kind);
            self.blit_px(
                &mut rc,
                self.tex(prop),
                [loc.x - pw * 0.5, loc.y - ph * 0.78, pw, ph],
                tint,
            );
        }
        let bob = (game.fx.time * 3.4).sin() * 0.006;
        self.blit_px(
            &mut rc,
            self.tex("kenney/prop/banner.png"),
            [
                game.world.party_x - 0.012,
                game.world.party_y - 0.038 + bob,
                0.024,
                0.046,
            ],
            [0.92, 0.88, 0.72, 1.0],
        );
        if self.images.contains_key("sprites/clare.png") {
            self.blit(
                &mut rc,
                self.tex("sprites/clare.png"),
                [
                    game.world.party_x - 0.016,
                    game.world.party_y - 0.05 + bob,
                    0.032,
                    0.058,
                ],
                [1.0, 1.0, 1.0, 1.0],
            );
        }
        self.blit_px(
            &mut rc,
            self.tex("kenney/ui/panel.png"),
            [0.0, 0.0, 1.0, 0.08],
            [0.42, 0.34, 0.26, 0.92],
        );
        self.text(
            &mut rc,
            &format!(
                "CLARE  NO.{}   {}   KARMA {}",
                game.world.rank,
                clock_label(game.world.hours),
                game.world.karma
            ),
            0.03,
            0.025,
            0.014,
        );
        self.prompt(&mut rc, "kenney/prompt/w.png", 0.03, 0.90, 0.04);
        self.prompt(&mut rc, "kenney/prompt/a.png", 0.075, 0.945, 0.04);
        self.prompt(&mut rc, "kenney/prompt/s.png", 0.12, 0.90, 0.04);
        self.prompt(&mut rc, "kenney/prompt/d.png", 0.165, 0.945, 0.04);
        self.text(&mut rc, "WALK", 0.22, 0.95, 0.012);
        self.kenney_btn(
            &mut rc,
            "kenney/ui/button-brown.png",
            hud::world_codex(),
            "CODEX",
            false,
        );
        self.draw_fx(&mut rc, game);
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
        self.rect(&mut rc, [0.0, 0.0, 1.0, 1.0], [0.04, 0.03, 0.02, 0.42]);
        self.blit_px(
            &mut rc,
            self.tex("kenney/ui/panel-brown.png"),
            [0.05, 0.07, 0.55, 0.28],
            [0.48, 0.38, 0.28, 0.78],
        );
        if let Some(loc) = loc {
            self.text(&mut rc, loc.name, 0.08, 0.10, 0.024);
            self.text(&mut rc, loc.region, 0.08, 0.16, 0.014);
            self.text(&mut rc, loc.blurb, 0.08, 0.24, 0.012);
        }
        self.kenney_btn(&mut rc, "kenney/ui/button.png", hud::town_hunt(), "HUNT", false);
        self.kenney_btn(&mut rc, "kenney/ui/button-grey.png", hud::town_rest(), "REST", false);
        self.kenney_btn(&mut rc, "kenney/ui/button-brown.png", hud::town_leave(), "LEAVE", false);
        self.draw_fx(&mut rc, game);
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
        let pan = Self::pan(game, w, h);
        let (ox, oy) = camera_origin(
            combat.cols,
            combat.rows,
            size,
            w,
            h,
            pan,
            game.ui.zoom,
        );
        let preview = game.preview_zone();
        let skill_on = game.ui.selected_skill.is_some();
        let overlay = if skill_on {
            "kenney/ui/hex-hit.png"
        } else {
            "kenney/ui/hex-move.png"
        };
        let otint = if skill_on {
            [0.85, 0.22, 0.16, 0.72]
        } else {
            [0.45, 0.62, 0.28, 0.65]
        };
        for hex in &preview {
            let (wx, wz) = axial_to_world(*hex, size);
            let (sx, sy) = world_to_iso(wx, size * 0.16, wz);
            let px = (ox + sx * game.ui.zoom) / w;
            let py = (oy + sy * game.ui.zoom) / h;
            let hw = (size * game.ui.zoom / w) * 1.2;
            let hh = (size * game.ui.zoom / h) * 0.7;
            self.blit_px(
                &mut rc,
                self.tex(overlay),
                [px - hw * 0.5, py - hh * 0.45, hw, hh],
                otint,
            );
        }
        for (hex, terrain) in &combat.terrain {
            if *terrain != Terrain::Ruin {
                continue;
            }
            let (wx, wz) = axial_to_world(*hex, size);
            let (sx, sy) = world_to_iso(wx, size * 0.42, wz);
            let px = (ox + sx * game.ui.zoom) / w;
            let py = (oy + sy * game.ui.zoom) / h;
            self.blit_px(
                &mut rc,
                self.tex("kenney/iso/column.png"),
                [px - 0.012, py - 0.055, 0.024, 0.07],
                [0.55, 0.5, 0.44, 0.9],
            );
        }
        for u in &combat.units {
            if u.dead {
                continue;
            }
            let c = core_hex(u);
            let (wx, wz) = axial_to_world(c, size);
            let (sx, sy) = world_to_iso(wx, size * 0.4, wz);
            let hash = (u.id.bytes().fold(0u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32))
                as f32)
                * 0.01;
            let bob = (game.fx.time * 3.1 + hash).sin() * 0.007;
            let px = (ox + sx * game.ui.zoom) / w - 0.04;
            let py = (oy + sy * game.ui.zoom) / h - 0.12 + bob;
            let spr = if self.images.contains_key(&u.sprite) {
                self.tex(&u.sprite)
            } else {
                self.white.view
            };
            let squash = 1.0 + (game.fx.time * 3.1 + hash).sin() * 0.03;
            self.blit(
                &mut rc,
                spr,
                [px, py, 0.08 * (2.0 - squash), 0.16 * squash],
                [1.0, 1.0, 1.0, 1.0],
            );
            let frac = if u.max_hp > 0 {
                u.hp as f32 / u.max_hp as f32
            } else {
                0.0
            };
            self.bar(&mut rc, px, py - 0.014, 0.08, 0.01, frac, BLOOD);
            if u.trans > 0 {
                self.bar(
                    &mut rc,
                    px,
                    py - 0.024,
                    0.08,
                    0.008,
                    u.trans as f32 / 100.0,
                    [0.72, 0.22, 0.28, 1.0],
                );
            }
        }
        self.blit_px(
            &mut rc,
            self.tex("kenney/ui/panel.png"),
            [0.0, 0.85, 1.0, 0.15],
            [0.38, 0.30, 0.22, 0.92],
        );
        if let Some(u) = current_unit(combat) {
            self.blit(
                &mut rc,
                self.tex(&u.portrait),
                [0.012, 0.862, 0.08, 0.128],
                [1.0; 4],
            );
            self.text(
                &mut rc,
                &format!("{}  AP {}", u.name, u.ap),
                0.10,
                0.862,
                0.012,
            );
            self.bar(
                &mut rc,
                0.10,
                0.888,
                0.22,
                0.016,
                u.hp as f32 / u.max_hp.max(1) as f32,
                BLOOD,
            );
            self.bar(
                &mut rc,
                0.10,
                0.908,
                0.22,
                0.012,
                u.trans as f32 / 100.0,
                [0.7, 0.22, 0.26, 1.0],
            );
        }
        let bar = hud::combat_bar();
        let sel = game.ui.selected_skill.as_deref();
        self.kenney_btn(&mut rc, "kenney/ui/button-grey.png", bar.wait, "WAIT", false);
        self.kenney_btn(&mut rc, "kenney/ui/button.png", bar.raise, "RAISE", false);
        self.kenney_btn(
            &mut rc,
            "kenney/ui/button-brown.png",
            bar.guard,
            "GUARD",
            sel == Some("guard"),
        );
        self.kenney_btn(
            &mut rc,
            "kenney/ui/button.png",
            bar.cut,
            "CUT",
            sel == Some("cut"),
        );
        let slot_label = current_unit(combat)
            .and_then(|u| u.skills.get(4))
            .map(|s| s.to_ascii_uppercase())
            .unwrap_or_else(|| "—".into());
        self.kenney_btn(
            &mut rc,
            "kenney/ui/button-grey.png",
            bar.slot,
            &slot_label,
            false,
        );
        self.kenney_btn(
            &mut rc,
            "kenney/ui/button-red.png",
            bar.forfeit,
            "FALL",
            false,
        );
        self.prompt(&mut rc, "kenney/prompt/space.png", bar.wait.x + 0.03, 0.868, 0.028);
        self.prompt(&mut rc, "kenney/prompt/1.png", bar.cut.x + 0.035, 0.868, 0.024);
        self.prompt(&mut rc, "kenney/prompt/2.png", bar.guard.x + 0.04, 0.868, 0.024);
        if let Some(line) = combat.log.first() {
            self.text(&mut rc, &line.text, 0.58, 0.862, 0.011);
        }
        self.text(&mut rc, combat.title.as_str(), 0.02, 0.02, 0.014);
        self.prompt(&mut rc, "kenney/prompt/esc.png", 0.92, 0.02, 0.04);
        self.draw_fx(&mut rc, game);
    }

    fn draw_result(&self, pass: &mut gpu::RenderCommandEncoder, game: &Game, _w: f32, _h: f32) {
        let mut rc = pass.with(&self.flat);
        self.rect(&mut rc, [0.0, 0.0, 1.0, 1.0], [0.05, 0.04, 0.03, 1.0]);
        self.blit_px(
            &mut rc,
            self.tex("kenney/ui/panel-brown.png"),
            [0.08, 0.28, 0.62, 0.32],
            [0.5, 0.4, 0.3, 0.85],
        );
        self.text(&mut rc, &game.result_title, 0.12, 0.34, 0.022);
        self.text(&mut rc, &game.result_body, 0.12, 0.44, 0.014);
        self.kenney_btn(
            &mut rc,
            "kenney/ui/button.png",
            hud::result_ok(),
            "RETURN",
            false,
        );
        self.draw_fx(&mut rc, game);
    }

    fn draw_codex(&self, pass: &mut gpu::RenderCommandEncoder, _game: &Game, _w: f32, _h: f32) {
        let mut rc = pass.with(&self.flat);
        self.rect(&mut rc, [0.0, 0.0, 1.0, 1.0], [0.05, 0.04, 0.03, 1.0]);
        self.blit_px(
            &mut rc,
            self.tex("kenney/ui/banner.png"),
            [0.06, 0.05, 0.4, 0.08],
            ASH,
        );
        self.text(&mut rc, "CODEX", 0.08, 0.07, 0.022);
        for (i, w) in WARRIORS.iter().take(8).enumerate() {
            let col = (i % 4) as f32;
            let row = (i / 4) as f32;
            let x = 0.08 + col * 0.22;
            let y = 0.18 + row * 0.32;
            self.blit_px(
                &mut rc,
                self.tex("kenney/ui/panel-brown.png"),
                [x, y, 0.20, 0.28],
                [0.48, 0.38, 0.28, 0.9],
            );
            self.blit(&mut rc, self.tex(w.portrait), [x + 0.03, y + 0.02, 0.14, 0.18], [1.0; 4]);
            self.text(&mut rc, w.name, x + 0.02, y + 0.21, 0.012);
            self.text(&mut rc, w.title, x + 0.02, y + 0.235, 0.010);
        }
        self.text(&mut rc, "CLICK TO CLOSE", 0.08, 0.90, 0.014);
    }


    fn draw_scene(&self, pass: &mut gpu::RenderCommandEncoder, game: &Game, _w: f32, _h: f32) {
        let mut rc = pass.with(&self.flat);
        let art = match game.scene.as_ref().map(|s| s.id) {
            Some(crate::dialog::SceneId::OpheliaIntro) => "art/battle-gonal.jpg",
            Some(crate::dialog::SceneId::TownDoga) => "art/tavern.jpg",
            Some(crate::dialog::SceneId::RecruitPaburo) => "art/battle-paburo.jpg",
            Some(crate::dialog::SceneId::RecruitPieta) => "art/battle-pieta.jpg",
            _ => "art/title.jpg",
        };
        self.blit(&mut rc, self.tex(art), [0.0, 0.0, 1.0, 1.0], [0.45, 0.42, 0.38, 1.0]);
        self.rect(&mut rc, [0.0, 0.0, 1.0, 1.0], [0.04, 0.03, 0.02, 0.55]);
        self.blit_px(
            &mut rc,
            self.tex("kenney/ui/panel-brown.png"),
            [0.08, 0.22, 0.70, 0.42],
            [0.50, 0.40, 0.30, 0.92],
        );
        if let Some(scene) = game.scene.as_ref() {
            if let Some(line) = scene.current() {
                self.text(&mut rc, line.speaker, 0.12, 0.28, 0.016);
                // wrap-ish: single line may be long; draw once
                self.text(&mut rc, line.text, 0.12, 0.38, 0.014);
                let step = format!("{}/{}", scene.step + 1, scene.lines().len());
                self.text(&mut rc, &step, 0.12, 0.52, 0.011);
            }
            if scene.at_end() {
                let choices = scene.choices();
                if choices.len() >= 1 {
                    self.kenney_btn(
                        &mut rc,
                        "kenney/ui/button.png",
                        crate::hud::scene_yes(),
                        choices[0].label,
                        true,
                    );
                }
                if choices.len() >= 2 {
                    self.kenney_btn(
                        &mut rc,
                        "kenney/ui/button-grey.png",
                        crate::hud::scene_no(),
                        choices[1].label,
                        false,
                    );
                }
            } else {
                self.prompt(&mut rc, "kenney/prompt/space.png", 0.12, 0.84, 0.05);
                self.text(&mut rc, "CONTINUE", 0.19, 0.855, 0.014);
            }
        }
        self.draw_fx(&mut rc, game);
    }

    fn draw_fx(&self, rc: &mut PipeEnc<'_>, game: &Game) {
        for p in &game.fx.particles {
            let a = (p.life / p.max).clamp(0.0, 1.0);
            let mut tint = p.tint;
            tint[3] *= a;
            self.blit_px(
                rc,
                self.tex(p.sprite),
                [p.x - p.size * 0.5, p.y - p.size * 0.5, p.size, p.size],
                tint,
            );
        }
        for b in &game.fx.bursts {
            let t = 1.0 - (b.life / b.max);
            let s = b.size * (0.55 + t * 0.8);
            let a = (b.life / b.max).clamp(0.0, 1.0);
            let mut tint = b.tint;
            tint[3] *= a;
            self.blit(
                rc,
                self.tex(b.sprite),
                [b.x - s * 0.5, b.y - s * 0.5, s, s],
                tint,
            );
        }
        for f in &game.fx.floaters {
            let a = (f.life / f.max).clamp(0.0, 1.0);
            let pop = 1.0 + (1.0 - a) * 0.25;
            let mut tint = f.tint;
            tint[3] *= a;
            self.text_tint(rc, &f.text, f.x, f.y, 0.016 * pop, tint);
        }
        if game.fx.flash > 0.02 {
            self.rect(rc, [0.0, 0.0, 1.0, 1.0], [0.92, 0.82, 0.72, game.fx.flash * 0.4]);
        }
    }

    fn kenney_btn(
        &self,
        rc: &mut PipeEnc<'_>,
        kind: &str,
        r: hud::Rect,
        label: &str,
        hot: bool,
    ) {
        let tex = if hot {
            "kenney/ui/button-line.png"
        } else {
            kind
        };
        self.blit_px(rc, self.tex(tex), r.pos(), ASH);
        if hot {
            self.rect(rc, r.pos(), [0.72, 0.55, 0.22, 0.22]);
        }
        self.text(rc, label, r.x + 0.016, r.y + r.h * 0.32, 0.013);
    }

    fn bar(
        &self,
        rc: &mut PipeEnc<'_>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        fill: f32,
        tint: [f32; 4],
    ) {
        self.blit_px(rc, self.tex("kenney/ui/bar.png"), [x, y, w, h], [0.22, 0.18, 0.14, 0.95]);
        let f = fill.clamp(0.0, 1.0);
        if f > 0.01 {
            self.blit_px(
                rc,
                self.tex("kenney/ui/bar-fill.png"),
                [x + w * 0.02, y + h * 0.15, w * 0.96 * f, h * 0.7],
                tint,
            );
        }
    }

    fn prompt(&self, rc: &mut PipeEnc<'_>, rel: &str, x: f32, y: f32, s: f32) {
        self.blit_px(rc, self.tex(rel), [x, y, s, s], [0.85, 0.8, 0.7, 0.95]);
    }

    fn blit(&self, rc: &mut PipeEnc<'_>, view: gpu::TextureView, pos: [f32; 4], tint: [f32; 4]) {
        self.blit_s(rc, view, self.sampler, pos, tint);
    }

    fn blit_px(&self, rc: &mut PipeEnc<'_>, view: gpu::TextureView, pos: [f32; 4], tint: [f32; 4]) {
        self.blit_s(rc, view, self.pixel, pos, tint);
    }

    fn blit_s(
        &self,
        rc: &mut PipeEnc<'_>,
        view: gpu::TextureView,
        sampler: gpu::Sampler,
        pos: [f32; 4],
        tint: [f32; 4],
    ) {
        rc.bind(
            0,
            &FlatFrame {
                globals: FlatGlobals { pad: [0.0; 4] },
                sprite_texture: view,
                sprite_sampler: sampler,
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
        self.text_tint(rc, s, x, y, glyph_h, [0.92, 0.88, 0.78, 1.0]);
    }

    fn text_tint(&self, rc: &mut PipeEnc<'_>, s: &str, x: f32, y: f32, glyph_h: f32, tint: [f32; 4]) {
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
                        tint,
                    },
                },
            );
            rc.bind_vertex(0, self.quad.into());
            rc.draw(0, 4, 0, 1);
            cx += glyph_w * 0.85;
        }
    }
}

fn loc_prop(id: &str, kind: &str) -> &'static str {
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

fn prop_size(kind: &str) -> (f32, f32) {
    match kind {
        "city" | "keep" | "office" => (0.055, 0.07),
        "wild" => (0.04, 0.06),
        _ => (0.045, 0.055),
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
