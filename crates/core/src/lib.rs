#![doc = include_str!("../README.md")]
#![forbid(unsafe_code, missing_docs, clippy::missing_panics_doc)]

mod context;
mod shader;
mod texture;

pub mod dynamic;
#[cfg(feature = "trace")]
pub mod trace;

pub use context::*;
pub use shader::*;
pub use texture::*;
