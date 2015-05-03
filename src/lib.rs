#![feature(core, custom_derive, plugin, unboxed_closures)]
#![plugin(num_macros)]
#![allow(dead_code)]

extern crate byteorder;
extern crate num;

pub mod filter;
pub mod music;
pub mod reader;
pub mod synthesizer;
pub mod wave;
pub mod writer;
