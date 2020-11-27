pub struct ThreadPool(rayon::ThreadPool);

impl ThreadPool {
    pub fn new() -> ThreadPool {
        rayon::ThreadPoolBuilder::new()
            .num_threads(3)
            .thread_name(name)
            .build()
            .map(ThreadPool)
            .unwrap()
    }

    pub fn spawn(&self, f: impl FnOnce() + Send + 'static) {
        self.0.spawn(f)
    }
}

fn name(index: usize) -> String {
    format!("API fetch thread {}", index + 1)
}
