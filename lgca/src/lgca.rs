pub mod cell;
pub mod movement_and_collision;
pub mod visualization;

pub use cell::Cell;
pub use movement_and_collision::{get_in_propagation, get_out_propagation, process_collisions};
pub use visualization::print_section;
