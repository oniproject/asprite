#![feature(range_contains)]
#![feature(iterator_step_by)]

#[macro_use]
extern crate bitflags;
extern crate nalgebra as na;

pub mod layer;
pub mod shape;
pub mod hex;
pub mod tile;
pub mod renderer;

pub mod rpg;

pub struct Simple {
}