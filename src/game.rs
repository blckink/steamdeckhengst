use crate::handler::{Handler, install_handler_from_file, scan_handlers};
use crate::paths::*;

use eframe::egui::{self, ImageSource};
use rfd::FileDialog;
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub enum Game {
    Executable { path: PathBuf, filename: String },
    HandlerRef(Handler),
}

impl ToOwned for Game {
    type Owned = Self;

    fn to_owned(&self) -> Self::Owned {
        match self {
            Game::Executable { path, filename } => Game::Executable {
                path: path.clone(),
                filename: filename.clone(),
            },
            Game::HandlerRef(handler) => Game::HandlerRef(handler.clone()),
        }
    }
}
impl Game {
    pub fn name(&self) -> &str {
        match self {
            Game::Executable { filename, .. } => filename,
            Game::HandlerRef(handler) => handler.display(),
        }
    }
    pub fn icon(&self) -> ImageSource {
        match self {
            Game::Executable { .. } => egui::include_image!("../res/executable_icon.png"),
            Game::HandlerRef(handler) => {
                let path = handler.path_handler.join("icon.png");
                if path.exists() {
                    format!("file://{}", path.display()).into()
                } else {
                    egui::include_image!("../.github/assets/sdh.svg")
                }
            }
        }
    }
}

pub fn scan_all_games() -> Vec<Game> {
    let mut games = Vec::new();

    // First, scan handlers
    for handler in scan_handlers() {
        games.push(Game::HandlerRef(handler));
    }

    // Scan executable paths from paths.json
    if let Ok(file) = std::fs::File::open(PATH_PARTY.join("paths.json")) {
        let json: Value = serde_json::from_reader(BufReader::new(file)).unwrap_or_default();

        if let Some(executables) = json[".executables"].as_array() {
            for executable in executables {
                if let Some(path_str) = executable.as_str() {
                    let path = PathBuf::from(path_str);
                    let filename = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("")
                        .to_string();
                    games.push(Game::Executable { path, filename });
                }
            }
        }
    }

    // Sort the games by name
    games.sort_by(|a, b| a.name().to_lowercase().cmp(&b.name().to_lowercase()));

    games
}

pub fn add_game() -> Result<(), Box<dyn Error>> {
    let file = FileDialog::new()
        .set_title("Select Linux/Windows Program or SteamDeckHengst Handler (.pdh)")
        .set_directory(&*PATH_HOME)
        .pick_file();

    if file.is_none() {
        return Ok(());
    }

    // Check if the file has a valid extension (pdh, exe, or no extension)
    let extension = file.as_ref().unwrap().extension().unwrap_or_default();
    if !["pdh", "exe", ""].contains(&extension.to_str().unwrap_or("")) {
        return Err("Invalid file type!".into());
    }

    let file = match file {
        Some(file) => file,
        None => return Ok(()),
    };

    if file.extension().unwrap_or_default() == "pdh" {
        install_handler_from_file(&file)?;
    }

    // Add executable path to the paths.json file
    if file.extension().unwrap_or_default() != "pdh" {
        // Prepare the JSON data - either load existing or create new
        let mut json = if let Ok(file) = File::open(PATH_PARTY.join("paths.json")) {
            serde_json::from_reader(BufReader::new(file))
                .unwrap_or(Value::Object(serde_json::Map::new()))
        } else {
            Value::Object(serde_json::Map::new())
        };

        // Ensure the executables array exists
        if !json.as_object().unwrap().contains_key(".executables") {
            json[".executables"] = serde_json::Value::Array(Vec::new());
        }

        // Add the file path to the executables array
        if let Some(executables) = json[".executables"].as_array_mut() {
            let file_path = file.to_string_lossy().to_string();

            // Only add if not already present
            if !executables.iter().any(|p| p.as_str() == Some(&file_path)) {
                let path_value = serde_json::Value::String(file_path);
                executables.push(path_value);
            }
        }

        // Save the updated paths.json
        let updated_data = serde_json::to_string_pretty(&json)
            .map_err(|e| format!("Failed to serialize paths.json: {}", e))?;

        std::fs::write(PATH_PARTY.join("paths.json"), updated_data)
            .map_err(|e| format!("Failed to write paths.json: {}", e))?;
    }

    Ok(())
}

pub fn remove_game(game: &Game) -> Result<(), Box<dyn Error>> {
    match game {
        Game::Executable { path, .. } => {
            // Load current paths.json
            let mut json = if let Ok(file) = File::open(PATH_PARTY.join("paths.json")) {
                serde_json::from_reader(BufReader::new(file))
                    .unwrap_or(Value::Object(serde_json::Map::new()))
            } else {
                Value::Object(serde_json::Map::new())
            };

            // Remove the file path from the executables array
            if let Some(executables) = json[".executables"].as_array_mut() {
                let file_path = path.to_string_lossy().to_string();
                executables.retain(|p| p.as_str() != Some(&file_path));
            }

            // Save the updated paths.json
            let updated_data = serde_json::to_string_pretty(&json)?;

            std::fs::write(PATH_PARTY.join("paths.json"), updated_data)?;
        }

        Game::HandlerRef(h) => {
            std::fs::remove_dir_all(h.path_handler.clone())?;
        }
    }

    Ok(())
}
