//use crate::compute_pipeline;
use cute::c;
use imagesize::size;
use rand::Rng;
use rayon::iter::IntoParallelIterator;
use std::time::{Duration, Instant};
use std::{iter, vec};
use std::{thread, time};
use image::{DynamicImage, GenericImageView, image_dimensions, ImageResult};
use wgpu::util::DeviceExt;
use winit::window::Window;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub async fn run() {
    // Initialize the logger for debugging and logging messages
    env_logger::init();

    // Create an event loop that will drive the application's event handling
    let event_loop = EventLoop::new();

    // Create a window using a builder and attach it to the event loop
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // Attempt to get the dimensions of the image file (city.png)
    let (w, h) = match size("src/images/city.png") {
        Ok(dim) => (dim.width, dim.height), // If successful, use the image dimensions
        Err(why) => {
            // Print an error message if dimensions cannot be retrieved
            println!("Error getting dimensions: {:?}", why);
            (0, 0) // Default to (0, 0) if an error occurs
        }
    };

    // Create a logical size structure for the window based on image dimensions
    let logical_size = LogicalSize {
        width: w as u32,
        height: h as u32,
    };

    // Set the window size using a physical conversion factor (0.5 scaling factor)
    window.set_inner_size(logical_size.to_physical::<u32>(0.5));

    // Create a new application state (handles WGPU setup and rendering)
    let mut state = State::new(&window).await;

    // Resize the internal WGPU surface to match the state size
    state.resize(state.size);

    // Initialize a time tracker for measuring frames per second
    let mut print_time = Instant::now() + Duration::from_secs(1);
    let mut draws = 0; // Counter for the number of frames rendered

    // Start the event loop, which continuously processes events
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::MainEventsCleared => {
                // The main event loop triggers this when all events are processed
                *control_flow = ControlFlow::Poll; // Continually request new frames

                if print_time < Instant::now() {
                    // Print frame count every second
                    println!("{} Frames in 1 Second", draws);
                    print_time = Instant::now() + Duration::from_secs(1);

                    // Update the application state and request a redraw
                    state.update();
                    window.request_redraw();
                }
            }

            Event::RedrawRequested(_) => {
                // This event is triggered when the window needs to be redrawn

                // Logic for generating and evaluating random shapes would go here
                // 1. Generate a random shape
                // 2. Generate a difference texture from the random shape
                // 3. Calculate score based on difference
                // 4. If score > 0, keep the shape, else repeat

                // Attempt to run a compute shader
                match state.compute() {
                    Ok(_) => {} // Successful execution
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size), // Reconfigure surface if lost
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit, // Exit if system is out of memory
                    Err(e) => eprintln!("{:?}", e), // Print other errors
                }
            }

            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                // Handle window-specific events such as resizing and keyboard input

                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                            ..
                        } => *control_flow = ControlFlow::Exit, // Exit if Escape is pressed

                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size); // Adjust WGPU surface on resize
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size); // Handle DPI scaling changes
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    });
}