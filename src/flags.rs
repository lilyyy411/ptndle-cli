
use std::fmt::Display;
use std::str::FromStr;

use crate::play::{NameAndGuess, NameAndGuessError};
#[derive(Debug)]
pub struct NameAndGuesses(pub Vec<NameAndGuess>);

impl FromStr for NameAndGuesses {
    type Err = NameAndGuessError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(',')
            .map(|s| s.trim().parse())
            .collect::<Result<_, Self::Err>>()
            .map(NameAndGuesses)
    }
}
#[derive(Debug)]
pub enum HelpCommand {
    Gather,
    Solve,
    Play,
    Help,
}
#[derive(Debug)]
pub struct UnknownCommandError(String);

impl FromStr for HelpCommand {
    type Err = UnknownCommandError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            | "gather" => Self::Gather,
            | "solve" => Self::Solve,
            | "play" => Self::Play,
            | "help" => Self::Help,
            | s => return Err(UnknownCommandError(s.to_owned())),
        })
    }
}
impl Display for UnknownCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Unknown command: `")?;
        f.write_str(&self.0)?;
        f.write_str("`")
    }
}
xflags::xflags! {
    /// A cli tool for both playing and solving games of Path to Nowordle (https://ptndle.com/),
    /// a game for guessing Path to Nowhere characters based on their characteristics.
    ///
    cmd ptndle-cli {
        /// Force-fetch the latest sinner data and store it in the cache.
        optional -f, --force-cache-update
         /// View in-depth help for a command
        cmd help {
            /// The command to view help for
            required command: HelpCommand
        }
        /// Play every possible game of Path To Nowordle and gather statistical data about
        /// the solver's performance
        cmd gather {}
        /// Play a game of Path to Nowordle from the terminal
        cmd play {}
        /// Solve a game of Path to Nowordle from an optional set of starting guesses.
        cmd solve {
            /// A list of previous guesses to pass to the solver in the form of a comma-separated list of name:guess.
            /// For more information, view the in-depth help.
            optional guesses: NameAndGuesses
        }

    }
}

