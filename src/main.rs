mod command_parsing;
mod scrabble;
mod scrabble_base_types;
mod term_evaluation;

use scrabble_base_types::ScrabbleLetter;
use std::io::{self, BufRead};
use std::str::FromStr;

use scrabble::ScrabbleGame;

fn main() {
    let stdin = io::stdin();
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.len() < 2 {
        println!("You need at least 2 players to play math scrabble!");
        return;
    }

    let player_letter_bags: Vec<Option<Vec<ScrabbleLetter>>> = args
        .into_iter()
        .map(|letters_str| {
            letters_str
                .chars()
                .map(|c| ScrabbleLetter::from_char(c))
                .collect()
        })
        .collect();
    if player_letter_bags.iter().any(|bag| bag.is_none()) {
        println!("At least one of the player bags contains invalid letters!");
        return;
    }
    // validity already checked
    let player_letter_bags_unwrapped = player_letter_bags
        .into_iter()
        .map(|bag| bag.unwrap())
        .collect();
    let mut scrabble_game = ScrabbleGame::<10>::new(&player_letter_bags_unwrapped);

    loop {
        let line = stdin
            .lock()
            .lines()
            .next()
            .expect("no next line")
            .expect("read err");

        let command = command_parsing::Command::from_str(line.as_str());

        match command {
            Err(err) => println!("{}", err),
            Ok(command_parsing::Command::Quit) => break,
            Ok(cmd) => match scrabble_game.execute_command(&cmd) {
                Err(err) => println!("{}", err),
                Ok(_) => (),
            },
        }
    }
}
