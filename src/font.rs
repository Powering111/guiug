use std::collections::HashMap;

pub type FontId = u32;

#[derive(Default)]
pub(crate) struct FontInfoManager<'a> {
    last_id: FontId,
    font_infos: HashMap<FontId, FontInfo<'a>>,
}

impl<'a> FontInfoManager<'a> {
    pub fn add_font_info(&mut self, data: &'a [u8]) -> FontId {
        let id = self.last_id;
        let font_info = FontInfo { data };
        self.last_id += 1;
        self.font_infos.entry(id).insert_entry(font_info);

        id
    }
}

pub(crate) struct FontInfo<'a> {
    data: &'a [u8],
}

pub(crate) struct FontManager {
    fonts: HashMap<FontId, Font>,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
}

impl FontManager {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            fonts: HashMap::new(),
            bind_group_layout,
            sampler,
        }
    }

    pub fn load(&mut self, font_infos: &FontInfoManager) {
        for (font_id, font_info) in &font_infos.font_infos {
            let font = Font::from_bytes(font_info.data);
            self.fonts.entry(*font_id).insert_entry(font);
        }
    }

    pub(crate) fn get_font(&self, font_id: FontId) -> Option<&Font> {
        self.fonts.get(&font_id)
    }

    pub(crate) fn get_font_mut(&mut self, font_id: FontId) -> Option<&mut Font> {
        self.fonts.get_mut(&font_id)
    }

    pub(crate) fn clear_cache(&mut self) {
        self.fonts
            .iter_mut()
            .for_each(|(_, font)| font.clear_cache());
    }
}

pub(crate) struct Font {
    inner: fontdue::Font,
    cache: std::collections::HashMap<(char, u16), Option<crate::texture::Texture>>,
}

impl Font {
    fn from_bytes(data: &[u8]) -> Self {
        Self {
            inner: fontdue::Font::from_bytes(data, fontdue::FontSettings::default()).unwrap(),
            cache: HashMap::new(),
        }
    }

    pub fn metrics(&self, character: char, size: u16) -> fontdue::Metrics {
        self.inner.metrics(character, size as f32)
    }

    pub fn prepare_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        character: char,
        size: u16,
    ) {
        self.cache.entry((character, size)).or_insert_with(|| {
            let (metrics, bitmap) = self.inner.rasterize(character, size as f32);

            if metrics.width > 0 && metrics.height > 0 {
                Some(crate::texture::Texture::from_bytes_r8(
                    device,
                    queue,
                    (metrics.width as u32, metrics.height as u32),
                    &bitmap,
                    bind_group_layout,
                    sampler,
                ))
            } else {
                None
            }
        });
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_texture(&self, character: char, size: u16) -> Option<&crate::texture::Texture> {
        self.cache.get(&(character, size)).and_then(|v| v.as_ref())
    }

    pub fn measure_width(&self, text: &str, size: u16) -> i32 {
        let mut res = 0;
        for character in text.chars() {
            let metrics = self.inner.metrics(character, size as f32);
            res += metrics.advance_width as i32;
        }
        res
    }
}
