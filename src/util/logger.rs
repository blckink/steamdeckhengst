use crate::paths::PATH_PARTY;
use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;

pub fn log_info(msg: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(PATH_PARTY.join("steamdeckhengst.log"))
    {
        let _ = writeln!(
            file,
            "[{}][INFO] {}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            msg
        );
    }
}

pub fn log_error(msg: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(PATH_PARTY.join("steamdeckhengst.log"))
    {
        let _ = writeln!(
            file,
            "[{}][ERROR] {}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            msg
        );
    }
}
