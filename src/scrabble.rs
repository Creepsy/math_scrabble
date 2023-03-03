use std::collections::HashMap;
use std::hash::Hash;

use crate::command_parsing::Command;
use crate::scrabble_base_types::{
    move_position, Direction, Placement, PlayerID, Position, ScrabbleLetter,
};
use crate::term_evaluation::Term;

#[derive(Debug)]
pub enum ScrabbleRuntimeError {
    PlayerIDOutOfBOunds(PlayerID),
    PositionOutOfBounds(Position),
    InvalidPlacement(String),
    MissingLetters,
    BlockedSpace,
}

impl std::fmt::Display for ScrabbleRuntimeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScrabbleRuntimeError::PlayerIDOutOfBOunds(player_id) => {
                write!(
                    formatter,
                    "Error: The player with the id {} doesn't exist!",
                    player_id + 1
                )
            }
            ScrabbleRuntimeError::PositionOutOfBounds(position) => {
                write!(
                    formatter,
                    "Error: The position {:?} is out of bounds!",
                    position
                )
            }
            ScrabbleRuntimeError::InvalidPlacement(cause) => {
                write!(formatter, "Error: {}", cause)
            }
            ScrabbleRuntimeError::MissingLetters => {
                write!(
                    formatter,
                    "Error: The bag of the current player doesn't contain the right letters for this placement!"
                )
            }
            ScrabbleRuntimeError::BlockedSpace => {
                write!(
                    formatter,
                    "Error: The placement is out of bounds or tried to overwrite existing letters!"
                )
            }
        }
    }
}

impl std::error::Error for ScrabbleRuntimeError {}

#[repr(isize)]
#[derive(Copy, Clone)]
enum TermDirection {
    Decreasing = -1,
    Increasing = 1,
}

pub struct ScrabbleGame<const N: usize> {
    players: Vec<Player>,
    current_player: usize,
    board: GameBoard<N>,
    is_first_placement: bool,
}

impl<const N: usize> ScrabbleGame<N> {
    pub fn new(player_bags: &Vec<Vec<ScrabbleLetter>>) -> ScrabbleGame<N> {
        ScrabbleGame {
            players: player_bags.into_iter().map(Player::new).collect(),
            current_player: 0,
            board: GameBoard::new(),
            is_first_placement: true,
        }
    }

    pub fn execute_command(&mut self, cmd: &Command) -> Result<(), ScrabbleRuntimeError> {
        match cmd {
            Command::Quit => unreachable!("Bug: Quit commands shouldn't be handled by the game!"),
            Command::Print => {
                print!("{}", self.board);
                Ok(())
            }
            Command::Score(player_id) => {
                if *player_id >= self.players.len() {
                    Err(ScrabbleRuntimeError::PlayerIDOutOfBOunds(*player_id))
                } else {
                    println!("{}", self.players[*player_id].score);
                    Ok(())
                }
            }
            Command::Bag(player_id) => {
                if *player_id >= self.players.len() {
                    Err(ScrabbleRuntimeError::PlayerIDOutOfBOunds(*player_id))
                } else {
                    println!(
                        "{}",
                        self.players[*player_id]
                            .letter_bag
                            .iter()
                            .map(ScrabbleLetter::to_string)
                            .collect::<String>()
                    );
                    Ok(())
                }
            }
            Command::Place(placement) => self.place_on_board(placement),
        }
    }

    fn place_on_board(&mut self, placement: &Placement) -> Result<(), ScrabbleRuntimeError> {
        self.get_current_player().try_consume(&placement.letters)?;

        match self.try_place(placement) {
            Ok(_) => (),
            Err(e) => {
                self.get_current_player()
                    .letter_bag
                    .append(&mut placement.letters.clone());
                return Err(e);
            }
        }

        let (terms, owners): (Vec<Term>, Vec<Owner>) = self
            .get_placement_terms(placement)
            .into_iter()
            .filter(|term| !term.0.is_singleton())
            .unzip();
        let results = terms
            .iter()
            .map(|to_eval| to_eval.evaluate())
            .collect::<Vec<Result<i32, String>>>();
        let are_terms_valid = results.iter().all(|res| res.is_ok());
        assert!(!self.is_first_placement || terms.len() == 1);

        // combine these
        if !are_terms_valid {
            self.get_current_player()
                .letter_bag
                .append(&mut placement.letters.clone());
            self.revert_placement(placement);
            return Err(ScrabbleRuntimeError::InvalidPlacement(
                "The placement leads to invalid terms!".to_string(),
            ));
        }
        if terms.is_empty() {
            self.get_current_player()
                .letter_bag
                .append(&mut placement.letters.clone());
            self.revert_placement(placement);
            return Err(ScrabbleRuntimeError::InvalidPlacement(
                "Terms of length 1 are not allowed!".to_string(),
            ));
        }
        // the following only makes sense with normal scrabble 
        // if !self.is_first_placement && terms.len() == 1 && terms[0] == Term::new(&placement.letters)
        // {
        //     self.get_current_player()
        //         .letter_bag
        //         .append(&mut placement.letters.clone());
        //     self.revert_placement(placement);
        //     return Err(ScrabbleRuntimeError::InvalidPlacement(
        //         "Your placement must include at least one already placed letter!".to_string(),
        //     ));
        // }

        // validity already checked -> are_terms_valid
        let results_unwrapped = results.into_iter().map(|res| res.unwrap());

        owners
            .into_iter()
            .zip(results_unwrapped.into_iter())
            .for_each(|(owner, score)| match owner {
                Owner::None => (),
                Owner::Owning(player_id) => self.players[player_id].score += score as isize,
            });

        self.next_player();
        self.is_first_placement = false;

        Ok(())
    }

    fn try_place(&mut self, placement: &Placement) -> Result<(), ScrabbleRuntimeError> {
        for offset in 0..placement.letters.len() {
            if let Err(err) = self.board.try_place(
                self.current_player,
                placement.letters[offset],
                move_position(placement.start_pos, offset as isize, &placement.direction),
            ) {
                self.revert_placement(&Placement::new(
                    &placement.letters[..offset].to_vec(),
                    &placement.start_pos,
                    &placement.direction,
                ));
                return Err(err);
            }
        }

        Ok(())
    }

    fn revert_placement(&mut self, placement: &Placement) {
        (0..placement.letters.len()).into_iter().for_each(|offset| {
            self.board.clear(move_position(
                placement.start_pos,
                offset as isize,
                &placement.direction,
            ))
        });
    }

    fn get_placement_terms(&self, placement: &Placement) -> Vec<(Term, Owner)> {
        let mut terms = Vec::new();
        let orthogonal = placement.direction.orthogonal();

        terms.push(self.get_term(placement.start_pos, &placement.direction));

        for offset in 0..placement.letters.len() as isize {
            terms.push(self.get_term(
                move_position(placement.start_pos, offset, &placement.direction),
                &orthogonal,
            ));
        }

        terms
    }

    fn collect_to_term_end(
        &self,
        position: Position,
        direction: &Direction,
        iter_dir: TermDirection,
    ) -> Vec<Position> {
        let mut curr_iter_offset = 0;

        std::iter::from_fn(move || {
            let curr_pos = move_position(position, curr_iter_offset, &direction);

            if self.board.is_out_of_bounds(curr_pos) || self.board.is_empty(curr_pos) {
                None
            } else {
                curr_iter_offset += iter_dir as isize;
                Some(curr_pos)
            }
        })
        .into_iter()
        .collect()
    }

    fn get_term(&self, position: Position, direction: &Direction) -> (Term, Owner) {
        let start_sequence =
            self.collect_to_term_end(position, direction, TermDirection::Decreasing);
        let end_sequence = self.collect_to_term_end(position, direction, TermDirection::Increasing);
        let term_sequence = start_sequence
            .into_iter()
            .rev()
            .chain(end_sequence.into_iter().skip(1));

        let (term, owners): (Vec<ScrabbleLetter>, Vec<Owner>) = term_sequence
            .map(|pos| self.board.try_get(pos))
            .collect::<Result<Vec<(ScrabbleLetter, Owner)>, ScrabbleRuntimeError>>()
            .expect("BUG: term is out of bounds!")
            .into_iter()
            .unzip();

        let mut frequencies = frequency(&owners);
        frequencies.sort_by(|a, b| b.1.cmp(&a.1));
        assert!(frequencies.len() > 0);

        if frequencies.len() == 1 {
            (Term::new(&term), frequencies[0].0)
        } else {
            assert!(frequencies.len() >= 2);
            let owner = if frequencies[0].1 == frequencies[1].1 {
                Owner::None
            } else {
                frequencies[0].0
            };
            (Term::new(&term), owner)
        }
    }

    fn get_current_player(&mut self) -> &mut Player {
        &mut self.players[self.current_player]
    }

    fn next_player(&mut self) {
        self.current_player = (self.current_player + 1) % self.players.len();
    }
}

pub struct GameBoard<const N: usize> {
    tiles: [[(ScrabbleLetter, Owner); N]; N],
}

impl<const N: usize> GameBoard<N> {
    pub fn try_place(
        &mut self,
        placer_id: PlayerID,
        to_place: ScrabbleLetter,
        pos: Position,
    ) -> Result<(), ScrabbleRuntimeError> {
        if !self.is_empty(pos) {
            return Err(ScrabbleRuntimeError::BlockedSpace);
        }
        self.tiles[pos.0 as usize][pos.1 as usize] = (to_place, Owner::Owning(placer_id));
        Ok(())
    }

    pub fn try_get(&self, pos: Position) -> Result<(ScrabbleLetter, Owner), ScrabbleRuntimeError> {
        if self.is_out_of_bounds(pos) {
            Err(ScrabbleRuntimeError::PositionOutOfBounds(pos))
        } else {
            Ok(self.tiles[pos.0 as usize][pos.1 as usize])
        }
    }

    pub fn clear(&mut self, pos: Position) {
        if self.is_out_of_bounds(pos) {
            return;
        }
        self.tiles[pos.0 as usize][pos.1 as usize] = (ScrabbleLetter::Empty, Owner::None);
    }

    pub fn is_empty(&self, pos: Position) -> bool {
        if self.is_out_of_bounds(pos) {
            return false;
        }
        self.tiles[pos.0 as usize][pos.1 as usize].0 == ScrabbleLetter::Empty
    }

    pub fn new() -> GameBoard<N> {
        GameBoard {
            tiles: [[(ScrabbleLetter::Empty, Owner::None); N]; N],
        }
    }

    pub fn is_out_of_bounds(&self, pos: Position) -> bool {
        pos.0 < 0 || pos.1 < 0 || pos.0 as usize >= N || pos.1 as usize >= N
    }
}

impl<const N: usize> std::fmt::Display for GameBoard<N> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..N {
            for x in 0..N {
                write!(formatter, "[{}]", &self.tiles[x][y].0)?;
            }
            writeln!(formatter, "")?;
        }

        Ok(())
    }
}

pub struct Player {
    letter_bag: Vec<ScrabbleLetter>,
    score: isize,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum Owner {
    None,
    Owning(PlayerID),
}

impl Player {
    pub fn new(letter_bag: &Vec<ScrabbleLetter>) -> Player {
        Player {
            letter_bag: letter_bag.clone(),
            score: 0,
        }
    }

    pub fn try_consume(
        &mut self,
        to_consume: &Vec<ScrabbleLetter>,
    ) -> Result<(), ScrabbleRuntimeError> {
        let mut modified_letter_bag = self.letter_bag.clone();

        for letter in to_consume {
            if let Some(position) = modified_letter_bag.iter().position(|val| val == letter) {
                modified_letter_bag.remove(position);
            } else {
                return Err(ScrabbleRuntimeError::MissingLetters);
            }
        }

        self.letter_bag = modified_letter_bag;

        Ok(())
    }
}

fn frequency<T: Eq + Hash + Copy>(elements: &Vec<T>) -> Vec<(T, usize)> {
    let mut occurences = HashMap::new();

    for element in elements {
        occurences
            .entry(*element)
            .and_modify(|counter| *counter += 1)
            .or_insert(1);
    }

    occurences.into_iter().collect()
}
