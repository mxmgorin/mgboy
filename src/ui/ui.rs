use crate::bus::Bus;
use crate::config::GraphicsConfig;
use crate::ppu::{Ppu, LCD_X_RES, LCD_Y_RES};
use crate::tile::PixelColor;
use crate::ui::debug_window::DebugWindow;
use crate::ui::events::{UiEvent, UiEventHandler};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::FRect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::EventPump;

pub const SCREEN_WIDTH: u32 = 640;
pub const SCREEN_HEIGHT: u32 = 480;

pub struct Ui {
    _sdl_context: sdl2::Sdl,
    event_pump: EventPump,

    main_canvas: Canvas<Window>,
    debug_window: Option<DebugWindow>,
    layout: Layout,

    pub config: GraphicsConfig,
    pub curr_pallet_idx: usize,
}

pub struct Layout {
    pub scale: f32,
    pub spacer: i32,
    pub y_spacer: i32,
    pub x_draw_start: i32,
    pub win_width: u32,
    pub win_height: u32,
}

impl Layout {
    pub fn new(scale: f32) -> Self {
        Self {
            scale,
            spacer: 8 * scale as i32,
            y_spacer: scale as i32,
            x_draw_start: scale as i32 / 2,
            win_width: LCD_X_RES as u32 * scale as u32,
            win_height: LCD_Y_RES as u32 * scale as u32,
        }
    }
}

impl Ui {
    pub fn new(config: GraphicsConfig, debug: bool) -> Result<Self, String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let layout = Layout::new(config.scale);

        let main_window = video_subsystem
            .window("GMBoy", layout.win_width, layout.win_height)
            .position_centered()
            .resizable()
            .build()
            .unwrap();
        let mut main_canvas = main_window.into_canvas().build().unwrap();
        main_canvas.set_scale(config.scale, config.scale)?;

        let (x, y) = main_canvas.window().position();
        let mut debug_window = DebugWindow::new(&video_subsystem);
        debug_window.set_position(x + SCREEN_WIDTH as i32 + 10, y);

        let curr_pallet_idx = config
            .pallets
            .iter()
            .position(|p| p.name == config.selected_pallet)
            .unwrap_or_default();

        Ok(Ui {
            event_pump: sdl_context.event_pump()?,
            _sdl_context: sdl_context,
            //ttf_context,
            main_canvas,
            debug_window: if debug { Some(debug_window) } else { None },
            layout,
            curr_pallet_idx,
            config,
        })
    }

    fn set_scale(&mut self, val: f32) -> Result<(), String> {
        self.layout = Layout::new(val);
        self.main_canvas.set_scale(val, val)?;
        let window = self.main_canvas.window_mut();
        window
            .set_size(self.layout.win_width, self.layout.win_height)
            .map_err(|e| e.to_string())?;
        window.set_position(
            sdl2::video::WindowPos::Centered,
            sdl2::video::WindowPos::Centered,
        );

        Ok(())
    }

    pub fn draw(&mut self, ppu: &Ppu, bus: &Bus) {
        self.draw_main(ppu);

        if let Some(debug_window) = self.debug_window.as_mut() {
            debug_window.draw(bus);
        }
    }

    fn draw_main(&mut self, ppu: &Ppu) {
        let mut rect = FRect::new(0.0, 0.0, self.layout.scale, self.layout.scale);
        self.main_canvas.clear();

        for y in 0..(LCD_Y_RES as usize) {
            for x in 0..(LCD_X_RES as usize) {
                let pixel = ppu.pipeline.buffer[x + (y * LCD_X_RES as usize)];
                rect.x = x as f32;
                rect.y = y as f32;
                let (r, g, b, a) = pixel.color.as_rgba();
                self.main_canvas.set_draw_color(Color::RGBA(r, g, b, a));
                self.main_canvas.fill_frect(rect).unwrap();
            }
        }

        //fill_rects2(&mut self.main_canvas, &self.rects_by_colors, rects_count);
        self.main_canvas.present();
    }

    pub fn handle_events(&mut self, bus: &mut Bus, event_handler: &mut impl UiEventHandler) {
        let mut new_scale = None;
        for event in self.event_pump.poll_iter() {
            match event {
                Event::DropFile { filename, .. } => {
                    event_handler.on_event(bus, UiEvent::DropFile(filename))
                }
                Event::Quit { .. } => event_handler.on_event(bus, UiEvent::Quit),
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    match keycode {
                        Keycode::EQUALS => new_scale = Some(self.layout.scale + 1.0),
                        Keycode::MINUS => new_scale = Some(self.layout.scale - 1.0),
                        Keycode::P => {
                            self.curr_pallet_idx = get_next_pallet_idx(
                                self.curr_pallet_idx,
                                self.config.pallets.len() - 1,
                            );

                            bus.io.lcd.set_pallet(into_pallet(
                                &self.config.pallets[self.curr_pallet_idx].hex_colors,
                            ));
                        }
                        _ => (),
                    }

                    event_handler.on_event(bus, UiEvent::Key(keycode, true))
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => event_handler.on_event(bus, UiEvent::Key(keycode, false)),
                Event::Window {
                    win_event: sdl2::event::WindowEvent::Close,
                    window_id,
                    ..
                } => {
                    if let Some(window) = self.debug_window.as_mut() {
                        if window.canvas.window().id() == window_id {
                            self.debug_window = None;
                        } else {
                            event_handler.on_event(bus, UiEvent::Quit);
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some(new_scale) = new_scale {
            self.set_scale(new_scale).unwrap();
        }
    }
}

pub fn into_pallet(hex_colors: &[String]) -> [PixelColor; 4] {
    let colors: Vec<PixelColor> = hex_colors
        .iter()
        .map(|hex| PixelColor::from_hex(u32::from_str_radix(hex, 16).unwrap()))
        .collect();

    colors[..4].try_into().unwrap()
}

pub fn get_next_pallet_idx(curr_idx: usize, max_idx: usize) -> usize {
    if curr_idx < max_idx {
        curr_idx + 1
    } else {
        0
    }
}
#[cfg(test)]
mod tests {
    use crate::ui::into_pallet;

    #[test]
    fn test_parse_hex_colors() {
        let colors = vec!["FF0F0F1B".to_string()];
        let colors = into_pallet(&colors);

        println!("{:?}", colors);
    }
}
