mod cpu;

use cpu::CPU;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use pixels::SurfaceTexture;
use pixels::wgpu::{PowerPreference, RequestAdapterOptions};


const WIDTH: usize = 64;
const HEIGHT: usize = 32;

const FONT_SPRITES: [u8; 80]  = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, //0 
    0x20, 0x60, 0x20, 0x20, 0x70, //1 
    0xF0, 0x10, 0xF0, 0x80, 0xF0, //2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, //3 
    0x90, 0x90, 0xF0, 0x10, 0x10, //4 
    0xF0, 0x80, 0xF0, 0x10, 0xF0, //5 
    0xF0, 0x80, 0xF0, 0x90, 0xF0, //6 
    0xF0, 0x10, 0x20, 0x10, 0x40, //7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, //8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, //9
    0xF0, 0x90, 0xF0, 0x90, 0x90, //A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, //B 
    0xF0, 0x80, 0x80, 0x80, 0xF0, //C 
    0xE0, 0x90, 0x90, 0x90, 0xE0, //D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, //E
    0xF0, 0x80, 0xF0, 0x80, 0x80, //F
];

#[derive(Default)]
struct Emulator {
    window: Option<Window>,
}

impl ApplicationHandler for Emulator {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(event_loop.create_window(Window::default_attributes()).unwrap());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event:WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                let mut cpu = CPU::new();
                cpu.load_font(&FONT_SPRITES);
                cpu.load_rom("4-flags.ch8");

                for _ in 0..9 {
                    cpu.step();
                }

                if cpu.draw {
                    let surface_texture = SurfaceTexture::new(WIDTH as u32, HEIGHT as u32, &self.window).try_into().unwrap();
                }

                self.window.as_ref().unwrap().request_redraw();
            },
            WindowEvent::KeyboardInput { 
                device_id, 
                event, 
                is_synthetic 
            } => {

            },
            _ => (),
        }
    }
}

fn main() {

    // Handle window logic
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut emu = Emulator::default();
    event_loop.run_app(&mut emu);

    
}