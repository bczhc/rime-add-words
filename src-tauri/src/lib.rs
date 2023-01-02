#[macro_export]
macro_rules! mutex_lock {
    ($e:expr) => {
        $e.lock().unwrap()
    };
}
