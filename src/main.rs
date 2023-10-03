#![feature(iter_next_chunk)]

mod structure;

use std::io::Read;

use glam::Vec2;
use pixels::{wgpu::Extent3d, Pixels, SurfaceTexture};
use structure::Structure;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::{dpi::PhysicalSize, window::WindowBuilder};
use winit_input_helper::WinitInputHelper;

fn main() -> Result<(), pixels::Error> {
    let mut structure = {
        let mut args = std::env::args().skip(1);
        let path = args.next().expect("no path to structure file specified");
        let mut structure_file =
            std::fs::File::open(path).expect("structure file not found: '{path}'");
        let mut gro = String::new();
        structure_file.read_to_string(&mut gro).unwrap();

        Structure::from_gro(gro).expect("gro file is invalid")
    };

    eprintln!("Structure loaded: '{}'", structure.title);
    eprintln!("         n_atoms: {}", structure.n_atoms());
    eprintln!("          center: {}", structure.center());
    eprintln!("             box: {:?}", structure.box_vecs);

    eprintln!("Centering the structure...");
    structure.center_structure();
    eprintln!("        centered: {}", structure.center());

    let mut zoom = 100.0;

    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = WindowBuilder::new()
        .with_title("laurel")
        .build(&event_loop)
        .unwrap();

    let mut pixels = {
        let PhysicalSize { width, height } = window.inner_size();
        let surface_texture = SurfaceTexture::new(width, height, &window);
        Pixels::new(width, height, surface_texture)?
    };

    event_loop.run(move |event, _, control_flow| {
        dbg!(&event);
        *control_flow = ControlFlow::Wait;

        if let Event::RedrawRequested(_) = event {
            eprintln!("INFO:  Redrawing");
            let Extent3d { width, height, .. } = pixels.texture().size();

            // Render the pixels.
            let frame = pixels.frame_mut();
            frame.fill(0x00); // Clear the screen.
            const PIXEL_SIZE: usize = 4;
            let screen_center = Vec2::new(width as f32 / 2.0, height as f32 / 2.0);
            let x_range = 0..width;
            let y_range = 0..height;
            for atom in &structure.atoms {
                // Find the render position.
                //
                // Orthographic projection.
                // |bx| = |sx  0  0||ax| + |cx|
                // |by| = | 0  0 sy||ay| + |cz|
                //                  |az|
                //
                // From this, we can derive:
                // bx = sx * ax + cx
                // by = sz * az + cz
                let a = atom.position;
                let cx = 1.0;
                let cz = 1.0;
                let sx = 1.0;
                let sz = 1.0;
                let bx = sx * a.x + cx;
                let by = sz * a.z + cz;

                let pos: Vec2 = Vec2::new(bx, by); // TODO

                // Render that onto the screen.
                let screen_pos = pos * zoom + screen_center;
                let (x, y) = (screen_pos.x as u32, screen_pos.y as u32);
                if x_range.contains(&x) && y_range.contains(&y) {
                    let depth = 1.0;
                    // let depth = (atom.position.z - structure.min_z()) / (structure.min_z().abs() + structure.max_z());
                    let px = {
                        let v = (depth * u8::MAX as f32 + 10.0) as u8;
                        [v, v, v, 0xff]
                    };
                    for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                        let (x, y) = (x + dx, y + dy);
                        let idx = (y * width + x) as usize * PIXEL_SIZE;
                        if idx + PIXEL_SIZE >= frame.len() {
                            continue;
                        }
                        frame[idx..idx + PIXEL_SIZE].copy_from_slice(&px);
                    }
                }
            }

            // Try to render.
            if let Err(err) = pixels.render() {
                eprintln!("ERROR: {err}");
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            // Close events.
            if input.close_requested() {
                eprintln!("INFO:  Close requested. Bye :)");
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window.
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_buffer(size.width, size.height) {
                    eprintln!("ERROR: {err}");
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    eprintln!("ERROR: {err}");
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            // Deal with key input.
            if input.key_held(VirtualKeyCode::Up) {
                // Rotate up.
            }
            if input.key_held(VirtualKeyCode::Down) {
                // Rotate down.
            }
            if input.key_held(VirtualKeyCode::Left) {
                // Rotate left.
            }
            if input.key_held(VirtualKeyCode::Right) {
                // Rotate right.
            }

            zoom = 0.0f32.max(zoom + input.scroll_diff() * 3.0);

            window.request_redraw();
        }
    });
}
