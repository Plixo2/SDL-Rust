extern crate sdl2;
use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::camera::Camera;
use crate::sdl2::sys;
use crate::shader::ShaderLoader;
use glam::{Vec2, Vec4, Mat4};
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::video::{GLProfile, WindowPos};
mod camera;
mod editor;
mod font;
mod image;
mod shader;
mod ui;
mod styling;

fn find_sdl_gl_driver() -> Option<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() {
        if item.name == "opengl" {
            return Some(index as u32);
        }
    }
    None
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let attr = video_subsystem.gl_attr();
    attr.set_context_profile(GLProfile::Core);
    attr.set_context_version(3, 3);
    //let surface = Surface::from_file(Path::new("window.PNG"))
    //    .map_err(|err| format!("failed to load cursor image: {}", err))?;

    //let h = surface.height();
    //let w = surface.width();
    let h = 500;
    let w = 800;
    println!("Window {} x {}", w, h);
    let window = video_subsystem
        .window("Window", w, h)
        .opengl() // this line DOES NOT enable opengl, but allows you to create/get an OpenGL context from your window.
        .build()
        .unwrap();

    //let title = CString::new("EDITOR").unwrap();
    //let mut window = unsafe {
    //    let raw = sys::SDL_CreateShapedWindow(
    //        title.as_ptr() as *const i8,
    //        to_ll_windowpos(WindowPos::Centered) as u32,
    //        to_ll_windowpos(WindowPos::Centered) as u32,
    //        w as u32,
    //        h as u32,
    //        sys::SDL_WindowFlags::SDL_WINDOW_OPENGL as u32,
    //    );

    //    Window::from_ll(video_subsystem.clone(), raw)
    //};

    //unsafe {
    //    let param = sys::SDL_WindowShapeParams {
    //        binarizationCutoff: 100,
    //    };
    //    let mut str = sys::SDL_WindowShapeMode {
    //        mode: sys::WindowShapeMode::ShapeModeDefault,
    //        parameters: param,
    //    };
    //    let c = sys::SDL_SetWindowShape(window.raw(), surface.raw(), &mut str);
    //    assert_eq!(c, 0, "Cant create custom SDL window shape");
    //}
    let mut event_pump = sdl_context.event_pump()?;

    let _context = window.gl_create_context()?;
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

    let mut canvas = window
        .into_canvas()
        .index(find_sdl_gl_driver().unwrap())
        .build()
        .unwrap();
    let texture_creator = canvas.texture_creator();
    canvas.window().gl_set_context_to_current()?;
    video_subsystem.gl_set_swap_interval(1)?;

    let texture = texture_creator
        .load_texture(&Path::new("resources/image/test.png"))
        .unwrap();

    let mut gui = ui::UIShader::new(&Path::new("resources/shaders/rounded"));
    let camera = Camera::new();

    let mut shader = image::ImageShader::new(&Path::new("resources/shaders/font"));

    let font_path = Path::new("resources/font/consola.ttf");
    let mut font_obj = font::FontRenderer::new(
        &font_path,
        16.75,
        &Path::new("resources/shaders/textured"),
        &texture_creator,
    );

    let mut editor = editor::Editor::default();
    editor.paste_lines(vec![String::from("Hello world")]);

    let background = ui::Color::hex(0xFF2D2D2D);

    //let tex = font_obj.textures.into_iter().nth(100).unwrap();

    let mut image = image::Image { texture: texture };

    //#2D2D2D
    //#2C2C2C
    //#171717
    //#1E1E1E
    //#171717 to #2D2D2D

    let mut rect = ui::UIRect::default();
    rect.size = Vec2::new(w as f32, h as f32);
    rect.pos = Vec2::ZERO;
    rect.roundness = 9.0;
    rect.color = background;
    rect.outline_color = ui::Color::hex(0x55_FF_FF_FF);
    rect.outline_width = 2.0;

    let mut top = ui::UIRect::default();
    top.pos = Vec2::new(20.0, 40.0);
    top.size = Vec2::new((w - 40) as f32, (h - 60) as f32);
    top.roundness = 15.0;
    top.hover_color = ui::Color::hex(0x55_00_00_00);
    top.color = ui::Color::hex(0xFF171717);

    let mut button = ui::UIRect::default();
    button.pos = Vec2::new(40.0, 14.0);
    button.size = Vec2::new(12.0, 12.0);
    button.roundness = 100.0;
    button.set_color(ui::TRANSPARENT, ui::WHITE, ui::TRANSPARENT);
    button.outline_color = ui::Color::hex(0x55_FF_FF_FF);
    button.outline_width = 1.0;
    rect.children.push(button);

    rect.children.push(top);

    let mut previous = SystemTime::now();
    let mut last_fps = SystemTime::now();
    let mut fps_count = 0;
    let mut micros = 0u128;
    let mut fps = 0;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::TextInput {
                    timestamp,
                    window_id,
                    text,
                } => {
                    println!("text {}", text);
                    editor.handle_text(text)
                }
                Event::KeyDown {
                    timestamp,
                    window_id,
                    keycode,
                    scancode,
                    keymod,
                    repeat,
                } => match keycode {
                    Some(Keycode::Escape) => {
                        break 'running;
                    }
                    Some(code) => {
                        editor.handle_input(code);
                    }
                    _ => {}
                },
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }
        let time = SystemTime::now();
        let delta = time.duration_since(previous).unwrap();
        micros += delta.as_micros();
        fps_count += 1;
        if time.duration_since(last_fps).unwrap().as_millis() >= 1000 {
            //  println!("{} fps ({} ms)", fps_count, (micros / fps_count) as f32 / 1000.0);
            fps = fps_count;
            fps_count = 0;
            micros = 0;
            last_fps = time;
        }
        previous = time;
        let mouse = event_pump.mouse_state();
        let _mouse_vec = Vec2::new(mouse.x() as f32, mouse.y() as f32);
        let slowmo = false;
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::DEPTH_TEST);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            gl::Disable(gl::CULL_FACE);
            let color: Vec4 = ui::Color::hex(0xFF282c34).into();
            gl::ClearColor(color.x, color.y, color.z, color.w);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            let camera = &camera.matrix(w, h);
            font_obj.set_view(camera);
            gui.set_view(camera);
            gui.transform = Mat4::IDENTITY;

            font_obj.draw(
                &format!("FPS {}", fps),
                00.0,
                300.0,
                ui::WHITE,
            );

            editor.render(&mut gui, &mut font_obj);

            //shader.shader.bind();
            //shader
            //    .shader
            //    .load_location_mat4(shader.location_projection, camera);
            //shader.upload(&mut image, ui::WHITE);
            //shader.shader.unbind();

            //shader.shader.bind();
            //shader
            //    .shader
            //    .load_location_mat4(shader.location_projection, &camera.matrix(w, h));
            //let delta = delta.as_micros() as f32 / (100000.0 * if slowmo { 1.0 } else { 10.0 });
            //shader.upload(&mut rect, UIPayload(delta, mouse_vec));
            //shader.shader.unbind();
        }
        if slowmo {
            std::thread::sleep(Duration::from_millis(166));
        }
        //std::thread::sleep(Duration::from_millis(16));

        canvas.window().gl_swap_window();
    }

    Ok(())
}
#[allow(dead_code)]
fn to_ll_windowpos(pos: WindowPos) -> i32 {
    match pos {
        WindowPos::Undefined => sys::SDL_WINDOWPOS_UNDEFINED_MASK as i32,
        WindowPos::Centered => sys::SDL_WINDOWPOS_CENTERED_MASK as i32,
        WindowPos::Positioned(x) => x as i32,
    }
}
