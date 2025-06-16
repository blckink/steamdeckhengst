use crate::app::config::*;
use crate::game::{Game::*, *};
use crate::handler::*;
use crate::input::*;
use crate::launch::{launch_executable, launch_from_handler};
use crate::paths::*;
use crate::task::Task;
use crate::util::*;

use dialog::DialogBox;
use eframe::egui::{self, Color32, Key, RichText, Ui};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use regex::Regex;
use std::path::PathBuf;

#[derive(Eq, PartialEq)]
pub enum MenuPage {
    Games,
    Settings,
    Profiles,
    Game,
    Players,
    About,
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
    pub md_cache: CommonMarkCache,
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
            update_check: Some(Task::spawn(|| {
                check_for_partydeck_update().unwrap_or(false)
            })),
            options: load_cfg(),
            cur_page: MenuPage::Games,
            infotext: String::new(),
            pads: scan_evdev_gamepads(),
            players: Vec::new(),
            games: scan_all_games(),
            profiles: Vec::new(),
            selected_game: 0,
            md_cache: CommonMarkCache::default(),
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
        let side_w = 150.0;
        egui::SidePanel::left("left_panel")
            .resizable(false)
            .exact_width(side_w)
            .frame(egui::Frame::default().fill(Color32::from_gray(40)))
            .show(ctx, |ui| {
                self.display_left_panel(ui);
            });

        egui::TopBottomPanel::top("nav_bar")
            .exact_height(60.0)
            .frame(egui::Frame::default().fill(Color32::from_gray(40)))
            .show(ctx, |ui| {
                self.display_nav_bar(ui, ctx);
            });

        if (self.cur_page != MenuPage::Games)
            && (self.cur_page != MenuPage::Players)
            && (self.cur_page != MenuPage::About)
        {
            self.display_info_panel(ctx);
        }

        egui::CentralPanel::default().show(ctx, |ui| match self.cur_page {
            MenuPage::Games => {
                self.display_page_games(ui);
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
            MenuPage::About => {
                self.display_page_about(ui);
            }
        });

        if self.cur_page == MenuPage::Games {
            egui::TopBottomPanel::bottom("games_buttons")
                .exact_height(40.0)
                .show(ctx, |ui| {
                    self.display_games_buttons(ui);
                });
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(33)); // 30 fps
    }
}

impl PartyApp {
    fn display_left_panel(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.vertical_centered(|ui| {
                ui.add(
                    egui::Image::new(egui::include_image!("../../.github/assets/sdh.svg"))
                        .max_height(60.0),
                );
            });
            ui.separator();
            ui.label("Gamepads:");
            if self.pads.is_empty() {
                ui.label("No Gamepads detected");
            } else {
                for pad in &self.pads {
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new(
                            RichText::new("\u{25CF}").color(Color32::GREEN),
                        ));
                        ui.label(pad.fancyname());
                    });
                }
            }
            if ui.button("Rescan").clicked() {
                self.players.clear();
                self.pads.clear();
                self.pads = scan_evdev_gamepads();
            }
            if self.update_check.is_some() {
                ui.label("Checking for updates...");
            }
        });
    }

    fn display_nav_bar(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ui.style_mut().spacing.item_spacing.x = 20.0;
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            let pages = [
                (MenuPage::Games, "GAMES"),
                (MenuPage::Profiles, "PROFILES"),
                (MenuPage::Settings, "SETTINGS"),
            ];
            for (page, label) in pages {
                let resp = ui.add(
                    egui::Label::new(RichText::new(label).text_style(egui::TextStyle::Button))
                        .sense(egui::Sense::click()),
                );
                if self.cur_page == page {
                    let stroke = egui::Stroke::new(1.0, Color32::from_rgb(0, 177, 227));
                    ui.painter().line_segment(
                        [
                            resp.rect.left_bottom() + egui::vec2(0.0, 2.0),
                            resp.rect.right_bottom() + egui::vec2(0.0, 2.0),
                        ],
                        stroke,
                    );
                }
                if resp.clicked() {
                    if page == MenuPage::Profiles {
                        self.profiles = scan_profiles(false);
                    }
                    self.cur_page = page;
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                let resp = ui.add(
                    egui::Label::new(RichText::new("ABOUT").text_style(egui::TextStyle::Button))
                        .sense(egui::Sense::click()),
                );
                if self.cur_page == MenuPage::About {
                    let stroke = egui::Stroke::new(1.0, Color32::from_rgb(0, 177, 227));
                    ui.painter().line_segment(
                        [
                            resp.rect.left_bottom() + egui::vec2(0.0, 2.0),
                            resp.rect.right_bottom() + egui::vec2(0.0, 2.0),
                        ],
                        stroke,
                    );
                }
                if resp.clicked() {
                    self.cur_page = MenuPage::About;
                }
            });
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

    fn display_games_grid(&mut self, ui: &mut Ui) {
        let tile_w = 220.0;
        let tile_h = tile_w * 9.0 / 16.0;
        let cols = ((ui.available_width() + 10.0) / (tile_w + 10.0))
            .floor()
            .max(1.0) as usize;
        let mut idx = 0;
        while idx < self.games.len() {
            ui.horizontal(|ui| {
                for _ in 0..cols {
                    if idx >= self.games.len() {
                        break;
                    }
                    ui.allocate_ui_with_layout(
                        egui::vec2(tile_w, tile_h + 20.0),
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
                        let selected = self.selected_game == idx;
                        let img_src = match &self.games[idx] {
                            HandlerRef(h) => {
                                if let Some(appid) = &h.steam_appid {
                                    format!("https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/{}/header.jpg", appid).into()
                                } else if !h.img_paths.is_empty() {
                                    format!("file://{}", h.img_paths[0].display()).into()
                                } else {
                                    self.games[idx].icon()
                                }
                            }
                            _ => self.games[idx].icon(),
                        };
                        let resp = ui.add(
                            egui::Image::new(img_src)
                                .fit_to_exact_size(egui::vec2(tile_w, tile_h))
                                .sense(egui::Sense::click()),
                        );
                        if selected {
                            ui.painter().rect_stroke(
                                resp.rect,
                                0.0,
                                ui.visuals().selection.stroke,
                                egui::StrokeKind::Inside,
                            );
                        }
                        if resp.clicked() {
                            self.selected_game = idx;
                            self.cur_page = MenuPage::Game;
                        }
                        ui.label(self.games[idx].name());
                    },
                    );
                    idx += 1;
                }
            });
            ui.add_space(10.0);
        }
    }

    fn display_page_games(&mut self, ui: &mut Ui) {
        self.display_games_grid(ui);
    }

    fn display_games_buttons(&mut self, ui: &mut Ui) {
        ui.horizontal_centered(|ui| {
            if ui.button("Add").clicked() {
                if let Err(err) = add_game() {
                    println!("Couldn't add game: {err}");
                    msg("Error", &format!("Couldn't add game: {err}"));
                }
            }
            if ui.button("Refresh").clicked() {
                self.games = scan_all_games();
            }
        });
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
        let vertical_two_player_check = ui.checkbox(
            &mut self.options.vertical_two_player,
            "Vertical split for 2 players",
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
        if vertical_two_player_check.hovered() {
            self.infotext = "Toggle how two player sessions are arranged. Enabled = vertical split (stacked). Disabled = horizontal split (side by side).".to_string();
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
            if ui
                .add_sized([150.0, 40.0], egui::Button::new("Play"))
                .clicked()
            {
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

    fn display_page_about(&mut self, ui: &mut Ui) {
        ui.heading(format!("About - Version {}", env!("CARGO_PKG_VERSION")));
        ui.separator();
        let raw = include_str!("../../README.md");
        let readme = clean_readme(raw);
        egui::ScrollArea::vertical()
            .max_height(ui.available_height())
            .show(ui, |ui| {
                CommonMarkViewer::new().show(ui, &mut self.md_cache, &readme);
            });
    }

    fn handle_gamepad_gui(&mut self, raw_input: &mut egui::RawInput) {
        let mut key: Option<egui::Key> = None;
        for pad in &mut self.pads {
            match pad.poll() {
                Some(PadButton::ABtn) => {
                    key = Some(Key::Enter);
                }
                Some(PadButton::BBtn) => {
                    self.cur_page = MenuPage::Games;
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
                        self.cur_page = MenuPage::Games;
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
        self.cur_page = MenuPage::Games;
    }

    pub fn start_handler_game(
        &mut self,
        handler: &Handler,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let _ = save_cfg(&self.options);
        log_info("Starting handler game launch");

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

        let script = if self.options.vertical_two_player {
            PATH_RES.join("splitscreen_kwin.js")
        } else {
            PATH_RES.join("splitscreen_kwin_horizontal.js")
        };
        kwin_dbus_start_script(script)?;

        std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()?;

        kwin_dbus_unload_script()?;
        log_info("Handler game finished");
        remove_guest_profiles()?;

        Ok(())
    }

    fn start_exec_game(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let _ = save_cfg(&self.options);
        log_info("Starting executable game launch");

        let cmd = launch_executable(path, &self.pads, &self.players, &self.options)?;

        let script = if self.options.vertical_two_player {
            PATH_RES.join("splitscreen_kwin.js")
        } else {
            PATH_RES.join("splitscreen_kwin_horizontal.js")
        };
        kwin_dbus_start_script(script)?;

        std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()?;

        kwin_dbus_unload_script()?;
        log_info("Executable game finished");

        Ok(())
    }
}

fn clean_readme(src: &str) -> String {
    let img_re = Regex::new(r#"<img\s+src=\"([^\"]+)\"[^>]*>"#).unwrap();
    let mut out = img_re.replace_all(src, "![]($1)").to_string();
    out = out.replace("<br />", "\n");
    let tags_re = Regex::new(r"</?[^>]+>").unwrap();
    tags_re.replace_all(&out, "").to_string()
}

static GUEST_NAMES: [&str; 21] = [
    "Blinky", "Pinky", "Inky", "Clyde", "Beatrice", "Battler", "Ellie", "Joel", "Leon", "Ada",
    "Madeline", "Theo", "Yokatta", "Wyrm", "Brodiee", "Supreme", "Conk", "Gort", "Lich", "Smores",
    "Canary",
];
