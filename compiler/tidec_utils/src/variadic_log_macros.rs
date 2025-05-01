#[macro_export]
macro_rules! v_debug {
    ($($arg:expr),+ $(,)?) => {
        tracing::debug!("{}", vec![$(format!("{:?}", $arg)),+].join(", "));
    };
}

#[macro_export]
macro_rules! v_trace {
    ($($arg:expr),+ $(,)?) => {
        tracing::trace!("{}", vec![$(format!("{:?}", $arg)),+].join(", "));
    };
}

#[macro_export]
macro_rules! v_info {
    ($($arg:expr),+ $(,)?) => {
        tracing::info!("{}", vec![$(format!("{:?}", $arg)),+].join(", "));
    };
}

#[macro_export]
macro_rules! v_warn {
    ($($arg:expr),+ $(,)?) => {
        tracing::warn!("{}", vec![$(format!("{:?}", $arg)),+].join(", "));
    };
}

#[macro_export]
macro_rules! v_error {
    ($($arg:expr),+ $(,)?) => {
        tracing::error!("{}", vec![$(format!("{:?}", $arg)),+].join(", "));
    };
}
