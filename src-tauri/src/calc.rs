//! Sensitivity math: DPI/sensitivity validation and cm-per-360° conversion.
//!
//! cm/360 uses the standard raw-input conversion:
//!     cm_per_360 = (2.54 * 360) / (dpi * sensitivity * yaw_per_count)
//!
//! `yaw_per_count` ("yaw constant") is the number of degrees the camera turns
//! per raw mouse count at sensitivity = 1. It is specific to a game's engine,
//! not to Glooblet, and the widely-cited community values below can drift
//! between game updates. Treat them as a documented starting point, not a
//! guarantee — `confidence` tells the UI how much to hedge, and every profile
//! can override the constant manually once a player has measured their own.

use crate::models::GameFormula;
use serde::Serialize;

pub const KNOWN_GAMES: &[&str] = &[
    "Counter-Strike 2",
    "VALORANT",
    "Apex Legends",
    "Overwatch",
    "Fortnite",
    "Call of Duty",
    "League of Legends",
    "Minecraft",
    "Custom game",
];

/// Returns the known formula entry for a game name, or `None` if the game
/// isn't in the starter catalogue at all (as opposed to being catalogued but
/// marked "not applicable").
pub fn formula_for(game: &str) -> Option<GameFormula> {
    let (yaw, confidence, note): (Option<f64>, &str, &str) = match game {
        "Counter-Strike 2" => (
            Some(0.022),
            "high",
            "Source-engine constant (0.022 deg/count), consistent across CS:GO and CS2 since the m_yaw default was introduced. Verify in-game with a measured 360 before relying on it competitively.",
        ),
        "VALORANT" => (
            Some(0.07),
            "high",
            "Riot's published in-game sensitivity multiplier. Widely used by third-party sens converters; re-check after major client updates.",
        ),
        "Apex Legends" => (
            Some(0.022),
            "high",
            "Apex runs on a Source-derived engine and shares CS:GO/CS2's 0.022 deg/count constant at the raw-input level.",
        ),
        "Overwatch" => (
            Some(0.0066),
            "community",
            "Commonly cited community constant, not an official Blizzard figure. Confirm against a measured 360 before competitive use.",
        ),
        "Fortnite" => (
            None,
            "not_applicable",
            "Fortnite's turn rate depends on FOV and input curve settings with no single stable community constant. Store DPI and sensitivity as reference values and convert manually in-game.",
        ),
        "Call of Duty" => (
            None,
            "not_applicable",
            "Recent Call of Duty titles use per-weapon ADS multipliers on top of base sensitivity, so a single yaw constant would misrepresent actual turn rate. Manual configuration only.",
        ),
        "League of Legends" => (
            None,
            "not_applicable",
            "League is a fixed top-down camera with no look-sensitivity/FOV turning, so cm/360 does not apply. DPI is still useful for cursor-speed tuning.",
        ),
        "Minecraft" => (
            None,
            "not_applicable",
            "Minecraft's 0-200% sensitivity slider is not standardized into a raw-input yaw constant across versions/editions. Manual configuration only.",
        ),
        _ => return None,
    };
    Some(GameFormula {
        game: game.to_string(),
        yaw_constant: yaw,
        confidence: confidence.to_string(),
        note: note.to_string(),
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct Cm360Result {
    pub cm_per_360: f64,
    pub yaw_constant_used: f64,
    pub source: String, // "game_default" | "custom_override"
    /// Populated only by `sensitivity_for_cm360`: the in-game sensitivity
    /// that achieves `cm_per_360` at the given DPI.
    pub resulting_sensitivity: Option<f64>,
}

#[derive(Debug, thiserror::Error)]
pub enum CalcError {
    #[error("DPI must be a positive number")]
    InvalidDpi,
    #[error("sensitivity must be a positive number")]
    InvalidSensitivity,
    #[error("no yaw constant available: {0}")]
    NoYawConstant(String),
}

/// Computes cm/360 for a DPI + in-game sensitivity pair.
///
/// `custom_yaw` overrides the game's known constant when the player has
/// measured their own, or is required outright for games with no documented
/// constant (`formula_for` returns `confidence: "not_applicable"`).
pub fn cm_per_360(
    dpi: f64,
    sensitivity: f64,
    game: &str,
    custom_yaw: Option<f64>,
) -> Result<Cm360Result, CalcError> {
    if !dpi.is_finite() || dpi <= 0.0 {
        return Err(CalcError::InvalidDpi);
    }
    if !sensitivity.is_finite() || sensitivity <= 0.0 {
        return Err(CalcError::InvalidSensitivity);
    }

    let (yaw, source) = if let Some(custom) = custom_yaw {
        if !custom.is_finite() || custom <= 0.0 {
            return Err(CalcError::NoYawConstant(
                "custom yaw constant must be positive".into(),
            ));
        }
        (custom, "custom_override")
    } else {
        let known = formula_for(game).and_then(|f| f.yaw_constant);
        match known {
            Some(yaw) => (yaw, "game_default"),
            None => {
                return Err(CalcError::NoYawConstant(format!(
                    "{game} has no documented yaw constant; provide a custom one"
                )))
            }
        }
    };

    let cm = (2.54 * 360.0) / (dpi * sensitivity * yaw);
    Ok(Cm360Result {
        cm_per_360: cm,
        yaw_constant_used: yaw,
        source: source.to_string(),
        resulting_sensitivity: None,
    })
}

/// Inverse of `cm_per_360`: given a desired physical distance for a full
/// turn, returns the in-game sensitivity to set at a given DPI.
pub fn sensitivity_for_cm360(
    target_cm: f64,
    dpi: f64,
    game: &str,
    custom_yaw: Option<f64>,
) -> Result<Cm360Result, CalcError> {
    if !target_cm.is_finite() || target_cm <= 0.0 {
        return Err(CalcError::InvalidSensitivity);
    }
    if !dpi.is_finite() || dpi <= 0.0 {
        return Err(CalcError::InvalidDpi);
    }
    let (yaw, source) = if let Some(custom) = custom_yaw {
        if !custom.is_finite() || custom <= 0.0 {
            return Err(CalcError::NoYawConstant(
                "custom yaw constant must be positive".into(),
            ));
        }
        (custom, "custom_override")
    } else {
        let known = formula_for(game).and_then(|f| f.yaw_constant);
        match known {
            Some(yaw) => (yaw, "game_default"),
            None => {
                return Err(CalcError::NoYawConstant(format!(
                    "{game} has no documented yaw constant; provide a custom one"
                )))
            }
        }
    };
    let sensitivity = (2.54 * 360.0) / (dpi * target_cm * yaw);
    Ok(Cm360Result {
        cm_per_360: target_cm,
        yaw_constant_used: yaw,
        source: source.to_string(),
        resulting_sensitivity: Some(sensitivity),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cs2_known_sensitivity_matches_hand_calculation() {
        // 400 dpi, 2.0 sens, Source yaw 0.022 => cm/360 = 914.4 / (400*2*0.022) = 51.9545...
        let result = cm_per_360(400.0, 2.0, "Counter-Strike 2", None).unwrap();
        assert!((result.cm_per_360 - 51.9545).abs() < 0.01);
        assert_eq!(result.source, "game_default");
    }

    #[test]
    fn custom_yaw_overrides_known_game_constant() {
        let result = cm_per_360(800.0, 1.0, "Counter-Strike 2", Some(0.05)).unwrap();
        assert_eq!(result.yaw_constant_used, 0.05);
        assert_eq!(result.source, "custom_override");
    }

    #[test]
    fn not_applicable_game_requires_custom_yaw() {
        let err = cm_per_360(800.0, 1.0, "Fortnite", None).unwrap_err();
        assert!(matches!(err, CalcError::NoYawConstant(_)));

        let ok = cm_per_360(800.0, 1.0, "Fortnite", Some(0.05));
        assert!(ok.is_ok());
    }

    #[test]
    fn rejects_non_positive_inputs() {
        assert!(matches!(
            cm_per_360(0.0, 1.0, "Counter-Strike 2", None),
            Err(CalcError::InvalidDpi)
        ));
        assert!(matches!(
            cm_per_360(800.0, -1.0, "Counter-Strike 2", None),
            Err(CalcError::InvalidSensitivity)
        ));
        assert!(matches!(
            cm_per_360(f64::NAN, 1.0, "Counter-Strike 2", None),
            Err(CalcError::InvalidDpi)
        ));
    }

    #[test]
    fn sensitivity_for_cm360_round_trips_with_cm_per_360() {
        let target_cm = 40.0;
        let sens = sensitivity_for_cm360(target_cm, 800.0, "VALORANT", None).unwrap();
        let resulting_sensitivity = sens.resulting_sensitivity.expect("sensitivity should be computed");
        let check = cm_per_360(800.0, resulting_sensitivity, "VALORANT", None).unwrap();
        assert!((check.cm_per_360 - target_cm).abs() < 0.001);
    }

    #[test]
    fn league_and_minecraft_are_marked_not_applicable() {
        assert_eq!(formula_for("League of Legends").unwrap().confidence, "not_applicable");
        assert_eq!(formula_for("Minecraft").unwrap().confidence, "not_applicable");
    }

    #[test]
    fn unknown_game_returns_none() {
        assert!(formula_for("Some Unlisted Game").is_none());
    }
}
