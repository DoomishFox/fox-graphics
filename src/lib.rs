//! Small GPU drawing backend. The caller owns the device, surface and event loop.
//! Positions are viewport-local logical pixels; CAD world coordinates stay outside this crate.
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Debug)]
pub struct Stroke {
    pub start: [f32; 2],
    pub end: [f32; 2],
    pub color: [f32; 4],
    pub width: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
    edge: f32,
    half_width: f32,
}

pub struct StrokeRenderer {
    pipeline: wgpu::RenderPipeline,
    vertices: wgpu::Buffer,
    capacity: u64,
    count: u32,
}

impl StrokeRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fox strokes"),
            source: wgpu::ShaderSource::Wgsl(include_str!("stroke.wgsl").into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fox strokes"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4, 2 => Float32, 3 => Float32],
                })],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        Self {
            pipeline,
            vertices: buffer(device, 256),
            capacity: 256,
            count: 0,
        }
    }

    /// Convert pixel strokes to clip space. Reuses the buffer until growth is needed.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        strokes: &[Stroke],
        size: [f32; 2],
    ) {
        let mut vertices = Vec::with_capacity(strokes.len() * 6);
        if size[0] > 0.0 && size[1] > 0.0 {
            for stroke in strokes {
                let dx = stroke.end[0] - stroke.start[0];
                let dy = stroke.end[1] - stroke.start[1];
                let length = dx.hypot(dy);
                if !length.is_finite() || length < 0.001 {
                    continue;
                }
                let half = stroke.width.max(0.5) * 0.5;
                let extent = half + 0.75;
                let normal = [-dy / length * extent, dx / length * extent];
                for (point, side) in [
                    (stroke.start, -1.0),
                    (stroke.end, -1.0),
                    (stroke.end, 1.0),
                    (stroke.start, -1.0),
                    (stroke.end, 1.0),
                    (stroke.start, 1.0),
                ] {
                    let x = point[0] + normal[0] * side;
                    let y = point[1] + normal[1] * side;
                    vertices.push(Vertex {
                        position: [2.0 * x / size[0] - 1.0, 1.0 - 2.0 * y / size[1]],
                        color: stroke.color,
                        edge: extent * side,
                        half_width: half,
                    });
                }
            }
        }
        self.count = vertices.len() as u32;
        let bytes = bytemuck::cast_slice(&vertices);
        if bytes.len() as u64 > self.capacity {
            self.capacity = (bytes.len() as u64).next_power_of_two();
            self.vertices = buffer(device, self.capacity);
        }
        if !bytes.is_empty() {
            queue.write_buffer(&self.vertices, 0, bytes);
        }
    }

    /// The caller sets viewport/scissor and chooses attachment/load operations.
    pub fn render(&self, pass: &mut wgpu::RenderPass<'_>) {
        if self.count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vertices.slice(..));
        pass.draw(0..self.count, 0..1);
    }
}

fn buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("fox stroke vertices"),
        size,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
