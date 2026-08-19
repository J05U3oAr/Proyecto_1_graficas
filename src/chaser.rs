//! Pared perseguidora con activacion por rango y pathfinding por celdas.
//!
//! Permanece quieta hasta que el jugador entra en su radio de deteccion. Una
//! vez activa, recalcula una ruta por la grilla y se apaga si el jugador logra
//! alejarse lo suficiente.

use std::collections::VecDeque;

use crate::{
    config::{
        CHASER_HIT_DISTANCE, CHASER_LOSE_DISTANCE, CHASER_RADIUS, CHASER_REPATH_INTERVAL,
        CHASER_SPEED, CHASER_WAKE_DISTANCE,
    },
    map::Map,
    player::Player,
};

/// Evento producido por la pared perseguidora durante el frame.
pub enum ChaserEvent {
    /// El jugador entro al radio de deteccion.
    Spotted,
    /// El jugador logro salir del radio de persecucion.
    Lost,
    /// La pared alcanzo al jugador.
    HitPlayer,
}

/// Estado de la pared perseguidora.
pub struct Chaser {
    /// Posicion X en coordenadas de mapa.
    pub x: f32,
    /// Posicion Y en coordenadas de mapa.
    pub y: f32,
    spawn_x: f32,
    spawn_y: f32,
    active: bool,
    path: Vec<(i32, i32)>,
    repath_timer: f32,
}

impl Chaser {
    /// Crea la pared perseguidora en su posicion inicial.
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            spawn_x: x,
            spawn_y: y,
            active: false,
            path: Vec::new(),
            repath_timer: 0.0,
        }
    }

    /// Indica si la pared esta persiguiendo actualmente.
    pub fn active(&self) -> bool {
        self.active
    }

    /// Restaura la pared a su posicion inicial y la deja dormida.
    pub fn reset(&mut self) {
        self.x = self.spawn_x;
        self.y = self.spawn_y;
        self.active = false;
        self.path.clear();
        self.repath_timer = 0.0;
    }

    /// Actualiza deteccion, ruta y movimiento.
    pub fn update(&mut self, map: &Map, player: &Player, dt: f32) -> Option<ChaserEvent> {
        if map.completed() {
            self.active = false;
            self.path.clear();
            return None;
        }

        let distance_to_player = self.distance_to_player(player);

        if !self.active {
            if distance_to_player <= CHASER_WAKE_DISTANCE {
                self.active = true;
                self.repath(map, player);
                return Some(ChaserEvent::Spotted);
            }

            return None;
        }

        if distance_to_player >= CHASER_LOSE_DISTANCE {
            self.active = false;
            self.path.clear();
            return Some(ChaserEvent::Lost);
        }

        self.repath_timer -= dt;

        if self.repath_timer <= 0.0 {
            self.repath(map, player);
        }

        self.follow_path(map, dt);

        if self.distance_to_player(player) <= CHASER_HIT_DISTANCE {
            return Some(ChaserEvent::HitPlayer);
        }

        None
    }

    fn distance_to_player(&self, player: &Player) -> f32 {
        (self.x - player.x).hypot(self.y - player.y)
    }

    fn repath(&mut self, map: &Map, player: &Player) {
        self.repath_timer = CHASER_REPATH_INTERVAL;
        let start = (self.x.floor() as i32, self.y.floor() as i32);
        let goal = (player.x.floor() as i32, player.y.floor() as i32);
        self.path = find_path(map, start, goal);
    }

    fn follow_path(&mut self, map: &Map, dt: f32) {
        let Some(&(target_cell_x, target_cell_y)) = self.path.first() else {
            return;
        };

        let target_x = target_cell_x as f32 + 0.5;
        let target_y = target_cell_y as f32 + 0.5;
        let delta_x = target_x - self.x;
        let delta_y = target_y - self.y;
        let distance = delta_x.hypot(delta_y);

        if distance < 0.04 {
            self.path.remove(0);
            return;
        }

        let step = (CHASER_SPEED * dt).min(distance);
        let next_x = self.x + delta_x / distance * step;
        let next_y = self.y + delta_y / distance * step;

        if map.can_stand_at(next_x, next_y, CHASER_RADIUS) {
            self.x = next_x;
            self.y = next_y;
        } else {
            self.path.clear();
        }
    }
}

fn find_path(map: &Map, start: (i32, i32), goal: (i32, i32)) -> Vec<(i32, i32)> {
    if start == goal {
        return Vec::new();
    }

    if !map.is_walkable_cell(goal.0, goal.1) {
        return Vec::new();
    }

    let width = map.width();
    let height = map.height();
    let Some(start_index) = cell_index(map, start.0, start.1) else {
        return Vec::new();
    };
    let Some(goal_index) = cell_index(map, goal.0, goal.1) else {
        return Vec::new();
    };

    let mut queue = VecDeque::from([start]);
    let mut came_from = vec![None; width * height];
    came_from[start_index] = Some(start);

    while let Some((x, y)) = queue.pop_front() {
        if (x, y) == goal {
            break;
        }

        for (next_x, next_y) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
            let Some(next_index) = cell_index(map, next_x, next_y) else {
                continue;
            };

            if came_from[next_index].is_some() || !map.is_walkable_cell(next_x, next_y) {
                continue;
            }

            came_from[next_index] = Some((x, y));
            queue.push_back((next_x, next_y));
        }
    }

    if came_from[goal_index].is_none() {
        return Vec::new();
    }

    let mut path = Vec::new();
    let mut current = goal;

    while current != start {
        path.push(current);
        let current_index = cell_index(map, current.0, current.1).expect("path cell in bounds");
        current = came_from[current_index].expect("path predecessor exists");
    }

    path.reverse();
    path
}

fn cell_index(map: &Map, x: i32, y: i32) -> Option<usize> {
    if x < 0 || y < 0 || x >= map.width() as i32 || y >= map.height() as i32 {
        return None;
    }

    Some(y as usize * map.width() + x as usize)
}
