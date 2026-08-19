//! winit loop. Identical on desktop and wasm.

use crate::game::Game;
use crate::gpu::Renderer;
use blade_graphics as gpu;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

fn surface_config(size: winit::dpi::PhysicalSize<u32>) -> gpu::SurfaceConfig {
    gpu::SurfaceConfig {
        size: gpu::Extent {
            width: size.width.max(1),
            height: size.height.max(1),
            depth: 1,
        },
        usage: gpu::TextureUsage::TARGET,
        display_sync: gpu::DisplaySync::Recent,
        ..Default::default()
    }
}

struct App {
    game: Game,
    renderer: Option<Renderer>,
    encoder: Option<gpu::CommandEncoder>,
    prev_sync: Option<gpu::SyncPoint>,
    surface: Option<gpu::Surface>,
    context: Option<gpu::Context>,
    window: Option<Window>,
    cursor: [f32; 2],
}

impl Default for App {
    fn default() -> Self {
        Self {
            game: Game::new(),
            renderer: None,
            encoder: None,
            prev_sync: None,
            surface: None,
            context: None,
            window: None,
            cursor: [0.0, 0.0],
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("Claymore")
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 800.0));
        let window = event_loop.create_window(attrs).expect("window");

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowExtWebSys as _;
            let canvas = window.canvas().expect("canvas");
            canvas.set_id(gpu::CANVAS_ID);
            let el: web_sys::HtmlElement = canvas.clone().unchecked_into();
            let _ = el.style().set_property("width", "100%");
            let _ = el.style().set_property("height", "100%");
            let _ = el.style().set_property("display", "block");
            web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.body())
                .and_then(|b| b.append_child(&web_sys::Element::from(canvas)).ok())
                .expect("append canvas");
        }

        let context = unsafe {
            gpu::Context::init(gpu::ContextDesc {
                presentation: true,
                validation: cfg!(debug_assertions),
                overlay: false,
                ..Default::default()
            })
        }
        .expect("blade gpu context");
        log::info!("{:?}", context.device_information());

        let size = window.inner_size();
        let surface = context
            .create_surface_configured(&window, surface_config(size))
            .expect("surface");
        let screen = gpu::Extent {
            width: size.width.max(1),
            height: size.height.max(1),
            depth: 1,
        };
        let renderer = Renderer::new(&context, screen, surface.info().format);
        let encoder = context.create_command_encoder(gpu::CommandEncoderDesc {
            name: "main",
            buffer_count: 2,
            manual_barriers: false,
        });
        self.renderer = Some(renderer);
        self.encoder = Some(encoder);
        self.surface = Some(surface);
        self.context = Some(context);
        self.window = Some(window);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let (Some(context), Some(surface), Some(renderer)) =
                    (&self.context, self.surface.as_mut(), self.renderer.as_mut())
                {
                    let screen = gpu::Extent {
                        width: size.width.max(1),
                        height: size.height.max(1),
                        depth: 1,
                    };
                    renderer.resize(context, screen);
                    context.reconfigure_surface(surface, surface_config(size));
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    if code == KeyCode::Escape && event.state == ElementState::Pressed {
                        #[cfg(not(target_arch = "wasm32"))]
                        if self.game.mode == crate::game::Mode::Title {
                            event_loop.exit();
                            return;
                        }
                    }
                    self.game.key(code, event.state == ElementState::Pressed);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = [position.x as f32, position.y as f32];
                if let Some(window) = &self.window {
                    let s = window.inner_size();
                    let w = s.width.max(1) as f32;
                    let h = s.height.max(1) as f32;
                    let nx = self.cursor[0] / w;
                    let ny = self.cursor[1] / h;
                    if self.game.ui.dragging {
                        self.game.ui.pan[0] += self.cursor[0] - self.game.ui.last_mouse[0];
                        self.game.ui.pan[1] += self.cursor[1] - self.game.ui.last_mouse[1];
                    }
                    self.game.ui.last_mouse = self.cursor;
                    self.game.hover_hex(nx, ny, [w, h]);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let down = state == ElementState::Pressed;
                if button == MouseButton::Right || button == MouseButton::Middle {
                    self.game.ui.dragging = down;
                }
                if button == MouseButton::Left && down {
                    if let Some(window) = &self.window {
                        let s = window.inner_size();
                        let w = s.width.max(1) as f32;
                        let h = s.height.max(1) as f32;
                        self.game
                            .click(self.cursor[0] / w, self.cursor[1] / h, [w, h]);
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 * 0.04,
                };
                self.game.ui.zoom = (self.game.ui.zoom * (1.0 + dy * 0.08)).clamp(0.55, 2.2);
            }
            WindowEvent::RedrawRequested => self.redraw(),
            _ => {}
        }
    }
}

impl App {
    fn redraw(&mut self) {
        self.game.tick(1.0 / 60.0);
        let (Some(context), Some(surface), Some(renderer), Some(encoder)) = (
            self.context.as_ref(),
            self.surface.as_mut(),
            self.renderer.as_mut(),
            self.encoder.as_mut(),
        ) else {
            return;
        };
        if renderer.is_zero_screen() {
            return;
        }
        let frame = surface.acquire_frame();
        encoder.start();
        encoder.init_texture(frame.texture());
        encoder.init_texture(renderer.depth_texture());
        renderer.render(encoder, frame.texture_view(), &self.game);
        encoder.present(frame);
        let sp = context.submit(encoder);
        if let Some(prev) = self.prev_sync.take() {
            #[cfg(not(target_arch = "wasm32"))]
            let _ = context.wait_for(&prev, !0);
            #[cfg(target_arch = "wasm32")]
            let _ = prev;
        }
        self.prev_sync = Some(sp);
    }
}

pub fn run() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = env_logger::try_init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Info);
    }

    let event_loop = EventLoop::new().expect("event loop");
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("run");

    #[cfg(target_arch = "wasm32")]
    {
        // winit returns after scheduling the browser loop; keep GPU alive.
        std::mem::forget(app);
        return;
    }

    #[cfg(not(target_arch = "wasm32"))]
    if let Some(context) = app.context.as_ref() {
        if let Some(sp) = app.prev_sync.take() {
            let _ = context.wait_for(&sp, !0);
        }
        if let Some(mut r) = app.renderer.take() {
            r.destroy(context);
        }
        if let Some(mut enc) = app.encoder.take() {
            context.destroy_command_encoder(&mut enc);
        }
        if let Some(mut surface) = app.surface.take() {
            context.destroy_surface(&mut surface);
        }
    }
}
