use std::fmt::Write;
use std::str::FromStr;

use facet::Facet;
use owo_colors::OwoColorize;

use crate::compare::Comparison;
use crate::data::{Sinner, MOST_COMMON_HEIGHT};

/// A packed representation of a result from guessing
/// a sinner based on a target
///
/// BITS\\
/// 0..3 -> code comparison
/// 3..6 -> height comparison
/// 6 -> alignment correct
/// 7 -> code comparison valid
/// 8 -> tendency correct
/// 9 -> birthplace correct
#[derive(Clone, Copy, Facet)]
pub struct Guess(u16);

const HEIGHT_OFFSET: u8 = 3;
const ALIGN_OFFSET: u8 = 6;
const CODE_VALID_OFFSET: u8 = 7;
const TENDENCY_OFFSET: u8 = 8;
const BIRTHPLACE_OFFSET: u8 = 9;
const CODE_BITS: u16 = 0b111;

impl Guess {
    pub const fn new(
        code: Option<Comparison>,
        alignment: bool,
        tendency: bool,
        height: Comparison,
        birthplace: bool,
    ) -> Self {
        let mut data = 0;
        if let Some(code) = code {
            data = (1 << CODE_VALID_OFFSET) + code as u16;
        }
        data |= (height as u16) << HEIGHT_OFFSET;
        data |= (alignment as u16) << ALIGN_OFFSET;
        data |= (tendency as u16) << TENDENCY_OFFSET;
        data |= (birthplace as u16) << BIRTHPLACE_OFFSET;
        Self(data)
    }
    /// The code comparison for the guess
    pub const fn code(self) -> Option<Comparison> {
        #[expect(clippy::cast_possible_truncation, reason = "we're intentionally truncating to extract the is valid bit")]
        if (self.0 as i8).is_negative() {
            // SAFETY: Guess can only be constructed with bits 0..3 being a valid comparison
            Some(unsafe { std::mem::transmute::<u8, Comparison>((self.0 & CODE_BITS) as u8) })
        } else {
            None
        }
    }
    /// The height comparison for the guess
    pub const fn height(self) -> Comparison {
        // SAFETY: Guess can only be constructed with bits 3..6 being a valid comparison
        unsafe {
            std::mem::transmute::<u8, Comparison>(((self.0 >> HEIGHT_OFFSET) & CODE_BITS) as u8)
        }
    }
    /// The alignment for the guess
    pub const fn alignment(self) -> bool { (self.0 >> ALIGN_OFFSET) & 1 != 0 }
    /// The tendency for the guess
    pub const fn tendency(self) -> bool { (self.0 >> TENDENCY_OFFSET) & 1 != 0 }
    /// The birthplace for the guess
    pub const fn birthplace(self) -> bool { (self.0 >> BIRTHPLACE_OFFSET) & 1 != 0 }
}

impl std::fmt::Debug for Guess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Guess")
            .field("code", &self.code())
            .field("alignment", &self.alignment())
            .field("tendency", &self.tendency())
            .field("height", &self.height())
            .field("birthplace", &self.birthplace())
            .finish()
    }
}
fn fmt_bool(f: &mut std::fmt::Formatter<'_>, b: bool) -> std::fmt::Result {
    if b {
        write!(f, "{}", " 1".green())
    } else {
        write!(f, "{}", " 0".red())
    }
}
impl std::fmt::Display for Guess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(code) = self.code() {
            std::fmt::Display::fmt(&code, f)?;
        } else {
            write!(f, "{}", " x".red())?;
        }
        f.write_char(' ')?;
        fmt_bool(f, self.alignment())?;
        f.write_char(' ')?;
        fmt_bool(f, self.tendency())?;
        f.write_char(' ')?;
        write!(f, "{}", self.height())?;
        f.write_char(' ')?;
        fmt_bool(f, self.birthplace())
    }
}
/// Checks whether comparing `guess` and `target` codes would yield
/// `comparison`. This is the [`Threshold::compare`] for sinner code, but solved
/// for `target`
#[expect(clippy::float_arithmetic, clippy::float_cmp, reason = "TODO: remove this float arithmetic")]
fn code_matches(guess: u16, target: u16, comparison: Comparison) -> bool {
    // TODO: remove ugly float arithmetic
    let [target, guess]: [f32; 2] = [target, guess].map(<_>::into);
    match comparison {
        | Comparison::Correct => target == guess,
        | Comparison::FarLess => target < (guess - 50.) / 1.35,
        | Comparison::Less => (guess - 50.) / 1.35 <= target && target <= (guess - 5.) / 1.1,
        | Comparison::Near => {
            target != guess && (guess - 5.) / 1.1 <= target && target <= (guess + 5.) / 0.9
        },
        | Comparison::Greater => (guess + 5.) * 0.9 < target && target <= (guess + 50.) / 0.65,
        | Comparison::FarGreater => target > (guess + 50.) / 0.65,
    }
}
fn height_below_upper_near_threshold(guess: u8, target: u8) -> bool {
    if i16::from(guess) <= MOST_COMMON_HEIGHT - 3 {
        i16::from(target) <= (10 * i16::from(guess) + MOST_COMMON_HEIGHT + 30) / 11
    } else {
        i16::from(target) <= (10 * i16::from(guess) - MOST_COMMON_HEIGHT + 30) / 9
    }
}
fn height_above_lower_near_threshold(guess: u8, target: u8) -> bool {
    if i16::from(guess) <= MOST_COMMON_HEIGHT + 3 {
        i16::from(target) >= (10 * i16::from(guess) - MOST_COMMON_HEIGHT - 30 + 8) / 9
    } else {
        i16::from(target) >= (10 * i16::from(guess) + MOST_COMMON_HEIGHT - 30 + 10) / 11
    }
}
fn height_below_upper_far_threshold(guess: u8, target: u8) -> bool {
    if i16::from(guess) <= MOST_COMMON_HEIGHT - 15 {
        i32::from(target) <= (20 * i32::from(guess) + 7 * i32::from(MOST_COMMON_HEIGHT) + 300) / 27
    } else {
        i32::from(target) <= (20 * i32::from(guess) - 7 * i32::from(MOST_COMMON_HEIGHT) + 300) / 13
    }
}

fn height_above_lower_far_threshold(guess: u8, target: u8) -> bool {
    if i16::from(guess) <= MOST_COMMON_HEIGHT + 15 {
        i32::from(target) >= (20 * i32::from(guess) - 7 * i32::from(MOST_COMMON_HEIGHT) - 300 + 12) / 13
    } else {
        i32::from(target) >= (20 * i32::from(guess) + 7 * i32::from(MOST_COMMON_HEIGHT) - 300 + 26) / 27
    }
}

/// Checks whether comparing `guess` and `target` heights would yield
/// `comparison`. This is the [`Threshold::compare`] for sinner height, but
/// solved for `target`.
fn height_matches(guess: u8, target: u8, comparison: Comparison) -> bool {
    match comparison {
        | Comparison::Correct => guess == target,
        | Comparison::Greater => {
            !height_below_upper_near_threshold(guess, target) &&
                height_below_upper_far_threshold(guess, target)
        },
        | Comparison::FarGreater => {
            !height_below_upper_near_threshold(guess, target) &&
                !height_below_upper_far_threshold(guess, target)
        },
        | Comparison::Less => {
            !height_above_lower_near_threshold(guess, target) &&
                height_above_lower_far_threshold(guess, target)
        },
        | Comparison::FarLess => {
            !height_above_lower_near_threshold(guess, target) &&
                !height_above_lower_far_threshold(guess, target)
        },
        | Comparison::Near => {
            guess != target &&
                height_below_upper_near_threshold(guess, target) &&
                height_above_lower_near_threshold(guess, target)
        },
    }
}
impl Sinner {
    /// Guesses a sinner based on this sinner being the target, returning a
    /// [`Guess`]
    pub fn guess(&self, guess: &Self) -> Guess {
        let threshes = self.thresholds();
        let mut code = self
            .code
            .zip(guess.code)
            .map(|(target, guess)| threshes.code.unwrap().compare(target.into(), guess.into()));
        // the only non-numeric code is NOX, so we want to make sure NOX can be guessed
        if self.code.is_none() && guess.code.is_none() {
            code = Some(Comparison::Correct);
        }
        let height = threshes
            .height
            .compare(self.height.into(), guess.height.into());
        Guess::new(
            code,
            self.alignment == guess.alignment,
            self.tendency == guess.tendency,
            height,
            self.birthplace == guess.birthplace,
        )
    }

    /// Checks whether `guess` matches the guess result `result` with `self` as
    /// the target
    pub fn matches_result(&self, result: Guess, guess: &Self) -> bool {
        if (self.tendency == guess.tendency) != result.tendency() {
            return false;
        }
        if (self.alignment == guess.alignment) != result.alignment() {
            return false;
        }
        if (self.birthplace == guess.birthplace) != result.birthplace() {
            return false;
        }
        if let Some(code) = result.code() {
            let Some((guess, candidate)) = self.code.zip(guess.code) else {
                return false;
            };
            if !code_matches(guess, candidate, code) {
                return false;
            }
        } else if self.code.is_some() && guess.code.is_some() {
            return false;
        }

        // println!("{} {}\n{} {}", self.name, self.height, guess.name, guess.height);
        height_matches(self.height, guess.height, result.height())
    }
}

impl FromStr for Guess {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn from_str_impl(s: &str) -> Option<Guess> {
            let mut iter = s.trim().split_ascii_whitespace();
            let code = iter.next()?.parse::<MaybeComparison>().ok()?.0;
            let alignment = iter.next()?.parse::<HumanBool>().ok()?.0;
            let tendency = iter.next()?.parse::<HumanBool>().ok()?.0;
            let height = iter.next()?.parse::<MaybeComparison>().ok()?.0?;
            let birthplace = iter.next()?.parse::<HumanBool>().ok()?.0;
            Some(Guess::new(code, alignment, tendency, height, birthplace))
        }
        from_str_impl(s).ok_or(())
    }
}
struct HumanBool(bool);
impl FromStr for HumanBool {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            return Err(());
        }
        Ok(Self(match s.as_bytes()[0] {
            | b'y' | b'Y' | b't' | b'T' | b'1' => true,
            | b'n' | b'N' | b'f' | b'F' | b'0' => false,
            | _ => return Err(()),
        }))
    }
}
struct MaybeComparison(Option<Comparison>);
impl FromStr for MaybeComparison {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Some(match s.trim() {
            | "x" | "X" => return Ok(Self(None)),
            | "vv" => Comparison::FarLess,
            | "v" => Comparison::Less,
            | "~" => Comparison::Near,
            | "=" => Comparison::Correct,
            | "^" => Comparison::Greater,
            | "^^" => Comparison::FarGreater,
            | _ => return Err(()),
        })))
    }
}
