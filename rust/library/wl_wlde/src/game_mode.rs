//! Deterministic Game Mode profile resolution for WLDE.
//!
//! This module consumes foreground/fullscreen/system signals from a platform
//! adapter. It does not enumerate processes, inspect executable files, hash
//! binaries, install hooks or poll the operating system.

use crate::GameMode;

/// Maximum number of user-selected application profiles evaluated per update.
pub const MAX_GAME_MODE_PROFILES: usize = 64;

/// A per-application mode keyed by the executable's SHA-256 digest.
///
/// The host computes the digest when creating/updating the profile. Filenames
/// and executable basenames are deliberately not considered stable identities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameModeProfile {
    executable_sha256: [u8; 32],
    mode: GameMode,
}

impl GameModeProfile {
    pub const fn new(executable_sha256: [u8; 32], mode: GameMode) -> Self {
        Self {
            executable_sha256,
            mode,
        }
    }

    pub const fn executable_sha256(&self) -> &[u8; 32] {
        &self.executable_sha256
    }

    pub const fn mode(&self) -> GameMode {
        self.mode
    }
}

/// Signals sampled by the host for the current foreground application.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GameModeSignals {
    pub executable_sha256: [u8; 32],
    pub is_foreground: bool,
    pub is_fullscreen: bool,
    pub system_game_mode_active: bool,
}

/// Optional automatic fallbacks. Both are disabled by default.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GameModePolicy {
    /// Apply this mode when the foreground application is reported fullscreen.
    pub fullscreen_mode: Option<GameMode>,
    /// Apply this mode when the host reports the OS Game Mode signal active.
    pub system_game_mode: Option<GameMode>,
}

impl GameModePolicy {
    pub const fn new(
        fullscreen_mode: Option<GameMode>,
        system_game_mode: Option<GameMode>,
    ) -> Self {
        Self {
            fullscreen_mode,
            system_game_mode,
        }
    }
}

/// Validation errors for the bounded profile resolver.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameModeError {
    TooManyProfiles,
    DuplicateExecutableFingerprint,
}

impl std::fmt::Display for GameModeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManyProfiles => {
                write!(
                    f,
                    "Game Mode profile count exceeds the {MAX_GAME_MODE_PROFILES}-profile limit"
                )
            }
            Self::DuplicateExecutableFingerprint => {
                f.write_str("Game Mode profiles must have unique executable SHA-256 fingerprints")
            }
        }
    }
}

impl std::error::Error for GameModeError {}

/// Resolves the current foreground application's Game Mode.
///
/// Precedence is deliberate:
/// 1. An exact user-selected executable fingerprint profile (including Normal).
/// 2. An enabled OS Game Mode signal policy.
/// 3. An enabled fullscreen policy.
/// 4. Normal.
///
/// Non-foreground applications always resolve to Normal. At most 64 profiles
/// are accepted. The lookup performs no allocation and does not use filenames.
pub fn resolve_game_mode(
    signals: GameModeSignals,
    policy: GameModePolicy,
    profiles: &[GameModeProfile],
) -> Result<GameMode, GameModeError> {
    if profiles.len() > MAX_GAME_MODE_PROFILES {
        return Err(GameModeError::TooManyProfiles);
    }

    for (index, profile) in profiles.iter().enumerate() {
        if profiles[..index]
            .iter()
            .any(|previous| previous.executable_sha256 == profile.executable_sha256)
        {
            return Err(GameModeError::DuplicateExecutableFingerprint);
        }
    }

    if !signals.is_foreground {
        return Ok(GameMode::Normal);
    }

    if let Some(profile) = profiles
        .iter()
        .find(|profile| profile.executable_sha256 == signals.executable_sha256)
    {
        return Ok(profile.mode);
    }

    if signals.system_game_mode_active {
        if let Some(mode) = policy.system_game_mode {
            return Ok(mode);
        }
    }

    if signals.is_fullscreen {
        if let Some(mode) = policy.fullscreen_mode {
            return Ok(mode);
        }
    }

    Ok(GameMode::Normal)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fingerprint(value: u8) -> [u8; 32] {
        [value; 32]
    }

    #[test]
    fn exact_user_profile_overrides_automatic_signals_including_normal() {
        let profiles = [
            GameModeProfile::new(fingerprint(0xA1), GameMode::Normal),
            GameModeProfile::new(fingerprint(0xB2), GameMode::Suspend),
        ];
        let signals = GameModeSignals {
            executable_sha256: fingerprint(0xA1),
            is_foreground: true,
            is_fullscreen: true,
            system_game_mode_active: true,
        };
        let policy = GameModePolicy::new(Some(GameMode::Optimize), Some(GameMode::Suspend));

        assert_eq!(
            resolve_game_mode(signals, policy, &profiles),
            Ok(GameMode::Normal)
        );

        let mut changed = signals;
        changed.executable_sha256 = fingerprint(0xB2);
        assert_eq!(
            resolve_game_mode(changed, policy, &profiles),
            Ok(GameMode::Suspend)
        );
    }

    #[test]
    fn system_signal_precedes_fullscreen_fallback() {
        let signals = GameModeSignals {
            executable_sha256: fingerprint(0xC3),
            is_foreground: true,
            is_fullscreen: true,
            system_game_mode_active: true,
        };
        let policy = GameModePolicy::new(Some(GameMode::Suspend), Some(GameMode::Optimize));

        assert_eq!(
            resolve_game_mode(signals, policy, &[]),
            Ok(GameMode::Optimize)
        );
    }

    #[test]
    fn fullscreen_policy_requires_foreground_and_is_disabled_by_default() {
        let signals = GameModeSignals {
            executable_sha256: fingerprint(0xD4),
            is_foreground: true,
            is_fullscreen: true,
            system_game_mode_active: false,
        };
        assert_eq!(
            resolve_game_mode(signals, GameModePolicy::default(), &[]),
            Ok(GameMode::Normal)
        );
        assert_eq!(
            resolve_game_mode(
                signals,
                GameModePolicy::new(Some(GameMode::Optimize), None),
                &[]
            ),
            Ok(GameMode::Optimize)
        );

        let mut background = signals;
        background.is_foreground = false;
        assert_eq!(
            resolve_game_mode(
                background,
                GameModePolicy::new(Some(GameMode::Suspend), Some(GameMode::Optimize)),
                &[]
            ),
            Ok(GameMode::Normal)
        );
    }

    #[test]
    fn duplicate_and_oversized_profile_sets_are_rejected() {
        let duplicate = [
            GameModeProfile::new(fingerprint(0xE5), GameMode::Optimize),
            GameModeProfile::new(fingerprint(0xE5), GameMode::Suspend),
        ];
        let signals = GameModeSignals {
            is_foreground: true,
            ..GameModeSignals::default()
        };
        assert_eq!(
            resolve_game_mode(signals, GameModePolicy::default(), &duplicate),
            Err(GameModeError::DuplicateExecutableFingerprint)
        );

        let many = vec![
            GameModeProfile::new(fingerprint(0xF6), GameMode::Normal);
            MAX_GAME_MODE_PROFILES + 1
        ];
        assert_eq!(
            resolve_game_mode(signals, GameModePolicy::default(), &many),
            Err(GameModeError::TooManyProfiles)
        );
    }
}
