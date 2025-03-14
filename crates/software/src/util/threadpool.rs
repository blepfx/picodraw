use std::{
    ptr::NonNull,
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicBool, Ordering},
    },
    thread::{Thread, available_parallelism, spawn},
};

pub struct ThreadPool {
    inner: Arc<Inner>,
    threads: Vec<Thread>,
}

impl ThreadPool {
    pub fn new() -> Self {
        Self::with_threads(available_parallelism().map(|x| x.get()).unwrap_or(1))
    }

    pub fn with_threads(threads: usize) -> Self {
        let inner = Arc::new(Inner::new());

        Self {
            threads: (0..threads.max(1) - 1)
                .map(|thread_idx| {
                    let inner = inner.clone();
                    spawn(move || {
                        while !inner.is_closed() {
                            match inner.pop_job() {
                                Some(job) => inner.invoke_runner(job, thread_idx + 1),
                                None => std::thread::park(),
                            }
                        }
                    })
                    .thread()
                    .clone()
                })
                .collect(),
            inner,
        }
    }

    pub fn num_threads(&self) -> usize {
        1 + self.threads.len()
    }

    pub fn execute<'a, T: 'a + Send>(
        &mut self,
        jobs: impl IntoIterator<Item = &'a T>,
        func: impl Fn(&'a T, usize) + Send + Sync,
    ) {
        let job_runner = |job: *const (), i: usize| {
            func(unsafe { &*(job as *const T) }, i);
        };

        self.inner.with_runner(&job_runner, || {
            let num_jobs = self
                .inner
                .push_jobs(jobs.into_iter().map(|job| job as *const _ as *const _));

            self.threads
                .iter()
                .take(num_jobs.max(1) - 1)
                .for_each(|t| t.unpark());

            while let Some(job) = self.inner.pop_job() {
                self.inner.invoke_runner(job, 0);
            }
        });
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.inner.close();
        self.threads.iter().for_each(|t| t.unpark());
    }
}

unsafe impl Send for Inner {}
unsafe impl Sync for Inner {}

struct Inner {
    closed: AtomicBool,
    job_list: Mutex<Vec<*const ()>>,
    job_runner: RwLock<Option<NonNull<dyn Fn(*const (), usize) + Send + Sync>>>,
}

impl Inner {
    fn new() -> Self {
        Self {
            closed: AtomicBool::new(false),
            job_list: Mutex::new(Vec::new()),
            job_runner: RwLock::new(None),
        }
    }

    fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }

    fn close(&self) {
        self.closed.store(true, Ordering::Release);
    }

    fn pop_job(&self) -> Option<*const ()> {
        let mut stack = self.job_list.lock().unwrap();
        stack.pop()
    }

    fn push_jobs(&self, jobs: impl IntoIterator<Item = *const ()>) -> usize {
        let mut stack = self.job_list.lock().unwrap();
        stack.extend(jobs);
        stack.len()
    }

    fn invoke_runner(&self, job: *const (), thread: usize) {
        let task = self.job_runner.read().unwrap();
        if let Some(task) = *task {
            unsafe {
                task.as_ref()(job, thread);
            }
        }
    }

    fn with_runner(&self, runner: &(dyn Fn(*const (), usize) + Send + Sync), f: impl FnOnce()) {
        {
            let mut task = self.job_runner.write().unwrap();
            *task = Some(NonNull::new(runner as *const _ as *mut _).unwrap());
        }

        f();

        {
            let mut task = self.job_runner.write().unwrap();
            *task = None;
        }
    }
}

//TODO: miri test
#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::atomic::AtomicUsize, thread, time::Duration};

    #[test]
    fn test_thread_pool() {
        let mut pool = ThreadPool::new();
        let counter = AtomicUsize::new(0);

        let data = (1..=1000).collect::<Vec<_>>();
        pool.execute(&data, |x, _| {
            thread::sleep(Duration::from_millis(15));
            counter.fetch_add(*x, Ordering::Relaxed);
        });

        assert_eq!(counter.load(Ordering::Relaxed), 500500);
    }
}
