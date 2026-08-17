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

/// Textura de ladrillo/piedra para paredes normales.
fn stone_brick(x: usize, y: usize) -> u32 {
    let row = y / 16;
    // Desfase por fila para que los ladrillos no queden alineados verticalmente.
    let shifted_x = (x + (row % 2) * 16) % TEXTURE_SIZE;
    let mortar = y % 16 == 0 || y % 16 == 15 || shifted_x % 32 == 0 || shifted_x % 32 == 31;
    let grain = noise(x, y, 11) as i32 - 4;

    if mortar {
        rgb(43, 63, 76)
    } else {
        rgb_i(103 + grain, 151 + grain, 176 + grain)
    }
}

/// Textura de puerta con barras, marco, refuerzos y remaches.
fn gate(x: usize, y: usize) -> u32 {
    let border = x < 4 || x > 59 || y < 4 || y > 59;
    let bar = x % 16 < 5;
    let brace = x.abs_diff(y) < 3 || (TEXTURE_SIZE - 1 - x).abs_diff(y) < 3;
    let rivet = (x % 16).abs_diff(8) <= 2 && (y % 16).abs_diff(8) <= 2;
    let highlight = x % 16 == 5;

    if rivet {
        rgb(255, 239, 166)
    } else if border || brace {
        rgb(154, 90, 20)
    } else if bar {
        rgb(238, 174, 56)
    } else if highlight {
        rgb(255, 210, 99)
    } else {
        rgb(75, 47, 31)
    }
}

/// Textura de panel metalico con uniones, remaches y rayones.
fn metal_panel(x: usize, y: usize) -> u32 {
    let seam = x % 32 == 0 || y % 32 == 0;
    let rivet_centered = (x % 32).abs_diff(16) <= 3 && (y % 32).abs_diff(16) <= 3;
    let scratch = (x * 3 + y * 5) % 29 == 0 || (x + y * 2) % 37 == 0;
    let grain = noise(x, y, 29) as i32 - 5;

    if rivet_centered {
        rgb(255, 176, 76)
    } else if seam {
        rgb(82, 72, 64)
    } else if scratch {
        rgb(251, 190, 118)
    } else {
        rgb_i(185 + grain, 105 + grain, 43 + grain / 2)
    }
}

/// Textura de ruinas con bloques, grietas y musgo.
fn cracked_ruins(x: usize, y: usize) -> u32 {
    let block_line = x % 21 == 0 || y % 18 == 0;
    let crack = (x * 5 + y * 9) % 41 < 2 || (x * 11).abs_diff(y * 7) % 53 < 2;
    let moss = y > 42 && noise(x, y, 7) > 10;
    let grain = noise(x, y, 43) as i32 - 7;

    if moss {
        rgb(80, 128, 91)
    } else if crack {
        rgb(48, 34, 60)
    } else if block_line {
        rgb(77, 58, 91)
    } else {
        rgb_i(143 + grain, 103 + grain / 2, 184 + grain)
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
