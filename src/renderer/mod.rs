/// Módulo de renderização GPU
/// wgpu com backend Metal para Apple Silicon

pub mod glyph;

use anyhow::Result;
use wgpu::util::DeviceExt;
use crate::config::{BG_COLOR, CELL_WIDTH, CELL_HEIGHT, PADDING_X, PADDING_Y, CURSOR_COLOR, CURSOR_TEXT_COLOR};
use crate::term::Grid;
use glyph::GlyphCache;

/// Vertex para renderização de células
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
    pub fg_color: [f32; 4],
    pub bg_color: [f32; 4],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Float32x4,
        3 => Float32x4,
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Renderer principal
pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    glyph_cache: GlyphCache,
    bind_group: wgpu::BindGroup,
    pub size: winit::dpi::PhysicalSize<u32>,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    // Estado do cursor
    cursor_visible: bool,
    last_blink: std::time::Instant,
}

impl Renderer {
    pub async fn new(window: std::sync::Arc<winit::window::Window>) -> Result<Self> {
        let size = window.inner_size();

        // Instância wgpu com preferência para Metal
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::METAL,
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

        // Adapter - preferência para GPU de alta performance
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Falha ao obter adapter GPU"))?;

        // Device e Queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("RTerm Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        // Configuração da surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Glyph cache
        let glyph_cache = GlyphCache::new(&device, &queue);

        // Shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // Bind group layout para texture
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Glyph Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Glyph Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&glyph_cache.texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&glyph_cache.sampler),
                },
            ],
        });

        // Pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Buffers iniciais
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            glyph_cache,
            bind_group,
            size,
            vertices: Vec::new(),
            indices: Vec::new(),
            cursor_visible: true,
            last_blink: std::time::Instant::now(),
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Calcula dimensões do grid baseado no tamanho da janela (com padding)
    pub fn grid_dimensions(&self) -> (usize, usize) {
        let usable_width = self.size.width as f32 - (PADDING_X * 2.0);
        let usable_height = self.size.height as f32 - (PADDING_Y * 2.0);
        let cols = (usable_width / CELL_WIDTH) as usize;
        let rows = (usable_height / CELL_HEIGHT) as usize;
        (cols.max(1), rows.max(1))
    }

    /// Atualiza estado de blink do cursor
    fn update_cursor_blink(&mut self) {
        let elapsed = self.last_blink.elapsed().as_millis() as u64;
        if elapsed >= crate::config::CURSOR_BLINK_RATE_MS {
            self.cursor_visible = !self.cursor_visible;
            self.last_blink = std::time::Instant::now();
        }
    }

    /// Renderiza o grid
    pub fn render(&mut self, grid: &Grid) -> Result<()> {
        self.update_cursor_blink();
        
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Constrói vertices para todas as células
        self.build_vertices(grid);

        // Atualiza buffers
        if !self.vertices.is_empty() {
            self.vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            self.index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&self.indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        }

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: BG_COLOR[0] as f64,
                            g: BG_COLOR[1] as f64,
                            b: BG_COLOR[2] as f64,
                            a: BG_COLOR[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if !self.vertices.is_empty() {
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn build_vertices(&mut self, grid: &Grid) {
        self.vertices.clear();
        self.indices.clear();

        let scale_x = 2.0 / self.size.width as f32;
        let scale_y = 2.0 / self.size.height as f32;

        for y in 0..grid.rows {
            for x in 0..grid.cols {
                let cell = grid.get_cell(x, y);
                
                // Skip espaços vazios com background padrão
                if cell.c == ' ' && cell.style.bg == BG_COLOR {
                    continue;
                }

                // Coordenadas em clip space (-1 a 1) com padding
                let px = PADDING_X + x as f32 * CELL_WIDTH;
                let py = PADDING_Y + y as f32 * CELL_HEIGHT;
                
                let x0 = px * scale_x - 1.0;
                let y0 = 1.0 - py * scale_y;
                let x1 = (px + CELL_WIDTH) * scale_x - 1.0;
                let y1 = 1.0 - (py + CELL_HEIGHT) * scale_y;

                // Obtém UV do glyph
                let (fg, bg) = if cell.style.inverse {
                    (cell.style.bg, cell.style.fg)
                } else {
                    (cell.style.fg, cell.style.bg)
                };

                let uv = self.glyph_cache.get_uv(cell.c);
                
                let base = self.vertices.len() as u32;
                
                // 4 vertices por célula
                self.vertices.push(Vertex {
                    position: [x0, y0],
                    tex_coords: [uv.0, uv.1],
                    fg_color: fg,
                    bg_color: bg,
                });
                self.vertices.push(Vertex {
                    position: [x1, y0],
                    tex_coords: [uv.2, uv.1],
                    fg_color: fg,
                    bg_color: bg,
                });
                self.vertices.push(Vertex {
                    position: [x1, y1],
                    tex_coords: [uv.2, uv.3],
                    fg_color: fg,
                    bg_color: bg,
                });
                self.vertices.push(Vertex {
                    position: [x0, y1],
                    tex_coords: [uv.0, uv.3],
                    fg_color: fg,
                    bg_color: bg,
                });

                // 2 triângulos por célula
                self.indices.extend_from_slice(&[
                    base, base + 1, base + 2,
                    base, base + 2, base + 3,
                ]);
            }
        }

        // Cursor (com blink)
        if self.cursor_visible {
            let cx = grid.cursor_x;
            let cy = grid.cursor_y;
            if cx < grid.cols && cy < grid.rows {
                let px = PADDING_X + cx as f32 * CELL_WIDTH;
                let py = PADDING_Y + cy as f32 * CELL_HEIGHT;
                
                let x0 = px * scale_x - 1.0;
                let y0 = 1.0 - py * scale_y;
                let x1 = (px + CELL_WIDTH) * scale_x - 1.0;
                let y1 = 1.0 - (py + CELL_HEIGHT) * scale_y;

                let base = self.vertices.len() as u32;

                // Cursor block elegante com a cor do tema
                self.vertices.push(Vertex {
                    position: [x0, y0],
                    tex_coords: [0.0, 0.0],
                    fg_color: CURSOR_TEXT_COLOR,
                    bg_color: CURSOR_COLOR,
                });
                self.vertices.push(Vertex {
                    position: [x1, y0],
                    tex_coords: [0.0, 0.0],
                    fg_color: CURSOR_TEXT_COLOR,
                    bg_color: CURSOR_COLOR,
                });
                self.vertices.push(Vertex {
                    position: [x1, y1],
                    tex_coords: [0.0, 0.0],
                    fg_color: CURSOR_TEXT_COLOR,
                    bg_color: CURSOR_COLOR,
                });
                self.vertices.push(Vertex {
                    position: [x0, y1],
                    tex_coords: [0.0, 0.0],
                    fg_color: CURSOR_TEXT_COLOR,
                    bg_color: CURSOR_COLOR,
                });

                self.indices.extend_from_slice(&[
                    base, base + 1, base + 2,
                    base, base + 2, base + 3,
                ]);
            }
        }
    }
}
