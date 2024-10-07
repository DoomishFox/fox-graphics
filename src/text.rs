use crate::AppSkeleton;

pub struct GlyphAtlas {
    pub glyph_bind_group_layout: wgpu::BindGroupLayout,
    pub glyph_bind_group: wgpu::BindGroup,
}

pub struct Text2D {
    pub vertex_buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    //pub bind_group: wgpu::BindGroup,
    pub glyph_atlas: GlyphAtlas,
    pub screen_uniform_buffer: wgpu::Buffer,
}

impl AppSkeleton {
    pub fn create_glyph_atlas(&self, path: &str) -> GlyphAtlas {
        let atlas_bytes: Vec<u8> = std::fs::read(path)
            .unwrap().iter()
            .map(|v| match v { 0 => 0, _ => 255 })
            .collect();

        let atlas_size = wgpu::Extent3d {
            width: 160,
            height: 144,
            depth_or_array_layers: 1,
        };

        let atlas_texture = self.device.create_texture(
            &wgpu::TextureDescriptor {
                size: atlas_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some(format!("Glyph Atlas - {}", path).as_str()),
                view_formats: &[]
            }
        );

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &atlas_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &atlas_bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(160 * 4),
                rows_per_image: Some(144),
            },
            atlas_size,
        );

        let atlas_texture_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let atlas_sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let glyph_bind_group_layout = self.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            //sample_type: wgpu::TextureSampleType::Uint,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("Glyph Atlas Bind Group Layout"),
            }
        );

        let glyph_bind_group = self.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &glyph_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&atlas_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&atlas_sampler),
                    }
                ],
                label: Some("Glyph Atlas Bind Group"),
            }
        );

        GlyphAtlas {
            glyph_bind_group,
            glyph_bind_group_layout,
        }
    }

    /*
    pub fn create_text2d_for_atlas(&self, glyph_atlas: GlyphAtlas) -> Text2D {
        let text_vertices = vec![
            Vertex::at(0.0,0.0,0.0),
            Vertex::at(1.0,1.0,0.0),
            Vertex::at(0.0,1.0,0.0),
            Vertex::at(1.0,0.0,0.0),
            Vertex::at(1.0,1.0,0.0),
            Vertex::at(0.0,0.0,0.0),
        ];
        let vertex_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Text Vertex Buffer"),
                contents: bytemuck::cast_slice(text_vertices.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let screen_uniform_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Screen Metadata Buffer"),
                contents: bytemuck::cast_slice(self.screen_size.size()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        Text2D {
            vertex_buffer,
            glyph_atlas,
            screen_uniform_buffer,
        }
    }
    */
}