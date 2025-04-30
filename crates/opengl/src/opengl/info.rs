use super::BUFFER_ALIGNMENT;
use glow::{HasContext, MAX_TEXTURE_IMAGE_UNITS, MAX_TEXTURE_SIZE, MAX_UNIFORM_BLOCK_SIZE};
use std::collections::HashSet;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct GlInfo {
    pub version: (u32, u32),
    pub is_gles: bool,

    pub vendor: String,
    pub extensions: HashSet<String>,

    pub max_texture_size: u32,
    pub max_texture_units: u32,
    pub max_uniform_block_size: u32,
}

impl GlInfo {
    pub fn query(gl: &impl HasContext) -> Self {
        unsafe {
            let version = gl.version();
            let max_texture_size = gl.get_parameter_i32(MAX_TEXTURE_SIZE as _) as u32;
            let max_texture_units = gl.get_parameter_i32(MAX_TEXTURE_IMAGE_UNITS as _) as u32;
            let max_uniform_block_size = gl.get_parameter_i32(MAX_UNIFORM_BLOCK_SIZE as _) as u32;

            Self {
                version: (version.major, version.minor),
                is_gles: version.is_embedded,

                vendor: version.vendor_info.clone(),
                extensions: gl.supported_extensions().clone(),

                max_texture_size,
                max_texture_units,
                max_uniform_block_size,
            }
        }
    }

    pub fn glsl_version(&self) -> u32 {
        if self.is_gles && self.version >= (3, 0) {
            330
        } else if self.is_gles && self.version >= (2, 0) {
            120
        } else if self.is_gles {
            100
        } else if self.version >= (3, 3) {
            (self.version.0 * 100 + self.version.1 * 10) as u32
        } else if self.version >= (3, 2) {
            150
        } else if self.version >= (3, 1) {
            140
        } else if self.version >= (3, 0) {
            130
        } else if self.version >= (2, 1) {
            120
        } else if self.version >= (2, 0) {
            110
        } else {
            100
        }
    }

    pub fn is_baseline_supported(&self) -> bool {
        self.version >= (3, 1) || self.extensions.contains("GL_ARB_uniform_buffer_object")
    }

    pub fn is_timer_query_supported(&self) -> bool {
        self.extensions.contains("GL_ARB_timer_query") || self.version >= (3, 3)
    }

    pub fn target_ubo_size(&self) -> u32 {
        let target = self.max_uniform_block_size.min(262144);
        target - target % BUFFER_ALIGNMENT // align to 16 bytes
    }
}
