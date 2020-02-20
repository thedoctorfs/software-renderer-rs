use software_renderer_rs::*;
use std::fs::File;
use std::io::BufReader;
use obj::*;
use std::time::{Duration, Instant};

mod vec;
use vec::{Vec2, Vec3, Vec4};
mod mat;
use mat::{Mat4};
use image::RgbImage;

pub struct Vertex {
    pub v: Vec3<f32>,
    pub n: Vec3<f32>,
    pub t: Vec2<f32>,
}

pub struct VertexOut {
    pub v: Vec4<f32>,
    pub n: Vec3<f32>,
    pub t: Vec2<f32>,
}

fn viewport(x: f32, y: f32, width: f32, height: f32, depth: f32) -> Mat4<f32> {
    Mat4(
        width / 2.0, 0.0, 0.0, x + width / 2.0,
        0.0, height / 2.0, 0.0, y + height / 2.0,
        0.0, 0.0, depth / 2.0, depth / 2.0,
        0.0, 0.0 , 0.0, 1.0)
}

pub trait Vary {
    fn vary(var0: &Self, var1: &Self, var2: &Self, bc: &Vec3<f32>) -> Self;
}

#[derive(Copy, Clone)]
struct Varyings {
    n: Vec3<f32>,
    t: Vec2<f32>,
}

impl Vary for Varyings {
    fn vary(var0: &Self, var1: &Self, var2: &Self, bc: &Vec3<f32>) -> Self {
        Varyings {
            n: (var0.n * bc.x + var1.n * bc.y + var2.n * bc.z).normalize(),
            t: (var0.t * bc.x + var1.t * bc.y + var2.t * bc.z),
        }
    }
}

pub trait Shader<V: Vary> {
    fn vertex(&self, in_v: &Vec3<f32>, var: &V) -> (Vec4<f32>, V);
    fn fragment(&self, in_f: &Vec2<f32>, var: V) -> Option<Color>;
}

struct BasicShader<'a>
{
    mat: &'a Mat4<f32>,
    tex: &'a RgbImage,
    light_direction: Vec3<f32>,
}

impl<'a> Shader<Varyings> for BasicShader<'a> {
    fn vertex(&self, in_v: &Vec3<f32>, var: &Varyings) -> (Vec4<f32>, Varyings) {
        (*self.mat * Vec4::new(in_v.x, in_v.y, in_v.z, 1.0),
         *var)
    }
    fn fragment(&self, in_f: &Vec2<f32>, var: Varyings) -> Option<Color> {
        let pixel = self.tex.get_pixel((var.t.x * self.tex.width() as f32) as u32, self.tex.height() - 1 - (var.t.y * self.tex.height() as f32) as u32);
        let r = pixel[0]; let g = pixel[1]; let b = pixel[2];
        let out_color = Color{ r: (r as f32) as u8, g: (g as f32) as u8, b: (b as f32) as u8, a: 255 };
        Some(out_color)
    }
}

fn draw_line(v0: Vec3<f32>, v1: Vec3<f32>, color: Color, canvas: &mut Canvas) {
    let mut steep = false;
    let mut x0 = v0.x;
    let mut y0 = v0.y;
    let mut x1 = v1.x;
    let mut y1 = v1.y;
    if (v1.x - v0.x).abs() < (v0.y - v1.y).abs() {
        std::mem::swap(&mut x0, &mut y0);
        std::mem::swap(&mut x1, &mut y1);
        steep = true;
    }
    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
        std::mem::swap(&mut y0, &mut y1);
    }
    if steep {
        for x in x0 as i32..x1 as i32 + 1 {
            let t: f32 = (x as f32 - x0) / (x1 - x0);
            let y = y0 * (1.0 - t) + y1 * t;
            canvas.set(y as usize, x as usize, &color);
        }
    } else {
        for x in x0 as i32..x1 as i32 + 1 {
            let t: f32 = (x as f32 - x0) / (x1 - x0);
            let y = y0 * (1.0 - t) + y1 * t;
            canvas.set(x as usize, y as usize, &color);
        }
    }
}

fn barycentric(v0: &Vec2<i32>, v1: &Vec2<i32>, v2: &Vec2<i32>, p: Vec2<i32>) -> Vec3<f32> {
    let x_vec: Vec3<i32> = Vec3::new(v2.x - v0.x, v1.x - v0.x, v0.x - p.x);
    let y_vec: Vec3<i32> = Vec3::new(v2.y - v0.y, v1.y - v0.y, v0.y - p.y);
    let u: Vec3<i32> = x_vec.cross(y_vec);
    if u.z.abs() < 1 {
        return Vec3::new(-1.0, 1.0, 1.0);
    }
    return Vec3::new(1.0 - (u.x as f32 + u.y as f32) / u.z as f32, u.y as f32 / u.z as f32, u.x as f32 / u.z as f32);
}

fn draw_triangle<V: Vary, S: Shader<V>>(shader: &S, v0: (Vec3<f32>, V), v1: (Vec3<f32>, V), v2: (Vec3<f32>, V), canvas: &mut Canvas, width: usize, height: usize) {
    let p0 = shader.vertex(&v0.0, &v0.1);
    let p1 = shader.vertex(&v1.0, &v1.1);
    let p2 = shader.vertex(&v2.0, &v2.1);
    let xy0 = p0.0.xy();
    let xy1 = p1.0.xy();
    let xy2 = p2.0.xy();
    let v0i: Vec2<i32> = Vec2::new(xy0.x as i32, xy0.y as i32);
    let v1i: Vec2<i32> = Vec2::new(xy1.x as i32, xy1.y as i32);
    let v2i: Vec2<i32> = Vec2::new(xy2.x as i32, xy2.y as i32);
    let x_min = std::cmp::max(0, std::cmp::min(v0i.x, std::cmp::min(v1i.x, v2i.x)));
    let x_max = std::cmp::min(width as i32, std::cmp::max(v0i.x, std::cmp::max(v1i.x, v2i.x)));
    let y_min = std::cmp::max(0, std::cmp::min(v0i.y, std::cmp::min(v1i.y, v2i.y)));
    let y_max = std::cmp::min(height as i32, std::cmp::max(v0i.y, std::cmp::max(v1i.y, v2i.y)));
    for x in x_min..x_max {
        for y in y_min..y_max {
            let bs = barycentric(&v0i, &v1i, &v2i, Vec2::new(x, y));
            if bs.x >= 0.0 && bs.y >= 0.0 && bs.z >= 0.0 {
                //w' = ( 1 / v0.v.w ) * bs.x + ( 1 / v1.v.w ) * bs.y + ( 1 / v2.v.w ) * bs.z
                //u' = ( v0.t.u / v0.t.w ) * bs.x + ( v1.t.u / v1.t.w ) * bs.y + ( v2.t.u / v2.t.w ) * bs.z
                //v' = ( v0.t.v / v0.t.w ) * bs.x + ( v1.t.v / v1.t.w ) * bs.y + ( v2.t.v / v2.t.w ) * bs.z
                //perspCorrU = u' / w'
                //perspCorrV = v' / w'

                let depth: f32 = bs.x * v0.0.z + bs.y * v1.0.z + bs.z * v2.0.z;
                match shader.fragment(&Vec2::new(x as f32, y as f32), V::vary(&v0.1, &v1.1, &v2.1, &bs)) {
                    Some(c) => canvas.set_with_depth(x as usize, y as usize, depth, &c),
                    None => (),
                }
            }
        }
    }
}
fn load_triangle() -> Vec<[Vertex; 3]> {
    let first: Vec3<f32> = Vec3::new(1.0, 0.0, 0.0);
    let second: Vec3<f32> = Vec3::new(0.0, 1.0, 1.0);
    let third: Vec3<f32> = Vec3::new(-1.0, 0.0, 0.0);
    let n: Vec3<f32> = (third - first).cross(second - first);
    let t: Vec2<f32> = Vec2::new(0.0, 0.0);
    let mut triangle : Vec<[Vertex; 3]>= Vec::new();
    triangle.push([ Vertex {v: first, n: n, t: Vec2::new(1.0, 0.0)},
                          Vertex{v: second, n: n, t: Vec2::new(0.5, 1.0)},
                          Vertex{v: third, n: n, t: Vec2::new(0.0, 0.0)}]);
    triangle
}

fn load_model<R: std::io::BufRead>(r: R) -> Result<Vec<[Vertex; 3]>, ObjError> {
    let model_obj: Obj<TexturedVertex> = load_obj(r)?;
    let mut model : Vec<[Vertex; 3]>= Vec::new();
    for indices in model_obj.indices.chunks(3) {
        let first = model_obj.vertices[indices[0] as usize];
        let second = model_obj.vertices[indices[1] as usize];
        let third = model_obj.vertices[indices[2] as usize];
        model.push([ Vertex{ v: Vec3::new(first.position[0], first.position[1], first.position[2]), n:  Vec3::new(first.normal[0], first.normal[1], first.normal[2]), t: Vec2::new(first.texture[0], first.texture[1]) },
                            Vertex{ v: Vec3::new(second.position[0], second.position[1], second.position[2]), n: Vec3::new(second.normal[0], second.normal[1], second.normal[2]), t: Vec2::new(second.texture[0], second.texture[1]) },
                            Vertex{ v: Vec3::new(third.position[0], third.position[1], third.position[2]), n: Vec3::new(third.normal[0], third.normal[1], third.normal[2]), t: Vec2::new(third.texture[0], third.texture[1]) }]);
    }
	Ok(model)
}

fn main() -> Result<(), ObjError> {
    let width: usize = 800;
    let height: usize = 800;
    let img: RgbImage = image::open("/Users/bjornmartens/projects/software-renderer-rs/obj/ah/african_head_diffuse.tga").unwrap().to_rgb(); // use try/? but convert to generic error to standard error and change result of main into that error.
    let mat = viewport(0.0, 0.0, 800.0, 800.0, 255.0);
    let shader = BasicShader {
        mat: &mat,
        tex: &img,
        light_direction: Vec3::new(0.0, 0.0, -1.0),
    };
    let mut canvas = Canvas::new(width, height, Color{r: 0, g:0, b: 0, a: 255});
    let window: Window = Window::new(&canvas);


    let input = &mut BufReader::new(File::open("/Users/bjornmartens/projects/software-renderer-rs/obj/ah/african_head.obj")?);
	let model = load_model(input)?;
    //let model = load_triangle();
    let mut previous_time = Instant::now();
    while window.pump() {
        let c = &Color {r: 255, g: 255, b: 255, a: 255 };
        let c_lines = &Color {r: 0, g: 0, b: 255, a: 255 };
        let mut triangle_count: i32 = 0;
        for t in &model {
            triangle_count = triangle_count + 1;
            let var0 = Varyings { n: t[0].n, t: t[0].t };
            let var1 = Varyings { n: t[1].n, t: t[1].t };
            let var2 = Varyings { n: t[2].n, t: t[2].t };
            draw_triangle(&shader,(t[0].v, var0), (t[1].v, var1), (t[2].v, var2), &mut canvas, width, height);
        }
        println!("triangle_count: {}", triangle_count);
        let current_time = Instant::now();
        println!("fps: {}", (current_time - previous_time).as_millis());
        previous_time = current_time;
        window.update();
        //canvas.clear_zbuffer();
    }
    Ok(())
}


