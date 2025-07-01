use eyre::eyre;
use getrandom::getrandom;

use crate::data::load_sinners;
use crate::flags::{Help, HelpCommand, PtndleCli, PtndleCliCmd, Solve};
use crate::play::{gather_data, play_game, solve, HumanPlayer};

mod compare;
mod data;
mod flags;
mod guess;
mod play;

const HELP_IN_DEPTH_HELP: &str = "USAGE: ptndle-cli help [command]

View in-depth help for a command";

const GATHER_IN_DEPTH_HELP: &str = "USAGE: ptndle-cli gather

Play every possible game of Path To Nowordle and gather statistical data about
the solver's performance. 

The results of playing each game are sent to stdout along with a summary of the gathered data
containing the following information:
    - The first sinner the solver chooses to play
    - The maximum number of guesses it takes to guess any sinner
    - The distribution of the number of guesses it takes to guess sinners
    - The sinners that take the maximum number of guesses to guess
    - The mean number of guesses it takes to guess a sinner";

const PLAY_IN_DEPTH_HELP: &str = "USAGE: ptndle-cli play

Play a game of Path to Nowordle from the terminal

You will be put into an interactive shell with the following commands:

info [sinner]:  View info on a sinner
guess [sinner]: Guess a sinner
quit:           Quit";

const SOLVE_IN_DEPTH_HELP: &str = "
USAGE: ptndle-cli solve [guesses]

Solve a game of Path to Nowordle from an optional set of starting guesses.

Guesses are made up of 5 whitespace-separated components, Code (comparison),
Alignment (boolean), Tendency (boolean), Height (comparison), and Birthplace (boolean).
Booleans are entered as 0 or 1 and comparisons are entered as follows:

    N/A:         x
    Correct:     =
    Far Less:    vv
    Less:        v
    Near:        ~
    Greater:     ^
    Far Greater: ^^

An example input for a guess is ^^ 0 0 ~ 1 and an example input for the guesses argument
is \"L.L.:^ 0 0 vv 0,Angell:^^ 0 0 vv 0\"";

const PLAY_WELCOME: &str = r" 
      __ 
     /  \
     |,_,|______
     /        `-----.___
    ,|                  `
    /       __       __  |
    |      /  \_____/  \  |
    `     |   O    X   | |
    `+___ `-----------`__^
      /   \__\      /__/ \
      |   ,--, --- ,--,   |
      |   | .|     | .|   |
      |   `-*   >  `-*    |
       \                 /
        \      ._>      /
         \             /
          `-----------`

Welcome to Path to Nowordle CLI edition. 
To guess a sinner, use the `guess` command.
To view a sinner's info, use the `info` command.
To quit, type `quit` or press Ctrl + C.

You can press tab to attempt to complete a command at any time";

fn get_in_depth_help(cmd: &HelpCommand) -> &'static str {
    match cmd {
        | HelpCommand::Gather => GATHER_IN_DEPTH_HELP,
        | HelpCommand::Solve => SOLVE_IN_DEPTH_HELP,
        | HelpCommand::Play => PLAY_IN_DEPTH_HELP,
        | HelpCommand::Help => HELP_IN_DEPTH_HELP,
    }
}
fn main() -> eyre::Result<()> {
    let cli = PtndleCli::from_env_or_exit();
    match cli.subcommand {
        | PtndleCliCmd::Help(Help { command }) => {
            eprintln!("{}", get_in_depth_help(&command));
        },
        | PtndleCliCmd::Gather(_) => gather_data(cli.force_cache_update)?,
        | PtndleCliCmd::Play(_) => {
            println!("{PLAY_WELCOME}");
            let random_num = {
                let mut buf = 0usize.to_ne_bytes();
                getrandom(&mut buf).map_err(|e| eyre!("Failed to get random number: {e}"))?;
                usize::from_le_bytes(buf)
            };
            let sinner_data = load_sinners(cli.force_cache_update)?;
            let target = &sinner_data[random_num  % sinner_data.len()];
            play_game(target, HumanPlayer::new(sinner_data.clone()));
        },
        | PtndleCliCmd::Solve(Solve { guesses }) => {
            solve(
                cli.force_cache_update,
                &guesses.map(|x| x.0).unwrap_or_default(),
            )?;
        },
    }
    Ok(())
}
