use dialog::{Choice, DialogBox};
use std::error::Error;
use std::path::PathBuf;
use x11rb::connection::Connection;

use crate::util::{log_error, log_info};

pub fn msg(title: &str, contents: &str) {
    let _ = dialog::Message::new(contents).title(title).show();
}

pub fn yesno(title: &str, contents: &str) -> bool {
    if let Ok(prompt) = dialog::Question::new(contents).title(title).show() {
        if prompt == Choice::Yes {
            return true;
        }
    }
    false
}

pub fn get_screen_resolution() -> (u32, u32) {
    if let Ok(conn) = x11rb::connect(None) {
        let screen = &conn.0.setup().roots[0];
        log_info(&format!(
            "Got screen resolution: {}x{}",
            screen.width_in_pixels, screen.height_in_pixels
        ));
        return (
            screen.width_in_pixels as u32,
            screen.height_in_pixels as u32,
        );
    }
    // Fallback to a common resolution if detection fails
    log_error("Failed to detect screen resolution, using fallback 1920x1080");
    (1920, 1080)
}

// Gets the resolution for a specific instance based on the number of instances
pub fn get_instance_resolution(
    playercount: usize,
    i: usize,
    basewidth: u32,
    baseheight: u32,
    two_player_vertical: bool,
) -> (u32, u32) {
    let (w, h) = match playercount {
        1 => (basewidth, baseheight),
        2 => {
            if two_player_vertical {
                (basewidth, baseheight / 2)
            } else {
                (basewidth / 2, baseheight)
            }
        }
        3 => {
            if i == 0 {
                (basewidth, baseheight / 2)
            } else {
                (basewidth / 2, baseheight / 2)
            }
        }
        4 => (basewidth / 2, baseheight / 2),
        // 5 => {
        //     if i < 2 {
        //         (basewidth / 2, baseheight / 2)
        //     } else {
        //         (basewidth / 3, baseheight / 2)
        //     }
        // }
        // 6 => (basewidth / 3, baseheight / 2),
        // 7 => {
        //     if i < 2 || i > 4 {
        //         (basewidth / 2, baseheight / 3)
        //     } else {
        //         (basewidth / 3, baseheight / 3)
        //     }
        // }
        // 8 => (basewidth / 2, baseheight / 4),
        _ => (basewidth, baseheight),
    };
    log_info(&format!(
        "Resolution for instance {}/{playercount}: {w}x{h}",
        i + 1
    ));
    return (w, h);
}

// Sends the splitscreen script to the active KWin session through DBus
pub fn kwin_dbus_start_script(file: PathBuf) -> Result<(), Box<dyn Error>> {
    log_info(&format!("Loading script {}...", file.display()));
    if !file.exists() {
        log_error("Script file doesn't exist!");
        return Err("Script file doesn't exist!".into());
    }

    let conn = zbus::blocking::Connection::session()?;
    let proxy = zbus::blocking::Proxy::new(
        &conn,
        "org.kde.KWin",
        "/Scripting",
        "org.kde.kwin.Scripting",
    )?;

    let _: i32 = proxy.call("loadScript", &(file.to_string_lossy(), "splitscreen"))?;
    log_info("Script loaded. Starting...");
    let _: () = proxy.call("start", &())?;

    log_info("KWin script started.");
    Ok(())
}

pub fn kwin_dbus_unload_script() -> Result<(), Box<dyn Error>> {
    log_info("Unloading splitscreen script...");
    let conn = zbus::blocking::Connection::session()?;
    let proxy = zbus::blocking::Proxy::new(
        &conn,
        "org.kde.KWin",
        "/Scripting",
        "org.kde.kwin.Scripting",
    )?;

    let _: bool = proxy.call("unloadScript", &("splitscreen"))?;

    log_info("Script unloaded.");
    Ok(())
}
