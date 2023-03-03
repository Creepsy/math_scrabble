use crate::scrabble_base_types::{Direction, Placement, PlayerID, ScrabbleLetter};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum CommandParseError {
    UnknownCommand {
        input: String,
    },
    InvalidPlayerID {
        id: String,
    },
    InvalidPlacement {
        placement: String,
    },
    InvalidLetters {
        letters: String,
    },
    InvalidArgumentCount {
        command: String,
        expected: usize,
        received: usize,
    },
}

impl std::fmt::Display for CommandParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandParseError::UnknownCommand { input } => {
                write!(formatter, "Error: '{}' is not a valid command!", input)
            }
            CommandParseError::InvalidPlayerID { id } => {
                write!(formatter, "Error: '{}' is not a valid player id!", id)
            }
            CommandParseError::InvalidPlacement { placement } => write!(
                formatter,
                "Error: '{}' is not a valid placement!",
                placement
            ),
            CommandParseError::InvalidLetters { letters } => {
                write!(formatter, "Error: '{}' contains invalid letters!", letters)
            }
            CommandParseError::InvalidArgumentCount {
                command,
                expected,
                received,
            } => {
                write!(
                    formatter,
                    "Error: The command '{}' expects {} arguments, but received {}!",
                    command, expected, received
                )
            }
        }
    }
}

impl std::error::Error for CommandParseError {}

#[derive(Debug)]
pub enum Command {
    Quit,
    Print,
    Score(PlayerID),
    Bag(PlayerID),
    Place(Placement),
}

impl FromStr for Command {
    type Err = CommandParseError;

    fn from_str(command_str: &str) -> Result<Self, Self::Err> {
        let arg_count = command_str.split(' ').collect::<Vec<&str>>().len() - 1;

        match &command_str.split(' ').collect::<Vec<&str>>()[..] {
            ["quit"] => Ok(Command::Quit),
            ["print"] => Ok(Command::Print),
            ["score", player_id] => player_id_from_str(player_id).map(|id| Command::Score(id)),
            ["bag", player_id] => player_id_from_str(player_id).map(|id| Command::Bag(id)),
            ["place", placement] => {
                placement_from_str(placement).map(|placement| Command::Place(placement))
            }

            ["quit", ..] => Err(CommandParseError::InvalidArgumentCount {
                command: "quit".to_string(),
                expected: 0,
                received: arg_count,
            }),
            ["print", ..] => Err(CommandParseError::InvalidArgumentCount {
                command: "print".to_string(),
                expected: 0,
                received: arg_count,
            }),
            ["score", ..] => Err(CommandParseError::InvalidArgumentCount {
                command: "score".to_string(),
                expected: 1,
                received: arg_count,
            }),
            ["bag", ..] => Err(CommandParseError::InvalidArgumentCount {
                command: "bag".to_string(),
                expected: 1,
                received: arg_count,
            }),
            ["place", ..] => Err(CommandParseError::InvalidArgumentCount {
                command: "place".to_string(),
                expected: 1,
                received: arg_count,
            }),

            _ => Err(CommandParseError::UnknownCommand {
                input: command_str.to_string(),
            }),
        }
    }
}

fn player_id_from_str(id_str: &str) -> Result<PlayerID, CommandParseError> {
    if !id_str.starts_with("P") || id_str.starts_with("P0") {
        Err(CommandParseError::InvalidPlayerID {
            id: id_str.to_string(),
        })
    } else {
        id_str[1..]
            .parse::<PlayerID>()
            .map_err(|_| CommandParseError::InvalidPlayerID {
                id: id_str.to_string(),
            })
            .map(|id| id - 1)
    }
}

fn placement_from_str(placement_str: &str) -> Result<Placement, CommandParseError> {
    let invalid_placement_err = CommandParseError::InvalidPlacement {
        placement: placement_str.to_string(),
    };

    if let [letters, start_x, start_y, direction] =
        placement_str.split(';').collect::<Vec<&str>>()[..]
    {
        //TODO: prevent input of negative numbers!!!
        let start_x: isize = start_x.parse().map_err(|_| invalid_placement_err.clone())?;
        let start_y: isize = start_y.parse().map_err(|_| invalid_placement_err.clone())?;

        if letters.len() < 1 || letters.len() > 3 {
            return Err(invalid_placement_err);
        }

        match direction {
            "H" => Ok(Placement {
                letters: letters
                    .chars()
                    .map(ScrabbleLetter::from_char)
                    .collect::<Option<Vec<ScrabbleLetter>>>()
                    .ok_or(CommandParseError::InvalidLetters {
                        letters: letters.to_string(),
                    })?,
                start_pos: (start_x, start_y),
                direction: Direction::Horizontal,
            }),
            "V" => Ok(Placement {
                letters: letters
                    .chars()
                    .map(ScrabbleLetter::from_char)
                    .collect::<Option<Vec<ScrabbleLetter>>>()
                    .ok_or(CommandParseError::InvalidLetters {
                        letters: letters.to_string(),
                    })?,
                start_pos: (start_x, start_y),
                direction: Direction::Vertical,
            }),
            _ => Err(invalid_placement_err),
        }
    } else {
        Err(invalid_placement_err)
    }
}
