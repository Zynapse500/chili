#![allow(non_snake_case)]

// Disable console window if on windows
#![cfg_attr(windows, windows_subsystem = "windows")]

extern crate crank;

use std::path::Path;
use std::env;

pub fn main() {
    let settings = crank::AppSettings {
        clear_color: [0.2, 0.2, 0.2, 1.0]
    };

    crank::run_app::<App>(680, 680,"Chili", settings).unwrap();
}


struct App {
    running: bool,
    window: crank::WindowHandle,

    batch: crank::RenderBatch,

    zoom: f32,
    max_zoom: f32,
    min_zoom: f32,

    focus: [f32; 2],

    texture: Option<crank::Texture>,
    image_size: (f32, f32)
}


impl App {
    fn draw(&mut self) {
        self.batch.clear();

        let view = self.calculate_view();
        self.batch.set_view(view);

        let (w, h) = self.image_size;

        self.batch.set_texture(self.texture);
        self.batch.set_fill_color([1.0; 4]);
        self.batch.draw_rectangle([-w / 2.0, -h / 2.0], [w, h]);
    }


    fn calculate_view(&self) -> crank::CenteredView {
        let w = self.window.get_width() as f32 / self.zoom;
        let h = self.window.get_height() as f32 / self.zoom;

        crank::CenteredView {
            center: self.focus,
            size: [w, h]
        }
    }

    fn create_texture(image: Option<crank::Image>) -> Option<crank::Texture> {
        if let Some(image) = image {
            let mut texture = crank::Texture::from(image);
            texture.set_min_mag_filter(crank::TextureFilter::Linear,
                                       crank::TextureFilter::Nearest);

            Some(texture)
        } else {
            let pixels = crank::TextureData::RGBA(
                &[
                    255, 0, 255, 255, 0,   0,   0, 255,
                    0,   0,   0, 255, 255, 0, 255, 255
                ]
            );

            let mut texture = crank::Texture::new(2, 2, pixels);

            texture.set_min_mag_filter(crank::TextureFilter::Linear,
                                       crank::TextureFilter::Nearest);

            Some(texture)
        }
    }
}


impl crank::App for App {
    fn setup(window: crank::WindowHandle) -> Self {
        let args: Vec<String> = env::args().collect();
        let (image, size) = if args.len() > 1 {
            match crank::Image::load_png(Path::new(&args[1])) {
                Ok(image) => {
                    let (w, h) = image.get_size();
                    (Some(image), (w as f32, h as f32))
                },
                Err(e) => panic!(e)
            }
        } else {
            (None, (1.0, 1.0))
        };

        let window_size = window.get_size();

        let zoom = zoom_from_sizes(window_size, [size.0 as u32, size.1 as u32]);

        App {
            running: true,
            window,

            batch: crank::RenderBatch::new(),

            zoom,
            min_zoom: 0.1 * zoom,
            max_zoom: 100.0 * zoom,

            focus: [0.0, 0.0],

            texture: App::create_texture(image),
            image_size: size
        }
    }

    fn render(&self, renderer: &mut crank::Renderer) {
        renderer.submit_batch(&self.batch);
    }

    fn is_running(&self) -> bool {
        self.running
    }
}


impl crank::WindowEventHandler for App {
    fn size_changed(&mut self, _width: u32, _height: u32) {
        let zoom = zoom_from_sizes(self.window.get_size(), [self.image_size.0 as u32, self.image_size.1 as u32]);

        self.min_zoom = 0.1 * zoom;
        self.max_zoom = 100.0 * zoom;

        self.draw();
    }


    fn key_pressed(&mut self, key: crank::KeyCode) {
        match key {
            crank::KeyCode::Escape => self.running = false,

            _ => ()
        }
    }


    fn mouse_scrolled(&mut self, delta: crank::ScrollDelta) {
        match delta {
            crank::ScrollDelta::LineDelta(_x, y) => {

                const ZOOM_AMOUNT: f32 = 1.5;

                if y > 0.0 {
                    self.zoom *= ZOOM_AMOUNT;
                } else {
                    self.zoom /= ZOOM_AMOUNT;
                }

                if self.zoom > self.max_zoom {
                    self.zoom = self.max_zoom;
                }
                if self.zoom < self.min_zoom {
                    self.zoom = self.min_zoom;
                }

                self.draw();
            }

            crank::ScrollDelta::PixelDelta(_x, _y) => {}
        }
    }


    fn mouse_moved(&mut self, x: i32, y: i32) {
        use crank::View;

        // Pan
        if self.window.mouse_down(crank::MouseButton::Left) ||
            self.window.mouse_down(crank::MouseButton::Right){
            let last_position = self.calculate_view().ndc_to_world(
                self.window.window_to_ndc(self.window.get_cursor_position())
            );

            let current_position = self.calculate_view().ndc_to_world(
                self.window.window_to_ndc([x, y])
            );

            let delta = crank::vec2_sub(last_position, current_position);

            self.focus = crank::vec2_add(self.focus, delta);
        }

        self.draw();
    }
}


impl crank::WindowFileHandler for App {
    fn file_dropped(&mut self, path: &Path) {
        println!("Dropped file: {:?}", path.to_str());

        let image = crank::Image::load_png(path);

        match image {
            Err(e) => println!("{}", e),

            Ok(image) => {
                let (w, h) = image.get_size();
                self.image_size = (w as f32, h as f32);

                self.texture = App::create_texture(Some(image));

                self.zoom = zoom_from_sizes(self.window.get_size(), [w, h]);
                self.min_zoom = 0.1 * self.zoom;
                self.max_zoom = 100.0 * self.zoom;


                self.draw();
            }
        }
    }
}





fn zoom_from_sizes(bound: [u32; 2], rect: [u32; 2]) -> f32 {
    let zoom_x = bound[0] as f32 / rect[0] as f32;
    let zoom_y = bound[1] as f32 / rect[1] as f32;

    if zoom_x < zoom_y {zoom_x} else {zoom_y}
}


