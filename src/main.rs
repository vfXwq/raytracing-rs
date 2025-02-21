use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use rayon::prelude::*;
use std::time::Instant;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use std::io::{self, Write};
use std::env;
use wgpu::Instance;
use sysinfo::{System, CpuRefreshKind, MemoryRefreshKind, RefreshKind};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

const CIRCLE_X: f32 = 850.0;
const CIRCLE_Y: f32 = 720.0/2.0;
const CIRCLE_R: f32 = 150.0;

const LIGHT_X: f32 = 200.0;
const LIGHT_Y: f32 = 720.0/2.0;
const LIGHT_R: f32 = 25.0;

struct World {
    dragging: bool,
    light_x: f32,
    light_y: f32,
    circle_y: f32,
    circle_vy: f32,
}

struct SystemMonitor {
    sys: System,
    cpu_name: String,
}

impl SystemMonitor {
    fn new() -> Self {
        let sys = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
        );
        let cpu_name = sys.cpus()[0].name().to_string();
        Self { sys, cpu_name }
    }

    fn update(&mut self) -> (f32, f32, f32) {
        self.sys.refresh_cpu();
        self.sys.refresh_memory();
        
        let cpu_usage = self.sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / 
                       self.sys.cpus().len() as f32;
        let memory_used = self.sys.used_memory() as f32 / (1024.0 * 1024.0); // Convert to GB
        let memory_total = self.sys.total_memory() as f32 / (1024.0 * 1024.0);
        let memory_percent = (memory_used / memory_total) * 100.0;
        
        (cpu_usage, memory_used, memory_percent)
    }
}

async fn get_gpu_info() {
    let instance = Instance::new(wgpu::InstanceDescriptor::default());
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..Default::default()
    }).await.unwrap();

    println!("Using GPU: {}", adapter.get_info().name);
}

fn main() -> Result<(), Error> {
    env_logger::init();
    
    let mut sys_monitor = SystemMonitor::new();
    
    println!("CPU: {}", sys_monitor.cpu_name);
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(get_gpu_info());
    
    unsafe { env::set_var("WGPU_POWER_PREF", "high") };
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Raytracing ")
            .with_resizable(false)
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = World::new();
    let mut last_time = Instant::now();
    let mut frames = 0;

    let res = event_loop.run(|event, elwt| {
        // Draw the current frame
        if let Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } = event
        {
            frames += 1;
            let elapsed = last_time.elapsed().as_secs_f32();
            if elapsed >= 0.1 {
                let fps = frames as f32 / elapsed;
                let (cpu_usage, mem_used, mem_percent) = sys_monitor.update();
                print!("\rFPS: {:.1} | CPU: {:.1}% | RAM: {:.1}GB ({:.1}%)", 
                    fps, cpu_usage, mem_used, mem_percent);
                io::stdout().flush().unwrap();
                frames = 0;
                last_time = Instant::now();
            }

            world.draw(pixels.frame_mut());
            if let Err(err) = pixels.render() {
                log_error("pixels.render", err);
                elwt.exit();
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                elwt.exit();
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    elwt.exit();
                    return;
                }
            }

            // Update internal state and request a redraw
            world.update(&input);
            window.request_redraw();
        }
    });
    res.map_err(|e| Error::UserDefined(Box::new(e)))
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}

impl World {
    fn new() -> Self {
        Self {
            dragging: false,
            light_x: LIGHT_X,
            light_y: LIGHT_Y,
            circle_y: CIRCLE_Y,
            circle_vy: 0.2,
        }
    }

    fn update(&mut self, input: &WinitInputHelper) {
        // Check for mouse press inside the light circle
        if input.mouse_pressed(0) {
            if let Some((mx, my)) = input.cursor() {
                let dx = mx as f32 - self.light_x;
                let dy = my as f32 - self.light_y;
                if (dx * dx + dy * dy).sqrt() <= LIGHT_R {
                    self.dragging = true;
                }
            }
        }

        // While dragging, follow the mouse
        if self.dragging && input.mouse_held(0) {
            if let Some((mx, my)) = input.cursor() {
                self.light_x = mx as f32;
                self.light_y = my as f32;
            }
        }

        // Stop dragging when released
        if input.mouse_released(0) {
            self.dragging = false;
        }

        // Move the circle up and down
        self.circle_y += self.circle_vy;

        // Bounce off top/bottom
        if self.circle_y < CIRCLE_R || self.circle_y > (HEIGHT as f32 - CIRCLE_R) {
            self.circle_vy = -self.circle_vy;
        }
    }

    fn draw(&self, frame: &mut [u8]) {
        frame.par_chunks_exact_mut(4)
             .enumerate()
             .for_each(|(i, pixel)| {
                 let xi = (i % WIDTH as usize) as f32;
                 let yi = (i / WIDTH as usize) as f32;

                 let dist_light = ((xi - self.light_x).powi(2) + (yi - self.light_y).powi(2)).sqrt();
                 let dist_circle = ((xi - CIRCLE_X).powi(2) + (yi - self.circle_y).powi(2)).sqrt();

                 // If inside the light circle => white
                 let rgba = if dist_light <= LIGHT_R {
                     [0xff, 0xff, 0xff, 0xff]
                 // Else if inside main circle => white
                 } else if dist_circle <= CIRCLE_R {
                     [0xff, 0xff, 0xff, 0xff]
                 // Else check if in shadow => black, else => yellow
                 } else if is_shadowed(self.light_x, self.light_y, xi, yi, CIRCLE_X, self.circle_y, CIRCLE_R) {
                     [0x00, 0x00, 0x00, 0xff]
                 } else {
                     [0xff, 0xff, 0x00, 0xff]
                 };

                 pixel.copy_from_slice(&rgba);
             });
    }
}

/// Return true if the line from (lx, ly) to (px, py) intersects the circle at (cx, cy) with radius r.
fn is_shadowed(lx: f32, ly: f32, px: f32, py: f32, cx: f32, cy: f32, r: f32) -> bool {
    let dx = px - lx;
    let dy = py - ly;
    let fx = lx - cx;
    let fy = ly - cy;

    let a = dx*dx + dy*dy;
    let b = 2.0 * (fx*dx + fy*dy);
    let c = fx*fx + fy*fy - r*r;

    let disc = b*b - 4.0*a*c;
    if disc < 0.0 {
        return false; // no intersection
    }

    let disc_sqrt = disc.sqrt();
    let t1 = (-b - disc_sqrt) / (2.0*a);
    let t2 = (-b + disc_sqrt) / (2.0*a);

    // If either t is between 0 and 1, we have an intersection before reaching (px, py).
    (0.0..=1.0).contains(&t1) || (0.0..=1.0).contains(&t2)
}