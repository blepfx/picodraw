use super::BUFFER_ALIGNMENT;
use glow::{HasContext, MAX_TEXTURE_BUFFER_SIZE, MAX_TEXTURE_IMAGE_UNITS, MAX_TEXTURE_SIZE, MAX_UNIFORM_BLOCK_SIZE};
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
    pub max_uniform_block_size_bytes: u32,
    pub max_texture_buffer_size_texels: u32,
}

impl GlInfo {
    pub fn query(gl: &impl HasContext) -> Self {
        unsafe {
            let version = gl.version();
            let max_texture_size = gl.get_parameter_i32(MAX_TEXTURE_SIZE as _) as u32;
            let max_texture_units = gl.get_parameter_i32(MAX_TEXTURE_IMAGE_UNITS as _) as u32;
            let max_uniform_block_size = gl.get_parameter_i32(MAX_UNIFORM_BLOCK_SIZE as _) as u32;
            let max_texture_buffer_size = gl.get_parameter_i32(MAX_TEXTURE_BUFFER_SIZE as _) as u32;

            Self {
                version: (version.major, version.minor),
                is_gles: version.is_embedded,

                vendor: version.vendor_info.clone(),
                extensions: gl.supported_extensions().clone(),

                max_texture_size,
                max_texture_units,
                max_uniform_block_size_bytes: max_uniform_block_size,
                max_texture_buffer_size_texels: max_texture_buffer_size,
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

    pub(crate) fn is_baseline_supported(&self) -> bool {
        if self.is_gles {
            let baseline = self.version >= (2, 0);
            let any_buffer = self.is_uniform_buffer_supported() || self.is_texture_buffer_supported();

            baseline && any_buffer
        } else {
            let baseline = self.version >= (2, 0);
            let framebuffer = self.extensions.contains("GL_ARB_framebuffer_object") || self.version >= (3, 0);
            let any_buffer = self.is_uniform_buffer_supported() || self.is_texture_buffer_supported();

            baseline && framebuffer && any_buffer
        }
    }

    pub(crate) fn is_uniform_buffer_supported(&self) -> bool {
        if self.is_gles {
            self.version >= (3, 0) || self.extensions.contains("GL_ARB_uniform_buffer_object")
        } else {
            self.version >= (3, 1) || self.extensions.contains("GL_ARB_uniform_buffer_object")
        }
    }

    pub(crate) fn is_texture_buffer_supported(&self) -> bool {
        let tbo = self.extensions.contains("GL_ARB_texture_buffer_object") || (self.version >= (3, 1) && !self.is_gles);
        let bit = self.extensions.contains("GL_ARB_shader_bit_encoding") || (self.version >= (3, 3) && !self.is_gles);
        tbo && bit
    }

    pub(crate) fn is_timer_query_supported(&self) -> bool {
        self.extensions.contains("GL_ARB_timer_query") || (self.version >= (3, 3) && !self.is_gles)
    }

    pub(crate) fn prefer_tbo_over_ubo(&self) -> bool {
        if !self.is_uniform_buffer_supported() {
            return true;
        }

        self.is_texture_buffer_supported()
            && self.target_tbo_size() > self.target_ubo_size()
            && self.max_texture_units > 8
    }

    pub(crate) fn target_ubo_size(&self) -> u32 {
        let target = self.max_uniform_block_size_bytes.min(65536);
        target - target % BUFFER_ALIGNMENT // align to 16 bytes
    }

    pub(crate) fn target_tbo_size(&self) -> u32 {
        let target = (self.max_texture_buffer_size_texels * BUFFER_ALIGNMENT).min(2097152);
        target - target % BUFFER_ALIGNMENT // align to 16 bytes
    }
}
