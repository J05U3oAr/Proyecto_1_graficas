//! Audio del juego: musica del perseguidor y habilidad sonora.

use std::{
    fs::File,
    path::PathBuf,
    time::{Duration, Instant},
};

use rodio::Source;

use crate::{chaser::Chaser, player::Player};

const CHASER_AUDIO_FILE: &str = "I Wonder.mp3";
const DISGUST_AUDIO_FILE: &str = "The Fate of Ophelia.mp3";
const DISGUST_AUDIO_DURATION: Duration = Duration::from_secs(5);
const DISGUST_AUDIO_VOLUME: f32 = 0.20;
const CHASER_MAX_VOLUME: f32 = 0.42;
const CHASER_AUDIBLE_DISTANCE: f32 = 14.0;
const EAR_DISTANCE: f32 = 0.7;

/// Controla las pistas del juego y su mezcla espacial.
pub struct AudioSystem {
    _device_sink: Option<rodio::stream::MixerDeviceSink>,
    chaser_player: Option<rodio::SpatialPlayer>,
    ability_player: Option<rodio::Player>,
    ability_until: Option<Instant>,
}

impl AudioSystem {
    /// Inicializa el dispositivo de audio y deja la musica del chaser preparada.
    pub fn new() -> Self {
        let Ok(device_sink) = rodio::DeviceSinkBuilder::open_default_sink() else {
            eprintln!("Audio device not available. Continuing without sound.");
            return Self::disabled();
        };

        let chaser_player = rodio::SpatialPlayer::connect_new(
            device_sink.mixer(),
            [0.0, 0.0, 1.0],
            [-EAR_DISTANCE / 2.0, 0.0, 0.0],
            [EAR_DISTANCE / 2.0, 0.0, 0.0],
        );
        let ability_player = rodio::Player::connect_new(device_sink.mixer());

        let mut audio = Self {
            _device_sink: Some(device_sink),
            chaser_player: Some(chaser_player),
            ability_player: Some(ability_player),
            ability_until: None,
        };
        audio.load_chaser_loop();
        audio.pause_chaser();
        audio
    }

    /// Actualiza posicion, direccion y volumen del audio del chaser.
    pub fn update_chaser_audio(&mut self, player: &Player, chaser: &Chaser, playing: bool) {
        self.update_ability_timer();

        let Some(chaser_player) = &self.chaser_player else {
            return;
        };

        if !playing || self.ability_until.is_some() {
            chaser_player.pause();
            return;
        }

        let rel_x = chaser.x - player.x;
        let rel_y = chaser.y - player.y;
        let distance = rel_x.hypot(rel_y);
        let right = rel_x * -player.angle.sin() + rel_y * player.angle.cos();
        let forward = rel_x * player.angle.cos() + rel_y * player.angle.sin();
        let distance_ratio = (distance / CHASER_AUDIBLE_DISTANCE).clamp(0.0, 1.0);
        let closeness = 1.0 - distance_ratio * distance_ratio;
        let active_boost = if chaser.active() { 1.0 } else { 0.58 };
        let volume = (closeness * CHASER_MAX_VOLUME * active_boost).clamp(0.0, CHASER_MAX_VOLUME);

        chaser_player.set_emitter_position([right, 0.0, forward]);
        chaser_player.set_left_ear_position([-EAR_DISTANCE / 2.0, 0.0, 0.0]);
        chaser_player.set_right_ear_position([EAR_DISTANCE / 2.0, 0.0, 0.0]);
        chaser_player.set_volume(volume);
        chaser_player.set_speed(if chaser.active() { 1.04 } else { 0.94 });

        if chaser_player.is_paused() && volume > 0.01 {
            chaser_player.play();
        } else if volume <= 0.01 {
            chaser_player.pause();
        }
    }

    /// Reproduce la habilidad y pausa la musica espacial del chaser mientras dura.
    pub fn play_disgust_sound(&mut self) {
        self.update_ability_timer();

        if self.ability_until.is_some() {
            return;
        }

        self.pause_chaser();

        let Some(ability_player) = &self.ability_player else {
            return;
        };
        let Some(path) = asset_path(DISGUST_AUDIO_FILE) else {
            eprintln!("Disgust audio not found: {DISGUST_AUDIO_FILE}");
            return;
        };
        let Ok(file) = File::open(&path) else {
            eprintln!("Could not open disgust audio: {}", path.display());
            return;
        };
        let Ok(source) = rodio::Decoder::new_mp3(file) else {
            eprintln!("Could not decode disgust audio: {}", path.display());
            return;
        };

        ability_player.stop();
        ability_player.set_volume(1.0);
        ability_player.append(
            source
                .take_duration(DISGUST_AUDIO_DURATION)
                .amplify(DISGUST_AUDIO_VOLUME),
        );
        ability_player.play();
        self.ability_until = Some(Instant::now() + DISGUST_AUDIO_DURATION);
    }

    /// Pausa el audio del perseguidor, util al salir al menu.
    pub fn pause_chaser(&self) {
        if let Some(chaser_player) = &self.chaser_player {
            chaser_player.pause();
        }
    }

    fn disabled() -> Self {
        Self {
            _device_sink: None,
            chaser_player: None,
            ability_player: None,
            ability_until: None,
        }
    }

    fn load_chaser_loop(&mut self) {
        let Some(chaser_player) = &self.chaser_player else {
            return;
        };
        let Some(path) = asset_path(CHASER_AUDIO_FILE) else {
            eprintln!("Chaser audio not found: {CHASER_AUDIO_FILE}");
            return;
        };
        let Ok(file) = File::open(&path) else {
            eprintln!("Could not open chaser audio: {}", path.display());
            return;
        };
        let Ok(source) = rodio::Decoder::new_mp3(file) else {
            eprintln!("Could not decode chaser audio: {}", path.display());
            return;
        };

        chaser_player.append(source.repeat_infinite());
    }

    fn update_ability_timer(&mut self) {
        let Some(until) = self.ability_until else {
            return;
        };

        if Instant::now() >= until {
            if let Some(ability_player) = &self.ability_player {
                ability_player.stop();
            }

            self.ability_until = None;
        }
    }
}

fn asset_path(file_name: &str) -> Option<PathBuf> {
    asset_roots()
        .into_iter()
        .map(|root| root.join(file_name))
        .find(|path| path.exists())
}

fn asset_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"));

    if let Ok(current_dir) = std::env::current_dir() {
        roots.push(current_dir.join("assets"));
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            roots.push(exe_dir.join("assets"));
        }
    }

    roots
}
