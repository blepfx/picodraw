use crate::{DebugCallback, DebugMessage};
use glow::{BLEND, DEBUG_OUTPUT, DEBUG_OUTPUT_SYNCHRONOUS, HasContext, ONE, ONE_MINUS_SRC_ALPHA, SRC_ALPHA};

pub fn viewport(gl: &impl HasContext, x: i32, y: i32, w: u32, h: u32) {
    unsafe {
        gl.viewport(x as _, y as _, w as _, h as _);
        gl.scissor(x as _, y as _, w as _, h as _);
    }
}

pub fn enable_blend_normal(gl: &impl HasContext) {
    unsafe {
        gl.enable(BLEND);
        gl.blend_func_separate(SRC_ALPHA, ONE_MINUS_SRC_ALPHA, ONE, ONE_MINUS_SRC_ALPHA);
    }
}

pub fn enable_debug(gl: &mut impl HasContext, logger: DebugCallback) {
    unsafe {
        gl.enable(DEBUG_OUTPUT);
        gl.enable(DEBUG_OUTPUT_SYNCHRONOUS);
        gl.debug_message_callback(move |_, type_, _, _, message| {
            let type_ = match type_ {
                glow::DEBUG_TYPE_ERROR => DebugMessage::Error,
                glow::DEBUG_TYPE_DEPRECATED_BEHAVIOR => DebugMessage::Deprecated,
                glow::DEBUG_TYPE_UNDEFINED_BEHAVIOR => DebugMessage::UndefinedBehavior,
                glow::DEBUG_TYPE_PORTABILITY => DebugMessage::Portability,
                glow::DEBUG_TYPE_PERFORMANCE => DebugMessage::Performance,
                glow::DEBUG_TYPE_MARKER => DebugMessage::Marker,
                _ => DebugMessage::Other,
            };

            logger(type_, message);
        });
    }
}
