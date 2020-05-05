mod obj;
mod rasterizer;
mod canvas;
mod sdl_window;
mod window;
mod camera;
mod simulation;
mod command_queue;
mod counter;
mod commands;
mod local_client;
mod client;
mod packet;
mod recording;

use rasterizer::{Vary, Shader};
use sdl_window::*;
use canvas::{Canvas, Color};

use image::RgbImage;
use nalgebra_glm::*;

use std::fs::File;
use std::io::BufReader;
use std::time::{Instant};
use command_queue::CommandQueue;
use std::path::PathBuf;
use structopt::StructOpt;
use crate::commands::Command;

#[derive(Copy, Clone)]
pub struct Varyings {
    n: Vec3,
    t: Vec2,
}

impl Vary for Varyings {
    fn vary(var0: &Self, var1: &Self, var2: &Self, bc: &Vec3) -> Self {
        Varyings {
            n: (var0.n * bc.x + var1.n * bc.y + var2.n * bc.z).normalize(),
            t: (var0.t * bc.x + var1.t * bc.y + var2.t * bc.z),
        }
    }
}

struct BasicShader<'a>
{
    viewport: &'a Vec4,
    projection: &'a Mat4,
    view: Mat4,
    model: Mat4,
    tex: &'a RgbImage,
    light_direction: Vec3,
}

impl<'a> Shader<Varyings> for BasicShader<'a> {
    fn vertex(&self, in_v: &Vec3, var: &Varyings) -> (Vec4, Varyings) {
        let modelview = self.view * self.model;
        let projected = project(in_v, &modelview, self.projection, *self.viewport);
        let n = inverse_transpose(modelview) *  vec3_to_vec4(&var.n);
        let out_var = Varyings { n: vec4_to_vec3(&n), t: var.t };
        (vec4(projected.x, projected.y, projected.z, 1.0), out_var)
    }
    fn fragment(&self, _: &I32Vec2, var: &Varyings) -> Option<Color> {
        let intensity = dot(&var.n, &self.light_direction);
        if intensity > 0.0 {
            let pixel = self.tex.get_pixel((var.t.x * self.tex.width() as f32) as u32, self.tex.height() - 1 - (var.t.y * self.tex.height() as f32) as u32);
            let r = pixel[0] as f32 * intensity;
            let g = pixel[1] as f32 * intensity;
            let b = pixel[2] as f32 * intensity;
            let out_color = Color { r: r as u8, g: g as u8, b: b as u8, a: 255 };
            Some(out_color)
        } else {
            None
        }
    }
}

fn _load_triangle() -> obj::ObjResult<Vec<[(Vec3, Varyings); 3]>> {
    let first = vec3(1.0, 0.0, 0.0);
    let second = vec3(0.0, 1.0, 1.0);
    let third = vec3(-1.0, 0.0, 0.0);
    let n = cross(&(third - first),&(second - first));
    let mut triangle = Vec::new();
    triangle.push([ (first, Varyings { n, t: Vec2::new(1.0, 0.0)}),
                           (second, Varyings { n, t: Vec2::new(0.5, 1.0)}),
                          (third, Varyings{ n, t: Vec2::new(0.0, 0.0)})]);
    Ok(triangle)
}

pub fn load_mesh<R>(reader: R) -> obj::ObjResult<Vec<[(Vec3, Varyings); 3]>>
    where R: std::io::BufRead {
    let vertices = obj::parse_obj(reader)?;
    let mut result = Vec::new();
    for v in vertices {
        result.push([(v[0].0, Varyings { n: (v[0].1).0, t: (v[0].1).1} ),
                            (v[1].0, Varyings { n: (v[1].1).0, t: (v[1].1).1} ),
                            (v[2].0, Varyings { n: (v[2].1).0, t: (v[2].1).1 })]);
    }
    Ok(result)
}

fn _example_viewport_projection_view_model() -> std::result::Result<(), obj::ObjError> {
    let original = vec3(1.0, 1.0, -10.0);
    let projection = perspective(800.0 / 800.0, 45.0, 1.0, 1000.0);
    let viewport = vec4(0.0, 0.0, 800.0, 800.0);
    let modelview = look_at(&vec3(0.0, 0.0, -4.0, ), &vec3(0.0, 0.0, 0.0), &vec3(0.0, 1.0, 0.0));
    let projected = project(&original, &modelview, &projection, viewport.clone());
    let unprojected = unproject(&projected, &modelview, &projection, viewport.clone());
    println!("original: {}", original);
    println!("projected: {}", projected);
    println!("unprojected: {}", unprojected);
    Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(name = "options", about = "command line options")]
pub struct Options {
    #[structopt(long = "recording", parse(from_os_str))]
    record_path: Option<PathBuf>,

    #[structopt(long = "replay", parse(from_os_str))]
    replay_path: Option<PathBuf>,
}

fn game(options: Options) -> std::result::Result<(), obj::ObjError> {
    let width: usize = 800;
    let height: usize = 800;

    let mut canvas = Canvas::new(width, height, &Color{r: 0, g:0, b: 0, a: 255});
    let window = SDLWindow::new(&canvas);

    let img: RgbImage = image::open("obj/ah/african_head_diffuse.tga").unwrap().to_rgb(); // use try/? but convert to generic error to standard error and change result of main into that error.
    let input = &mut BufReader::new(File::open("obj/ah/african_head.obj")?);
    let mesh = load_mesh(input)?;

    let viewport = vec4(0.0, 0.0, 800.0, 800.0);
    let projection = perspective(800.0 / 800.0, 45.0, 1.0, 1000.0);
    let model: Mat4 = identity();

    let mut previous_time = Instant::now();
    let mut rot: f32 = 0.0;

    let mut inputs = window::InputQueue::new();
    let mut commands_queue = CommandQueue::new();
    let mut simulation = simulation::Simulation::new();
    let mut client= local_client::LocalClient::new();
    let mut record = recording::try_create_recorder(options.record_path);
    let replaying = options.replay_path != None;
    let mut replay = recording::try_create_replayer(options.replay_path);
    let mut shader = BasicShader {
        viewport: &viewport,
        projection: &projection,
        view: camera::view(&simulation.camera_position, &simulation.camera_direction),
        model: model,
        tex: &img,
        light_direction: vec3(0.0, 0.0, 1.0),
    };

    /* every client creates if possible for every frame its commands, if there are no commands then
       the server will do nothing. every client runs its own local simulation. when the server
       returns the frames it merged from all clients (this means the server also needs to wait
        on slow clients). the "real" simulation is applied and the
       local simulation is rebased on that. In this way user input acts fast and the simulation
       state is equal on all clients at the cost rebasing the simulation every time the server
       sends an update.
       */
    let mut quit = false;
    let mut frame_counter = counter::FrameCounter::new(60);
    while !quit {
        frame_counter.run();
        canvas.clear(&Color{r: 0, g:0, b: 0, a: 255});
        canvas.clear_zbuffer();

        quit = !inputs.pump(&(*window));
        if !replaying {
            let input_commands = commands_queue.handle_input(&mut inputs, frame_counter.count());
            client::send(&mut client, input_commands.as_slice());
        } else {
            let replay_commands = client::receive(&mut *replay, frame_counter.count());
            client::send(&mut client, replay_commands.as_slice());
        }
        let commands_received = client::receive(&mut client,frame_counter.count());
        client::send(&mut *record, commands_received.as_slice());
        for frame in &commands_received {
            let _ = simulation.handle_frame(frame);
        }

        rot = rot + 0.01;
        shader.model = rotate(&identity(), rot, &vec3(0.0, 1.0, 0.0));
        shader.view = camera::view(&simulation.camera_position, &simulation.camera_direction);

        let mut triangle_count: i32 = 0;
        for t in &mesh {
            triangle_count = triangle_count + 1;
            rasterizer::draw_triangle(t[0], t[1], t[2], &shader, &mut canvas);
        }

        //println!("triangle_count: {}", triangle_count);
        let current_time = Instant::now();
        //println!("fps: {}", (current_time - previous_time).as_millis());
        previous_time = current_time;
        window.update();
    }
    Ok(())
}

fn main() -> std::result::Result<(), obj::ObjError> {
    let options = Options::from_args();
    game(options)
}
