use std::time::{Duration, Instant};

use minifb::{Key, Scale, Window, WindowOptions};

use crate::{
    config::{MESSAGE_DURATION, SCREEN_HEIGHT, SCREEN_WIDTH, TARGET_FPS},
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
    message: &'static str,
    message_timer: f32,
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

        let map = Map::level_one();
        let (player_x, player_y, player_angle) = map.player_spawn();

        Ok(Self {
            window,
            renderer: Renderer::new(SCREEN_WIDTH, SCREEN_HEIGHT),
            map,
            player: Player::new(player_x, player_y, player_angle),
            previous_frame: Instant::now(),
            fps_timer: Instant::now(),
            frame_counter: 0,
            displayed_fps: 0,
            message: "FIND KEY",
            message_timer: MESSAGE_DURATION,
        })
    }

    pub fn run(&mut self) -> Result<(), minifb::Error> {
        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            let now = Instant::now();
            let dt = (now - self.previous_frame).as_secs_f32();
            self.previous_frame = now;

            let input = InputState::from_window(&self.window);
            self.player.update(&input, &self.map, dt);
            self.update_interactions(dt);
            self.update_fps(now);

            self.renderer
                .render(&self.map, &self.player, self.displayed_fps, self.message);

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

    fn update_interactions(&mut self, dt: f32) {
        self.message_timer = (self.message_timer - dt).max(0.0);

        if self.player.touched_hazard() {
            self.player.take_hit_and_respawn();
            self.message = "SPIKES HIT";
            self.message_timer = MESSAGE_DURATION;
        } else if let Some(message) = self.map.update_player_interactions(&mut self.player) {
            self.message = message;
            self.message_timer = MESSAGE_DURATION;
        } else if self.message_timer <= 0.0 {
            self.message = self.current_goal_message();
        }
    }

    fn current_goal_message(&self) -> &'static str {
        if self.map.completed() {
            "LEVEL COMPLETE"
        } else if !self.map.has_key() {
            "FIND KEY"
        } else if !self.map.switch_pressed() {
            "PRESS SWITCH"
        } else {
            "REACH EXIT"
        }
    }
}
