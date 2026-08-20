//! Efectos sonoros pequenos del juego.

/// Emite un sonido corto cuando el jugador usa la habilidad de disgusto.
#[cfg(target_os = "windows")]
pub fn play_disgust_sound() {
    std::thread::spawn(|| unsafe {
        // 0x30 es MB_ICONWARNING. Usar MessageBeep evita agregar una dependencia de audio.
        winapi::um::winuser::MessageBeep(0x30);
    });
}

/// Fallback simple para plataformas sin `MessageBeep`.
#[cfg(not(target_os = "windows"))]
pub fn play_disgust_sound() {
    use std::io::Write;

    let _ = std::io::stdout().write_all(b"\x07");
    let _ = std::io::stdout().flush();
}
