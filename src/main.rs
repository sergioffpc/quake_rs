use std::{
    thread,
    time::{Duration, Instant},
};

use log::debug;
use quake_rs::{
    camera::Camera,
    hid::{self, HIDEvent, GLOBAL_HID_EVENT_BUS},
    renderer, resource,
    scene::Scene,
    send_hid_event,
};
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    env_logger::init();
    hid::init();
    resource::init("res/PAK0.PAK");

    let width = 1280;
    let height = 720;
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Quake::rs")
        .with_inner_size(PhysicalSize::new(width, height))
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    let renderer = renderer::Renderer::new(&window).unwrap();

    let mut camera = Camera::new(width, height);
    GLOBAL_HID_EVENT_BUS
        .get()
        .unwrap()
        .subscribe(&mut |event| camera.update(event));

    let mut scene = Scene::load(&renderer, "").unwrap();

    let target_fps = 60;
    let target_frame_time = Duration::from_secs_f64(1.0 / target_fps as f64);
    let mut last_frame_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        // Calculate delta time
        let delta_time = last_frame_time.elapsed();
        last_frame_time = Instant::now();

        // Handle input events
        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => handle_keyboard_input(input),
                _ => (),
            },
            Event::DeviceEvent { event, .. } => handle_mouse_input(event),
            _ => (),
        }

        // Update game logic
        scene.update(&renderer.queue, &delta_time);

        // Render game state
        renderer
            .render(&camera, scene.visible_entities(&camera))
            .unwrap();

        // Control frame rate
        let elapsed_frame_time = last_frame_time.elapsed();
        if elapsed_frame_time < target_frame_time {
            let sleep_duration = target_frame_time - elapsed_frame_time;
            thread::sleep(sleep_duration);
        }
    });
}

fn handle_keyboard_input(input: KeyboardInput) {
    match input {
        KeyboardInput {
            state: ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::W),
            ..
        } => {
            send_hid_event!(HIDEvent::MoveForward(0.5));
        }
        KeyboardInput {
            state: ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::S),
            ..
        } => {
            send_hid_event!(HIDEvent::MoveBackward(0.5));
        }
        _ => (),
    }
}

fn handle_mouse_input(event: DeviceEvent) {
    match event {
        DeviceEvent::MouseMotion { delta } => {
            debug!("MouseMotion {:?}", delta)
        }
        DeviceEvent::MouseWheel { delta } => match delta {
            winit::event::MouseScrollDelta::PixelDelta(ref pos) => {
                debug!("MouseWheel {:?}", pos)
            }
            _ => (),
        },
        DeviceEvent::Button { button, state } => debug!("Button {:?} {:?}", button, state),
        _ => (),
    }
}
