#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod backend;
mod compiler;
mod dispatch;
mod opengl;

pub use crate::backend::*;

/// Draw call statistics.
#[derive(Debug, Clone, Default)]
pub struct Stats {
    /// Total GPU render time of one of the previous draw calls.
    /// Does not necessarily correspond to the time of the last draw call (there is a small delay due to the asynchronous nature of GPUs).
    pub gpu_time: Option<std::time::Duration>,

    /// Total CPU time spent building and issuing draw calls for the last call to [`Context::draw`](picodraw_core::Context::draw).
    /// Includes time spent within the `draw` closure.
    pub cpu_time: Option<std::time::Duration>,

    /// Number of GPU draw calls/context switches
    pub draw_calls: u32,

    /// Total number of bytes sent to the GPU, including quad lists and quad data
    pub bytes_sent: u64,

    /// Number of quads sent to the GPU
    pub total_quads: u32,

    /// Number of objects sent to the GPU
    pub total_objects: u32,

    /// Number of pixels written
    pub total_pixels: u64,
}

/// An error that occurred during the creation of an OpenGL `picodraw` context.
#[derive(Debug, Clone)]
pub enum InitError {
    /// The OpenGL thread-context is invalid or could not be used.
    InvalidContext,

    /// The detected OpenGL version is not supported.
    ///
    /// See [`Backend::new`] for more details on supported versions.
    UnsupportedVersion {
        /// OpenGL version information.
        version: (u32, u32),

        /// Whether the context is OpenGL ES.
        is_gles: bool,

        /// List of supported extensions.
        extensions: Vec<String>,
    },
}

/// Configuration options for the OpenGL backend.
pub struct Config {
    /// A callback function for OpenGL debug messages. Useful for logging.
    ///
    /// Enables debug output if set, might have a performance impact.
    pub debug_logger: Option<DebugCallback>,

    /// Enable GPU time queries.
    ///
    /// Without this option, GPU time measurements will not be available.
    ///
    /// Might have a small performance impact.
    pub enable_gpu_time_queries: bool,

    /// Prefer using Uniform Buffer Objects (UBOs) over Texture Buffer Objects (TBOs) for uploading shader data.
    pub prefer_ubo_over_tbo: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            debug_logger: None,
            enable_gpu_time_queries: true,
            prefer_ubo_over_tbo: false,
        }
    }
}

/// The type of OpenGL debug message sent to the debug callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DebugMessage {
    /// An error message.
    Error,

    /// Deprecated functionality was used.
    Deprecated,

    /// Undefined behavior was detected.
    UndefinedBehavior,

    /// Performance issues were detected.   
    Performance,

    /// A portability issue was detected.
    Portability,

    /// Marker?
    Marker,

    ///  Other/unknown message.
    Other,
}

/// A callback function for OpenGL debug messages.
pub type DebugCallback = Box<dyn Fn(DebugMessage, &str) + Send + Sync>;
