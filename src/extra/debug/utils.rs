#[macro_export]
macro_rules! debug_info {
    ($($arg:tt)*) => {
        #[cfg(feature = "debug")]
        {
            info!($($arg)*);
        }
    };
}
