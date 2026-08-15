use std::time::{Duration, Instant};

use minifb::{Key, Scale, Window, WindowOptions};

use crate::{
    config::{SCREEN_HEIGHT, SCREEN_WIDTH, TARGET_FPS},
    input::InputState,
    map::Map,
    player::Player,
    renderer::Renderer,
};

pub struct Game {
    window: Window,
    renderer: Renderer,
    map: Map,
    player: Player,
    previous_frame: Instant,
    fps_timer: Instant,
    frame_counter: u32,
    displayed_fps: u32,
}

impl Game {
    pub fn new() -> Result<Self, minifb::Error> {
        let mut window = Window::new(
            "Ray Caster - Proyecto 1",
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            WindowOptions {
                resize: false,
                scale: Scale::X1,
                ..WindowOptions::default()
            },
        )?;

        window.set_target_fps(TARGET_FPS);

        Ok(Self {
            window,
            renderer: Renderer::new(SCREEN_WIDTH, SCREEN_HEIGHT),
            map: Map::level_one(),
            player: Player::new(2.5, 2.5, 0.0),
            previous_frame: Instant::now(),
            fps_timer: Instant::now(),
            frame_counter: 0,
            displayed_fps: 0,
        })
    }

    pub fn run(&mut self) -> Result<(), minifb::Error> {
        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            let now = Instant::now();
            let dt = (now - self.previous_frame).as_secs_f32();
            self.previous_frame = now;

            let input = InputState::from_window(&self.window);
            self.player.update(&input, &self.map, dt);
            self.update_fps(now);

            self.renderer
                .render(&self.map, &self.player, self.displayed_fps);

            self.window
                .update_with_buffer(self.renderer.buffer(), SCREEN_WIDTH, SCREEN_HEIGHT)?;
        }

        Ok(())
    }

    fn update_fps(&mut self, now: Instant) {
        self.frame_counter += 1;

        if now.duration_since(self.fps_timer) >= Duration::from_secs(1) {
            self.displayed_fps = self.frame_counter;
            self.frame_counter = 0;
            self.fps_timer = now;

            self.window.set_title(&format!(
                "Ray Caster - Proyecto 1 | FPS: {}",
                self.displayed_fps
            ));
        }
    }
}
