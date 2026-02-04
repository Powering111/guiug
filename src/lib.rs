pub mod scene;
pub mod texture;
pub use scene::{Display, Node, Scene};

use std::sync::Arc;

pub use glam::f32::{Vec2, Vec3, Vec4};
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

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: Vec3,
    uv: Vec2,
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

trait Instance {
    type Raw: bytemuck::Pod;
    fn desc() -> wgpu::VertexBufferLayout<'static>;
    fn raw(&self) -> Self::Raw;
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct FlatInstance {
    position: Vec3,
    scale: Vec3,
    color: Vec4,
}

impl FlatInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![2 => Float32x3, 3 => Float32x3, 4 => Float32x4];
}

impl Instance for FlatInstance {
    type Raw = FlatInstance;
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self::Raw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }

    fn raw(&self) -> Self::Raw {
        *self
    }
}

#[derive(Clone, Debug)]
struct TextureInstance {
    position: Vec3,
    scale: Vec3,
    texture_id: texture::TextureId,
}
impl TextureInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![2 => Float32x3, 3 => Float32x3];
}

impl Instance for TextureInstance {
    type Raw = TextureInstanceRaw;
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self::Raw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }

    fn raw(&self) -> Self::Raw {
        TextureInstanceRaw {
            position: self.position,
            scale: self.scale,
        }
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TextureInstanceRaw {
    position: Vec3,
    scale: Vec3,
}

const RECT_VERTICES: &[Vertex] = &[
    Vertex {
        position: Vec3::new(0.5, 0.5, 0.0),
        uv: Vec2::new(1.0, 0.0),
    },
    Vertex {
        position: Vec3::new(-0.5, 0.5, 0.0),
        uv: Vec2::new(0.0, 0.0),
    },
    Vertex {
        position: Vec3::new(-0.5, -0.5, 0.0),
        uv: Vec2::new(0.0, 1.0),
    },
    Vertex {
        position: Vec3::new(0.5, -0.5, 0.0),
        uv: Vec2::new(1.0, 1.0),
    },
];

const RECT_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

const TRIANGLE_VERTICES: &[Vertex] = &[
    Vertex {
        position: Vec3::new(0.0, 0.577350, 0.0),
        uv: Vec2::new(0.5, 0.066987),
    },
    Vertex {
        position: Vec3::new(-0.5, -0.288675, 0.0),
        uv: Vec2::new(0.0, 0.933013),
    },
    Vertex {
        position: Vec3::new(0.5, -0.288675, 0.0),
        uv: Vec2::new(1.0, 0.933013),
    },
];
const TRIANGLE_INDICES: &[u16] = &[0, 1, 2];

struct VertexBuffer<I: Instance> {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,

    instance_buffer: wgpu::Buffer,

    _p: std::marker::PhantomData<I>,
}

impl<I: Instance> VertexBuffer<I> {
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
            size: 1024 * size_of::<I>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: index.len() as u32,

            instance_buffer,
            _p: std::marker::PhantomData,
        }
    }

    fn draw(&self, queue: &mut wgpu::Queue, render_pass: &mut wgpu::RenderPass, instances: &[I]) {
        if instances.is_empty() {
            return;
        }

        let raw_instances: Vec<I::Raw> = instances.iter().map(|instance| instance.raw()).collect();
        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&raw_instances),
        );

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        render_pass.draw_indexed(0..self.index_count, 0, 0..instances.len() as u32);
    }
}

fn create_render_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    buffer_layout: &[wgpu::VertexBufferLayout],
    bind_group_layout: &[&wgpu::BindGroupLayout],
    surface_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: bind_group_layout,
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: buffer_layout,
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
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
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
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
    flat_render_pipeline: wgpu::RenderPipeline,
    texture_render_pipeline: wgpu::RenderPipeline,

    rect_vbuf: VertexBuffer<FlatInstance>,
    triangle_vbuf: VertexBuffer<FlatInstance>,
    texture_vbuf: VertexBuffer<TextureInstance>,

    texture_manager: texture::TextureManager,
    depth_texture_view: wgpu::TextureView,
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

        // texture
        let mut texture_manager = texture::TextureManager::new(&device);

        // textures
        let icon_texture_id = texture_manager.load_texture_from_bytes(
            &device,
            &queue,
            include_bytes!("res/awesomeface_3d.png"),
        );
        assert_eq!(icon_texture_id, 0);

        // shader
        let flat_shader = device.create_shader_module(wgpu::include_wgsl!("shader/flat.wgsl"));
        let texture_shader =
            device.create_shader_module(wgpu::include_wgsl!("shader/texture.wgsl"));

        // pipeline
        let flat_render_pipeline = create_render_pipeline(
            &device,
            &flat_shader,
            &[Vertex::desc(), FlatInstance::desc()],
            &[],
            surface_format,
        );
        let texture_render_pipeline = create_render_pipeline(
            &device,
            &texture_shader,
            &[Vertex::desc(), TextureInstance::desc()],
            &[&texture_manager.bind_group_layout],
            surface_format,
        );

        // vertex buffers
        let rect_vbuf = VertexBuffer::new(&device, RECT_VERTICES, RECT_INDICES);
        let triangle_vbuf = VertexBuffer::new(&device, TRIANGLE_VERTICES, TRIANGLE_INDICES);
        let texture_vbuf = VertexBuffer::new(&device, RECT_VERTICES, RECT_INDICES);

        let depth_texture_view = texture::create_depth_texture(&device, &surface_configuration);

        Self {
            scene,

            window,
            surface,
            device,
            queue,
            surface_configuration,
            flat_render_pipeline,
            texture_render_pipeline,

            rect_vbuf,
            triangle_vbuf,
            texture_vbuf,

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
            let mut triangle_instances = Vec::new();
            let mut texture_instances = Vec::new();
            for node in &self.scene.nodes {
                match node.display {
                    Display::Rectangle(color) => rect_instances.push(FlatInstance {
                        position: node.position,
                        scale: node.size,
                        color,
                    }),
                    Display::Triangle(color) => triangle_instances.push(FlatInstance {
                        position: node.position,
                        scale: node.size,
                        color,
                    }),
                    Display::Texture(texture_id) => texture_instances.push(TextureInstance {
                        position: node.position,
                        scale: node.size,
                        texture_id,
                    }),
                };
            }

            // Flat rendering
            render_pass.set_pipeline(&self.flat_render_pipeline);

            self.rect_vbuf
                .draw(&mut self.queue, &mut render_pass, &rect_instances);
            self.triangle_vbuf
                .draw(&mut self.queue, &mut render_pass, &triangle_instances);

            // Texture rendering
            render_pass.set_pipeline(&self.texture_render_pipeline);

            for texture_id in 0..self.texture_manager.last_id {
                let mut instances = texture_instances
                    .iter()
                    .filter(|instance| instance.texture_id == texture_id)
                    .peekable();
                if instances.peek().is_some() {
                    // there are nodes rendered using this texture
                    let texture = self.texture_manager.get_texture(texture_id).unwrap();

                    render_pass.set_bind_group(0, texture.bind_group.as_ref().unwrap(), &[]);

                    let instances: Vec<TextureInstance> = instances.cloned().collect();
                    self.texture_vbuf
                        .draw(&mut self.queue, &mut render_pass, &instances);
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
