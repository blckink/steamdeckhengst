// Re-export all utility functions from submodules
mod filesystem;
mod logger;
mod profiles;
mod sys;
mod updates;

// Re-export functions from profiles
pub use profiles::{create_gamesave, create_profile, remove_guest_profiles, scan_profiles};

// Re-export functions from filesystem
pub use filesystem::{SanitizePath, copy_dir_recursive, get_rootpath, get_rootpath_handler};

// Re-export functions from launcher
pub use sys::{
    get_instance_resolution, get_screen_resolution, kwin_dbus_start_script,
    kwin_dbus_unload_script, msg, yesno,
};

pub use logger::{log_error, log_info};

// Re-export functions from updates
pub use updates::{check_for_steamdeckhengst_update, update_goldberg_emu, update_umu_launcher};
