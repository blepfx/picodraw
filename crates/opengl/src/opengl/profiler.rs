use glow::{HasContext, QUERY_RESULT, QUERY_RESULT_AVAILABLE, TIME_ELAPSED};
use std::cell::Cell;

pub struct GlProfiler<T: HasContext> {
    query: Option<T::Query>,
    last: Cell<u32>,
    check: Cell<bool>,
}

impl<T: HasContext> GlProfiler<T> {
    pub fn new(gl: &T) -> Self {
        unsafe {
            Self {
                query: gl.create_query().ok(),
                last: Cell::new(0),
                check: Cell::new(true),
            }
        }
    }

    pub fn dummy() -> Self {
        Self {
            query: None,
            last: Cell::new(0),
            check: Cell::new(true),
        }
    }

    pub fn query(&self) -> u32 {
        self.last.get()
    }

    pub fn wrap(&self, gl: &T, c: impl FnOnce()) {
        match self.query.as_ref() {
            Some(query) => unsafe {
                if self.check.replace(false) {
                    gl.begin_query(TIME_ELAPSED, *query);
                    c();
                    gl.end_query(TIME_ELAPSED);
                } else {
                    c();
                }

                let available = gl.get_query_parameter_u32(*query, QUERY_RESULT_AVAILABLE);
                if available != 0 {
                    let result = gl.get_query_parameter_u32(*query, QUERY_RESULT);
                    self.last.set(result);
                    self.check.set(true);
                }
            },
            None => c(),
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
