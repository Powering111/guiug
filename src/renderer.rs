use glam::{UVec2, UVec3, Vec2, Vec3, Vec4};
use wgpu::util::DeviceExt;

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

// Flat Renderer
pub struct FlatRenderer {
    render_pipeline: wgpu::RenderPipeline,
    instance_buffer: wgpu::Buffer,
    vbuf: VertexBuffer,
}

impl FlatRenderer {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        screen_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader/flat.wgsl"));
        let render_pipeline = create_render_pipeline(
            device,
            &shader,
            &[Vertex::desc(), FlatInstance::desc()],
            &[screen_bind_group_layout],
            surface_format,
        );

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 1024 * size_of::<FlatInstance>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let vbuf = VertexBuffer::new(device, RECT_VERTICES, RECT_INDICES);

        Self {
            render_pipeline,
            instance_buffer,
            vbuf,
        }
    }

    pub fn draw(
        &self,
        render_pass: &mut wgpu::RenderPass,
        queue: &wgpu::Queue,
        instances: Vec<FlatInstance>,
    ) {
        if instances.is_empty() {
            return;
        }

        queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));

        render_pass.set_pipeline(&self.render_pipeline);
        self.vbuf.set(render_pass);
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.draw_indexed(0..self.vbuf.index_count, 0, 0..instances.len() as u32);
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct FlatInstance {
    pub position: UVec3,
    pub scale: UVec2,
    pub color: Vec4,
}

impl FlatInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![2 => Uint32x3, 3 => Uint32x2, 4 => Float32x4];
}

impl FlatInstance {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

// Texture Renderer

pub struct TextureRenderer {
    render_pipeline: wgpu::RenderPipeline,
    instance_buffer: wgpu::Buffer,
    vbuf: VertexBuffer,
}

impl TextureRenderer {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        screen_bind_group_layout: &wgpu::BindGroupLayout,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader/texture.wgsl"));
        let render_pipeline = create_render_pipeline(
            device,
            &shader,
            &[Vertex::desc(), TextureInstanceRaw::desc()],
            &[screen_bind_group_layout, texture_bind_group_layout],
            surface_format,
        );

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 1024 * 32,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let vbuf = VertexBuffer::new(device, RECT_VERTICES, RECT_INDICES);
        Self {
            render_pipeline,
            instance_buffer,
            vbuf,
        }
    }

    pub fn draw(
        &self,
        render_pass: &mut wgpu::RenderPass,
        queue: &wgpu::Queue,
        texture_manager: &crate::texture::TextureManager,
        mut instances: Vec<TextureInstance>,
    ) {
        if instances.is_empty() {
            return;
        }

        instances.sort_by_key(|instance| instance.texture_id);
        let instances_raw: Vec<TextureInstanceRaw> =
            instances.iter().map(|instance| instance.raw()).collect();

        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&instances_raw),
        );
        render_pass.set_pipeline(&self.render_pipeline);
        self.vbuf.set(render_pass);
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        // 'instances' is sorted by texture_id.
        let mut last_texture_id = instances[0].texture_id;
        let mut instance_start = 0;
        for (num, instance) in instances.iter().enumerate() {
            if last_texture_id != instance.texture_id {
                if let Some(texture) = texture_manager.get_texture(last_texture_id) {
                    render_pass.set_bind_group(1, &texture.bind_group, &[]);
                    render_pass.draw_indexed(
                        0..self.vbuf.index_count,
                        0,
                        instance_start..num as u32,
                    );
                }

                instance_start = num as u32;
                last_texture_id = instance.texture_id;
            }
        }
        if let Some(texture) = texture_manager.get_texture(last_texture_id) {
            render_pass.set_bind_group(1, &texture.bind_group, &[]);
            render_pass.draw_indexed(
                0..self.vbuf.index_count,
                0,
                instance_start..instances.len() as u32,
            );
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TextureInstance {
    pub position: UVec3,
    pub scale: UVec2,
    pub texture_id: crate::texture::TextureId,
}

impl TextureInstance {
    fn raw(&self) -> TextureInstanceRaw {
        TextureInstanceRaw {
            position: self.position,
            scale: self.scale,
        }
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TextureInstanceRaw {
    position: UVec3,
    scale: UVec2,
}

impl TextureInstanceRaw {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![2 => Uint32x3, 3 => Uint32x2];

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
        position: Vec3::new(1.0, 1.0, 0.0),
        uv: Vec2::new(1.0, 0.0),
    },
    Vertex {
        position: Vec3::new(0.0, 1.0, 0.0),
        uv: Vec2::new(0.0, 0.0),
    },
    Vertex {
        position: Vec3::new(0.0, 0.0, 0.0),
        uv: Vec2::new(0.0, 1.0),
    },
    Vertex {
        position: Vec3::new(1.0, 0.0, 0.0),
        uv: Vec2::new(1.0, 1.0),
    },
];

const RECT_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

struct VertexBuffer {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
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

        Self {
            vertex_buffer,
            index_buffer,
            index_count: index.len() as u32,
        }
    }

    fn set(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
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
