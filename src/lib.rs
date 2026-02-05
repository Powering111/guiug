mod renderer;
mod scene;
mod texture;

pub use glam::{UVec2, UVec3, Vec2, Vec3, Vec4};
pub use scene::{Node, NodeId, Position, Scene};
use std::sync::Arc;
use wgpu::{BindGroupDescriptor, BindGroupLayoutDescriptor, util::DeviceExt};

#[derive(Default)]
pub struct Guiug<'a> {
    scene: Scene,
    texture_info_manager: texture::TextureInfoManager<'a>,
}

impl<'a> Guiug<'a> {
    // Add texture to be loaded and used later. You can use the returned TextureId.
    pub fn add_texture(&mut self, texture_data: &'a [u8]) -> texture::TextureId {
        self.texture_info_manager.add_texture_info(texture_data)
    }
}

impl core::ops::Deref for Guiug<'_> {
    type Target = scene::Scene;

    fn deref(&self) -> &Self::Target {
        &self.scene
    }
}

impl core::ops::DerefMut for Guiug<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scene
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
    screen_uniform_buffer: wgpu::Buffer,
    screen_uniform_bind_group: wgpu::BindGroup,

    texture_manager: texture::TextureManager,
}

impl<'a> State<'a> {
    async fn new(window: Arc<winit::window::Window>, guiug: Guiug<'a>) -> Self {
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

        // screen uniform
        let screen_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let screen_size = UVec2::new(size.width, size.height);
        let screen_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("screen uniform buffer"),
            contents: bytemuck::cast_slice(&[screen_size]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let screen_uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("screen uniform bind group"),
            layout: &screen_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &screen_uniform_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        // renderer
        let flat_renderer =
            renderer::FlatRenderer::new(&device, surface_format, &screen_bind_group_layout);

        let texture_renderer = renderer::TextureRenderer::new(
            &device,
            surface_format,
            &screen_bind_group_layout,
            &texture_manager.bind_group_layout,
        );

        let depth_texture_view = texture::create_depth_texture(&device, &surface_configuration);

        Self {
            scene: guiug.scene,

            window,
            surface,
            device,
            queue,
            surface_configuration,
            flat_renderer,
            texture_renderer,
            screen_uniform_buffer,
            screen_uniform_bind_group,

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

            let visitor = NodeVisitor::visit(
                self.surface_configuration.width,
                self.surface_configuration.height,
                &self.scene,
            );

            // bind screen uniform
            render_pass.set_bind_group(0, &self.screen_uniform_bind_group, &[]);

            // Flat rendering
            self.flat_renderer
                .draw(&mut render_pass, &self.queue, visitor.rect_instances);

            // Texture rendering
            self.texture_renderer.draw(
                &mut render_pass,
                &self.queue,
                &self.texture_manager,
                visitor.texture_instances,
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

        self.queue.write_buffer(
            &self.screen_uniform_buffer,
            0,
            bytemuck::cast_slice(&[UVec2::new(width, height)]),
        );
    }
}

#[derive(Clone, Copy, Debug)]
struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}
pub(crate) struct NodeVisitor {
    rect_instances: Vec<renderer::FlatInstance>,
    texture_instances: Vec<renderer::TextureInstance>,
    z_index: u32,
}

impl NodeVisitor {
    pub fn visit(screen_width: u32, screen_height: u32, scene: &Scene) -> Self {
        let mut visitor = Self {
            rect_instances: Vec::new(),
            texture_instances: Vec::new(),
            z_index: 0,
        };
        if let Some(root_node) = scene.root_node {
            let rect = Rect {
                x: 0,
                y: 0,
                w: screen_width,
                h: screen_height,
            };

            visitor.do_visit(scene, root_node, rect);
        }
        visitor
    }

    pub fn do_visit(&mut self, scene: &Scene, node_id: NodeId, rect: Rect) {
        if let Some(node) = scene.get(&node_id) {
            match node {
                Node::Layer { inner } => {
                    for (position, child_node_id) in inner {
                        let child_rect = match position {
                            Position::Full => rect,
                            Position::Absolute { position, size } => Rect {
                                x: position.x,
                                y: position.y,
                                w: size.x,
                                h: size.y,
                            },
                        };
                        self.do_visit(scene, *child_node_id, child_rect);
                        self.z_index += 1;
                    }
                }
                Node::Rect { color } => self.rect_instances.push(renderer::FlatInstance {
                    position: UVec3::new(rect.x, rect.y, self.z_index),
                    scale: UVec2::new(rect.w, rect.h),
                    color: *color,
                }),
                Node::Texture { texture_id } => {
                    self.texture_instances.push(renderer::TextureInstance {
                        position: UVec3::new(rect.x, rect.y, self.z_index),
                        scale: UVec2::new(rect.w, rect.h),
                        texture_id: *texture_id,
                    })
                }
            }
        }
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
                    .with_inner_size(winit::dpi::PhysicalSize::new(1000, 1000))
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
