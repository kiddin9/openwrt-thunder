mod home_dir_ne_windows {
    use std::{env::var_os, path::PathBuf};

    const HOME: &str = "HOME";

    /// Return the user's home directory.
    ///
    /// ```
    /// //  "/home/USER"
    /// let path = simple_home_dir::home_dir().unwrap();
    /// ```
    pub fn home_dir() -> Option<PathBuf> {
        var_os(HOME).map(Into::into)
    }
}

pub use home_dir_ne_windows::*;
