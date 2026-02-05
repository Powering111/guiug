mod renderer;
pub mod scene;
mod texture;

pub use glam::f32::{Vec2, Vec3, Vec4};
pub use scene::{Display, Node, Scene};
use std::sync::Arc;

#[derive(Default)]
pub struct Guiug<'a> {
    scene: Option<Scene>,
    texture_info_manager: texture::TextureInfoManager<'a>,
}

impl<'a> Guiug<'a> {
    pub fn set_scene(&mut self, scene: Scene) {
        self.scene = Some(scene);
    }

    // Add texture to be loaded and used later. You can use the returned TextureId.
    pub fn add_texture(&mut self, texture_data: &'a [u8]) -> texture::TextureId {
        self.texture_info_manager.add_texture_info(texture_data)
    }
}

pub fn run(title: &str, guiug: Guiug) {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let mut app = Handler {
        state: None,
        guiug: Some(guiug),
        title,
    };
    event_loop.run_app(&mut app).unwrap();
}

struct State<'a> {
    // scene
    scene: Scene,

    // winit-related
    window: Arc<winit::window::Window>,

    // wgpu-related
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_configuration: wgpu::SurfaceConfiguration,
    depth_texture_view: wgpu::TextureView,

    flat_renderer: renderer::FlatRenderer,
    texture_renderer: renderer::TextureRenderer,

    texture_manager: texture::TextureManager,
}

impl<'a> State<'a> {
    async fn new(window: Arc<winit::window::Window>, guiug: Guiug<'_>) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        // surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|format| format.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let size = window.inner_size();
        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_configuration);

        // texture manager
        let mut texture_manager = texture::TextureManager::new(&device);
        texture_manager.load(&device, &queue, &guiug.texture_info_manager);

        // renderer
        let flat_renderer = renderer::FlatRenderer::new(&device, surface_format);
        let texture_renderer = renderer::TextureRenderer::new(
            &device,
            surface_format,
            &texture_manager.bind_group_layout,
        );

        let depth_texture_view = texture::create_depth_texture(&device, &surface_configuration);

        Self {
            scene: guiug.scene.unwrap_or_default(),

            window,
            surface,
            device,
            queue,
            surface_configuration,
            flat_renderer,
            texture_renderer,

            texture_manager,
            depth_texture_view,
        }
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            let mut rect_instances = Vec::new();
            let mut texture_instances = Vec::new();
            for node in &self.scene.nodes {
                match node.display {
                    Display::Rectangle(color) => rect_instances.push(renderer::FlatInstance {
                        position: node.position,
                        scale: node.size,
                        color,
                    }),
                    Display::Texture(texture_id) => {
                        texture_instances.push(renderer::TextureInstance {
                            position: node.position,
                            scale: node.size,
                            texture_id,
                        })
                    }
                };
            }

            // Flat rendering
            self.flat_renderer
                .draw(&mut render_pass, &self.queue, rect_instances);

            // Texture rendering
            self.texture_renderer.draw(
                &mut render_pass,
                &self.queue,
                &self.texture_manager,
                texture_instances,
            );
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        self.window.pre_present_notify();
        output.present();

        // if you want to render every frame
        // self.window.request_redraw();
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.surface_configuration.width = width;
        self.surface_configuration.height = height;
        self.surface
            .configure(&self.device, &self.surface_configuration);

        self.depth_texture_view =
            texture::create_depth_texture(&self.device, &self.surface_configuration);
    }
}

struct Handler<'a> {
    state: Option<State<'a>>,
    guiug: Option<Guiug<'a>>,
    title: &'a str,
}

impl<'a> winit::application::ApplicationHandler for Handler<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(
                winit::window::Window::default_attributes()
                    .with_inner_size(winit::dpi::PhysicalSize::new(800, 800))
                    .with_title(self.title),
            )
            .unwrap();
        self.state = Some(pollster::block_on(State::new(
            Arc::new(window),
            self.guiug.take().unwrap(),
        )));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(state) => state,
            None => return,
        };

        match event {
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            winit::event::WindowEvent::RedrawRequested => {
                state.update();
                if let Err(wgpu::SurfaceError::Lost) | Err(wgpu::SurfaceError::Outdated) =
                    state.render()
                {
                    let size = state.window.inner_size();
                    state.resize(size.width, size.height);
                }
            }
            winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize { width, height }) => {
                state.resize(width, height);
            }
            _ => (),
        }
    }
}
