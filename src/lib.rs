#![feature(core, custom_derive, lang_items, old_path, std_misc, unboxed_closures)]
#![allow(dead_code, unused_features)]

// unused_features is here only for std_misc, which is used in some assert! macros.

extern crate byteorder;
extern crate num;

pub mod filter;
pub mod music;
pub mod reader;
pub mod synthesizer;
pub mod wave;
pub mod writer;
