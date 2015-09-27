#![feature(core, convert, custom_derive, plugin, unboxed_closures)]
#![plugin(num_macros)]
#![allow(dead_code)]

#[macro_use]
extern crate nom;

extern crate byteorder;
extern crate num;

pub mod filter;
pub mod midi;
pub mod music;
pub mod synthesizer;
pub mod wave;
pub mod writer;

