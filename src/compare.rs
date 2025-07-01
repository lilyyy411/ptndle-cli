use std::fmt::Display;

use facet::Facet;
use owo_colors::OwoColorize;

/// A comparison result of comparing 2 numerical values
#[derive(Copy, Clone, Debug, PartialEq, Eq, Facet)]
#[repr(u8)]
pub enum Comparison {
    Correct,
    FarLess,
    Less,
    Near,
    Greater,
    FarGreater,
}

impl Comparison {
    pub const fn to_str(self) -> &'static str {
        match self {
            | Self::Correct => " =",
            | Self::FarLess => "↓↓",
            | Self::Less => " ↓",
            | Self::Near => " ≅",
            | Self::Greater => " ↑",
            | Self::FarGreater => "↑↑",
        }
    }
}

impl Display for Comparison {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            | Self::Correct => write!(f, "{}", self.to_str().green()),
            | Self::Near => write!(f, "{}", self.to_str().yellow()),
            | _ => write!(f, "{}", self.to_str().red()),
        }
    }
}
#[derive(Copy, Clone, Debug, PartialEq, Facet)]
pub struct Threshold {
    pub near: f32,
    pub far: f32,
}

impl Threshold {
    #[expect(clippy::float_arithmetic, clippy::float_cmp, reason = "TODO: remove all float arithmetic")]
    pub fn compare(self, target: f32, guess: f32) -> Comparison {
        let distance = target - guess;
        if target == guess {
            return Comparison::Correct;
        }
        if distance > self.near {
            if distance > self.far {
                Comparison::FarGreater
            } else {
                Comparison::Greater
            }
        } else if distance < -self.near {
            if distance < -self.far {
                Comparison::FarLess
            } else {
                Comparison::Less
            }
        } else {
            Comparison::Near
        }
    }
}
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Thresholds {
    pub code: Option<Threshold>,
    pub height: Threshold,
}
