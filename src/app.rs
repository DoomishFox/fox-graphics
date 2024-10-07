use crate::data::{ScreenSize, Vertex};
use crate::text::{GlyphAtlas, Text2D};
use crate::camera::{Camera, Camera3D, CameraController, CameraUniform};
use crate::camera::fps_camera::CameraLegacy;

use crate::pipeline::NamedPipeline;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window, dpi::PhysicalSize,
};

use wgpu::util::DeviceExt;

pub struct AppSkeleton {
    pub window: Window,
    pub event_loop: EventLoop<()>,
    pub _instance: wgpu::Instance,
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub screen_size: ScreenSize,
}

pub trait Application: 'static + Sized {
    fn optional_features() -> wgpu::Features {
        wgpu::Features::empty()
    }
    fn required_features() -> wgpu::Features {
        wgpu::Features::empty()
    }
    fn required_downlevel_capabilities() -> wgpu::DownlevelCapabilities {
        wgpu::DownlevelCapabilities {
            flags: wgpu::DownlevelFlags::empty(),
            shader_model: wgpu::ShaderModel::Sm5,
            ..wgpu::DownlevelCapabilities::default()
        }
    }
    fn required_limits() -> wgpu::Limits {
        wgpu::Limits::default()
    }
    fn input(&mut self, event: &WindowEvent) -> bool;
    fn update(&mut self, queue: &wgpu::Queue);
    fn render(
        &mut self,
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), wgpu::SurfaceError>;
}


pub struct AppBuilder<'a, S> {
    pub skeleton: AppSkeleton,
    pub state: Option<S>,

    pub camera3d: Option<Camera3D<CameraLegacy>>,
    pub text2d: Option<Text2D>,

    pub named_pipelines: Vec<NamedPipeline<'a>>,
}

impl<'a, S> AppBuilder<'a, S> {
    pub fn state(mut self, state: S) -> Self {
        self.state = Some(state);
        self
    }

    pub fn add_pipeline(mut self, pipeline: NamedPipeline<'a>) -> Self {
        // you need a skeleton and a camera3d (or text2d) to even create a NamedPipeline so i dont need
        // to explicitly check.
        //let skeleton = self.skeleton.as_ref().expect("cannot add pipeline with no skeleton!");
        //let camera = self.camera3d.as_ref().expect("cannot add pipeline with no camera!");
        self.named_pipelines.push(pipeline);
        self
    }

    pub fn add_camera3d(mut self, camera: CameraLegacy, controller: CameraController) -> Self {
        //let skeleton = self.skeleton.as_ref().expect("cannot add camera3d without skeleton!");
        let camera = CameraLegacy {
            aspect: self.skeleton.config.width as f32 / self.skeleton.config.height as f32,
            ..camera
        };

        let mut uniform = CameraUniform::new();
        uniform.update_view_proj(&camera);

        let buffer = self.skeleton.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let bind_group_layout = self.skeleton.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
                label: Some("camera_bind_group_layout"),
            }
        );

        let bind_group = self.skeleton.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }
                ],
                label: Some("camera_bind_group"),
            }
        );
        
        self.camera3d = Some(Camera3D {
            camera,
            uniform,
            buffer,
            bind_group_layout,
            bind_group,
            controller,
        });
        self
    }

    pub fn add_text2d(mut self, glyph_atlas: GlyphAtlas) -> Self {
        //let skeleton = self.skeleton.as_ref().expect("cannot add text2d without skeleton!");
        let text_vertices = vec![
            Vertex::at(0.0,0.0,0.0),
            Vertex::at(1.0,1.0,0.0),
            Vertex::at(0.0,1.0,0.0),
            Vertex::at(1.0,0.0,0.0),
            Vertex::at(1.0,1.0,0.0),
            Vertex::at(0.0,0.0,0.0),
        ];
        let vertex_buffer = self.skeleton.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Text Vertex Buffer"),
                contents: bytemuck::cast_slice(text_vertices.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let bind_group_layout = self.skeleton.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry { // screen metadata
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry { // text metadata
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer  {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None
                        },
                        count: None,
                    }
                ],
                label: Some("text_bind_group_layout"),
            }
        );
        let screen_uniform_buffer = self.skeleton.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Screen Metadata Buffer"),
                contents: bytemuck::cast_slice(self.skeleton.screen_size.size()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        self.text2d = Some(Text2D {
            vertex_buffer,
            bind_group_layout,
            glyph_atlas,
            screen_uniform_buffer,
        });
        self
    }

    pub fn build(self) -> (AppSkeleton, App<'a, S>) {
        (
            self.skeleton,
            App {
                state: self.state.expect("failed to build: no app state!"),

                camera3d: self.camera3d,
                text2d: self.text2d,

                named_pipelines: self.named_pipelines,
            }
        )
    }
}


pub struct App<'a, S> {
    // owns the skeleton and should then ensure all the lifetimes are correct
    //skeleton: graphics::AppSkeleton,
    pub state: S,

    pub camera3d: Option<Camera3D<CameraLegacy>>,
    pub text2d: Option<Text2D>,

    pub named_pipelines: Vec<NamedPipeline<'a>>,
}

impl<'a, S> App<'_, S> {
    //fn builder() -> FigBuilder<'a> {
    //    FigBuilder::default()
    //}
    pub fn for_skeleton(skeleton: AppSkeleton) -> AppBuilder<'a, S> {
        AppBuilder {
            skeleton: skeleton,
            state: None,
            camera3d: None,
            text2d: None,
            named_pipelines: Vec::<NamedPipeline>::default(),
        }
    }
}