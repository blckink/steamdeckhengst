use crate::app::config::*;
use crate::game::{Game::*, *};
use crate::handler::*;
use crate::input::*;
use crate::launch::{launch_executable, launch_from_handler};
use crate::paths::*;
use crate::util::*;
use crate::task::Task;

use dialog::DialogBox;
use eframe::egui::{self, Key, Ui};
use std::path::PathBuf;

#[derive(Eq, PartialEq)]
pub enum MenuPage {
    Main,
    Settings,
    Profiles,
    Game,
    Players,
}

pub struct PartyApp {
    pub needs_update: bool,
    pub update_check: Option<Task<bool>>,
    pub options: PartyConfig,
    pub cur_page: MenuPage,
    pub infotext: String,
    pub pads: Vec<Gamepad>,
    pub players: Vec<Player>,
    pub games: Vec<Game>,
    pub profiles: Vec<String>,
    pub selected_game: usize,
}

macro_rules! cur_game {
    ($self:expr) => {
        &$self.games[$self.selected_game]
    };
}

impl Default for PartyApp {
    fn default() -> Self {
        Self {
            needs_update: false,
            update_check: Some(Task::spawn(|| check_for_partydeck_update().unwrap_or(false))),
            options: load_cfg(),
            cur_page: MenuPage::Main,
            infotext: String::new(),
            pads: scan_evdev_gamepads(),
            players: Vec::new(),
            games: scan_all_games(),
            profiles: Vec::new(),
            selected_game: 0,
        }
    }
}

impl eframe::App for PartyApp {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        match self.cur_page {
            MenuPage::Players => self.handle_gamepad_players(),
            _ => self.handle_gamepad_gui(raw_input),
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(task) = &self.update_check {
            if let Some(res) = task.try_join() {
                self.needs_update = res;
                self.update_check = None;
            }
        }
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.display_top_panel(ui);
        });
        egui::SidePanel::left("games_panel")
            .resizable(false)
            .exact_width(200.0)
            .show(ctx, |ui| {
                self.display_games_panel(ui);
            });
        if (self.cur_page != MenuPage::Main) && (self.cur_page != MenuPage::Players) {
            self.display_info_panel(ctx);
        }
        egui::CentralPanel::default().show(ctx, |ui| match self.cur_page {
            MenuPage::Main => {
                self.display_page_main(ui);
            }
            MenuPage::Settings => {
                self.display_page_settings(ui);
            }
            MenuPage::Profiles => {
                self.display_page_profiles(ui);
            }
            MenuPage::Game => {
                self.display_page_game(ui);
            }
            MenuPage::Players => {
                self.display_page_players(ui);
            }
        });
        ctx.request_repaint_after(std::time::Duration::from_millis(33)); // 30 fps
    }
}

impl PartyApp {
    fn display_top_panel(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui
                .add(egui::Button::image_and_text(
                    egui::include_image!("../../res/BTN_NORTH.png"),
                    "⛭",
                ))
                .clicked()
            {
                self.cur_page = MenuPage::Settings;
            }
            if ui
                .add(egui::Button::image_and_text(
                    egui::include_image!("../../res/BTN_WEST.png"),
                    "👥",
                ))
                .clicked()
            {
                self.profiles = scan_profiles(false);
                self.cur_page = MenuPage::Profiles;
            }
            if ui
                .add(egui::Button::image_and_text(
                    egui::include_image!("../../res/BTN_EAST.png"),
                    "🏠",
                ))
                .clicked()
            {
                self.cur_page = MenuPage::Main;
            }
            if ui.button("🎮 Rescan").clicked() {
                self.players.clear();
                self.pads.clear();
                self.pads = scan_evdev_gamepads();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌ Quit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                let version_label = match self.needs_update {
                    true => format!("v{} (Update Available)", env!("CARGO_PKG_VERSION")),
                    false => format!("v{}", env!("CARGO_PKG_VERSION")),
                };
                if self.update_check.is_some() {
                    ui.label("Checking for updates...");
                } else {
                    ui.hyperlink_to(
                        version_label,
                        "https://github.com/blckink/steamdeckhengst/releases",
                    );
                }
                ui.add(egui::Separator::default().vertical());
                ui.hyperlink_to(
                    "Open Source Licenses",
                    "https://github.com/blckink/steamdeckhengst/tree/main?tab=License-2-ov-file",
                );
            });
        });
    }

    fn display_games_panel(&mut self, ui: &mut Ui) {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.heading("Games");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("➕").clicked() {
                    if let Err(err) = add_game() {
                        println!("Couldn't add game: {err}");
                        msg("Error", &format!("Couldn't add game: {err}"));
                    }
                    let dir_tmp = PATH_PARTY.join("tmp");
                    if dir_tmp.exists() {
                        std::fs::remove_dir_all(&dir_tmp).unwrap();
                    }
                    self.games.clear();
                    self.games = crate::game::scan_all_games();
                }
                if ui.button("🔄").clicked() {
                    self.games.clear();
                    self.games = crate::game::scan_all_games();
                }
            });
        });
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.display_game_list(ui);
        });
    }

    fn display_info_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("info_panel")
            .exact_height(100.0)
            .show(ctx, |ui| {
                match self.cur_page {
                    MenuPage::Game => {
                        match cur_game!(self){
                            Game::Executable { path, .. } => {
                                self.infotext = format!("{}", path.display());
                            }
                            Game::HandlerRef(h) => {
                                self.infotext = h.info.to_owned();
                            }
                        }
                    }
                    MenuPage::Profiles => {
                        self.infotext = "Create profiles to persistently store game save data, settings, and stats.".to_string();
                    }
                    _ => {}
                }
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label(&self.infotext);
                });
            });
    }

    fn display_game_list(&mut self, ui: &mut Ui) {
        let mut refresh_games = false;
        for (i, game) in self.games.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.add(
                    egui::Image::new(game.icon())
                        .max_width(16.0)
                        .corner_radius(2),
                );
                let btn = ui.selectable_value(&mut self.selected_game, i, game.name());
                if btn.has_focus() {
                    btn.scroll_to_me(None);
                }
                if btn.clicked() {
                    self.cur_page = MenuPage::Game;
                };

                let popup_id = ui.make_persistent_id(format!("gamectx{}", i));

                egui::popup::popup_below_widget(
                    ui,
                    popup_id,
                    &btn,
                    egui::popup::PopupCloseBehavior::CloseOnClick,
                    |ui| {
                        if ui.button("Remove").clicked() {
                            if yesno(
                                "Remove game?",
                                &format!("Are you sure you want to remove {}?", game.name()),
                            ) {
                                if let Err(err) = remove_game(&self.games[i]) {
                                    println!("Failed to remove game: {}", err);
                                    msg("Error", &format!("Failed to remove game: {}", err));
                                }
                            }
                            refresh_games = true;
                        }
                        if let HandlerRef(h) = game {
                            if ui.button("Open Handler Folder").clicked() {
                                if let Err(_) = std::process::Command::new("sh")
                                    .arg("-c")
                                    .arg(format!("xdg-open {}", h.path_handler.display()))
                                    .status()
                                {
                                    msg("Error", "Couldn't open handler folder!");
                                }
                            }
                        }
                    },
                );

                if btn.secondary_clicked() {
                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                }
            });
        }
        // Hacky workaround to avoid borrowing conflicts from inside the loop
        if refresh_games {
            self.games.clear();
            self.games = scan_all_games();
        }
    }

    fn display_page_main(&mut self, ui: &mut Ui) {
        ui.heading("Welcome to PartyDeck");
        ui.separator();
        ui.label("Press SELECT/BACK or Tab to unlock gamepad navigation.");
        ui.label("PartyDeck is in the very early stages of development; as such, you will likely encounter bugs, issues, and strange design decisions.");
        ui.label("For debugging purposes, it's recommended to read terminal output (stdout) for further information on errors.");
        ui.label("If you have found this software useful, consider donating to support further development!");
        ui.hyperlink_to("Ko-fi", "https://ko-fi.com/wunner");
        ui.label("If you've encountered issues or want to suggest improvements, criticism and feedback are always appreciated!");
        ui.hyperlink_to("GitHub", "https://github.com/blckink/steamdeckhengst");
    }

    fn display_page_settings(&mut self, ui: &mut Ui) {
        self.infotext.clear();
        ui.heading("Settings");
        ui.separator();
        let force_sdl2_check = ui.checkbox(&mut self.options.force_sdl, "Force Steam Runtime SDL2");
        let render_scale_slider = ui.add(
            egui::Slider::new(&mut self.options.render_scale, 35..=200)
                .text("Instance resolution scale"),
        );
        let gamescope_sdl_backend_check = ui.checkbox(
            &mut self.options.gamescope_sdl_backend,
            "Use SDL backend for Gamescope",
        );

        if force_sdl2_check.hovered() {
            self.infotext = "Forces games to use the version of SDL2 included in the Steam Runtime. Only works on native Linux games, may fix problematic game controller support (incorrect mappings) in some games, may break others. If unsure, leave this unchecked.".to_string();
        }
        if render_scale_slider.hovered() {
            self.infotext = "PartyDeck divides each instance by a base resolution. 100% render scale = your monitor's native resolution. Lower this value to increase performance, but may cause graphical issues or even break some games. If you're using a small screen like the Steam Deck's handheld screen, increase this to 150% or higher.".to_string();
        }
        if gamescope_sdl_backend_check.hovered() {
            self.infotext = "Runs gamescope sessions using the SDL backend. If unsure, leave this checked. If gamescope sessions only show a black screen or give an error (especially on Nvidia + Wayland), try disabling this.".to_string();
        }

        ui.horizontal(|ui| {
        let proton_ver_label = ui.label("Proton version");
        let proton_ver_editbox = ui.add(
            egui::TextEdit::singleline(&mut self.options.proton_version)
                .hint_text("GE-Proton"),
        );
        if proton_ver_label.hovered() || proton_ver_editbox.hovered() {
            self.infotext = "Specify a Proton version. This can be a path, e.g. \"/path/to/proton\" or just a name, e.g. \"GE-Proton\" for the latest version of Proton-GE. If left blank, this will default to \"GE-Proton\". If unsure, leave this blank.".to_string();
        }
        });

        ui.horizontal(|ui| {
        if ui.button("Erase Proton Prefix").clicked() {
            if yesno("Erase Prefix?", "This will erase the Wine prefix used by PartyDeck. This shouldn't erase profile/game-specific data, but exercise caution. Are you sure?") && PATH_PARTY.join("gamesyms").exists() {
                if let Err(err) = std::fs::remove_dir_all(PATH_PARTY.join("pfx")) {
                    msg("Error", &format!("Couldn't erase pfx data: {}", err));
                }
                else if let Err(err) = std::fs::create_dir_all(PATH_PARTY.join("pfx")) {
                    msg("Error", &format!("Couldn't re-create pfx directory: {}", err));
                }
                else {
                    msg("Data Erased", "Proton prefix data successfully erased.");
                }
            }
        }
        if ui.button("Erase Symlink Data").clicked() {
            if yesno("Erase Symlink Data?", "This will erase all game symlink data. This shouldn't erase profile/game-specific data, but exercise caution. Are you sure?") && PATH_PARTY.join("gamesyms").exists() {
                if let Err(err) = std::fs::remove_dir_all(PATH_PARTY.join("gamesyms")) {
                    msg("Error", &format!("Couldn't erase symlink data: {}", err));
                }
                else if let Err(err) = std::fs::create_dir_all(PATH_PARTY.join("gamesyms")) {
                    msg("Error", &format!("Couldn't re-create symlink directory: {}", err));
                }
                else {
                    msg("Data Erased", "Game symlink data successfully erased.");
                }
            }
        }
        });

        ui.horizontal(|ui| {
            if ui.button("Update Goldberg Steam Emu").clicked() {
                if let Err(err) = update_goldberg_emu() {
                    msg("Error", &format!("Couldn't update: {}", err));
                }
            }
            if ui.button("Update UMU Launcher").clicked() {
                if let Err(err) = update_umu_launcher() {
                    msg("Error", &format!("Couldn't update: {}", err));
                }
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Open PartyDeck Data Folder").clicked() {
                if let Err(_) = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(format!("xdg-open {}/", PATH_PARTY.display()))
                    .status()
                {
                    msg("Error", "Couldn't open PartyDeck Data Folder!");
                }
            }
            if ui.button("Edit game paths").clicked() {
                if let Err(_) = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(format!("xdg-open {}/paths.json", PATH_PARTY.display(),))
                    .status()
                {
                    msg("Error", "Couldn't open paths.json!");
                }
            }
        });
    }

    fn display_page_profiles(&mut self, ui: &mut Ui) {
        ui.heading("Profiles");
        ui.separator();
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 16.0)
            .auto_shrink(false)
            .show(ui, |ui| {
                for profile in &self.profiles {
                    if ui.selectable_value(&mut 0, 0, profile).clicked() {
                        if let Err(_) = std::process::Command::new("sh")
                            .arg("-c")
                            .arg(format!(
                                "xdg-open {}/profiles/{}",
                                PATH_PARTY.display(),
                                profile
                            ))
                            .status()
                        {
                            msg("Error", "Couldn't open profile directory!");
                        }
                    };
                }
            });
        if ui.button("New").clicked() {
            if let Some(name) = dialog::Input::new("Enter name (must be alphanumeric):")
                .title("New Profile")
                .show()
                .expect("Could not display dialog box")
            {
                if !name.is_empty() && name.chars().all(char::is_alphanumeric) {
                    create_profile(&name).unwrap();
                } else {
                    msg("Error", "Invalid name");
                }
            }
            self.profiles = scan_profiles(false);
        }
    }

    fn display_page_game(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.image(cur_game!(self).icon());
            ui.heading(cur_game!(self).name());
        });

        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("Play").clicked() {
                self.players.clear();
                self.profiles = scan_profiles(true);
                self.cur_page = MenuPage::Players;
            }
            if let HandlerRef(h) = cur_game!(self) {
                ui.add(egui::Separator::default().vertical());
                if h.win {
                    ui.label(" Proton");
                } else {
                    ui.label("🐧 Native");
                }
                ui.add(egui::Separator::default().vertical());
                ui.label(format!("Author: {}", h.author));
                ui.add(egui::Separator::default().vertical());
                ui.label(format!("Version: {}", h.version));
            }
        });

        if let HandlerRef(h) = cur_game!(self) {
            egui::ScrollArea::horizontal()
                .max_width(f32::INFINITY)
                .show(ui, |ui| {
                    let available_height = ui.available_height();
                    ui.horizontal(|ui| {
                        for img in h.img_paths.iter() {
                            ui.add(
                                egui::Image::new(format!("file://{}", img.display()))
                                    .fit_to_exact_size(egui::vec2(
                                        available_height * 1.77,
                                        available_height,
                                    ))
                                    .maintain_aspect_ratio(true),
                            );
                        }
                    });
                });
        }
    }

    fn display_page_players(&mut self, ui: &mut Ui) {
        ui.heading("Players");
        ui.separator();

        ui.horizontal(|ui| {
            ui.add(
                egui::Image::new(egui::include_image!("../../res/BTN_SOUTH.png")).max_height(12.0),
            );
            ui.label("Add");
            ui.add(
                egui::Image::new(egui::include_image!("../../res/BTN_EAST.png")).max_height(12.0),
            );
            ui.label("Remove");
        });

        let mut i = 0;
        for player in &mut self.players {
            ui.horizontal(|ui| {
                ui.label("👤");
                if let HandlerRef(_) = cur_game!(self) {
                    egui::ComboBox::from_id_salt(format!("{i}")).show_index(
                        ui,
                        &mut player.profselection,
                        self.profiles.len(),
                        |i| self.profiles[i].clone(),
                    );
                } else {
                    ui.label(format!("Player {}", i + 1));
                }
                ui.label(format!("🎮 {}", self.pads[player.pad_index].fancyname(),));
                ui.small(format!("({})", self.pads[player.pad_index].path(),));
            });
            i += 1;
        }
        if self.players.len() > 0 {
            ui.separator();
            if ui.button("Start").clicked() {
                self.start_game();
            }
        }
    }

    fn handle_gamepad_gui(&mut self, raw_input: &mut egui::RawInput) {
        let mut key: Option<egui::Key> = None;
        for pad in &mut self.pads {
            match pad.poll() {
                Some(PadButton::ABtn) => {
                    key = Some(Key::Enter);
                }
                Some(PadButton::BBtn) => {
                    self.cur_page = MenuPage::Main;
                }
                Some(PadButton::XBtn) => {
                    self.profiles = scan_profiles(false);
                    self.cur_page = MenuPage::Profiles;
                }
                Some(PadButton::YBtn) => {
                    self.cur_page = MenuPage::Settings;
                }
                Some(PadButton::SelectBtn) => {
                    key = Some(Key::Tab);
                }
                Some(PadButton::Up) => {
                    key = Some(Key::ArrowUp);
                }
                Some(PadButton::Down) => {
                    key = Some(Key::ArrowDown);
                }
                Some(PadButton::Left) => {
                    key = Some(Key::ArrowLeft);
                }
                Some(PadButton::Right) => {
                    key = Some(Key::ArrowRight);
                }
                Some(_) => {}
                None => {}
            }
        }

        if let Some(key) = key {
            raw_input.events.push(egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::default(),
            });
        }
    }

    fn handle_gamepad_players(&mut self) {
        for (i, pad) in self.pads.iter_mut().enumerate() {
            if is_pad_in_players(i, &self.players) {
                continue;
            }
            match pad.poll() {
                Some(PadButton::ABtn) => {
                    if self.players.len() < 4 {
                        self.players.push(Player {
                            pad_index: i,
                            profname: String::new(),
                            profselection: 0,
                        });
                    }
                }
                Some(PadButton::BBtn) => {
                    if self.players.len() == 0 {
                        self.cur_page = MenuPage::Main;
                    }
                }
                _ => {}
            }
        }

        let mut i = 0;
        while i < self.players.len() {
            match self.pads[self.players[i].pad_index].poll() {
                Some(PadButton::BBtn) => {
                    self.players.remove(i);
                    continue;
                }
                Some(PadButton::StartBtn) => {
                    self.start_game();
                }
                _ => {}
            }
            i += 1;
        }
    }

    pub fn start_game(&mut self) {
        let game = cur_game!(self).to_owned();
        match game {
            HandlerRef(handler) => {
                if let Err(err) = self.start_handler_game(&handler) {
                    println!("{}", err);
                    msg("Launch Error", &format!("{err}"));
                }
            }
            Executable { path, .. } => {
                if let Err(err) = self.start_exec_game(&path) {
                    println!("{}", err);
                    msg("Launch Error", &format!("{err}"));
                }
            }
        }
        self.cur_page = MenuPage::Main;
    }

    pub fn start_handler_game(
        &mut self,
        handler: &Handler,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let _ = save_cfg(&self.options);

        let mut guests = GUEST_NAMES.to_vec();
        for player in &mut self.players {
            if player.profselection == 0 {
                let i = fastrand::usize(..guests.len());
                player.profname = format!(".{}", guests[i]);
                guests.swap_remove(i);
            } else {
                player.profname = self.profiles[player.profselection].to_owned();
            }
            create_profile(player.profname.as_str())?;
            create_gamesave(player.profname.as_str(), handler)?;
        }
        if handler.symlink_dir {
            create_symlink_folder(handler)?;
        }

        let cmd = launch_from_handler(handler, &self.pads, &self.players, &self.options)?;
        println!("\nCOMMAND:\n{}\n", cmd);

        kwin_dbus_start_script(PATH_RES.join("splitscreen_kwin.js"))?;

        std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()?;

        kwin_dbus_unload_script()?;
        remove_guest_profiles()?;

        Ok(())
    }

    fn start_exec_game(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let _ = save_cfg(&self.options);

        let cmd = launch_executable(path, &self.pads, &self.players, &self.options)?;

        kwin_dbus_start_script(PATH_RES.join("splitscreen_kwin.js"))?;

        std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()?;

        kwin_dbus_unload_script()?;

        Ok(())
    }
}

static GUEST_NAMES: [&str; 21] = [
    "Blinky", "Pinky", "Inky", "Clyde", "Beatrice", "Battler", "Ellie", "Joel", "Leon", "Ada",
    "Madeline", "Theo", "Yokatta", "Wyrm", "Brodiee", "Supreme", "Conk", "Gort", "Lich", "Smores",
    "Canary",
];
