use std::{cell::RefCell, fmt::Debug};

// tag::direction_consts[]
pub const TO_WEST: u8 = 0b0000_0001;
pub const TO_NORTH_WEST: u8 = 0b00000010;
pub const TO_NORTH_EAST: u8 = 0b00000100;
pub const TO_EAST: u8 = 0b00001000;
pub const TO_SOUTH_EAST: u8 = 0b00010000;
pub const TO_SOUTH_WEST: u8 = 0b00100000;
// end::direction_consts[]

use rand::prelude::*;

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

// thread_local! {
thread_local!(pub static RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_entropy()));
// }

impl Cell {
    pub fn new() -> Self {
        Self { raw: 0 }
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

    pub fn get_direction(&self) -> (f32, f32) {
        let mut x: f32 = 0.0;
        let mut y: f32 = 0.0;
        if self.to_east() {
            x += 1.0;
        }
        if self.to_south_east() {
            x += 0.5;
            y += 0.866;
        }
        if self.to_north_east() {
            x += 0.5;
            y -= 0.866;
        }

        if self.to_west() {
            x -= 1.0;
        }
        if self.to_south_west() {
            x -= 0.5;
            y += 0.866;
        }
        if self.to_north_west() {
            x -= 0.5;
            y -= 0.866;
        }

        let angle = y.atan2(x) / std::f32::consts::PI * 2.0;
        let length = (x * x + y * y).sqrt();

        return (angle, length);
    }

    pub fn get_particles(&self) -> u8 {
        // Return the one bits
        self.raw.count_ones() as u8
    }

    // tag::collision_real[]
    pub fn process_collision(&mut self) {
        self.raw = match self.raw {
            // Two opposing particles
            const { TO_WEST | TO_EAST } => {
                if RNG.with(|f| f.borrow_mut().gen::<bool>()) {
                    TO_NORTH_EAST | TO_SOUTH_WEST
                } else {
                    TO_SOUTH_EAST | TO_NORTH_WEST
                }
            }
            const { TO_SOUTH_EAST | TO_NORTH_WEST } => {
                if RNG.with(|f| f.borrow_mut().gen::<bool>()) {
                    TO_NORTH_EAST | TO_SOUTH_WEST
                } else {
                    TO_EAST | TO_WEST
                }
            }
            const { TO_SOUTH_WEST | TO_NORTH_EAST } => {
                if RNG.with(|f| f.borrow_mut().gen::<bool>()) {
                    TO_SOUTH_EAST | TO_NORTH_WEST
                } else {
                    TO_EAST | TO_WEST
                }
            }

            // Three particles
            const { TO_SOUTH_WEST | TO_NORTH_WEST | TO_EAST } => {
                TO_SOUTH_EAST | TO_NORTH_EAST | TO_WEST
            }
            const { TO_SOUTH_EAST | TO_NORTH_EAST | TO_WEST } => {
                TO_SOUTH_WEST | TO_NORTH_WEST | TO_EAST
            }

            // Four particles with opposing holes
            0b00110110 => {
                if RNG.with(|f| f.borrow_mut().gen::<bool>()) {
                    0b00011011
                } else {
                    0b00101101
                }
            }
            0b00011011 => {
                if RNG.with(|f| f.borrow_mut().gen::<bool>()) {
                    0b00101101
                } else {
                    0b00110110
                }
            }
            0b00101101 => {
                if RNG.with(|f| f.borrow_mut().gen::<bool>()) {
                    0b00011011
                } else {
                    0b00110110
                }
            }

            // Everything else
            _ => self.raw,
        }
    }
    // end::collision_real[]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_collision() {
        let mut cell = Cell::new();
        cell.raw = 0b00001001;
        cell.process_collision();
        assert_eq!(
            cell.raw, 0b00010010,
            "{:#08b} != {:#08b}",
            cell.raw, 0b00010010
        );
        cell.process_collision();
        assert_eq!(
            cell.raw, 0b00100100,
            "{:#08b} != {:#08b}",
            cell.raw, 0b00100100
        );
    }
}
