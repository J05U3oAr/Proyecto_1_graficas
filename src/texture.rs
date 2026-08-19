//! Texturas procedural para paredes y obstaculos.
//!
//! Cada funcion recibe coordenadas dentro de una textura virtual de 64x64
//! y devuelve un color RGB. Mas adelante este modulo puede cambiarse para
//! leer pixeles desde imagenes PNG sin alterar el raycaster.

use crate::map::{TILE_GATE, TILE_METAL, TILE_RUINS, TILE_WALL};

/// Tamano fijo de cada textura cuadrada.
pub const TEXTURE_SIZE: usize = 64;

/// Devuelve el color de textura para un tile bloqueante.
pub fn wall_texel(tile: u8, x: usize, y: usize) -> u32 {
    // El modulo permite repetir la textura si una coordenada se sale del rango.
    let x = x % TEXTURE_SIZE;
    let y = y % TEXTURE_SIZE;

    match tile {
        TILE_WALL => stone_brick(x, y),
        TILE_GATE => gate(x, y),
        TILE_METAL => metal_panel(x, y),
        TILE_RUINS => cracked_ruins(x, y),
        _ => fallback(x, y),
    }
}

/// Textura de bloque de concreto para los pasillos del complejo.
fn stone_brick(x: usize, y: usize) -> u32 {
    let row = y / 16;
    // Desfase por fila para que los bloques no queden alineados verticalmente.
    let shifted_x = (x + (row % 2) * 16) % TEXTURE_SIZE;
    let mortar = y % 16 == 0 || y % 16 == 15 || shifted_x % 32 == 0 || shifted_x % 32 == 31;
    let grain = noise(x, y, 11) as i32 - 4;

    if mortar {
        // Junta oscura, casi sin luz ambiental.
        rgb(24, 27, 26)
    } else {
        // Concreto lavado, con un tinte verdoso de luz fluorescente enferma.
        rgb_i(118 + grain, 128 + grain, 118 + grain)
    }
}

/// Textura de compuerta de contencion con franjas de peligro y remaches.
fn gate(x: usize, y: usize) -> u32 {
    let border = x < 4 || x > 59 || y < 4 || y > 59;
    let hazard_stripe = ((x as i32 + y as i32) / 6) % 2 == 0;
    let rivet = (x % 16).abs_diff(8) <= 2 && (y % 16).abs_diff(8) <= 2;
    let center_seam = x.abs_diff(TEXTURE_SIZE / 2) < 2;

    if rivet {
        rgb(255, 214, 74)
    } else if center_seam {
        rgb(18, 18, 20)
    } else if border {
        // Franja de peligro amarillo/negro alrededor del marco.
        if hazard_stripe {
            rgb(255, 196, 0)
        } else {
            rgb(20, 20, 22)
        }
    } else {
        rgb(46, 49, 53)
    }
}

/// Textura de panel metalico de mantenimiento con uniones, remaches y rayones.
fn metal_panel(x: usize, y: usize) -> u32 {
    let seam = x % 32 == 0 || y % 32 == 0;
    let rivet_centered = (x % 32).abs_diff(16) <= 3 && (y % 32).abs_diff(16) <= 3;
    let scratch = (x * 3 + y * 5) % 29 == 0 || (x + y * 2) % 37 == 0;
    let grain = noise(x, y, 29) as i32 - 5;

    if rivet_centered {
        rgb(226, 232, 235)
    } else if seam {
        rgb(28, 32, 35)
    } else if scratch {
        rgb(200, 210, 214)
    } else {
        // Acero azulado, frio y aseptico.
        rgb_i(120 + grain, 130 + grain, 138 + grain)
    }
}

/// Textura de sector colapsado con grietas y contaminacion biologica.
fn cracked_ruins(x: usize, y: usize) -> u32 {
    let block_line = x % 21 == 0 || y % 18 == 0;
    let crack = (x * 5 + y * 9) % 41 < 2 || (x * 11).abs_diff(y * 7) % 53 < 2;
    let growth = y > 42 && noise(x, y, 7) > 10;
    let grain = noise(x, y, 43) as i32 - 7;

    if growth {
        // Crecimiento organico toxico, un aviso de que algo salio mal aqui.
        rgb(58, 138, 46)
    } else if crack {
        rgb(8, 9, 8)
    } else if block_line {
        rgb(46, 52, 46)
    } else {
        rgb_i(96 + grain / 2, 108 + grain, 96 + grain / 2)
    }
}

/// Textura de respaldo para tiles no reconocidos.
fn fallback(x: usize, y: usize) -> u32 {
    if (x / 8 + y / 8) % 2 == 0 {
        rgb(210, 210, 210)
    } else {
        rgb(180, 180, 180)
    }
}

/// Ruido determinista pequeno para variaciones de color.
fn noise(x: usize, y: usize, seed: usize) -> u8 {
    let mut value = x
        .wrapping_mul(73_856_093)
        .wrapping_add(y.wrapping_mul(19_349_663))
        .wrapping_add(seed.wrapping_mul(83_492_791));
    value ^= value >> 13;
    value = value.wrapping_mul(1_274_126_177);
    ((value >> 24) & 0x0f) as u8
}

/// Variante de `rgb` que acepta enteros con signo y recorta a 0..255.
fn rgb_i(r: i32, g: i32, b: i32) -> u32 {
    rgb(
        r.clamp(0, 255) as u32,
        g.clamp(0, 255) as u32,
        b.clamp(0, 255) as u32,
    )
}

/// Empaca componentes RGB en el formato `0xRRGGBB`.
fn rgb(r: u32, g: u32, b: u32) -> u32 {
    (r.min(255) << 16) | (g.min(255) << 8) | b.min(255)
}