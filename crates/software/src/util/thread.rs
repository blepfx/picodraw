use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    ptr::null_mut,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering},
    },
    thread::{Thread, current, park, spawn},
};

pub struct ThreadPool {
    workers: Vec<Worker>,
}

impl ThreadPool {
    pub fn with_threads(workers: usize) -> Self {
        Self {
            workers: (1..workers).map(Worker::new).collect(),
        }
    }

    pub fn num_workers(&self) -> usize {
        1 + self.workers.len()
    }

    pub fn run_indexed(&mut self, jobs: usize, run: impl Fn(usize, usize) + Send + Sync) {
        let mut scope = Scope::new(&run, jobs);

        unsafe {
            for worker in self.workers.iter_mut() {
                worker.fork(&scope);
            }

            scope.run(0);

            for worker in self.workers.iter_mut() {
                worker.join();
            }
        }

        scope.consume_panic();
    }

    pub fn run_arrays<'a, Worker: 'a + Send + Sync, Job: 'a + Send + Sync>(
        &mut self,
        workers: &'a mut [Worker],
        jobs: &'a [Job],
        run: impl Fn(&'a mut Worker, &'a Job) + Send + Sync,
    ) {
        #[derive(Clone, Copy)]
        struct WorkerList<T>(*mut T);
        unsafe impl<T> Send for WorkerList<T> {}
        unsafe impl<T> Sync for WorkerList<T> {}
        impl<T> WorkerList<T> {
            #[allow(clippy::mut_from_ref)]
            unsafe fn get<'a>(&self, index: usize) -> &'a mut T {
                unsafe { &mut *self.0.add(index) }
            }
        }

        assert_eq!(workers.len(), self.num_workers());

        let workers = WorkerList(workers.as_mut_ptr());
        self.run_indexed(jobs.len(), move |worker, job| {
            run(unsafe { workers.get(worker) }, &jobs[job]);
        });
    }
}

#[repr(align(64))]
struct Scope<'a> {
    coordinator: Thread,
    panic: Mutex<Option<Box<dyn Any + Send>>>,

    runner: &'a (dyn Fn(usize, usize) + Send + Sync),
    count: AtomicUsize,
    total: usize,
}

#[repr(align(128))]
struct WorkerData {
    closed: AtomicBool,
    scope: AtomicPtr<()>,
}

struct Worker {
    thread: Thread,
    worker: Arc<WorkerData>,
}

impl<'a> Scope<'a> {
    fn new(runner: &'a (dyn Fn(usize, usize) + Send + Sync), jobs: usize) -> Self {
        Self {
            coordinator: current(),
            panic: Mutex::new(None),
            count: AtomicUsize::new(0),
            total: jobs,
            runner,
        }
    }

    #[inline]
    unsafe fn run(&self, thread: usize) {
        loop {
            let task = self.count.fetch_add(1, Ordering::Relaxed);
            if task >= self.total {
                return;
            }

            let result = catch_unwind(AssertUnwindSafe(|| {
                (self.runner)(thread, task);
            }));

            if let Err(err) = result {
                *self.panic.lock().unwrap() = Some(err);
            }
        }
    }

    fn consume_panic(&mut self) {
        if let Some(err) = self.panic.get_mut().unwrap().take() {
            resume_unwind(err);
        }
    }
}

impl Worker {
    fn new(index: usize) -> Self {
        let worker = Arc::new(WorkerData {
            closed: AtomicBool::new(false),
            scope: AtomicPtr::new(null_mut()),
        });

        let thread = {
            let worker = worker.clone();
            spawn(move || {
                loop {
                    park();

                    let scope = worker.scope.load(Ordering::Acquire);
                    if !scope.is_null() {
                        unsafe {
                            let scope = &*(scope as *mut Scope);
                            let coordinator = scope.coordinator.clone();
                            scope.run(index);

                            // scope might be dropped after we set scope to null, do NOT use it
                            // that is the reason we clone `coordinator` instead of using it via the `scope` reference

                            worker.scope.store(null_mut(), Ordering::Release);
                            coordinator.unpark();
                        }
                    }

                    if worker.closed.load(Ordering::Acquire) {
                        return;
                    }
                }
            })
            .thread()
            .clone()
        };

        Self { worker, thread }
    }

    /// SAFETY:
    /// - `scope` should be alive until `is_running` returns false
    /// - thread index should be unique per scope      
    unsafe fn fork(&mut self, scope: &Scope) {
        self.worker.scope.store(scope as *const _ as *mut (), Ordering::Release);
        self.thread.unpark();
    }

    /// SAFETY: should only be run as the coordinator thread passed to the last call to `fork`
    unsafe fn join(&mut self) {
        while !self.worker.scope.load(Ordering::Acquire).is_null() {
            park();
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.worker.closed.store(true, Ordering::Release);
        self.thread.unpark();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn run_indexed() {
        const ITERS: usize = if cfg!(miri) { 10 } else { 10000 };

        let mut pool = ThreadPool::with_threads(8);
        for _ in 0..ITERS {
            let counter = AtomicUsize::new(0);

            let data = (1..=10000).collect::<Vec<_>>();
            pool.run_indexed(data.len(), |_, i| {
                counter.fetch_add(data[i], Ordering::Relaxed);
            });

            assert_eq!(counter.load(Ordering::Relaxed), 50005000);
        }
    }

    #[test]
    fn run_arrays() {
        const ITERS: usize = if cfg!(miri) { 10 } else { 10000 };

        let mut pool = ThreadPool::with_threads(8);
        for i in 0..ITERS {
            let data = (1..=10000).collect::<Vec<_>>();
            let mut workers = (0..pool.num_workers()).map(|_| 0usize).collect::<Vec<_>>();

            pool.run_arrays(&mut workers[..], &data[..], |worker, data| {
                //counter.fetch_add(*data, Ordering::Relaxed);
                *worker += data;
            });

            assert_eq!(
                workers.iter().copied().sum::<usize>(),
                50005000,
                "i={} workers={:?}",
                i,
                workers
            );
        }
    }
}
