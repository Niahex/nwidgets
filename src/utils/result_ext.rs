use std::fmt;

/// Extension trait for Result to log errors
pub trait ResultExt<T> {
    fn log_err(self) -> Option<T>;
}

impl<T, E: fmt::Display> ResultExt<T> for Result<T, E> {
    fn log_err(self) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(e) => {
                log::error!("{}", e);
                None
            }
        }
    }
}

/// Extension trait for Option to log None cases
pub trait OptionExt<T> {
    fn log_none(self, msg: &str) -> Option<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn log_none(self, msg: &str) -> Option<T> {
        if self.is_none() {
            log::warn!("{}", msg);
        }
        self
    }
}
