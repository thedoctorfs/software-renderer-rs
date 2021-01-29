use crate::renderer::{depth_texture::DepthTexture, error::RendererError, Renderer};
use nalgebra_glm::{identity, vec3, Mat4, Vec3};
use std::io::Read;
use wgpu::util::DeviceExt;
use winit::window::Window;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: Vec3,
}
unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Uniforms {
    pub m: Mat4,
    pub v: Mat4,
    pub p: Mat4,
}

unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}

#[repr(C)]
#[derive(Debug)]
pub struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    len: u32,
    model: Mat4,
}

impl Mesh {
    pub fn create_mesh_from(renderer: &Renderer, asset: &Asset) -> Self {
        let vertex_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(asset.vertices.as_slice()),
                usage: wgpu::BufferUsage::VERTEX,
            });
        let index_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[0, 1, 2]),
                usage: wgpu::BufferUsage::INDEX,
            });
        Self {
            vertex_buffer,
            index_buffer,
            len: 3,
            model: identity(),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Asset {
    pub vertices: Vec<Vertex>,
}

impl Default for Asset {
    fn default() -> Self {
        Self {
            vertices: vec![
                Vertex {
                    position: vec3(0.5, -0.5, 0.0),
                    normal: vec3(0.0, 0.0, 1.0),
                    color: vec3(1.0, 0.0, 0.0),
                },
                Vertex {
                    position: vec3(0.0, 0.5, 0.0),
                    normal: vec3(0.0, 0.0, 1.0),
                    color: vec3(1.0, 0.0, 0.0),
                },
                Vertex {
                    position: vec3(-0.5, -0.5, 0.0),
                    normal: vec3(0.0, 0.0, 1.0),
                    color: vec3(1.0, 0.0, 0.0),
                },
            ],
        }
    }
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: 2 * mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float3,
                },
            ],
        }
    }
}

pub struct Pipeline {
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    pub async fn new(renderer: &Renderer) -> Result<Self, RendererError> {
        let (mut spirv_vs_bytes, mut spirv_fs_bytes) = (Vec::new(), Vec::new());
        match glsl_to_spirv::compile(
            include_str!("shaders/shader.vert"),
            glsl_to_spirv::ShaderType::Vertex,
        ) {
            Ok(mut spirv_vs_output) => {
                spirv_vs_output.read_to_end(&mut spirv_vs_bytes).unwrap();
            }
            Err(ref e) => return Err(RendererError::from(e.clone())),
        }
        match glsl_to_spirv::compile(
            include_str!("shaders/shader.frag"),
            glsl_to_spirv::ShaderType::Fragment,
        ) {
            Ok(mut spirv_vs_output) => {
                spirv_vs_output.read_to_end(&mut spirv_fs_bytes).unwrap();
            }
            Err(ref e) => return Err(RendererError::from(e.clone())),
        }
        let vs_module_source = wgpu::util::make_spirv(spirv_vs_bytes.as_slice());
        let fs_module_source = wgpu::util::make_spirv(spirv_fs_bytes.as_slice());
        let vs_module = renderer.device.create_shader_module(vs_module_source);
        let fs_module = renderer.device.create_shader_module(fs_module_source);

        let uniforms = Uniforms {
            m: identity(),
            v: identity(),
            p: identity(),
        };

        let uniform_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            size: (std::mem::size_of::<Uniforms>()) as u64,
            mapped_at_creation: false,
        });

        let uniform_bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: None,
                });

        let uniform_bind_group = renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(..)),
                }],
            });

        let render_pipeline_layout =
            renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&uniform_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            renderer
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&render_pipeline_layout),
                    vertex_stage: wgpu::ProgrammableStageDescriptor {
                        module: &vs_module,
                        entry_point: "main",
                    },
                    fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                        module: &fs_module,
                        entry_point: "main",
                    }),
                    rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: wgpu::CullMode::Back,
                        clamp_depth: false,
                        depth_bias: 0,
                        depth_bias_slope_scale: 0.0,
                        depth_bias_clamp: 0.0,
                    }),
                    color_states: &[wgpu::ColorStateDescriptor {
                        format: renderer.swap_chain_descriptor.format,
                        color_blend: wgpu::BlendDescriptor::REPLACE,
                        alpha_blend: wgpu::BlendDescriptor::REPLACE,
                        write_mask: wgpu::ColorWrite::ALL,
                    }],
                    primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                    depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                        format: DepthTexture::DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilStateDescriptor {
                            front: wgpu::StencilStateFaceDescriptor::IGNORE,
                            back: wgpu::StencilStateFaceDescriptor::IGNORE,
                            read_mask: 0,
                            write_mask: 0,
                        },
                    }),
                    vertex_state: wgpu::VertexStateDescriptor {
                        index_format: wgpu::IndexFormat::Uint32,
                        vertex_buffers: &[Vertex::desc()],
                    },
                    sample_count: 1,
                    sample_mask: !0,
                    alpha_to_coverage_enabled: false,
                });
        Ok(Self {
            uniform_buffer,
            uniform_bind_group,
            render_pipeline,
        })
    }

    pub fn render(&self, mesh: &Mesh, renderer: &mut Renderer) {
        let target = &renderer
            .swap_chain
            .get_current_frame()
            .expect("Could not get next frame texture_view")
            .output
            .view;
        let mut encoder = renderer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &renderer.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: true,
                    }),
                }),
            });
            let uniforms = Uniforms {
                m: mesh.model.clone(),
                v: identity(),
                p: identity(),
            };
            renderer
                .queue
                .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.draw(0..mesh.len, 0..1);
        }
        renderer.queue.submit(std::iter::once(encoder.finish()));
    }
}