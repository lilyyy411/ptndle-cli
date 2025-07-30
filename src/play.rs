use std::cell::RefCell;
use std::fmt::Display;
use std::io::{stdin, stdout, Write};
use std::str::FromStr;

use eyre::eyre;
use facet::Facet;
use ordered_float::NotNan;
use owo_colors::OwoColorize;
use reedline::{default_emacs_keybindings, ColumnarMenu, DefaultCompleter, DefaultPrompt, Emacs,
               ExampleHighlighter, KeyModifiers, MenuBuilder, Reedline, ReedlineEvent, Signal};

use crate::data::Sinner;
use crate::guess::Guess;

#[derive(Debug, Clone)]
pub struct Game<'game> {
    target: &'game Sinner,
    guess_num: u8,
}

impl<'game> Game<'game> {
    pub fn new(target: &'game Sinner) -> Self {
        Self {
            target,
            guess_num: 1,
        }
    }
    pub fn guess_num(&self) -> u8 { self.guess_num }
    pub fn guess(&mut self, character: &'_ Sinner) -> Option<Guess> {
        if character == self.target {
            return None;
        }
        let guess = self.target.guess(character);
        self.guess_num += 1;
        Some(guess)
    }
}

pub trait Player {
    /// Updates the state of the player based on a given guess and the character
    /// guessed
    fn update(&mut self, result: Guess, character: &Sinner);
    /// Gets the next guess from the player. May return `None` if there is a
    /// contradiction in the state.
    fn next_guess(&self) -> Option<&Sinner>;
}

/// A [`Player`] that guesses sinners based on the mean number of sinners
/// remaining after a guess.
#[derive(Debug, Clone)]
pub struct OptimalPlayer {
    candidates: Vec<Sinner>,
}

impl Player for OptimalPlayer {
    fn update(&mut self, result: Guess, character: &Sinner) {
        self.candidates = std::mem::take(&mut self.candidates)
            .into_iter()
            .filter(|x| character.matches_result(result, x) && x.code != character.code)
            .collect();
    }
    #[expect(clippy::float_arithmetic, reason = "statistics")]
    fn next_guess(&self) -> Option<&Sinner> {
        if self.candidates.len() == 1 {
            return Some(&self.candidates[0]);
        }
        #[expect(
            clippy::cast_precision_loss,
            reason = "The sum will not get big enough for it to be an issue"
        )]
        Some(
            self.candidates
                .iter()
                .map(|guess| {
                    (
                        guess,
                        self.candidates
                            .iter()
                            .filter(|target| guess != *target)
                            .map(|target| {
                                self.candidates
                                    .iter()
                                    .filter(|x| guess.matches_result(target.guess(guess), x))
                                    .count()
                            })
                            .sum::<usize>() as f64 /
                            self.candidates.len() as f64,
                    )
                })
                .min_by_key(|(_, mean)| NotNan::new(*mean).unwrap())?
                .0,
        )
    }
}

impl OptimalPlayer {
    pub fn new(candidates: Vec<Sinner>) -> OptimalPlayer { OptimalPlayer { candidates } }
}

/// A [`Player`] connected to the terminal
pub struct HumanPlayer {
    line_editor: RefCell<Reedline>,
    choices: Vec<Sinner>,
}
impl HumanPlayer {
    pub fn new(choices: Vec<Sinner>) -> Self {
        let commands = choices
            .iter()
            .map(|x| "info ".to_owned() + &x.name)
            .chain(choices.iter().map(|x| "guess ".to_owned() + &x.name))
            .chain(std::iter::once("quit".to_owned()))
            .collect();
        let mut completer = DefaultCompleter::with_inclusions(&['.', '-']);
        completer.insert(commands);
        let completer = Box::new(completer);
        let highlighter = Box::new(ExampleHighlighter::new(vec![
            "info".to_owned(),
            "guess".to_owned(),
            "quit".to_owned(),
        ]));
        let mut keybindings = default_emacs_keybindings();
        keybindings.add_binding(
            KeyModifiers::NONE,
            reedline::KeyCode::Tab,
            ReedlineEvent::UntilFound(vec![
                ReedlineEvent::Menu("completion_menu".to_owned()),
                ReedlineEvent::MenuNext,
            ]),
        );
        let completion_menu = Box::new(ColumnarMenu::default().with_name("completion_menu"));
        let line_editor = RefCell::new(
            Reedline::create()
                .with_edit_mode(Box::new(Emacs::new(keybindings)))
                .with_completer(completer)
                .with_menu(reedline::ReedlineMenu::EngineCompleter(completion_menu))
                .with_highlighter(highlighter),
        );
        Self {
            line_editor,
            choices,
        }
    }
}
impl Player for HumanPlayer {
    fn next_guess(&self) -> Option<&Sinner> {
        loop {
            match self.line_editor.borrow_mut().read_line(&DefaultPrompt::new(
                reedline::DefaultPromptSegment::Basic("ptndle >>".to_owned()),
                reedline::DefaultPromptSegment::Basic("Hella yeah!".to_owned()),
            )) {
                | Ok(Signal::Success(buffer)) => {
                    let buffer = buffer.trim();
                    if buffer == "quit" {
                        std::process::exit(0);
                    }
                    let Some((cmd, arg)) = buffer.split_once(' ') else {
                        eprintln!("Unknown command: `{buffer}`");
                        continue;
                    };

                    match cmd {
                        | "info" => {
                            let Some(sinner) = self
                                .choices
                                .iter()
                                .find(|x| x.name.eq_ignore_ascii_case(arg))
                            else {
                                eprintln!("Unknown Sinner: `{arg}`");
                                continue;
                            };
                            println!("Name: {}", sinner.name);
                            println!(
                                "Code: {}",
                                sinner
                                    .code
                                    .as_ref()
                                    .map_or_else(|| "NOX".to_owned(), <_>::to_string)
                            );
                            println!("Alignment: {:?}", sinner.alignment);
                            println!("Tendency: {:?}", sinner.tendency);
                            println!("Height: {}cm", sinner.height);
                            println!("Birthplace: {:?}", sinner.birthplace);
                        },
                        | "guess" => {
                            let Some(to_play) = self
                                .choices
                                .iter()
                                .find(|x| x.name.eq_ignore_ascii_case(arg))
                            else {
                                eprintln!("Unknown Sinner: `{arg}`");
                                continue;
                            };
                            break Some(to_play);
                        },
                        | _ => {
                            eprintln!("Unknown command: `{cmd}`");
                        },
                    }
                },
                | Ok(Signal::CtrlC | Signal::CtrlD) => {
                    eprintln!("Aborted!");
                    std::process::exit(1);
                },
                | Err(e) => {
                    eprintln!("Failed to read line of input: {e}");
                    std::process::exit(1);
                },
            }
        }
    }
    fn update(&mut self, _result: Guess, _character: &Sinner) {}
}

pub fn play_game<P: Player>(target: &Sinner, mut player: P) -> u8 {
    let mut game = Game::new(target);

    loop {
        let Some(play) = player.next_guess() else {
            eprintln!("No possible guesses in this state. There is likely a contradiction.");
            return 255;
        };
        println!("Guessed {}", play.name);
        if let Some(guess) = game.guess(play) {
            println!("{guess}");
            assert!(
                play.matches_result(guess, target),
                "ERROR: Target ({target:?}) does not match its own result ({guess}) based on \
                 guess ({play:?}). This is a bug."
            );
            let c = play.clone();

            player.update(guess, &c);
        } else {
            println!("{}", " =  1  1  =  1".green());
            println!("Won! The sinner was {}!", target.name);
            println!("Won in {} guesses!\n", game.guess_num());
            break game.guess_num();
        }
    }
}

#[expect(clippy::float_arithmetic, reason = "statistics")]
#[expect(clippy::unnecessary_wraps, reason = "maybe fallible later")]
pub fn gather_data(sinners: &[Sinner]) -> eyre::Result<()> {
    let sinner_data: Vec<(u8, &Sinner)> = sinners
        .iter()
        .map(|target| {
            (
                play_game(target, OptimalPlayer::new(sinners.to_owned())),
                target,
            )
        })
        .collect();

    println!(
        "Goto first sinner to play: {}",
        OptimalPlayer::new(sinners.to_owned())
            .next_guess()
            .unwrap()
            .name
    );
    let (max_rounds, _) = sinner_data
        .iter()
        .max_by_key(|(guesses, _)| *guesses)
        .unwrap();
    let max_round_sinners = sinner_data
        .iter()
        .filter(|(guesses, _)| guesses == max_rounds);
    println!("It takes {max_rounds} or less guesses to guess any sinner.");

    #[expect(clippy::cast_precision_loss, reason = "It doesn't matter.")]
    for rounds in 1..=*max_rounds {
        let count = sinner_data.iter().filter(|(v, _)| *v == rounds).count();
        println!(
            "    {count} sinners take {rounds} guesses ({:.2}%)",
            count as f64 * 100. / sinner_data.len() as f64
        );
    }
    println!("The sinners that take the maximum number of guesses rounds are:");
    for (_, sinner) in max_round_sinners {
        println!("    {}", sinner.name);
    }
    let sum: u32 = sinner_data.iter().map(|(r, _)| u32::from(*r)).sum();

    #[expect(clippy::cast_precision_loss, reason = "It doesn't matter.")]
    {
        println!(
            "The mean number of guesses is {:.2}",
            f64::from(sum) / sinner_data.len() as f64
        );
    };

    Ok(())
}

pub fn solve(initial_state: &[NameAndGuess], sinners: Vec<Sinner>) -> eyre::Result<()> {
    println!("======== Welcome to the Path to Nowordle Solver ========");
    println!(
        "This solver always wins within 4 guesses from an unknown sinner target, but typically \
         wins in 3 or less.\n"
    );
    println!("======== Instructions ========");
    println!(
        "Enter a row as seen on the website when prompted and guess the sinner you are prompted \
         to play."
    );
    println!("Entries in the row are separated by whitespace.");
    println!("Comparisons are entered as vv/v/~/=/^/^^ and booleans are entered as 0 or 1.");
    println!("An example input is ^^ 0 0 ~ 1");
    println!("==============================");
    let sinners_clone = sinners.clone();
    let mut player = OptimalPlayer::new(sinners);

    for NameAndGuess { name, guess } in initial_state {
        let sinner = sinners_clone
            .iter()
            .find(|x| x.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| eyre!("No sinner with name {name} found"))?
            .clone();
        player.update(*guess, &sinner);
    }
    if !initial_state.is_empty() {
        let names = player.candidates.iter().map(|x| x.name.as_str());
        println!("Possible Sinners: {}", names.collect::<Vec<_>>().join(", "));
    }
    'outer: loop {
        let sinner = player
            .next_guess()
            .ok_or_else(|| {
                eyre!("No possible guesses in this state. There is likely a contradiction.")
            })?
            .clone();
        println!("Guess {}", sinner.name);
        if player.candidates.len() == 1 {
            println!("GG! You won.");
            break;
        }

        let guess;
        loop {
            let mut line = <_>::default();
            print!("Enter row or q to quit: ");
            stdout().flush()?;
            stdin().read_line(&mut line)?;
            if line.trim() == "q" {
                break 'outer;
            }
            if let Ok(g) = line.parse::<Guess>() {
                guess = g;
                break;
            }
        }

        player.update(guess, &sinner);
        let names = player.candidates.iter().map(|x| x.name.as_str());

        println!("Possible Sinners: {}", names.collect::<Vec<_>>().join(", "));
    }
    Ok(())
}

#[derive(Debug, Facet)]
pub struct NameAndGuess {
    pub name: String,
    pub guess: Guess,
}

#[derive(Debug)]
pub enum NameAndGuessError {
    NoColon,
    InvalidGuess,
}
impl Display for NameAndGuessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            | Self::NoColon => "No : in input",
            | Self::InvalidGuess => "Invalid guess format.",
        })
    }
}
impl FromStr for NameAndGuess {
    type Err = NameAndGuessError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((name, guess)) = s.split_once(':') else {
            return Err(NameAndGuessError::NoColon);
        };
        Ok(NameAndGuess {
            name: name.trim().to_owned(),
            guess: guess
                .parse()
                .map_err(|()| NameAndGuessError::InvalidGuess)?,
        })
    }
}
