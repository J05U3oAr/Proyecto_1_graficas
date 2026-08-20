//! Efectos sonoros pequenos del juego.

use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(target_os = "windows")]
static PLAYBACK_GENERATION: AtomicU64 = AtomicU64::new(0);

/// Reproduce la cancion cuando el jugador usa la habilidad de disgusto.
#[cfg(target_os = "windows")]
pub fn play_disgust_sound() {
    let generation = PLAYBACK_GENERATION.fetch_add(1, Ordering::Relaxed) + 1;

    std::thread::spawn(move || {
        if let Some(path) = ophelia_audio_path() {
            play_mp3_from_start(&path, generation);
        }
    });
}

/// Fallback simple para plataformas donde no se controla el audio del sistema.
#[cfg(not(target_os = "windows"))]
pub fn play_disgust_sound() {}

fn ophelia_audio_path() -> Option<PathBuf> {
    const AUDIO_FILE: &str = "The Fate of Ophelia.mp3";
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
        .into_iter()
        .map(|root| root.join(AUDIO_FILE))
        .find(|path| path.exists())
}

#[cfg(target_os = "windows")]
fn play_mp3_from_start(path: &std::path::Path, generation: u64) {
    const ALIAS: &str = "disgust_ophelia";
    const VOLUME: u16 = 200;
    const DURATION: std::time::Duration = std::time::Duration::from_secs(5);

    let path = path.display();
    let _ = send_mci_command(&format!("close {ALIAS}"));
    let opened = send_mci_command(&format!("open \"{path}\" type mpegvideo alias {ALIAS}"));

    if opened == 0 {
        let _ = send_mci_command(&format!("setaudio {ALIAS} volume to {VOLUME}"));
        let _ = send_mci_command(&format!("play {ALIAS} from 0"));
        std::thread::sleep(DURATION);

        if PLAYBACK_GENERATION.load(Ordering::Relaxed) == generation {
            let _ = send_mci_command(&format!("close {ALIAS}"));
        }
    }
}

#[cfg(target_os = "windows")]
fn send_mci_command(command: &str) -> u32 {
    use std::{ffi::OsStr, os::windows::ffi::OsStrExt, ptr::null_mut};

    #[link(name = "winmm")]
    unsafe extern "system" {
        fn mciSendStringW(
            lpszCommand: *const u16,
            lpszReturnString: *mut u16,
            cchReturn: u32,
            hwndCallback: *mut std::ffi::c_void,
        ) -> u32;
    }

    let wide_command: Vec<u16> = OsStr::new(command).encode_wide().chain(Some(0)).collect();

    unsafe { mciSendStringW(wide_command.as_ptr(), null_mut(), 0, null_mut()) }
}
