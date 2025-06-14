use iced::widget::{Column, Row, button, column, image, row, scrollable, text};
use iced::{Application, Command, Element, Settings, Theme, executor};
use std::collections::HashMap;

use crate::game::{self, Game};
use crate::handler::Handler;
use crate::image_cache::get_image;
use crate::input::{Gamepad, scan_evdev_gamepads};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    MyGames,
    Controllers,
    Profiles,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectTab(Tab),
    RefreshPads,
    Play(Game),
}

pub struct PartyUI {
    tab: Tab,
    games: Vec<Game>,
    pads: Vec<Gamepad>,
    images: HashMap<String, iced::widget::image::Handle>,
}

impl Application for PartyUI {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                tab: Tab::MyGames,
                games: game::scan_all_games(),
                pads: scan_evdev_gamepads(),
                images: HashMap::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("PartyDeck")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::SelectTab(t) => self.tab = t,
            Message::RefreshPads => self.pads = scan_evdev_gamepads(),
            Message::Play(_g) => {}
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let tabs = row![
            button(text("My Games")).on_press(Message::SelectTab(Tab::MyGames)),
            button(text("Controllers")).on_press(Message::SelectTab(Tab::Controllers)),
            button(text("Profiles")).on_press(Message::SelectTab(Tab::Profiles)),
            button(text("Settings")).on_press(Message::SelectTab(Tab::Settings)),
        ]
        .spacing(10);

        let content = match self.tab {
            Tab::MyGames => self.view_games(),
            Tab::Controllers => self.view_controllers(),
            Tab::Profiles => self.view_profiles(),
            Tab::Settings => self.view_settings(),
        };

        column![tabs, content].spacing(20).into()
    }
}

impl PartyUI {
    fn view_games(&self) -> Element<Message> {
        let mut rows = Column::new().spacing(10);
        for game in &self.games {
            let title = text(game.name());
            let mut r = Row::new().spacing(10);
            if let Game::HandlerRef(h) = game {
                if let Some(appid) = &h.steam_appid {
                    let local = h.img_paths.get(0).map(|p| p.as_path());
                    if let Some(path) = get_image(appid, local) {
                        let handle = image::Handle::from_path(path);
                        r = r.push(image(handle).width(100));
                    }
                }
            }
            r = r.push(title);
            r = r.push(button("Play").on_press(Message::Play(game.clone())));
            rows = rows.push(r);
        }
        scrollable(rows).into()
    }

    fn view_controllers(&self) -> Element<Message> {
        let mut col = Column::new().spacing(10);
        for (i, pad) in self.pads.iter().enumerate() {
            col = col.push(text(format!("{}: {}", i + 1, pad.fancyname())));
        }
        column![button("Refresh").on_press(Message::RefreshPads), col].into()
    }

    fn view_profiles(&self) -> Element<Message> {
        Column::new().push(text("Profiles coming soon")).into()
    }

    fn view_settings(&self) -> Element<Message> {
        Column::new().push(text("Settings")).into()
    }
}

pub fn run() -> iced::Result {
    PartyUI::run(Settings::default())
}
