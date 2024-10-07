use crate::app;
use crate::data::Vertex;
use crate::app::{AppBuilder, AppSkeleton, Application};

pub struct NamedPipeline<'a> {
    pub name: &'a str,
    pub pipeline: wgpu::RenderPipeline,
    // store some information about what bind groups im using and index they are at
    // this is necessary in the render function to tell the encoder the right bind groups
}

impl NamedPipeline<'_> {
    pub fn named_for<'a, S>(name: &'a str, app: AppBuilder<'a, S>) -> PipelineBuilder<'a, S> {
        // device: &'a wgpu::Device
        PipelineBuilder {
            app,
            name,
            shader: None,
            pipeline_layout: None,
            vertex_buffer_layout: Some(Vertex::desc()),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            front_face_format: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
        }
    }
}

pub struct PipelineBuilder<'a, S> {
    //device: &'a wgpu::Device,
    pub app: AppBuilder<'a, S>,
    pub name: &'a str,
    pub shader: Option<wgpu::ShaderModule>,
    pub pipeline_layout: Option<wgpu::PipelineLayout>,
    pub vertex_buffer_layout: Option<wgpu::VertexBufferLayout<'static>>,
    pub primitive_topology: wgpu::PrimitiveTopology,
    pub front_face_format: wgpu::FrontFace,
    pub cull_mode: Option<wgpu::Face>,
    pub polygon_mode: wgpu::PolygonMode,
}

impl<'a, S> PipelineBuilder<'a, S> {
    pub fn with_shader(mut self, source: wgpu::ShaderSource) -> Self {
        let device = &self.app.skeleton.device;
        self.shader = Some(device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(format!("{} shader", self.name).as_str()),
            source: source.into(),
        }));
        self
    }

    pub fn with_vertex_buffer_layout(mut self, vertex_buffer_layout: wgpu::VertexBufferLayout<'static>) -> Self {
        self.vertex_buffer_layout = Some(vertex_buffer_layout);
        self
    }

    pub fn with_primitive_topology(mut self, primitive_topology: wgpu::PrimitiveTopology) -> Self {
        self.primitive_topology = primitive_topology;
        self
    }

    pub fn with_front_face_format(mut self, front_face_format: wgpu::FrontFace) -> Self {
        self.front_face_format = front_face_format;
        self
    }

    pub fn with_cull_mode(mut self, cull_mode: Option<wgpu::Face>) -> Self {
        self.cull_mode = cull_mode;
        self
    }

    pub fn with_polygon_mode(mut self, polygon_mode: wgpu::PolygonMode) -> Self {
        self.polygon_mode = polygon_mode;
        self
    }

    pub fn layout_for_camera3d(mut self) -> Self {
        let camera = self.app.camera3d.as_ref().expect("cannot layout camera3d: no camera3d in use!");
        let device = &self.app.skeleton.device;
        assert!(self.pipeline_layout.is_none(), "cannot use camera3d, pipeline layout is already defined! (hint: did you already call a layout_for_*() function?)");
        self.pipeline_layout = Some(device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(format!("{} render pipeline layout", self.name).as_str()),
            bind_group_layouts: &[
                &camera.bind_group_layout,
            ],
            push_constant_ranges: &[],
        }));
        self
    }

    pub fn layout_for_text2d(mut self) -> Self {
        let text = self.app.text2d.as_ref().expect("cannot layout text2d: no text2d in use!");
        let device = &self.app.skeleton.device;
        assert!(self.pipeline_layout.is_none(), "cannot use camera3d, pipeline layout is already defined! (hint: did you already call a layout_for_*() function?)");
        self.pipeline_layout = Some(device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(format!("{} render pipeline layout", self.name).as_str()),
            bind_group_layouts: &[
                &text.bind_group_layout,
                &text.glyph_atlas.glyph_bind_group_layout,
            ],
            push_constant_ranges: &[],
        }));
        self
    }

    pub fn use_custom_bind_group_layouts(mut self, layouts: &[&wgpu::BindGroupLayout]) -> Self {
        let device = &self.app.skeleton.device;
        self.pipeline_layout = Some(device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(format!("{} render pipeline layout", self.name).as_str()),
            bind_group_layouts: layouts,
            push_constant_ranges: &[],
        }));
        self
    }

    pub fn build(self) -> (AppBuilder<'a, S>, NamedPipeline<'a>) {
        let device = &self.app.skeleton.device;
        let config = &self.app.skeleton.config;
        let shader = &self.shader.expect(format!("cannot build {} pipeline without shader!", self.name).as_str());
        let vertex_buffer_layout = self.vertex_buffer_layout.expect(format!("cannot build {} pipeline without vertex buffer layout!", self.name).as_str());
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(format!("{} render pipeline", self.name).as_str()),
            layout: self.pipeline_layout.as_ref(),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: "vs_main",
                buffers: &[
                    vertex_buffer_layout
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: self.primitive_topology,
                strip_index_format: None,
                front_face: self.front_face_format,
                cull_mode: self.cull_mode,
                polygon_mode: self.polygon_mode,
                // needs Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // needs Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        (
            self.app,
            NamedPipeline {
                name: self.name,
                pipeline: render_pipeline,
            }
        )
    }
}