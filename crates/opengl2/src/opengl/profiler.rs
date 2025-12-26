use glow::{HasContext, QUERY_RESULT, QUERY_RESULT_AVAILABLE, TIME_ELAPSED};
use std::cell::Cell;

pub struct GlProfiler<T: HasContext> {
    query: Option<T::Query>,
    last: Cell<Option<u32>>,
    check: Cell<bool>,
}

impl<T: HasContext> GlProfiler<T> {
    pub fn new(gl: &T) -> Self {
        unsafe {
            Self {
                query: gl.create_query().ok(),
                last: Cell::new(None),
                check: Cell::new(true),
            }
        }
    }

    pub fn dummy() -> Self {
        Self {
            query: None,
            last: Cell::new(None),
            check: Cell::new(true),
        }
    }

    pub fn query(&self) -> Option<u32> {
        self.last.get()
    }

    pub fn begin(&self, gl: &T) {
        if let Some(query) = self.query.as_ref() {
            if self.check.get() {
                unsafe {
                    gl.begin_query(TIME_ELAPSED, *query);
                }
            }
        }
    }

    pub fn end(&self, gl: &T) {
        if let Some(query) = self.query.as_ref() {
            unsafe {
                if self.check.replace(false) {
                    gl.end_query(TIME_ELAPSED);
                }

                let available = gl.get_query_parameter_u32(*query, QUERY_RESULT_AVAILABLE);
                if available != 0 {
                    let result = gl.get_query_parameter_u32(*query, QUERY_RESULT);
                    self.last.set(Some(result));
                    self.check.set(true);
                }
            }
        }
    }

    pub fn delete(self, gl: &T) {
        if let Some(query) = self.query {
            unsafe {
                gl.delete_query(query);
            }
        }
    }
}
