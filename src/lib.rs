use std::sync::Arc;

pub use glam::f32::{Vec2, Vec3};
use wgpu::util::DeviceExt;

pub fn run(title: &str, scene: Scene) {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let mut app = Handler {
        state: None,
        scene: Some(scene),
        title,
    };
    event_loop.run_app(&mut app).unwrap();
}

#[derive(Default)]
pub struct Scene {
    pub nodes: Vec<Node>,
}

pub struct Node {
    display: Display,
    position: Vec2,
    size: Vec2,
}

impl Node {
    // create new rectangle node
    pub fn rectangle(position: Vec2, size: Vec2) -> Self {
        Self {
            display: Display::Rectangle,
            position,
            size,
        }
    }

    // create new triangle node
    pub fn triangle(position: Vec2, size: Vec2) -> Self {
        Self {
            display: Display::Triangle,
            position,
            size,
        }
    }
}

pub enum Display {
    Rectangle,
    Triangle,
    Texture,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: Vec2,
    uv: Vec2,
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    position: Vec2,
    scale: Vec2,
}

impl Instance {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![2 => Float32x2, 3 => Float32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

const RECT_VERTICES: &[Vertex] = &[
    Vertex {
        position: Vec2::new(0.5, 0.5),
        uv: Vec2::new(1.0, 0.0),
    },
    Vertex {
        position: Vec2::new(-0.5, 0.5),
        uv: Vec2::new(0.0, 0.0),
    },
    Vertex {
        position: Vec2::new(-0.5, -0.5),
        uv: Vec2::new(0.0, 1.0),
    },
    Vertex {
        position: Vec2::new(0.5, -0.5),
        uv: Vec2::new(1.0, 1.0),
    },
];

const RECT_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

const TRIANGLE_VERTICES: &[Vertex] = &[
    Vertex {
        position: Vec2::new(0.0, 0.577350),
        uv: Vec2::new(0.5, 0.066987),
    },
    Vertex {
        position: Vec2::new(-0.5, -0.288675),
        uv: Vec2::new(0.0, 0.933013),
    },
    Vertex {
        position: Vec2::new(0.5, -0.288675),
        uv: Vec2::new(1.0, 0.933013),
    },
];
const TRIANGLE_INDICES: &[u16] = &[0, 1, 2];

struct VertexBuffer {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,

    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
}

impl VertexBuffer {
    fn new(device: &wgpu::Device, vertex: &[Vertex], index: &[u16]) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(vertex),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(index),
            usage: wgpu::BufferUsages::INDEX,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 128 * size_of::<Instance>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: index.len() as u32,
            instances: Vec::new(),
            instance_buffer,
        }
    }

    fn set_buffer(&self, queue: &mut wgpu::Queue, render_pass: &mut wgpu::RenderPass) {
        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&self.instances),
        );

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
    }
}

struct State<'a> {
    // scene
    scene: Scene,

    // winit-related
    pub window: Arc<winit::window::Window>,

    // wgpu-related
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_configuration: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,

    rect_vbuf: VertexBuffer,
    triangle_vbuf: VertexBuffer,
}

impl<'a> State<'a> {
    async fn new(window: Arc<winit::window::Window>, scene: Scene) -> Self {
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

        // shader
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader/default.wgsl"));

        // pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Vertex::desc(), Instance::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // vertex buffers
        let rect_vbuf = VertexBuffer::new(&device, RECT_VERTICES, RECT_INDICES);
        let triangle_vbuf = VertexBuffer::new(&device, TRIANGLE_VERTICES, TRIANGLE_INDICES);

        Self {
            scene,

            window,
            surface,
            device,
            queue,
            surface_configuration,
            render_pipeline,

            rect_vbuf,
            triangle_vbuf,
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
                depth_stencil_attachment: None,
                ..Default::default()
            });

            render_pass.set_pipeline(&self.render_pipeline);

            for node in &self.scene.nodes {
                let instance = Instance {
                    position: node.position,
                    scale: node.size,
                };
                match node.display {
                    Display::Rectangle => self.rect_vbuf.instances.push(instance),
                    Display::Triangle => self.triangle_vbuf.instances.push(instance),
                    Display::Texture => todo!(),
                };
            }
            for vbuf in [&self.rect_vbuf, &self.triangle_vbuf] {
                if !vbuf.instances.is_empty() {
                    vbuf.set_buffer(&mut self.queue, &mut render_pass);
                    render_pass.draw_indexed(
                        0..vbuf.index_count,
                        0,
                        0..vbuf.instances.len() as u32,
                    );
                }
            }
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
    }
}

struct Handler<'a> {
    state: Option<State<'a>>,
    scene: Option<Scene>,
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
            self.scene.take().unwrap(),
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
