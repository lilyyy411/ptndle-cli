use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use eyre::{eyre, Context};
use facet::Facet;

use crate::compare::{Threshold, Thresholds};
#[expect(dead_code, reason = "Facet constructs these")]
/// A sinner's alignment
#[derive(Facet, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Alignment {
    Death,
    Fraud,
    Limbo,
    Anger,
    Love,
    Greed,
    Heresy,
    Sloth,
    Pestilence,
    Immortal,
    Famine,
    Violence,
    Treachery,
}
/// A sinner's tendency
#[expect(dead_code, reason = "Facet constructs these")]
#[derive(Facet, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Tendency {
    Catalyst,
    Arcane,
    Endura,
    Fury,
    Reticle,
    Umbra,
}

/// A sinner's birthplace
#[expect(dead_code, reason = "Facet constructs these")]
#[derive(Facet, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum BirthPlace {
    Other,
    Syndicate,
    Eastside,
}

#[derive(Facet)]
struct RawSinner {
    name: String,
    code: String,
    alignment: Alignment,
    tendency: Tendency,
    height: String,
    birthplace: BirthPlace,
}
impl RawSinner {
    fn into_sinner(self) -> eyre::Result<Sinner> {
        let code = self.code.parse::<u16>().ok();
        let height: u8 = self
            .height
            .strip_suffix("cm")
            .and_then(|x| x.parse().ok())
            .ok_or_else(|| eyre::eyre!("invalid height"))?;
        Ok(Sinner {
            name: self.name,
            code,
            height,
            alignment: self.alignment,
            tendency: self.tendency,
            birthplace: self.birthplace,
        })
    }
}

/// The parsed data for a sinner
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Sinner {
    pub name: String,
    /// The numerical code of the sinner if it is a number. NOX is the only
    /// sinner with a non-numeric code.
    pub code: Option<u16>,
    pub alignment: Alignment,
    pub tendency: Tendency,
    /// The height of the sinner in cm
    pub height: u8,
    pub birthplace: BirthPlace,
}
pub const MOST_COMMON_HEIGHT: i16 = 168;
impl Sinner {
    /// Gets the height and code thresholds based on this sinner's data
    #[expect(clippy::float_arithmetic, reason = "we don't care for now")]
    pub fn thresholds(&self) -> Thresholds {
        Thresholds {
            code: self.code.map(|code| {
                Threshold {
                    near: 5. + f32::from(code) * 0.1,
                    far: 50. + f32::from(code) * 0.35,
                }
            }),
            height: Threshold {
                near: 3. + f32::from((i16::from(self.height) - MOST_COMMON_HEIGHT).abs()) * 0.1,
                far: 15. + f32::from((i16::from(self.height) - MOST_COMMON_HEIGHT).abs()) * 0.35,
            },
        }
    }
}

fn cache_dir() -> PathBuf {
    dirs::cache_dir().map_or_else(|| "path-to-nowordle-cli-cache".into(), |x| x.join("Path-To-Nowordle-CLI"))
}
fn make_and_get_cache_dir() -> eyre::Result<PathBuf> {
    let cache = cache_dir();
    std::fs::create_dir_all(&cache).with_context(|| "Failed to create sinner cache directory")?;
    Ok(cache)
}

fn load_sinners_from_json(bytes: &[u8]) -> eyre::Result<Vec<Sinner>> {
    let raw_sinners = facet_json::from_slice::<Vec<RawSinner>>(bytes).map_err(|e| eyre!("{e}"))?;
    raw_sinners
        .into_iter()
        .map(RawSinner::into_sinner)
        .collect()
}

static FALLBACK_SINNER_DATA: &[u8] = include_bytes!("../sinners.json");
static SINNER_DATA_URL: &str = "https://raw.githubusercontent.com/Kaseioo/pathtonowordle/refs/heads/main/src/character_data/characters.json";

fn is_cache_outdated<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref()
        .metadata()
        .map(|meta| {
            meta.modified()
                .map(|modified| {
                    SystemTime::now()
                        .duration_since(modified)
                        .map(|x| x > Duration::from_secs(24 * 3600))
                        .unwrap_or(true)
                })
                .unwrap_or(true)
        })
        .unwrap_or(true)
}

pub fn load_sinners(force_update: bool) -> eyre::Result<Vec<Sinner>> {
    let cache_path = make_and_get_cache_dir()?.join("sinners.json");
    let load_cache = || {
        std::fs::read(&cache_path).unwrap_or_else(|e| {
            eprintln!("[WARNING] Could not read cache: {e}. Falling back to hard-coded data.");
            FALLBACK_SINNER_DATA.to_vec()
        })
    };

    let json = if force_update || is_cache_outdated(&cache_path) {
        if let Ok(json) = ureq::get(SINNER_DATA_URL)
            .call()
            .map(|mut x| x.body_mut().read_to_vec())
            .and_then(|x| x)
            .inspect_err(|e| {
                eprintln!(
                    "[WARNING]: Failed to update sinner data: {e}. Falling back to reading cache \
                     instead."
                );
            })
        {
            // I don't care if the write fails... just try
            _ = std::fs::write(&cache_path, &json);
            json
        } else {
            load_cache()
        }
    } else {
        load_cache()
    };
    load_sinners_from_json(&json)
}
