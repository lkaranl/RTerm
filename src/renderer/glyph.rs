/// Glyph Cache - Texture atlas para caracteres
/// Rasteriza fontes com fontdue

use std::collections::HashMap;
use crate::config::{FONT_DATA, FONT_SIZE};

/// Cache de glyphs com texture atlas
pub struct GlyphCache {
    font: fontdue::Font,
    cache: HashMap<char, (f32, f32, f32, f32)>, // UV coords
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    atlas_data: Vec<u8>,
    atlas_size: u32,
    next_x: u32,
    next_y: u32,
    row_height: u32,
}

impl GlyphCache {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        // Carrega fonte
        let font = fontdue::Font::from_bytes(FONT_DATA, fontdue::FontSettings::default())
            .expect("Falha ao carregar fonte");

        let atlas_size = 1024u32;
        let atlas_data = vec![0u8; (atlas_size * atlas_size * 4) as usize];

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Glyph Atlas"),
            size: wgpu::Extent3d {
                width: atlas_size,
                height: atlas_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let mut cache = Self {
            font,
            cache: HashMap::new(),
            texture,
            texture_view,
            sampler,
            atlas_data,
            atlas_size,
            next_x: 0,
            next_y: 0,
            row_height: 0,
        };

        // Pre-rasteriza ASCII printable
        for c in 32u8..127 {
            cache.rasterize(c as char, queue);
        }

        cache
    }

    /// Obtém UV coords para um caractere
    pub fn get_uv(&self, c: char) -> (f32, f32, f32, f32) {
        self.cache.get(&c).copied().unwrap_or((0.0, 0.0, 0.0, 0.0))
    }

    /// Rasteriza um caractere e adiciona ao atlas
    fn rasterize(&mut self, c: char, queue: &wgpu::Queue) {
        let (metrics, bitmap) = self.font.rasterize(c, FONT_SIZE);
        
        if metrics.width == 0 || metrics.height == 0 {
            self.cache.insert(c, (0.0, 0.0, 0.0, 0.0));
            return;
        }

        let w = metrics.width as u32;
        let h = metrics.height as u32;

        // Próxima linha se não couber
        if self.next_x + w >= self.atlas_size {
            self.next_x = 0;
            self.next_y += self.row_height + 1;
            self.row_height = 0;
        }

        if self.next_y + h >= self.atlas_size {
            // Atlas cheio, ignora
            self.cache.insert(c, (0.0, 0.0, 0.0, 0.0));
            return;
        }

        // Copia bitmap para atlas (convertendo grayscale para RGBA)
        let x = self.next_x;
        let y = self.next_y;

        for row in 0..h {
            for col in 0..w {
                let src_idx = (row * w + col) as usize;
                let dst_idx = ((y + row) * self.atlas_size + (x + col)) as usize * 4;
                
                if src_idx < bitmap.len() && dst_idx + 3 < self.atlas_data.len() {
                    let alpha = bitmap[src_idx];
                    self.atlas_data[dst_idx] = 255;     // R
                    self.atlas_data[dst_idx + 1] = 255; // G
                    self.atlas_data[dst_idx + 2] = 255; // B
                    self.atlas_data[dst_idx + 3] = alpha; // A
                }
            }
        }

        // Atualiza texture
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            &self.extract_region(x, y, w, h),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(w * 4),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );

        // Calcula UV coords
        let u0 = x as f32 / self.atlas_size as f32;
        let v0 = y as f32 / self.atlas_size as f32;
        let u1 = (x + w) as f32 / self.atlas_size as f32;
        let v1 = (y + h) as f32 / self.atlas_size as f32;

        self.cache.insert(c, (u0, v0, u1, v1));

        self.next_x += w + 1;
        self.row_height = self.row_height.max(h);
    }

    fn extract_region(&self, x: u32, y: u32, w: u32, h: u32) -> Vec<u8> {
        let mut data = Vec::with_capacity((w * h * 4) as usize);
        
        for row in y..(y + h) {
            for col in x..(x + w) {
                let idx = (row * self.atlas_size + col) as usize * 4;
                data.extend_from_slice(&self.atlas_data[idx..idx + 4]);
            }
        }
        
        data
    }
}
