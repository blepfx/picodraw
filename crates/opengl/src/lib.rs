mod backend;
mod compiler;
mod dispatch;
mod opengl;

pub use crate::backend::*;
pub use crate::opengl::GlInfo as OpenGlInfo;

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

#[derive(Debug, Clone)]
pub enum InitError {
    InvalidContext,
    UnsupportedVersion { info: OpenGlInfo },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DebugMessage {
    Error,
    Deprecated,
    UndefinedBehavior,
    Performance,
    Portability,
    Marker,
    Other,
}

pub type DebugCallback = Box<dyn Fn(DebugMessage, &str) + Send + Sync>;
