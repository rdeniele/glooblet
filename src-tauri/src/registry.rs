//! Game/device compatibility registry (spec section 10). This is a static,
//! versioned table reviewed and shipped with each Glooblet release — not
//! something the app edits at runtime, and not a claim of publisher
//! endorsement. Every entry is "manual configuration only" or "limited
//! support" because Glooblet does not read game memory, inject code, or use
//! any undocumented per-game mechanism; the registry exists so the UI can
//! say exactly that, per game, instead of leaving compatibility implicit.

use crate::models::IntegrationStatus;

const REVIEW_DATE: &str = "2026-01-01";

pub fn compatibility_registry() -> Vec<IntegrationStatus> {
    let manual = |game: &str, note: &str| IntegrationStatus {
        subject: game.to_string(),
        method: "Manual: Glooblet stores DPI/sensitivity as reference values; apply them in the game's own settings menu.".to_string(),
        source: "Glooblet engineering review (no publisher API available)".to_string(),
        last_reviewed: REVIEW_DATE.to_string(),
        supported_functionality: vec![
            "Profile storage (DPI, sensitivity, notes)".to_string(),
            "cm/360 reference calculation where a formula is documented".to_string(),
        ],
        limitations: vec![
            "No automatic in-game application of settings".to_string(),
            "No automatic profile switching".to_string(),
            note.to_string(),
        ],
        status: "manual_configuration_only".to_string(),
    };

    vec![
        manual("Counter-Strike 2 / Counter-Strike: Global Offensive", "Source-engine sensitivity formula is well documented; verify cm/360 in-game before relying on it competitively."),
        manual("VALORANT", "Riot does not expose a settings API; Vanguard anti-cheat means Glooblet never attaches to or inspects the game process."),
        manual("Apex Legends", "Source-derived engine; same manual-apply model as CS2."),
        manual("Overwatch", "No public settings API; yaw constant is community-sourced, not Blizzard-confirmed."),
        manual("Fortnite", "Turn rate depends on FOV and input curve; no single stable conversion constant."),
        manual("Call of Duty (modern titles)", "Per-weapon ADS multipliers make a single sensitivity constant unreliable."),
        manual("League of Legends", "Top-down fixed camera; no look-sensitivity concept, cm/360 does not apply."),
        manual("Minecraft", "Sensitivity is a 0-200% slider with no standardized raw-input constant across versions/editions."),
        IntegrationStatus {
            subject: "Custom game".to_string(),
            method: "Manual, with an optional user-supplied custom yaw constant for cm/360.".to_string(),
            source: "N/A".to_string(),
            last_reviewed: REVIEW_DATE.to_string(),
            supported_functionality: vec!["Profile storage".to_string(), "Custom cm/360 formula".to_string()],
            limitations: vec!["Accuracy depends entirely on the constant the user provides".to_string()],
            status: "manual_configuration_only".to_string(),
        },
    ]
}
