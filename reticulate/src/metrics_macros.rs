#[macro_export]
macro_rules! start_timer {
    ($name:ident, as = $guard:ident) => {
        #[cfg(feature = "metrics-recording")]
        let timer = $crate::metrics::$name();
        #[cfg(feature = "metrics-recording")]
        let $guard = timer.start();
    };
    ($name:ident) => {
        #[cfg(feature = "metrics-recording")]
        let timer = $crate::metrics::$name();
        #[cfg(feature = "metrics-recording")]
        let _guard = timer.start();
    };
}

#[macro_export]
macro_rules! stop_timer {
    ($guard:ident) => {
        #[cfg(feature = "metrics-recording")]
        $guard.stop();
    };
}

#[macro_export]
macro_rules! increase_counter {
    ($name:ident) => {
        #[cfg(feature = "metrics-recording")]
        $crate::metrics::$name().inc();
    };
}
