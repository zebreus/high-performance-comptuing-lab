use std::fmt::Debug;

pub const TO_WEST: u8 = 0b0000_0001;
pub const TO_NORTH_WEST: u8 = 0b00000010;
pub const TO_NORTH_EAST: u8 = 0b00000100;
pub const TO_EAST: u8 = 0b00001000;
pub const TO_SOUTH_EAST: u8 = 0b00010000;
pub const TO_SOUTH_WEST: u8 = 0b00100000;

// const CANCEL_EAST_WEST: u8 = TO_WEST | TO_EAST;
// const CANCEL_NORTH_EAST_SOUTH_WEST: u8 = TO_NORTH_EAST | TO_SOUTH_WEST;
// const CANCEL_NORTH_WEST_SOUTH_EAST: u8 = TO_NORTH_EAST | TO_SOUTH_WEST;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Cell {
    pub raw: u8,
}

impl Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cell")
            .field("content", &self.raw)
            .field("to_west", &self.to_west())
            .field("to_north_west", &self.to_north_west())
            .field("to_north_east", &self.to_north_east())
            .field("to_east", &self.to_east())
            .field("to_south_east", &self.to_south_east())
            .field("to_south_west", &self.to_south_west())
            .finish()
    }
}

impl Cell {
    pub fn new() -> Self {
        Self { raw: 0 }
    }

    pub fn receive_from_west(&mut self, other: &Cell) {
        self.raw |= other.raw & TO_EAST;
    }
    pub fn receive_from_north_west(&mut self, other: &Cell) {
        self.raw |= other.raw & TO_SOUTH_EAST;
    }
    pub fn receive_from_north_east(&mut self, other: &Cell) {
        self.raw |= other.raw & TO_SOUTH_WEST;
    }
    pub fn receive_from_east(&mut self, other: &Cell) {
        self.raw |= other.raw & TO_WEST;
    }
    pub fn receive_from_south_east(&mut self, other: &Cell) {
        self.raw |= other.raw & TO_NORTH_WEST;
    }
    pub fn receive_from_south_west(&mut self, other: &Cell) {
        self.raw |= other.raw & TO_NORTH_EAST;
    }

    pub fn to_east(&self) -> bool {
        self.raw & TO_EAST != 0
    }
    pub fn to_south_east(&self) -> bool {
        self.raw & TO_SOUTH_EAST != 0
    }
    pub fn to_south_west(&self) -> bool {
        self.raw & TO_SOUTH_WEST != 0
    }
    pub fn to_west(&self) -> bool {
        self.raw & TO_WEST != 0
    }
    pub fn to_north_west(&self) -> bool {
        self.raw & TO_NORTH_WEST != 0
    }
    pub fn to_north_east(&self) -> bool {
        self.raw & TO_NORTH_EAST != 0
    }

    pub fn set_to_east(&mut self, value: bool) {
        if value {
            self.raw |= TO_EAST;
        } else {
            self.raw &= !TO_EAST;
        }
    }
    pub fn set_to_south_east(&mut self, value: bool) {
        if value {
            self.raw |= TO_SOUTH_EAST;
        } else {
            self.raw &= !TO_SOUTH_EAST;
        }
    }
    pub fn set_to_south_west(&mut self, value: bool) {
        if value {
            self.raw |= TO_SOUTH_WEST;
        } else {
            self.raw &= !TO_SOUTH_WEST;
        }
    }
    pub fn set_to_west(&mut self, value: bool) {
        if value {
            self.raw |= TO_WEST;
        } else {
            self.raw &= !TO_WEST;
        }
    }
    pub fn set_to_north_west(&mut self, value: bool) {
        if value {
            self.raw |= TO_NORTH_WEST;
        } else {
            self.raw &= !TO_NORTH_WEST;
        }
    }
    pub fn set_to_north_east(&mut self, value: bool) {
        if value {
            self.raw |= TO_NORTH_EAST;
        } else {
            self.raw &= !TO_NORTH_EAST;
        }
    }

    pub fn get_particles(&self) -> u8 {
        // Return the one bits
        self.raw.count_ones() as u8
    }

    pub fn process_collision(&mut self, seed: u32) {
        let rand_bool = seed & 1 == 1;
        self.raw = match self.raw {
            // Two opposing particles
            0b00001001 => {
                if rand_bool {
                    TO_NORTH_EAST | TO_SOUTH_WEST
                } else {
                    TO_SOUTH_EAST | TO_NORTH_WEST
                }
            }
            0b00010010 => {
                if rand_bool {
                    TO_NORTH_EAST | TO_SOUTH_WEST
                } else {
                    TO_EAST | TO_WEST
                }
            }
            0b00100100 => {
                if rand_bool {
                    TO_SOUTH_EAST | TO_NORTH_WEST
                } else {
                    TO_EAST | TO_WEST
                }
            }
            // Three particles
            0b00101010 => 0b00010101,
            0b00010101 => 0b00101010,

            // Four particles with opposing holes
            0b00110110 => {
                if rand_bool {
                    0b00011011
                } else {
                    0b00101101
                }
            }
            0b00011011 => {
                if rand_bool {
                    0b00101101
                } else {
                    0b00110110
                }
            }
            0b00101101 => {
                if rand_bool {
                    0b00011011
                } else {
                    0b00110110
                }
            }

            // Everything else
            _ => self.raw,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receive_from_west() {
        let mut cell = Cell::new();
        let mut other = Cell::new();
        cell.raw = TO_WEST;
        other.raw = TO_EAST;
        cell.receive_from_west(&other);
        assert_eq!(cell.to_east(), true, "Content is {:?}", cell);
        assert_eq!(cell.to_west(), true, "Content is {:?}", cell);
    }

    #[test]
    fn test_receive_from_north_west() {
        let mut cell = Cell::new();
        let mut other = Cell::new();
        cell.raw = TO_NORTH_WEST;
        other.raw = TO_SOUTH_EAST;
        cell.receive_from_north_west(&other);
        assert_eq!(cell.to_south_east(), true, "Content is {:?}", cell);
        assert_eq!(cell.to_north_west(), true, "Content is {:?}", cell);
    }

    #[test]
    fn test_receive_from_north_east() {
        let mut cell = Cell::new();
        let mut other = Cell::new();
        cell.raw = TO_NORTH_EAST;
        other.raw = TO_SOUTH_WEST;
        cell.receive_from_north_east(&other);
        assert_eq!(cell.to_south_west(), true, "Content is {:?}", cell);
        assert_eq!(cell.to_north_east(), true, "Content is {:?}", cell);
    }

    #[test]
    fn test_receive_from_east() {
        let mut cell = Cell::new();
        let mut other = Cell::new();
        cell.raw = TO_EAST;
        other.raw = TO_WEST;
        cell.receive_from_east(&other);
        assert_eq!(cell.to_west(), true, "Content is {:?}", cell);
        assert_eq!(cell.to_east(), true, "Content is {:?}", cell);
    }

    #[test]
    fn test_receive_from_south_east() {
        let mut cell = Cell::new();
        let mut other = Cell::new();
        cell.raw = TO_SOUTH_EAST;
        other.raw = TO_NORTH_WEST;
        cell.receive_from_south_east(&other);
        assert_eq!(cell.to_north_west(), true, "Content is {:?}", cell);
        assert_eq!(cell.to_south_east(), true, "Content is {:?}", cell);
    }

    #[test]
    fn test_receive_from_south_west() {
        let mut cell = Cell::new();
        let mut other = Cell::new();
        cell.raw = TO_SOUTH_WEST;
        other.raw = TO_NORTH_EAST;
        cell.receive_from_south_west(&other);
        assert_eq!(cell.to_north_east(), true, "Content is {:?}", cell);
        assert_eq!(cell.to_south_west(), true, "Content is {:?}", cell);
    }

    #[test]
    fn test_process_collision() {
        let mut cell = Cell::new();
        cell.raw = 0b00001001;
        cell.process_collision(0);
        assert_eq!(
            cell.raw, 0b00010010,
            "{:#08b} != {:#08b}",
            cell.raw, 0b00010010
        );
        cell.process_collision(1);
        assert_eq!(
            cell.raw, 0b00100100,
            "{:#08b} != {:#08b}",
            cell.raw, 0b00100100
        );
    }
}
