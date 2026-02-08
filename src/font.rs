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
}

pub(crate) struct Font {
    inner: fontdue::Font,
}

impl core::ops::Deref for Font {
    type Target = fontdue::Font;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Font {
    fn from_bytes(data: &[u8]) -> Self {
        Self {
            inner: fontdue::Font::from_bytes(data, fontdue::FontSettings::default()).unwrap(),
        }
    }
}
