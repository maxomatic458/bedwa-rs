use valence::Direction;

pub trait DirectionExt {
    fn as_u8(&self) -> u8;

    fn from_u8(value: u8) -> Direction;
}

impl DirectionExt for Direction {
    fn as_u8(&self) -> u8 {
        match self {
            Direction::Down => 0,
            Direction::Up => 1,
            Direction::North => 2,
            Direction::South => 3,
            Direction::West => 4,
            Direction::East => 5,
        }
    }

    fn from_u8(value: u8) -> Direction {
        match value {
            0 => Direction::Down,
            1 => Direction::Up,
            2 => Direction::North,
            3 => Direction::South,
            4 => Direction::West,
            5 => Direction::East,
            _ => panic!("Invalid direction value: {}", value),
        }
    }
}
