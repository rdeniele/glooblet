//! First-run seed data: the built-in sound library (pointing at the
//! procedurally generated, royalty-free WAV assets bundled as resources) and
//! the game compatibility registry. Both are static/versioned here rather
//! than editable at runtime, per the spec's "versioned registry" requirement.

use crate::db;
use crate::error::AppResult;
use crate::models::SoundPreset;
use rusqlite::Connection;

pub struct BuiltinSound {
    pub file_name: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub description: &'static str,
}

pub const BUILTIN_SOUNDS: &[BuiltinSound] = &[
    BuiltinSound { file_name: "soft-pop.wav", name: "Soft Pop", category: "Soft", description: "A gentle, rounded pop with no sharp edges." },
    BuiltinSound { file_name: "soft-thud.wav", name: "Soft Thud", category: "Soft", description: "Low, muffled thud — barely-there feedback." },
    BuiltinSound { file_name: "mechanical-click.wav", name: "Mechanical Click", category: "Mechanical", description: "Crisp switch snap, like a mechanical keyboard." },
    BuiltinSound { file_name: "mechanical-switch.wav", name: "Mechanical Switch", category: "Mechanical", description: "Slightly heavier tactile switch snap." },
    BuiltinSound { file_name: "retro-blip.wav", name: "Retro Blip", category: "Retro", description: "8-bit style square-wave blip." },
    BuiltinSound { file_name: "retro-beep.wav", name: "Retro Beep", category: "Retro", description: "Flat retro beep, low frequency." },
    BuiltinSound { file_name: "arcade-coin.wav", name: "Arcade Coin", category: "Arcade", description: "Bright rising chirp, coin-slot energy." },
    BuiltinSound { file_name: "arcade-zap.wav", name: "Arcade Zap", category: "Arcade", description: "Short zappy sweep with a harmonic edge." },
    BuiltinSound { file_name: "scifi-pulse.wav", name: "Sci-Fi Pulse", category: "Sci-Fi", description: "Detuned descending pulse." },
    BuiltinSound { file_name: "scifi-confirm.wav", name: "Sci-Fi Confirm", category: "Sci-Fi", description: "Modulated confirmation tone." },
    BuiltinSound { file_name: "minimal-tick.wav", name: "Minimal Tick", category: "Minimal", description: "Barely-audible high tick." },
    BuiltinSound { file_name: "minimal-tap.wav", name: "Minimal Tap", category: "Minimal", description: "Soft noise tap, extremely short." },
];

/// Inserts the built-in sound catalogue on first run only — if any built-in
/// row already exists we assume seeding already happened and leave the
/// table (and any user edits to volume, etc.) alone.
pub fn seed_builtin_sounds(conn: &Connection) -> AppResult<()> {
    let existing = db::sounds::list(conn)?;
    if existing.iter().any(|s| s.is_builtin) {
        return Ok(());
    }

    for sound in BUILTIN_SOUNDS {
        db::sounds::insert(
            conn,
            SoundPreset {
                id: String::new(),
                name: sound.name.to_string(),
                category: sound.category.to_string(),
                description: sound.description.to_string(),
                file_path: sound.file_name.to_string(),
                is_builtin: true,
                volume: 0.7,
                duration_ms: None,
                created_at: String::new(),
            },
        )?;
    }
    Ok(())
}

/// Known executable names per supported game, used only for the optional
/// Click Studio "suppress over a game" heuristic. This is a foreground-window
/// name check, not process injection or memory access.
pub fn known_game_executables() -> Vec<String> {
    vec![
        "cs2.exe", "csgo.exe", "valorant.exe", "valorant-win64-shipping.exe",
        "r5apex.exe", "overwatch.exe", "fortniteclient-win64-shipping.exe",
        "cod.exe", "modernwarfare.exe", "warzone.exe",
        "league of legends.exe", "leagueclient.exe", "minecraft.exe", "javaw.exe",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}
