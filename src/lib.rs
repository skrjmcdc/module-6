pub struct ThreadPool;

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        Self
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
    }
}
