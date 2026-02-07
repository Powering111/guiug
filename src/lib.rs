//! Declarative GUI library in Rust.
//! Create [Guiug] object and call [run] with it.

mod renderer;
mod scene;
mod texture;
mod types;

pub use glam::Vec4;
use glam::{IVec2, IVec3, UVec3};
pub use scene::{Anchor, Node, NodeId, Position, Scene, Size};
use std::sync::Arc;
use types::Rect;
use wgpu::{BindGroupDescriptor, BindGroupLayoutDescriptor, util::DeviceExt};

use crate::types::Dimension;

/// Interface for guiug application.
///
/// # Example
/// ```
/// let mut guiug = guiug::Guiug::default();
/// let root_node = guiug.layer_node(vec![]);
/// guiug.set_root(root_node);
/// guiug::run("awesome application", guiug);
/// ```
#[derive(Default)]
pub struct Guiug<'a> {
    scene: Scene,
    texture_info_manager: texture::TextureInfoManager<'a>,
}

impl<'a> Guiug<'a> {
    /// Add texture to be loaded and used later. You can use the returned TextureId to construct texture node.
    pub fn add_texture(&mut self, texture_data: &'a [u8]) -> texture::TextureId {
        self.texture_info_manager.add_texture_info(texture_data)
    }

    /// Set scene root. You have to set root in order to render anything on the screen. Root node will have same size as the screen.
    pub fn set_root(&mut self, root_node: NodeId) {
        self.scene.root_node = Some(root_node);
    }

    /// Create Layer node.
    /// First one will be visible when overlapped.
    pub fn layer_node(&mut self, inner: Vec<(Position, NodeId)>) -> NodeId {
        let node = Node::Layer { inner };
        self.scene.insert_node(node)
    }

    /// Create Rect node. It renders as solid rectangle. Color is RGBA0~1 Vec4.
    pub fn rect_node(&mut self, color: Vec4) -> NodeId {
        let node = Node::Rect { color };
        self.scene.insert_node(node)
    }

    /// Create texture node. It renders as rectangular image.
    /// To create texture, use [Self::add_texture]
    pub fn texture_node(&mut self, texture_id: texture::TextureId) -> NodeId {
        let node = Node::Texture { texture_id };
        self.scene.insert_node(node)
    }

    /// Create row node.
    pub fn row_node(&mut self, inner: Vec<(Size, NodeId)>) -> NodeId {
        let node = Node::Row { inner };
        self.scene.insert_node(node)
    }

    /// Create column node.
    pub fn column_node(&mut self, inner: Vec<(Size, NodeId)>) -> NodeId {
        let node = Node::Column { inner };
        self.scene.insert_node(node)
    }

    /// Create empty node. It can be used for space between row or column elements.
    pub fn empty_node(&mut self) -> NodeId {
        let node = Node::Empty;
        self.scene.insert_node(node)
    }
}

/// Run the given guiug application.
/// This function will not return until the window closes.
/// * `title` - window title
/// * `guiug` - guiug application to run
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

        let screen_size = UVec3::new(size.width, size.height, 0);
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

            let screen_size = Dimension::new(
                self.surface_configuration.width as i32,
                self.surface_configuration.height as i32,
            );
            let visitor = NodeVisitor::visit(screen_size, &self.scene);

            self.queue.write_buffer(
                &self.screen_uniform_buffer,
                0,
                bytemuck::cast_slice(&[UVec3::new(
                    screen_size.width as u32,
                    screen_size.height as u32,
                    visitor.z_index as u32,
                )]),
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
    }
}

pub(crate) struct NodeVisitor {
    screen_size: Dimension,
    rect_instances: Vec<renderer::FlatInstance>,
    texture_instances: Vec<renderer::TextureInstance>,
    z_index: i32,
}

impl NodeVisitor {
    pub fn visit(screen_size: Dimension, scene: &Scene) -> Self {
        let mut visitor = Self {
            screen_size,
            rect_instances: Vec::new(),
            texture_instances: Vec::new(),
            z_index: 0,
        };
        if let Some(root_node) = scene.root_node {
            let screen_rect = Rect::new(0, 0, screen_size.width, screen_size.height);

            visitor.do_visit(scene, root_node, screen_rect);
        }
        visitor
    }

    pub fn do_visit(&mut self, scene: &Scene, node_id: NodeId, rect: Rect) {
        if let Some(node) = scene.get_node(&node_id) {
            match node {
                Node::Layer { inner } => {
                    for (position, child_node_id) in inner {
                        let child_rect = position.apply(rect, self.screen_size);
                        self.do_visit(scene, *child_node_id, child_rect);
                        self.z_index += 1;
                    }
                }
                Node::Row { inner } => {
                    let mut total_size = rect.h;
                    let mut total_weight = 0.0;
                    for (size, _) in inner {
                        total_size -= size.resolve(rect.dimension(), self.screen_size);
                        if let Size::Weight(weight) = size {
                            total_weight += weight;
                        }
                    }

                    let mut pos = rect.y;
                    for (size, child_node_id) in inner {
                        let size = if let Size::Weight(weight) = size {
                            (total_size as f32 * (weight / total_weight)) as i32
                        } else {
                            size.resolve(rect.dimension(), self.screen_size)
                        }
                        .max(0);
                        self.do_visit(scene, *child_node_id, Rect::new(rect.x, pos, rect.w, size));
                        pos += size;
                    }
                }
                Node::Column { inner } => {
                    let mut total_size = rect.w;
                    let mut total_weight = 0.0;
                    for (size, _) in inner {
                        total_size -= size.resolve(rect.dimension(), self.screen_size);
                        if let Size::Weight(weight) = size {
                            total_weight += weight;
                        }
                    }

                    let mut pos = rect.x;
                    for (size, child_node_id) in inner {
                        let size = if let Size::Weight(weight) = size {
                            (total_size as f32 * (weight / total_weight)) as i32
                        } else {
                            size.resolve(rect.dimension(), self.screen_size)
                        }
                        .max(0);
                        self.do_visit(scene, *child_node_id, Rect::new(pos, rect.y, size, rect.h));
                        pos += size;
                    }
                }
                Node::Rect { color } => self.rect_instances.push(renderer::FlatInstance {
                    position: IVec3::new(rect.x, rect.y, self.z_index),
                    scale: IVec2::new(rect.w, rect.h),
                    color: *color,
                }),
                Node::Texture { texture_id } => {
                    self.texture_instances.push(renderer::TextureInstance {
                        position: IVec3::new(rect.x, rect.y, self.z_index),
                        scale: IVec2::new(rect.w, rect.h),
                        texture_id: *texture_id,
                    })
                }
                Node::Empty => (),
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
                    .with_inner_size(winit::dpi::PhysicalSize::new(800, 800))
                    .with_title(self.title)
                    .with_visible(false),
            )
            .unwrap();
        let window = Arc::new(window);
        self.state = Some(pollster::block_on(State::new(
            window.clone(),
            self.guiug.take().unwrap(),
        )));

        window.set_visible(true);
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
