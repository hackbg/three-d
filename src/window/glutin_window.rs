
use glutin::*;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use crate::window::frame_input;
use crate::context;

#[derive(Debug)]
pub enum Error {
    WindowCreationError(glutin::CreationError),
    ContextError(glutin::ContextError)
}

impl From<glutin::CreationError> for Error {
    fn from(other: glutin::CreationError) -> Self {
        Error::WindowCreationError(other)
    }
}

impl From<glutin::ContextError> for Error {
    fn from(other: glutin::ContextError) -> Self {
        Error::ContextError(other)
    }
}

pub struct Window
{
    windowed_context: ContextWrapper<PossiblyCurrent, window::Window>,
    event_loop: EventLoop<()>,
    gl: crate::Context
}

impl Window
{
    pub fn new(title: &str, size: Option<(u32, u32)>) -> Result<Window, Error>
    {
        let window_builder =
            if let Some((width, height)) = size {
                WindowBuilder::new()
                    .with_title(title)
                    .with_inner_size(dpi::LogicalSize::new(width as f64, height as f64))
                    .with_resizable(false)
            } else {
                WindowBuilder::new()
                    .with_title(title)
                    .with_maximized(true)
                    .with_resizable(false)
            };

        let event_loop = EventLoop::new();
        let windowed_context = ContextBuilder::new().with_vsync(true).with_srgb(true).build_windowed(window_builder, &event_loop)?;
        let windowed_context = unsafe { windowed_context.make_current().unwrap() };
        let gl = context::Glstruct::load_with(|s| windowed_context.get_proc_address(s) as *const std::os::raw::c_void);
        Ok(Window { windowed_context, event_loop, gl})
    }

    pub fn render_loop<F: 'static>(self, mut callback: F) -> Result<(), Error>
        where F: FnMut(frame_input::FrameInput)
    {
        let windowed_context = self.windowed_context;
        let mut last_time = std::time::Instant::now();
        let mut count = 0;
        let mut accumulated_time = 0.0;
        let mut events = Vec::new();
        let mut cursor_pos = None;
        self.event_loop.run(move |event, _, control_flow| {
                *control_flow = ControlFlow::Wait;
                match event {
                    Event::LoopDestroyed => {
                        return;
                    }
                    Event::MainEventsCleared => {
                        windowed_context.window().request_redraw();
                    }
                    Event::RedrawRequested(_) => {

                        let now = std::time::Instant::now();
                        let duration = now.duration_since(last_time);
                        last_time = now;
                        let elapsed_time = duration.as_secs() as f64 * 1000.0 + duration.subsec_nanos() as f64 * 1e-6;
                        accumulated_time += elapsed_time;
                        count += 1;
                        if accumulated_time > 1000.0 {
                            println!("FPS: {}", count as f64 / (accumulated_time * 0.001));
                            count = 0;
                            accumulated_time = 0.0;
                        }

                        let (physical_width, physical_height): (u32, u32) = windowed_context.window().inner_size().into();
                        let (width, height): (u32, u32) = windowed_context.window().inner_size().to_logical::<f64>(windowed_context.window().scale_factor()).into();
                        let frame_input = frame_input::FrameInput {
                            events: events.clone(),
                            elapsed_time,
                            viewport: crate::Viewport::new_at_origo(physical_width as usize, physical_height as usize),
                            window_width: width as usize,
                            window_height: height as usize
                        };
                        events.clear();
                        callback(frame_input);
                        windowed_context.swap_buffers().unwrap();
                    }
                    Event::WindowEvent { ref event, .. } => match event {
                        WindowEvent::Resized(physical_size) => {
                            windowed_context.resize(*physical_size);
                        }
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit
                        },
                        WindowEvent::KeyboardInput {input, ..} => {
                            if let Some(keycode) = input.virtual_keycode {
                                if keycode == event::VirtualKeyCode::Escape {
                                    *control_flow = ControlFlow::Exit;
                                }
                                let state = if input.state == event::ElementState::Pressed {frame_input::State::Pressed} else {frame_input::State::Released};
                                events.push(frame_input::Event::Key {state, kind: format!("{:?}", keycode)});
                            }
                        },
                        WindowEvent::MouseWheel {delta, ..} => {
                            if let Some(position) = cursor_pos
                            {
                                match delta {
                                    event::MouseScrollDelta::LineDelta(_, y) => {
                                        events.push(frame_input::Event::MouseWheel { delta: *y as f64, position });
                                    },
                                    event::MouseScrollDelta::PixelDelta(logical_position) => {
                                        events.push(frame_input::Event::MouseWheel { delta: logical_position.y, position });
                                    }
                                }
                            }
                        },
                        WindowEvent::MouseInput {state, button, ..} => {
                            if let Some(position) = cursor_pos
                            {
                                let state = if *state == event::ElementState::Pressed {frame_input::State::Pressed} else {frame_input::State::Released};
                                let button = match button {
                                    event::MouseButton::Left => Some(frame_input::MouseButton::Left),
                                    event::MouseButton::Middle => Some(frame_input::MouseButton::Middle),
                                    event::MouseButton::Right => Some(frame_input::MouseButton::Right),
                                    _ => None
                                };
                                if let Some(b) = button {
                                    events.push(frame_input::Event::MouseClick { state, button: b, position });
                                }
                            }
                        },
                        WindowEvent::CursorMoved {position, ..} => {
                            cursor_pos = Some((position.x, position.y));
                        },
                        _ => (),
                    },
                    Event::DeviceEvent{ event, .. } => match event {
                        event::DeviceEvent::MouseMotion {delta} => {
                            if let Some(position) = {cursor_pos}
                            {
                                events.push(frame_input::Event::MouseMotion { delta, position });
                            }
                        },
                        _ => {}
                    },
                    _ => (),
                }
            });
    }

    pub fn size(&self) -> (usize, usize)
    {
        let t: (u32, u32) = self.windowed_context.window().inner_size().to_logical::<f64>(self.windowed_context.window().scale_factor()).into();
        (t.0 as usize, t.1 as usize)
    }

    pub fn viewport(&self) -> crate::Viewport {
        let (w, h): (u32, u32) = self.windowed_context.window().inner_size().into();
        crate::Viewport::new_at_origo(w as usize, h as usize)
    }

    pub fn gl(&self) -> crate::Context
    {
        self.gl.clone()
    }
}
