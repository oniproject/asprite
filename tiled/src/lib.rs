#![feature(range_contains)]

#[macro_use]
extern crate bitflags;
extern crate nalgebra as na;

pub mod layer;
pub mod shape;
pub mod hex;
pub mod tile;