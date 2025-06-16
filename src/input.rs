pub struct Player {
    pub pad_index: usize,
    pub profname: String,
    pub profselection: usize,
}

pub fn is_pad_in_players(index: usize, players: &Vec<Player>) -> bool {
    for player in players {
        if player.pad_index == index {
            return true;
        }
    }
    false
}

use evdev::*;

pub struct Gamepad {
    path: String,
    dev: Device,
}

pub enum PadType {
    Xbox,
    PlayStation,
    Nintendo,
    Unknown,
}
pub enum PadButton {
    Left,
    Right,
    Up,
    Down,
    ABtn,
    BBtn,
    XBtn,
    YBtn,
    StartBtn,
    SelectBtn,
}
impl Gamepad {
    pub fn name(&self) -> &str {
        self.dev.name().unwrap_or_else(|| "")
    }
    pub fn fancyname(&self) -> &str {
        match self.dev.input_id().vendor() {
            0x045e => "Xbox Controller",
            0x054c => "PS Controller",
            0x057e => "NT Pro Controller",
            _ => self.name(),
        }
    }
    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn pad_type(&self) -> PadType {
        match self.dev.input_id().vendor() {
            0x045e => PadType::Xbox,
            0x054c => PadType::PlayStation,
            0x057e => PadType::Nintendo,
            _ => PadType::Unknown,
        }
    }

    pub fn event_id(&self) -> Option<&str> {
        self.path.rsplit_once("event").map(|(_, id)| id)
    }

    pub fn battery_percent(&self) -> Option<u8> {
        if let Some(b) = self.battery_percent_sys() {
            return Some(b);
        }
        #[cfg(feature = "hid-battery")]
        if self.dev.input_id().vendor() == 0x054c {
            return self.battery_percent_hid();
        }
        None
    }

    fn battery_percent_sys(&self) -> Option<u8> {
        let name = std::path::Path::new(&self.path)
            .file_name()
            .and_then(|p| p.to_str())?;
        let sys_path = std::path::Path::new("/sys/class/power_supply");
        for entry in std::fs::read_dir(sys_path).ok()? {
            let entry = entry.ok()?;
            if let Ok(content) = std::fs::read_to_string(entry.path().join("uevent")) {
                if content.contains(name) {
                    if let Ok(cap) = std::fs::read_to_string(entry.path().join("capacity")) {
                        if let Ok(val) = cap.trim().parse::<u8>() {
                            return Some(val);
                        }
                    }
                }
            }
        }
        None
    }

    #[cfg(feature = "hid-battery")]

    fn battery_percent_hid(&self) -> Option<u8> {
        use hidapi::HidApi;
        let vid = self.dev.input_id().vendor();
        let pid = self.dev.input_id().product();
        let api = HidApi::new().ok()?;
        for info in api.device_list() {
            if info.vendor_id() == vid && info.product_id() == pid {
                if let Ok(device) = info.open_device(&api) {
                    let mut buf = [0u8; 64];
                    buf[0] = 0x02;
                    if device.get_feature_report(&mut buf).is_ok() {
                        let level = buf[53];
                        if level <= 10 {
                            return Some(level * 10);
                        }
                    }
                }
            }
        }
        None
    }
    pub fn poll(&mut self) -> Option<PadButton> {
        let mut btn: Option<PadButton> = None;
        if let Ok(events) = self.dev.fetch_events() {
            for event in events {
                btn = match event.destructure() {
                    EventSummary::Key(_, KeyCode::BTN_SOUTH, 1) => Some(PadButton::ABtn),
                    EventSummary::Key(_, KeyCode::BTN_EAST, 1) => Some(PadButton::BBtn),
                    EventSummary::Key(_, KeyCode::BTN_NORTH, 1) => Some(PadButton::XBtn),
                    EventSummary::Key(_, KeyCode::BTN_WEST, 1) => Some(PadButton::YBtn),
                    EventSummary::Key(_, KeyCode::BTN_START, 1) => Some(PadButton::StartBtn),
                    EventSummary::Key(_, KeyCode::BTN_SELECT, 1) => Some(PadButton::SelectBtn),
                    EventSummary::AbsoluteAxis(_, AbsoluteAxisCode::ABS_HAT0X, -1) => {
                        Some(PadButton::Left)
                    }
                    EventSummary::AbsoluteAxis(_, AbsoluteAxisCode::ABS_HAT0X, 1) => {
                        Some(PadButton::Right)
                    }
                    EventSummary::AbsoluteAxis(_, AbsoluteAxisCode::ABS_HAT0Y, -1) => {
                        Some(PadButton::Up)
                    }
                    EventSummary::AbsoluteAxis(_, AbsoluteAxisCode::ABS_HAT0Y, 1) => {
                        Some(PadButton::Down)
                    }
                    _ => btn,
                };
            }
        }
        btn
    }
}

pub fn scan_evdev_gamepads(filter_steam: bool) -> Vec<Gamepad> {
    let mut pads: Vec<Gamepad> = Vec::new();
    for dev in evdev::enumerate() {
        if filter_steam {
            let vendor = dev.1.input_id().vendor();
            if vendor == 0x28de {
                continue;
            }
            if let Some(name) = dev.1.name() {
                if name.contains("Steam Virtual Gamepad") {
                    continue;
                }
            }
        }
        let has_btn_south = dev
            .1
            .supported_keys()
            .map_or(false, |keys| keys.contains(KeyCode::BTN_SOUTH));
        if has_btn_south {
            if dev.1.set_nonblocking(true).is_err() {
                println!("Failed to set non-blocking mode for {}", dev.0.display());
                continue;
            }
            pads.push(Gamepad {
                path: dev.0.to_str().unwrap().to_string(),
                dev: dev.1,
            });
        }
    }
    pads
}

#[allow(dead_code)]
pub fn scan_evdev_mice() -> Vec<Device> {
    let mut mice: Vec<Device> = Vec::new();
    for dev in evdev::enumerate() {
        let has_btn_left = dev
            .1
            .supported_keys()
            .map_or(false, |keys| keys.contains(KeyCode::BTN_LEFT));
        if has_btn_left {
            if dev.1.set_nonblocking(true).is_err() {
                println!("Failed to set non-blocking mode for {}", dev.0.display());
                continue;
            }
            mice.push(dev.1);
        }
    }
    mice
}
