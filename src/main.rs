use export::export_maps;
use nalgebra::Vector3;
use render::{ColorMode, EcosystemRenderable};
use sdl2::{
    keyboard::Keycode,
    sys::{SDL_GetPerformanceCounter, SDL_GetPerformanceFrequency},
};
use simulation::Simulation;
use std::{collections::HashSet, ffi::CString, thread::sleep, time::Duration};

use crate::export::export_height_map;

mod camera;
mod constants;
mod ecology; // apparently naming this "ecosystem" breaks rust analyzer :(
mod events;
mod export;
mod import;
mod render;
mod render_gl;
mod simulation;

#[derive(PartialEq, Eq, Hash)]
pub(crate) enum Direction {
    Up,
    Down,
    Left,
    Right,
    Forward,
    Back,
}

fn main() {
    // https://nercury.github.io/rust/opengl/tutorial/2018/02/08/opengl-in-rust-from-scratch-00-setup.html
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 1);

    let window = video_subsystem
        .window("Hummus", 900, 700)
        .opengl()
        .resizable()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    // set up shared state for window
    unsafe {
        gl::Viewport(
            0,
            0,
            constants::SCREEN_WIDTH as i32,
            constants::SCREEN_HEIGHT as i32,
        );
        gl::ClearColor(1.0, 1.0, 1.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        gl::Enable(gl::DEPTH_TEST);
    }

    let vert_shader = render_gl::Shader::from_vert_source(
        &CString::new(include_str!("../resources/shaders/shader.vert")).unwrap(),
    )
    .unwrap();
    let frag_shader = render_gl::Shader::from_frag_source(
        &CString::new(include_str!("../resources/shaders/shader.frag")).unwrap(),
    )
    .unwrap();
    let shader_program = render_gl::Program::from_shaders(&[vert_shader, frag_shader]).unwrap();

    // Set up simulation and tracking variables
    let mut simulation = Simulation::init();
    // let mut simulation = Simulation::init_with_height_map(constants::IMPORT_FILE_PATH);
    let export_terrain = false;

    let mut color_mode = ColorMode::Standard;
    let mut path = "".to_string();
    let mut count = 0;
    let mut paused = true;
    let mut prev_keys = HashSet::new();
    let now;
    unsafe {
        now = SDL_GetPerformanceCounter();
    }
    let mut loop_start;
    let mut loop_end = now;
    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
            if let sdl2::event::Event::Quit { .. } = event {
                break 'main;
            }
        }

        // draw
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        shader_program.set_used();
        simulation.draw(shader_program.id(), gl::TRIANGLES);
        // simulation.draw(shader_program.id(), gl::LINES);
        unsafe {
            let mut err: gl::types::GLenum = gl::GetError();
            while err != gl::NO_ERROR {
                // Process/log the error.
                println!("loop error {err}");
                err = gl::GetError();
            }
        }

        // handle ticks
        let elapsed_secs;
        unsafe {
            loop_start = SDL_GetPerformanceCounter();
            elapsed_secs = (loop_start - loop_end) as f64 / SDL_GetPerformanceFrequency() as f64;

            if !paused {
                println!("\nTime step {count}");
                println!("elapsed_secs {elapsed_secs}");
                simulation.take_time_step(&color_mode);
                count += 1;
                let duration = (0.1 - elapsed_secs) * 1000.0;
                println!("sleep duration {duration} ms");
                // sleep(Duration::from_millis(duration as u64));

                // export terrain
                if export_terrain {
                    if path.is_empty() {
                        // create directory for export
                        let now = chrono::Local::now();
                        let today = now.date_naive().format("%Y_%m_%d").to_string();
                        let time = now.time().format("%H_%M_%S").to_string();
                        path = format!("./output/{today}-{time}");
                        println!("{path}");
                        std::fs::create_dir(path.clone()).unwrap();
                    }
                    export_height_map(&simulation.ecosystem.ecosystem, count, &path);
                }
            }
            loop_end = SDL_GetPerformanceCounter();
        }

        // Handle key input
        // Create a set of pressed Keys.
        let keys: HashSet<Keycode> = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        // Get the difference between the new and old sets.
        let new_keys = &keys - &prev_keys;
        prev_keys = keys.clone();
        if new_keys.contains(&Keycode::Space) {
            // take one time step
            println!("\nTime step {count}");
            simulation.take_time_step(&color_mode);

            // export terrain
            if export_terrain {
                if path.is_empty() {
                    // create directory for export
                    let now = chrono::Local::now();
                    let today = now.date_naive().format("%Y_%m_%d").to_string();
                    let time = now.time().format("%H_%M_%S").to_string();
                    path = format!("./output/{today}-{time}");
                    println!("{path}");
                    std::fs::create_dir(path.clone()).unwrap();
                }
                export_height_map(&simulation.ecosystem.ecosystem, count, &path);
            }

            count += 1;
        } else if new_keys.contains(&Keycode::T) {
            // continuously take time steps
            paused = !paused;
        } else if new_keys.contains(&Keycode::P) {
            // export current data
            if path.is_empty() {
                // create directory for export
                let now = chrono::Local::now();
                let today = now.date_naive().format("%Y_%m_%d").to_string();
                let time = now.time().format("%H_%M_%S").to_string();
                path = format!("./output/{today}-{time}");
                println!("{path}");
                std::fs::create_dir(path.clone()).unwrap();
            }
            export_maps(&simulation.ecosystem.ecosystem, count, &path);
        } else if new_keys.contains(&Keycode::Num1) {
            // change color mode
            color_mode = ColorMode::Standard;
            simulation.change_color_mode(&color_mode);
        } else if new_keys.contains(&Keycode::Num2) {
            // change color mode
            color_mode = ColorMode::HypsometricTint;
            simulation.change_color_mode(&color_mode);
        } else if new_keys.contains(&Keycode::Num3) {
            // change color mode
            color_mode = ColorMode::Sunlight;
            simulation.change_color_mode(&color_mode);
        } else if new_keys.contains(&Keycode::Num4) {
            // change color mode
            color_mode = ColorMode::SoilMoisture;
            simulation.change_color_mode(&color_mode);
        }
        let dirs = keys.into_iter().filter_map(convert_key_to_dir).collect();
        move_camera(&mut simulation.ecosystem, dirs, elapsed_secs as f32);

        window.gl_swap_window();
    }
}

fn convert_key_to_dir(key: Keycode) -> Option<Direction> {
    match key {
        Keycode::W => Some(Direction::Up),
        Keycode::S => Some(Direction::Down),
        Keycode::A => Some(Direction::Left),
        Keycode::D => Some(Direction::Right),
        Keycode::LShift => Some(Direction::Forward),
        Keycode::LCtrl => Some(Direction::Back),
        _ => None,
    }
}

fn move_camera(ecosystem: &mut EcosystemRenderable, dirs: HashSet<Direction>, delta_seconds: f32) {
    let mut m_forward = 0.0;
    let mut m_sideways = 0.0;
    let mut m_vertical = 0.0;

    for dir in dirs {
        match dir {
            Direction::Up => m_vertical -= constants::SPEED,
            Direction::Down => m_vertical += constants::SPEED,
            Direction::Left => m_sideways += constants::SPEED,
            Direction::Right => m_sideways -= constants::SPEED,
            Direction::Forward => m_forward += constants::SPEED,
            Direction::Back => m_forward -= constants::SPEED,
        }
    }
    let mut look = ecosystem.m_camera.m_look;
    look.y = 0.0;
    look = look.normalize();
    let perp: Vector3<f32> = Vector3::new(-look.z, 0.0, look.x).normalize();
    let mut move_vec: Vector3<f32> =
        m_forward * look + m_sideways * perp + m_vertical * Vector3::new(0.0, 1.0, 0.0);
    move_vec *= delta_seconds;
    ecosystem.m_camera.move_camera(move_vec);
}
