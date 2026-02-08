use glam::{IVec2, IVec3, Vec2, Vec4};
use wgpu::util::DeviceExt;

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
    pub position: IVec3,
    pub scale: IVec2,
    pub color: Vec4,
}

impl FlatInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![2 => Sint32x3, 3 => Sint32x2, 4 => Float32x4];
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
    pub position: IVec3,
    pub scale: IVec2,
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
    position: IVec3,
    scale: IVec2,
}

impl TextureInstanceRaw {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![2 => Sint32x3, 3 => Sint32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct TextRenderer {
    render_pipeline: wgpu::RenderPipeline,
    instance_buffer: wgpu::Buffer,
    vbuf: VertexBuffer,
}

impl TextRenderer {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        screen_bind_group_layout: &wgpu::BindGroupLayout,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader/text.wgsl"));
        let render_pipeline = create_render_pipeline(
            device,
            &shader,
            &[Vertex::desc(), TextInstanceRaw::desc()],
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
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        font_manager: &mut crate::font::FontManager,
        mut instances: Vec<TextInstance>,
    ) {
        if instances.is_empty() {
            return;
        }

        instances.sort_by_key(|instance| instance.font_id);

        let mut instances_raw = Vec::new();
        for instance in instances.iter() {
            if let Some(font) = font_manager.get_font(instance.font_id) {
                let mut pos = instance.position;
                for character in instance.text.chars() {
                    let metrics = font.metrics(character, instance.size);
                    instances_raw.push(TextInstanceRaw {
                        position: pos
                            + IVec3::new(metrics.xmin, -metrics.ymin - metrics.height as i32, 0),
                        scale: IVec2::new(metrics.width as i32, metrics.height as i32),
                        color: instance.color,
                    });

                    pos.x += metrics.advance_width as i32;
                }
            }
        }
        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&instances_raw),
        );

        render_pass.set_pipeline(&self.render_pipeline);
        self.vbuf.set(render_pass);
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        let mut num = 0;
        for instance in instances.iter() {
            let bind_group_layout = font_manager.bind_group_layout.clone();
            let sampler = font_manager.sampler.clone();
            if let Some(font) = font_manager.get_font(instance.font_id) {
                for character in instance.text.chars() {
                    if let Some(texture) = font.get_texture(
                        device,
                        queue,
                        &bind_group_layout,
                        &sampler,
                        character,
                        instance.size,
                    ) {
                        render_pass.set_bind_group(1, &texture.bind_group, &[]);
                        render_pass.draw_indexed(
                            0..self.vbuf.index_count,
                            0,
                            num as u32..(num + 1) as u32,
                        );
                    }
                    num += 1;
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TextInstance {
    pub text: String,
    pub position: IVec3,
    pub size: u16,
    pub font_id: crate::font::FontId,
    pub color: Vec4,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TextInstanceRaw {
    position: IVec3,
    scale: IVec2,
    color: Vec4,
}

impl TextInstanceRaw {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![2 => Sint32x3, 3 => Sint32x2, 4 => Float32x4];

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
        position: Vec2::new(1.0, 0.0),
        uv: Vec2::new(1.0, 0.0),
    },
    Vertex {
        position: Vec2::new(0.0, 0.0),
        uv: Vec2::new(0.0, 0.0),
    },
    Vertex {
        position: Vec2::new(0.0, -1.0),
        uv: Vec2::new(0.0, 1.0),
    },
    Vertex {
        position: Vec2::new(1.0, -1.0),
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

pub(crate) struct Uniform {
    buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl Uniform {
    pub(crate) fn new(device: &wgpu::Device, size: u64) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });
        Self {
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub(crate) fn write(&self, queue: &wgpu::Queue, data: &[u8]) {
        queue.write_buffer(&self.buffer, 0, data);
    }

    pub(crate) fn set(&self, render_pass: &mut wgpu::RenderPass, bind_group_index: u32) {
        render_pass.set_bind_group(bind_group_index, &self.bind_group, &[]);
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
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
