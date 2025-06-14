mod app;
mod game;
mod handler;
mod image_cache;
mod input;
mod launch;
mod paths;
mod ui;
mod util;

use crate::paths::*;
use crate::util::*;

fn main() -> iced::Result {
    std::fs::create_dir_all(PATH_PARTY.join("gamesyms"))
        .expect("Failed to create gamesyms directory");
    std::fs::create_dir_all(PATH_PARTY.join("handlers"))
        .expect("Failed to create handlers directory");
    std::fs::create_dir_all(PATH_PARTY.join("profiles"))
        .expect("Failed to create profiles directory");

    remove_guest_profiles().unwrap();

    if PATH_PARTY.join("tmp").exists() {
        std::fs::remove_dir_all(PATH_PARTY.join("tmp")).unwrap();
    }
    if !PATH_RES.join("umu-run").exists() {
        msg(
            "Downloading Dependencies",
            "UMU Launcher not found in resources folder. PartyDeck uses UMU to launch Windows games with Proton. Click OK to automatically download from the internet.",
        );
        if let Err(e) = update_umu_launcher() {
            println!("Failed to download UMU Launcher: {}", e);
            msg("Error", &format!("Failed to download UMU Launcher: {}", e));
            std::fs::remove_file(PATH_RES.join("umu-run")).unwrap();
            return Ok(());
        }
    }
    if !PATH_RES.join("goldberg_linux").exists() || !PATH_RES.join("goldberg_win").exists() {
        msg(
            "Downloading Dependencies",
            "Goldberg Steam Emu not found in resources folder. PartyDeck uses Goldberg for LAN play. Click OK to automatically download from the internet.",
        );
        if let Err(e) = update_goldberg_emu() {
            println!("Failed to download Goldberg: {}", e);
            msg("Error", &format!("Failed to download Goldberg: {}", e));
            std::fs::remove_dir_all(PATH_PARTY.join("goldberg_linux")).unwrap();
            std::fs::remove_dir_all(PATH_PARTY.join("goldberg_win")).unwrap();
            return Ok(());
        }
    }

    println!("\n[PARTYDECK] started\n");

    let _fullscreen = std::env::args().any(|arg| arg == "--fullscreen");
    let _ = get_screen_resolution();
    ui::run()
}
