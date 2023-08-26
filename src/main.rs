pub mod colors;
pub mod config;
mod dla;
pub mod grid;
use dla::Dla;

use log::{debug, error};
use pixels::{Pixels, SurfaceTexture};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

use crate::gui::Framework;
mod gui;

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Trace).expect("error initializing logger");

        wasm_bindgen_futures::spawn_local(run());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();

        pollster::block_on(run());
    }
}

async fn run() {
    let sim: Dla = Dla::default();
    let scale_factor = 1.25;
    let (width, height) = sim.size();

    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(width as f64, height as f64);
        let scaled_size =
            LogicalSize::new(width as f64 * scale_factor, height as f64 * scale_factor);
        WindowBuilder::new()
            .with_title("DLA in Rust")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let window = Rc::new(window);

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowExtWebSys;

        // Retrieve current width and height dimensions of browser client window
        let get_window_size = || {
            let client_window = web_sys::window().unwrap();
            LogicalSize::new(
                client_window.inner_width().unwrap().as_f64().unwrap(),
                client_window.inner_height().unwrap().as_f64().unwrap(),
            )
        };

        let window = Rc::clone(&window);

        // Initialize winit window with current dimensions of browser client
        window.set_inner_size(get_window_size());

        let client_window = web_sys::window().unwrap();

        // Attach winit canvas to body element
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");

        // Listen for resize event on browser client. Adjust winit window dimensions
        // on event trigger
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::Event| {
            let size = get_window_size();
            window.set_inner_size(size)
        }) as Box<dyn FnMut(_)>);
        client_window
            .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    // can't move borrowed self into the event loop. So we move a clone in which has the same lifetime
    let self_clone = sim.clone();

    // The grid is shared between the event loop, live thread, and gui.
    let event_loop_grid = Arc::new(Mutex::new(self_clone));
    let gui_grid = Arc::clone(&event_loop_grid);

    let (mut pixels, mut framework) = {
        let window_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, window.as_ref());
        let pixels = Pixels::new_async(width as u32, height as u32, surface_texture)
            .await
            .expect("Pixels error");
        let framework = Framework::new(
            &event_loop,
            window_size.width,
            window_size.height,
            scale_factor,
            &pixels,
            gui_grid,
        );
        (pixels, framework)
    };

    // use worker thread on native
    #[cfg(not(target_arch = "wasm32"))]
    {
        Dla::spawn_worker_thread(&event_loop_grid);
    }

    event_loop.run(move |event, _, control_flow| {
        if input.update(&event) {
            if input.key_released(VirtualKeyCode::Escape) || input.quit() {
                debug!("Quit signal received. Exiting...");
                *control_flow = ControlFlow::Exit;
                return;
            }

            // egui scaling
            if let Some(scale_factor) = input.scale_factor() {
                debug!("Scale factor changed. Updating framework");
                framework.scale_factor(scale_factor);
            }

            // egui resizing
            if let Some(size) = input.window_resized() {
                let guard = event_loop_grid.lock().unwrap();
                let (grid_width, grid_height) = guard.size();
                std::mem::drop(guard);
                if size.width < grid_width as u32 || size.height < grid_height as u32 {
                    // There aren't enough pixels to resize the grid
                    // This would result in grid square size of < 1 pixel, which isn't possible (?)
                    debug!(
                        "Can't resize grid to ({} x{}), ignoring",
                        size.width, size.height
                    );
                } else {
                    debug!("Resizing framework & pixels surface. Size is {size:?}");
                    if let Err(err) = pixels.resize_surface(size.width, size.height) {
                        error!("pixels.resize_surface() failed: {err}");
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                    framework.resize(size.width, size.height);
                }
            }

            // single threaded when compiling to wasm
            #[cfg(target_arch = "wasm32")]
            {
                let mut guard = event_loop_grid.lock().unwrap();
                if !guard.paused() && !guard.complete() {
                    // 80,000 is a number that worked well on my HW to balance locking
                    // access and responsiveness of the UI within reason. Essentially this
                    // is saying we'd like 80,000 updates between redraws of the UI
                    for _ in 0..80_000 {
                        guard.update();
                    }
                }
                std::mem::drop(guard);
            }

            window.request_redraw(); // TODO: play with moving this
        }

        match event {
            Event::WindowEvent { event, .. } => {
                // for egui inputs
                framework.handle_event(&event);
            }
            Event::RedrawRequested(_) => {
                // pass in a pixels frame mutably. Modify it to produce the next frame
                let guard_grid = event_loop_grid.lock().unwrap();
                let need_resize = guard_grid.do_resize();
                std::mem::drop(guard_grid);

                if need_resize {
                    debug!("Resize needed. Updating winit, pixels, and egui framework");
                    // special resize handling
                    let mut guard_grid = event_loop_grid.lock().unwrap();
                    guard_grid.set_do_resize(false);
                    let (width, height) = guard_grid.size();
                    std::mem::drop(guard_grid);

                    let size = LogicalSize::new(width as f64, height as f64);
                    let scaled_size =
                        LogicalSize::new(width as f64 * scale_factor, height as f64 * scale_factor);

                    // update the window size to fit the new grid size the same way as when it's init'd outside the
                    // event loop
                    window.set_inner_size(scaled_size);
                    window.set_min_inner_size(Some(size));

                    let phys_size = window.inner_size();

                    // TWO parts of pixels need to be updated:
                    // 1. The size of the buffer itself (we changed grid size)
                    // 2. The size of the surface on top of which its renedered (relevant b/c window size changed)
                    if let Err(err) = pixels.resize_buffer(width as u32, height as u32) {
                        error!("pixels.resize_buffer() failed: {err}");
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                    if let Err(err) = pixels.resize_surface(phys_size.width, phys_size.height) {
                        error!("pixels.resize_surface() failed: {err}");
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    // The egui framework also needs to be resized to account for the new window size
                    framework.resize(phys_size.width, phys_size.height);

                    // We've changed the window, redraw it.
                    window.request_redraw();
                } else {
                    // normal case
                    let mut guard_grid = event_loop_grid.lock().unwrap();
                    guard_grid.draw(pixels.get_frame_mut());
                    std::mem::drop(guard_grid);

                    // This runs Gui::ui, which needs the lock! so make sure it's released
                    framework.prepare(&window);

                    let render_result = pixels.render_with(|encoder, render_target, context| {
                        context.scaling_renderer.render(encoder, render_target);

                        framework.render(encoder, render_target, context);

                        Ok(())
                    });

                    if let Err(err) = render_result {
                        error!("pixels.render() failed: {err}");
                        *control_flow = ControlFlow::Exit;
                    }
                }
            }
            _ => (), // ignore all other events
        }
    });
}
