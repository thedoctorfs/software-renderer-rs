#[repr(C)]
#[derive(Copy, Clone)]
pub struct Color
{
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[repr(u32)]
pub enum InputEvent {
    Quit,
    MouseMotion,
    NotImplemented,
    NoEvent,
}

extern "C" {
    fn window_create(width: libc::size_t, height: libc::size_t, buffer: *const Color, size: libc::size_t) -> *const libc::c_void;
    fn window_destroy(handle: *const libc::c_void);
    fn window_poll_event(handle: *const libc::c_void) -> InputEvent;
    fn window_update(handle: *const libc::c_void);
}

pub struct Window
{
    handle: *const libc::c_void
}

pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<Color>,
    pub zbuffer: Vec<f32>,
}

impl Canvas
{
    pub fn new(width: usize, height: usize, color: &Color) -> Canvas {
        Canvas {
            width,
            height,
            buffer: vec![*color; width * height],
            zbuffer: vec![std::f32::MIN; width * height]
        }
    }
    pub fn clear(&mut self, color: &Color) {
        for elem in self.buffer.iter_mut() { *elem = *color; }
    }
    pub fn set_with_depth(&mut self, x: usize, y: usize, depth: f32, color: &Color) {
        if x < self.width && y < self.height {
            let index = x + (self.height - 1 - y) * self.width;
            if self.zbuffer[index] < depth {
                self.zbuffer[index] = depth;
                self.buffer[index] = *color;
            }
        }
    }
    pub fn set(&mut self, x: usize, y: usize, color: &Color) {
        if x < self.width && y < self.height {
            self.buffer[x + (self.height - 1 - y) * self.width] = *color;
        }
    }
    pub fn clear_zbuffer(&mut self) {
        self.zbuffer = vec![std::f32::MIN; self.width * self.height];
    }
}

impl Window
{
    pub fn new(canvas: &Canvas) -> Window {
        unsafe {
            Window {
                handle: window_create(canvas.width as libc::size_t, canvas.height as libc::size_t, canvas.buffer.as_ptr(), canvas.buffer.len() as libc::size_t)
            }
        }
    }
    pub fn update(&self) {
        unsafe {
            window_update(self.handle);
        }
    }
    pub fn poll_event(&self) -> InputEvent {
        unsafe {
            window_poll_event(self.handle)
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            window_destroy(self.handle);
        }
    }
}
