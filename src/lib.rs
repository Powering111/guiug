//! Declarative GUI library in Rust.
//! Create [Guiug] object and call [run] with it.

mod font;
mod interaction;
mod renderer;
mod scene;
mod texture;
mod types;

pub use glam::Vec4;
use glam::{IVec2, IVec3, UVec3};
pub use interaction::Event;
pub use scene::{Anchor, Node, NodeId, Position, Scene, Size, TextAnchor};
use std::{
    collections::{HashSet, VecDeque},
    sync::Arc,
};
use types::Rect;
use winit::event::WindowEvent;
pub use winit::keyboard::{KeyCode, PhysicalKey};

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
    font_info_manager: font::FontInfoManager<'a>,
    interaction: interaction::Interaction<'a>,
}

impl core::ops::Deref for Guiug<'_> {
    type Target = Scene;

    fn deref(&self) -> &Self::Target {
        &self.scene
    }
}

impl core::ops::DerefMut for Guiug<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scene
    }
}

impl<'a> Guiug<'a> {
    /// Add texture to be loaded and used later. You can use the returned TextureId to construct texture node.
    pub fn add_texture(&mut self, texture_data: &'a [u8]) -> texture::TextureId {
        self.texture_info_manager.add_texture_info(texture_data)
    }

    /// Add font to be loaded and used later. You can use the returned FontId to construct text node.
    pub fn add_font(&mut self, font_data: &'a [u8]) -> font::FontId {
        self.font_info_manager.add_font_info(font_data)
    }

    /// Add interaction. It attaches event handler to the given event.
    pub fn interaction(&mut self, event: Event, handler: impl FnMut(&mut Runtime) + 'a) {
        self.interaction.insert_handler(event, handler);
    }
}

/// Run the given guiug application.
/// This function will not return until the window closes.
/// * `title` - window title
/// * `guiug` - guiug application to run
pub fn run(title: &str, mut guiug: Guiug) {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let interaction = core::mem::take(&mut guiug.interaction);
    let mut app = WindowHandler {
        runtime: None,
        guiug: Some(guiug),
        title,
        pressed_keys: HashSet::new(),
        cursor_position: Default::default(),
        interaction,
    };
    event_loop.run_app(&mut app).unwrap();
}

pub struct Runtime<'a> {
    // scene
    scene: Scene,
    visitor: Option<NodeVisitor>,

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
    text_renderer: renderer::TextRenderer,
    screen_uniform: renderer::Uniform,

    texture_manager: texture::TextureManager,
    font_manager: font::FontManager,

    // interactions
    events: VecDeque<Event>,
    should_exit: bool,
}

impl<'a> Runtime<'a> {
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

        // font manager
        let mut font_manager = font::FontManager::new(&device);
        font_manager.load(&guiug.font_info_manager);

        // screen uniform
        let screen_uniform = renderer::Uniform::new(&device, size_of::<UVec3>() as u64);

        // renderer
        let flat_renderer =
            renderer::FlatRenderer::new(&device, surface_format, &screen_uniform.bind_group_layout);

        let texture_renderer = renderer::TextureRenderer::new(
            &device,
            surface_format,
            &screen_uniform.bind_group_layout,
            &texture_manager.bind_group_layout,
        );

        let text_renderer = renderer::TextRenderer::new(
            &device,
            surface_format,
            &screen_uniform.bind_group_layout,
            &texture_manager.bind_group_layout,
        );

        let depth_texture_view = texture::create_depth_texture(&device, &surface_configuration);

        Self {
            scene: guiug.scene,
            visitor: None,

            window,
            surface,
            device,
            queue,
            surface_configuration,
            flat_renderer,
            texture_renderer,
            text_renderer,
            screen_uniform,

            texture_manager,
            font_manager,
            depth_texture_view,

            events: VecDeque::new(),
            should_exit: false,
        }
    }

    fn update(&mut self) {
        self.visitor = Some(NodeVisitor::visit(
            Dimension::new(
                self.surface_configuration.width as i32,
                self.surface_configuration.height as i32,
            ),
            &self.scene,
            &self.font_manager,
        ));
    }

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

            let visitor = self.visitor.as_ref().unwrap();
            let rect_instances = &visitor.rect_instances;
            let texture_instances = &visitor.texture_instances;
            let text_instances = &visitor.text_instances;

            // screen uniform
            self.screen_uniform.write(
                &self.queue,
                bytemuck::cast_slice(&[UVec3::new(
                    screen_size.width as u32,
                    screen_size.height as u32,
                    visitor.z_index as u32,
                )]),
            );
            self.screen_uniform.set(&mut render_pass, 0);

            // Flat rendering
            self.flat_renderer
                .draw(&mut render_pass, &self.device, &self.queue, rect_instances);

            // Texture rendering
            self.texture_renderer.draw(
                &mut render_pass,
                &self.device,
                &self.queue,
                &self.texture_manager,
                texture_instances,
            );

            // Text rendering
            self.text_renderer.draw(
                &mut render_pass,
                &self.device,
                &self.queue,
                &mut self.font_manager,
                text_instances,
            )
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

        self.font_manager.clear_cache();
    }

    fn record_event(&mut self, event: Event) {
        self.events.push_back(event);
        self.window.request_redraw();
    }

    fn handle_click(&mut self, position: glam::IVec2) {
        let hitboxes = &self.visitor.as_ref().unwrap().hitboxes;
        for (node_id, hitbox) in hitboxes.iter() {
            if hitbox.contains(position) {
                self.record_event(Event::Click(*node_id));
                break;
            }
        }
    }

    pub fn exit(&mut self) {
        self.should_exit = true;
    }
}

impl core::ops::Deref for Runtime<'_> {
    type Target = Scene;

    fn deref(&self) -> &Self::Target {
        &self.scene
    }
}

impl core::ops::DerefMut for Runtime<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scene
    }
}

pub(crate) struct NodeVisitor {
    screen_size: Dimension,
    rect_instances: Vec<renderer::FlatInstance>,
    texture_instances: Vec<renderer::TextureInstance>,
    text_instances: Vec<renderer::TextInstance>,
    hitboxes: Vec<(NodeId, Rect)>,
    z_index: i32,
}

impl NodeVisitor {
    pub fn visit(screen_size: Dimension, scene: &Scene, font_manager: &font::FontManager) -> Self {
        let mut visitor = Self {
            screen_size,
            rect_instances: Vec::new(),
            texture_instances: Vec::new(),
            text_instances: Vec::new(),
            hitboxes: Vec::new(),
            z_index: 0,
        };
        if let Some(root_node) = scene.root_node {
            let screen_rect = Rect::new(0, 0, screen_size.width, screen_size.height);

            visitor.do_visit(scene, root_node, screen_rect, font_manager);
        }
        visitor
    }

    pub fn do_visit(
        &mut self,
        scene: &Scene,
        node_id: NodeId,
        rect: Rect,
        font_manager: &font::FontManager,
    ) {
        if let Some(node) = scene.get_node(&node_id) {
            match node {
                Node::Layer { inner } => {
                    for (position, child_node_id) in inner.iter().rev() {
                        let child_rect = position.apply(rect, self.screen_size);
                        self.do_visit(scene, *child_node_id, child_rect, font_manager);
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
                        self.do_visit(
                            scene,
                            *child_node_id,
                            Rect::new(rect.x, pos, rect.w, size),
                            font_manager,
                        );
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
                        self.do_visit(
                            scene,
                            *child_node_id,
                            Rect::new(pos, rect.y, size, rect.h),
                            font_manager,
                        );
                        pos += size;
                    }
                }
                Node::Hitbox => {
                    self.hitboxes.push((node_id, rect));
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
                Node::Text {
                    text,
                    font_id,
                    size,
                    color,
                    horizontal,
                    vertical,
                } => {
                    let size = size.resolve(rect.dimension(), self.screen_size) as u16;
                    let font = font_manager.get_font(*font_id).expect("no such font");
                    let node_width = font.measure_width(text, size);

                    let x = horizontal.apply(
                        node_width,
                        rect.x,
                        rect.w,
                        rect.dimension(),
                        self.screen_size,
                    );
                    let y = vertical.apply(0, rect.y, rect.h, rect.dimension(), self.screen_size);

                    self.text_instances.push(renderer::TextInstance {
                        text: text.clone(),
                        position: IVec3::new(x, y, self.z_index),
                        size,
                        font_id: *font_id,
                        color: *color,
                    })
                }
                Node::Empty => (),
            }
        }
    }
}

struct WindowHandler<'a> {
    runtime: Option<Runtime<'a>>,
    guiug: Option<Guiug<'a>>,
    title: &'a str,
    pressed_keys: HashSet<PhysicalKey>,
    cursor_position: winit::dpi::PhysicalPosition<f64>,
    interaction: interaction::Interaction<'a>,
}

impl<'a> winit::application::ApplicationHandler for WindowHandler<'a> {
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
        self.runtime = Some(pollster::block_on(Runtime::new(
            window.clone(),
            self.guiug.take().unwrap(),
        )));

        window.set_visible(true);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let runtime = match &mut self.runtime {
            Some(state) => state,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let events = runtime.events.clone();
                for event in events.iter() {
                    let handlers = self.interaction.get_handlers(*event);
                    for handler in handlers.iter_mut() {
                        handler(runtime);
                    }
                }
                runtime.events.clear();
                if runtime.should_exit {
                    event_loop.exit();
                    return;
                }

                runtime.update();
                if let Err(wgpu::SurfaceError::Lost) | Err(wgpu::SurfaceError::Outdated) =
                    runtime.render()
                {
                    let size = runtime.window.inner_size();
                    runtime.resize(size.width, size.height);
                }
            }
            WindowEvent::Resized(winit::dpi::PhysicalSize { width, height }) => {
                runtime.resize(width, height);
            }
            WindowEvent::Focused(false) => {
                // lost focus
                for pressed_key in self.pressed_keys.iter() {
                    runtime.record_event(Event::KeyUp(*pressed_key));
                }
                self.pressed_keys.clear();
            }
            WindowEvent::KeyboardInput { event, .. } => match event {
                winit::event::KeyEvent {
                    physical_key,
                    state: winit::event::ElementState::Pressed,
                    repeat: false,
                    ..
                } => {
                    self.pressed_keys.insert(physical_key);
                    runtime.record_event(Event::KeyDown(physical_key));
                }
                winit::event::KeyEvent {
                    physical_key,
                    state: winit::event::ElementState::Released,
                    repeat: false,
                    ..
                } => {
                    self.pressed_keys.remove(&physical_key);
                    runtime.record_event(Event::KeyUp(physical_key));
                }
                _ => (),
            },
            WindowEvent::MouseInput {
                state: winit::event::ElementState::Pressed,
                button: winit::event::MouseButton::Left,
                ..
            } => {
                // left click
                runtime.handle_click(glam::IVec2::new(
                    self.cursor_position.x as i32,
                    self.cursor_position.y as i32,
                ));
                runtime.record_event(Event::Click(0));
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = position;
            }
            _ => (),
        }
    }
}
