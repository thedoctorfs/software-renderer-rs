use winit::window::Window;
use nalgebra_glm::*;
use std::collections::HashSet;
use std::convert::TryInto;
use crate::graphics::error::GraphicsError;
use crate::graphics::regular::{Vertex, Uniforms};

pub mod regular;
pub mod ui;
pub mod texture;
pub mod error;

type Result<T> = std::result::Result<T, GraphicsError>;

pub fn ensure_unique_provoking_vertices(vertices: &[[f32; 3]], indices: &[u32]) -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut new_vertices= vertices.to_vec();
    let mut new_indices = indices.to_vec();
    let mut provs_used: HashSet<u32> = HashSet::new();
    for face in indices.chunks(3).enumerate() {
        // first vertex of face is a provoking vertex
        if provs_used.contains(&face.1[0]) {
            new_vertices.push(vertices[face.1[0] as usize].clone());
            new_indices[&face.0 * 3] = new_vertices.len() as u32 - 1;
        } else {
            provs_used.insert(face.1[0]);
        }
    }
    (new_vertices, new_indices)
}

pub fn make_mesh_from_flat_obj(vertices_flat: &[f32], indices: &[u32], in_color: &[f32; 3]) -> Mesh<Vertex> {
    let mut vertices: Vec<Vertex> = vertices_flat.chunks(3).map(|v| Vertex { position: [v[0], v[1], v[2]], normal: [0.0, 0.0, 0.0], color: *in_color }).collect();
    let mut new_indices: Vec<u32> = Vec::new();
    let mut used_as_provoking: HashSet<u32> = HashSet::new();
    for face in indices.chunks(3) {
        if used_as_provoking.contains(&face[0]) {
            vertices.push(vertices[face[0] as usize].clone());
            new_indices.extend([vertices.len() as u32 - 1, face[1], face[2]].to_vec());
        } else {
            used_as_provoking.insert(face[0]);
            new_indices.extend(face);
        }
    }
    for face in new_indices.chunks(3) {
        let n = create_normal([vertices[face[0] as usize].position, vertices[face[1] as usize].position, vertices[face[2] as usize].position]);
        vertices[face[0] as usize].normal = n;
    }
    let mesh = Mesh { vertices, indices: new_indices };
    mesh
}

fn create_normal(in_positions: [[f32; 3]; 3]) -> [f32; 3] {
    triangle_normal(&make_vec3(&in_positions[0]), &make_vec3(&in_positions[1]), &make_vec3(&in_positions[2])).as_slice().try_into().unwrap()
}

pub fn enhance_provoking_vertices(vertices: &[[f32; 3]], indices: &[u32]) -> Vec<Vertex> {
    let mut mesh_vertices: Vec<Vertex> = vertices.iter().map(|v| Vertex { position: *v, normal: [0.0, 1.0, 0.0], color: [1.0, 0.0, 0.0] } ).collect();
    for face in indices.chunks(3) {
        let edge_0: Vec3 = make_vec3(&vertices[face[2] as usize]) - make_vec3(&vertices[face[0] as usize]);
        let edge_1: Vec3 = make_vec3(&vertices[face[1] as usize]) - make_vec3(&vertices[face[0] as usize]);
        let n: Vec3 = cross(&edge_1, &edge_0).normalize();
        mesh_vertices[face[0] as usize].normal = n.as_slice().try_into().unwrap();
    }
    mesh_vertices
}

pub fn enhance_provoking_vertices2(mut mesh: Mesh<Vertex>) -> Mesh<Vertex> {
    for face in mesh. indices.chunks(3) {
        let edge_0: Vec3 = make_vec3(&mesh.vertices[face[2] as usize].position) - make_vec3(&mesh.vertices[face[0] as usize].position);
        let edge_1: Vec3 = make_vec3(&mesh.vertices[face[1] as usize].position) - make_vec3(&mesh.vertices[face[0] as usize].position);
        let n: Vec3 = cross(&edge_1, &edge_0).normalize();
        mesh.vertices[face[0] as usize].normal = n.as_slice().try_into().unwrap();
    }
    mesh
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Instance {
    model: Mat4,
}

pub struct Text
{
    pub pos: (f32, f32),
    pub text: String,
    pub font_size: f32,
    pub color: [u8; 4],
}

pub struct Mesh<T> {
    pub vertices: Vec<T>,
    pub indices: Vec<u32>,
}

pub struct Drawable {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_buffer_len: u32,
}

unsafe impl bytemuck::Pod for Instance {}
unsafe impl bytemuck::Zeroable for Instance {}


pub struct Graphics {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_descriptor: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    ui_renderer: ui::UIRenderer,
    renderer: regular::Renderer,
    depth_texture: texture::Texture,
    glyph_brush: wgpu_glyph::GlyphBrush<()>,
    window_size: winit::dpi::PhysicalSize<u32>,
}

impl Graphics {
    pub fn build_glyph_brush(device: &wgpu::Device, texture_format: wgpu::TextureFormat) -> wgpu_glyph::GlyphBrush<()> {
        let font = wgpu_glyph::ab_glyph::FontArc::try_from_slice(include_bytes!("../JetBrainsMono-Regular.ttf")).expect("Can not load font");
        let glyph_brush = wgpu_glyph::GlyphBrushBuilder::using_font(font).build(&device, texture_format);
        glyph_brush
    }

    pub async fn new(window: &Window, ui_mesh: Mesh<ui::UIVertex>) -> Result<Self> {
        // from here device creation and surface swapchain
        let surface =  wgpu::Surface::create(window);
        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface) }, wgpu::BackendBit::PRIMARY).await;
        let adapter = match adapter {
            Some(adapter) => adapter,
            None => { return Err(GraphicsError::RequestAdapter); },
        };
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor
        { extensions: wgpu::Extensions { anisotropic_filtering: false, }, limits: Default::default(), }).await;
        let sc_descriptor = wgpu::SwapChainDescriptor{
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_descriptor);
        let depth_texture = texture::Texture::create_depth_texture(&device, &sc_descriptor);
        let renderer= regular::Renderer::new(&device, &sc_descriptor).await?;
        let ui_renderer = ui::UIRenderer::new(&device, &sc_descriptor, &queue, ui_mesh).await?;
        let glyph_brush = Graphics::build_glyph_brush(&device, wgpu::TextureFormat::Bgra8UnormSrgb);

        Ok(Self {
            surface,
            queue,
            sc_descriptor,
            swap_chain,
            device,
            renderer,
            ui_renderer,
            glyph_brush,
            depth_texture,
            window_size: window.inner_size(),
        })
    }

    pub async fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.window_size = size;
        self.sc_descriptor.width = size.width;
        self.sc_descriptor.height = size.height;
        self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.sc_descriptor);
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_descriptor);
    }

    pub fn create_drawable_from_mesh(&mut self, mesh: &Mesh<Vertex>) -> usize {
        let vertex_buffer = self.device.create_buffer_with_data(bytemuck::cast_slice(&mesh.vertices), wgpu::BufferUsage::VERTEX);
        let index_buffer = self.device.create_buffer_with_data(bytemuck::cast_slice(&mesh.indices), wgpu::BufferUsage::INDEX);
        self.renderer.drawables.push(Drawable { vertex_buffer, index_buffer, index_buffer_len: mesh.indices.len() as u32, });
        self.renderer.drawables.len() - 1
    }

    pub fn create_drawable_from_mesh2(&mut self, mesh: &Mesh<Vertex>) -> Drawable {
        let vertex_buffer = self.device.create_buffer_with_data(bytemuck::cast_slice(&mesh.vertices), wgpu::BufferUsage::VERTEX);
        let index_buffer = self.device.create_buffer_with_data(bytemuck::cast_slice(&mesh.indices), wgpu::BufferUsage::INDEX);
        Drawable { vertex_buffer, index_buffer, index_buffer_len: mesh.indices.len() as u32, }
    }

    pub fn update(&mut self, model_player: Mat4, model_terrain: Mat4, model_axis: Mat4) {
        let instances = [Instance { model: model_player }, Instance { model: model_terrain }, Instance {model: model_axis }];
        let buffer = self.device.create_buffer_with_data(bytemuck::cast_slice(&instances), wgpu::BufferUsage::COPY_SRC);
        let mut encoder =
            self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_buffer_to_buffer(&buffer, 0, &self.renderer.instance_buffer, 0,
                                      std::mem::size_of_val(&instances) as wgpu::BufferAddress);
        self.queue.submit(&[encoder.finish()]);
    }

    pub async fn render(&mut self, view: Mat4, render_ui: bool, ui_mesh: Option<(Mesh<ui::UIVertex>, Vec<Text>)>) {
        let projection = perspective(self.sc_descriptor.width as f32 / self.sc_descriptor.height as f32,45.0, 0.1, 100.0);
        let uniforms = Uniforms { projection: projection, view: view, };
        let buffer = self.device.create_buffer_with_data(bytemuck::cast_slice(&[uniforms]), wgpu::BufferUsage::COPY_SRC);
        let frame = self.swap_chain.get_next_texture().expect("failed to get next texture");
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
        });
        encoder.copy_buffer_to_buffer(&buffer, 0, &self.renderer.uniform_buffer, 0, std::mem::size_of_val(&uniforms) as u64);
        {
            let mut diffuse_scene_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }
                    }
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture.view,
                    depth_load_op: wgpu::LoadOp::Clear,
                    depth_store_op: wgpu::StoreOp::Store,
                    clear_depth: 1.0,
                    stencil_load_op: wgpu::LoadOp::Clear,
                    stencil_store_op: wgpu::StoreOp::Store,
                    clear_stencil: 0,
                }),
            });
            diffuse_scene_pass.set_pipeline(&self.renderer.render_pipeline);

            diffuse_scene_pass.set_vertex_buffer(0, &self.renderer.drawables[0].vertex_buffer, 0, 0);
            diffuse_scene_pass.set_index_buffer(&self.renderer.drawables[0].index_buffer, 0, 0);
            diffuse_scene_pass.set_bind_group(0, &self.renderer.uniform_bind_group, &[]);
            diffuse_scene_pass.draw_indexed(0..self.renderer.drawables[0].index_buffer_len, 0, 0..1);

            diffuse_scene_pass.set_vertex_buffer(0, &self.renderer.drawables[1].vertex_buffer, 0, 0);
            diffuse_scene_pass.set_index_buffer(&self.renderer.drawables[1].index_buffer, 0, 0);
            diffuse_scene_pass.set_bind_group(0, &self.renderer.uniform_bind_group, &[]);
            diffuse_scene_pass.draw_indexed(0..self.renderer.drawables[1].index_buffer_len, 0, 1..2);
        }
        // far and near plane are not used in UI rendering
        let ui_uniforms = ui::UIUniforms { projection: ortho(0.0, self.sc_descriptor.width as f32, 0.0, self.sc_descriptor.height as f32, -1.0, 1.0) };
        let buffer = self.device.create_buffer_with_data(bytemuck::cast_slice(&[ui_uniforms]), wgpu::BufferUsage::COPY_SRC);
        encoder.copy_buffer_to_buffer(&buffer, 0, &self.ui_renderer.ui_uniform_buffer, 0, std::mem::size_of_val(&ui_uniforms) as u64);
        if render_ui
        {
            let mut ui_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        load_op: wgpu::LoadOp::Load,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }
                    }
                ],
                depth_stencil_attachment: None
            });
            if let Some(ui_mesh) = ui_mesh {
                self.ui_renderer.ui_drawable = Drawable { vertex_buffer: self.device.create_buffer_with_data(bytemuck::cast_slice(&ui_mesh.0.vertices), wgpu::BufferUsage::VERTEX),
                    index_buffer: self.device.create_buffer_with_data(bytemuck::cast_slice(&ui_mesh.0.indices), wgpu::BufferUsage::INDEX),
                    index_buffer_len: ui_mesh.0.indices.len() as u32 };

                for text in &ui_mesh.1 {
                    let section = wgpu_glyph::Section {
                        screen_position: text.pos,
                        text: vec![wgpu_glyph::Text::new(&text.text.as_str()).with_color([1.0, 0.0, 0.0, 1.0]).with_scale(wgpu_glyph::ab_glyph::PxScale{ x: text.font_size, y: text.font_size })], ..wgpu_glyph::Section::default()
                    };
                    self.glyph_brush.queue(section);
                }
            }
            ui_pass.set_pipeline(&self.ui_renderer.ui_render_pipeline);
            ui_pass.set_vertex_buffer(0, &self.ui_renderer.ui_drawable.vertex_buffer, 0, 0);
            ui_pass.set_index_buffer(&self.ui_renderer.ui_drawable.index_buffer, 0, 0);
            ui_pass.set_bind_group(0, &self.ui_renderer.ui_uniform_bind_group, &[]);
            ui_pass.set_bind_group(1, &self.ui_renderer.ui_texture_bind_group, &[]);
            ui_pass.draw_indexed(0..self.ui_renderer.ui_drawable.index_buffer_len, 0, 0..1);
        }

        self.glyph_brush.draw_queued(&self.device, &mut encoder, &frame.view, self.sc_descriptor.width, self.sc_descriptor.height,).expect("Cannot draw glyph_brush");
        self.queue.submit(&[encoder.finish()]);
    }
}