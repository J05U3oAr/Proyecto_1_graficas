//! Pared perseguidora con activacion por rango y pathfinding por celdas.
//!
//! Patrulla pasillos hasta que el jugador entra en su radio de deteccion. Una
//! vez activa, recalcula una ruta por la grilla y se apaga si el jugador logra
//! alejarse lo suficiente.

use std::{
    collections::{HashMap, VecDeque},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    config::{
        CHASER_HIT_DISTANCE, CHASER_LOSE_DISTANCE, CHASER_RADIUS, CHASER_REPATH_INTERVAL,
        CHASER_SPEED, CHASER_WAKE_DISTANCE,
    },
    map::Map,
    player::Player,
};

/// Velocidad usada cuando la pared patrulla sin perseguir al jugador.
const CHASER_PATROL_SPEED: f32 = CHASER_SPEED * 0.68;

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
    patrol_target: Option<(i32, i32)>,
    patrol_direction: (i32, i32),
    visit_counts: HashMap<(i32, i32), u32>,
    rng_state: u32,
}

#[derive(Clone, Copy)]
struct PatrolChoice {
    cell: (i32, i32),
    direction: (i32, i32),
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
            patrol_target: None,
            patrol_direction: (-1, 0),
            visit_counts: HashMap::new(),
            rng_state: random_seed(x, y),
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
        self.patrol_target = None;
        self.visit_counts.clear();
        self.advance_rng();
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
                self.patrol_target = None;
                self.repath(map, player);
                return Some(ChaserEvent::Spotted);
            }

            self.patrol(map, dt);

            if self.distance_to_player(player) <= CHASER_WAKE_DISTANCE {
                self.active = true;
                self.patrol_target = None;
                self.repath(map, player);
                return Some(ChaserEvent::Spotted);
            }

            return None;
        }

        if distance_to_player >= CHASER_LOSE_DISTANCE {
            self.active = false;
            self.path.clear();
            self.patrol_target = None;
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

    fn patrol(&mut self, map: &Map, dt: f32) {
        if self.patrol_target.is_none() {
            self.patrol_target = self.next_patrol_cell(map);
        }

        let Some((target_cell_x, target_cell_y)) = self.patrol_target else {
            return;
        };

        let target_x = target_cell_x as f32 + 0.5;
        let target_y = target_cell_y as f32 + 0.5;
        let delta_x = target_x - self.x;
        let delta_y = target_y - self.y;
        let distance = delta_x.hypot(delta_y);

        if distance < 0.04 {
            self.x = target_x;
            self.y = target_y;
            self.record_visit((target_cell_x, target_cell_y));
            self.patrol_target = self.next_patrol_cell(map);
            return;
        }

        let step = (CHASER_PATROL_SPEED * dt).min(distance);
        let next_x = self.x + delta_x / distance * step;
        let next_y = self.y + delta_y / distance * step;

        if map.can_stand_at(next_x, next_y, CHASER_RADIUS) {
            self.x = next_x;
            self.y = next_y;
        } else {
            self.patrol_target = None;
        }
    }

    fn next_patrol_cell(&mut self, map: &Map) -> Option<(i32, i32)> {
        let current = (self.x.floor() as i32, self.y.floor() as i32);
        let direction = self.patrol_direction;
        let forward_cell = add_cell(current, direction);
        let right_direction = (-direction.1, direction.0);
        let left_direction = (direction.1, -direction.0);
        let right_cell = add_cell(current, right_direction);
        let left_cell = add_cell(current, left_direction);

        let mut choices = Vec::new();

        for (cell, direction) in [
            (forward_cell, direction),
            (right_cell, right_direction),
            (left_cell, left_direction),
        ] {
            if map.is_walkable_cell(cell.0, cell.1) {
                choices.push(PatrolChoice { cell, direction });
            }
        }

        if !choices.is_empty() {
            let min_visits = choices
                .iter()
                .map(|choice| self.visit_count(choice.cell))
                .min()
                .expect("choices no esta vacio");

            let least_visited: Vec<_> = choices
                .iter()
                .copied()
                .filter(|choice| self.visit_count(choice.cell) == min_visits)
                .collect();

            let choice = least_visited[self.random_index(least_visited.len())];
            self.patrol_direction = choice.direction;
            return Some(choice.cell);
        }

        let backward_direction = (-direction.0, -direction.1);
        let backward_cell = add_cell(current, backward_direction);

        if map.is_walkable_cell(backward_cell.0, backward_cell.1) {
            self.patrol_direction = backward_direction;
            return Some(backward_cell);
        }

        for fallback_direction in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let fallback_cell = add_cell(current, fallback_direction);

            if map.is_walkable_cell(fallback_cell.0, fallback_cell.1) {
                self.patrol_direction = fallback_direction;
                return Some(fallback_cell);
            }
        }

        None
    }

    fn visit_count(&self, cell: (i32, i32)) -> u32 {
        self.visit_counts.get(&cell).copied().unwrap_or(0)
    }

    fn record_visit(&mut self, cell: (i32, i32)) {
        *self.visit_counts.entry(cell).or_insert(0) += 1;
    }

    fn random_index(&mut self, len: usize) -> usize {
        (self.advance_rng() as usize) % len
    }

    fn advance_rng(&mut self) -> u32 {
        self.rng_state = self
            .rng_state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        self.rng_state
    }
}

fn add_cell(cell: (i32, i32), direction: (i32, i32)) -> (i32, i32) {
    (cell.0 + direction.0, cell.1 + direction.1)
}

fn random_seed(x: f32, y: f32) -> u32 {
    let time_seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.subsec_nanos());

    time_seed ^ x.to_bits().rotate_left(11) ^ y.to_bits().rotate_right(7)
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