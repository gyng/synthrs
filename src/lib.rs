#![feature(core, io, lang_items, path, std_misc, unboxed_closures)]
#![allow(dead_code, unused_features)]

// unused_features is here only for std_misc, which is used in some assert! macros.

pub mod filter;
pub mod music;
pub mod reader;
pub mod synthesizer;
pub mod wave;
pub mod writer;
