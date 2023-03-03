pub type PlayerID = usize;
pub type Position = (isize, isize);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ScrabbleLetter {
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Plus,
    Minus,
    Dot,
    Empty,
}

impl ScrabbleLetter {
    pub fn from_char(letter: char) -> Option<ScrabbleLetter> {
        match letter {
            '0' => Some(ScrabbleLetter::Num0),
            '1' => Some(ScrabbleLetter::Num1),
            '2' => Some(ScrabbleLetter::Num2),
            '3' => Some(ScrabbleLetter::Num3),
            '4' => Some(ScrabbleLetter::Num4),
            '5' => Some(ScrabbleLetter::Num5),
            '6' => Some(ScrabbleLetter::Num6),
            '7' => Some(ScrabbleLetter::Num7),
            '8' => Some(ScrabbleLetter::Num8),
            '9' => Some(ScrabbleLetter::Num9),
            '+' => Some(ScrabbleLetter::Plus),
            '-' => Some(ScrabbleLetter::Minus),
            '*' => Some(ScrabbleLetter::Dot),

            _ => None,
        }
    }
}

impl std::fmt::Display for ScrabbleLetter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{}",
            match self {
                ScrabbleLetter::Num0 => '0',
                ScrabbleLetter::Num1 => '1',
                ScrabbleLetter::Num2 => '2',
                ScrabbleLetter::Num3 => '3',
                ScrabbleLetter::Num4 => '4',
                ScrabbleLetter::Num5 => '5',
                ScrabbleLetter::Num6 => '6',
                ScrabbleLetter::Num7 => '7',
                ScrabbleLetter::Num8 => '8',
                ScrabbleLetter::Num9 => '9',
                ScrabbleLetter::Plus => '+',
                ScrabbleLetter::Minus => '-',
                ScrabbleLetter::Dot => '*',
                ScrabbleLetter::Empty => ' ',
            }
        )
    }
}

#[derive(Debug, Clone)]
pub enum Direction {
    Horizontal,
    Vertical,
}

impl Direction {
    pub fn as_vec(&self) -> (isize, isize) {
        match self {
            Direction::Horizontal => (1, 0),
            Direction::Vertical => (0, 1),
        }
    }

    pub fn orthogonal(&self) -> Direction {
        match self {
            Direction::Horizontal => Direction::Vertical,
            Direction::Vertical => Direction::Horizontal,
        }
    }
}

#[derive(Debug)]
pub struct Placement {
    pub letters: Vec<ScrabbleLetter>,
    pub start_pos: Position,
    pub direction: Direction,
}

impl Placement {
    pub fn new(
        letters: &Vec<ScrabbleLetter>,
        start_pos: &Position,
        direction: &Direction,
    ) -> Placement {
        Placement {
            letters: letters.clone(),
            start_pos: *start_pos,
            direction: direction.clone(),
        }
    }
}

pub fn move_position(position: Position, offset: isize, direction: &Direction) -> Position {
    (
        position.0 + offset * direction.as_vec().0,
        position.1 + offset * direction.as_vec().1,
    )
}
