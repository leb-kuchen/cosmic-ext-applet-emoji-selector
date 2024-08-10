use bitflags::bitflags;
use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};
use serde::{Deserialize, Serialize};
pub const CONFIG_VERSION: u64 = 1;

#[derive(Clone, CosmicConfigEntry, Debug, Deserialize, PartialEq, Serialize)]
pub struct Config {
    #[serde(default)]
    pub color_buttons: Vec<ColorButton>,
    #[serde(default)]
    pub left_click_action: ClickMode,
    #[serde(default)]
    pub right_click_action: ClickMode,
    #[serde(default)]
    pub middle_click_action: ClickMode,
    #[serde(default)]
    pub last_used_limit: usize,
    #[serde(default)]
    pub last_used: Vec<String>,
    #[serde(default)]
    pub font_family: String,
    #[serde(default)]
    pub show_preview: bool,
    #[serde(default)]
    pub skin_tone_mode: SkinToneMode,
}

impl Default for Config {
    fn default() -> Self {
        let colors = [
            0xFFCC22_ffu32,
            0xf7dece_ff,
            0xf3d2a2_ff,
            0xbf8d67_ff,
            0xaf7e57_ff,
            0x7c533e_ff,
        ];
        let color_buttons = SkinToneMode::ALL
            .iter()
            .take(6)
            .zip(colors)
            .map(|(skin_tone, color)| ColorButton {
                color: color.to_be_bytes().map(|c| c as f32 / 255.),
                skin_tone_mode: skin_tone,
                active: skin_tone == SkinToneMode::DEFAULT,
            })
            .collect();
        Self {
            color_buttons,
            last_used: Vec::new(),
            last_used_limit: 20,
            font_family: "Noto Color Emoji".into(),
            show_preview: true,
            left_click_action: ClickMode::CLOSE | ClickMode::COPY,
            right_click_action: ClickMode::APPEND | ClickMode::COPY,
            middle_click_action: ClickMode::COPY,
            skin_tone_mode: SkinToneMode::DEFAULT | SkinToneMode::NO_SKIN,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Hash, Debug, Default, Clone)]
pub struct Annotation {
    #[serde(default)]
    pub default: Vec<String>,
    #[serde(default)]
    pub tts: Vec<String>,
}
bitflags! {
    #[derive(Serialize, Deserialize, PartialEq, Hash, Debug, Default, Copy, Clone, Eq)]
    pub struct ClickMode: u8 {
        const NONE = 0;  0xFFCC22_ffu32,
            0xf7dece_ff,
            0xf3d2a2_ff,
            0xbf8d67_ff,
            0xaf7e57_ff,
            0x7c533e_ff,
        const COPY = 1;
        const CLOSE = 1 << 1;
        const APPEND = 1 << 2;
        const PRIVATE = 1 << 3;
    }

}

#[derive(Serialize, Deserialize, PartialEq, Hash, Debug, Default, Copy, Clone, Eq)]
#[serde(transparent)]
pub struct SkinToneMode(u32);
bitflags! {
    impl SkinToneMode: u32 {
        const DEFAULT = 1;

        const LIGHT = 1 << 1;
        const MEDIUM_LIGHT = 1 << 2;
        const MEDIUM = 1 << 3;
        const MEDIUM_DARK = 1 << 4;
        const DARK = 1 << 5;
        const NO_SKIN = 1 << 6;

        const OTHER = 1 << 7;

        const LIGHT_AND_MEDIUM_LIGHT = 1 << 8;
        const LIGHT_AND_MEDIUM = 1 << 9;
        const LIGHT_AND_MEDIUM_DARK = 1 << 10;
        const LIGHT_AND_DARK = 1 << 11;
        const MEDIUM_LIGHT_AND_LIGHT = 1 << 12;
        const MEDIUM_LIGHT_AND_MEDIUM = 1 << 13;
        const MEDIUM_LIGHT_AND_MEDIUM_DARK = 1 << 14;
        const MEDIUM_LIGHT_AND_DARK = 1 << 15;
        const MEDIUM_AND_LIGHT = 1 << 16;
        const MEDIUM_AND_MEDIUM_LIGHT = 1 << 17;
        const MEDIUM_AND_MEDIUM_DARK = 1 << 18;
        const MEDIUM_AND_DARK = 1 << 19;
        const MEDIUM_DARK_AND_LIGHT = 1 << 20;
        const MEDIUM_DARK_AND_MEDIUM_LIGHT = 1 << 21;
        const MEDIUM_DARK_AND_MEDIUM = 1 << 22;
        const MEDIUM_DARK_AND_DARK = 1 << 23;
        const DARK_AND_LIGHT = 1 << 24;
        const DARK_AND_MEDIUM_LIGHT = 1 << 25;
        const DARK_AND_MEDIUM = 1 << 26;
        const DARK_AND_MEDIUM_DARK = 1 << 27;

        const ALL = !0 >> 5;

        const FILTER_EXACT = 1 << 28;
        const ALL_EXACT = ((1 << 21) - 1) << 8;

        const _ = !0;
    }
}

impl SkinToneMode {
    pub fn new(skin_tone: emojis::SkinTone) -> Self {
        match skin_tone {
            emojis::SkinTone::Default => SkinToneMode::DEFAULT,
            emojis::SkinTone::Light => SkinToneMode::LIGHT,
            emojis::SkinTone::MediumLight => SkinToneMode::MEDIUM_LIGHT,
            emojis::SkinTone::Medium => SkinToneMode::MEDIUM,
            emojis::SkinTone::MediumDark => SkinToneMode::MEDIUM_DARK,
            emojis::SkinTone::Dark => SkinToneMode::DARK,
            emojis::SkinTone::LightAndMediumLight => {
                SkinToneMode::LIGHT | SkinToneMode::MEDIUM_LIGHT
            }
            emojis::SkinTone::LightAndMedium => SkinToneMode::LIGHT | SkinToneMode::MEDIUM,
            emojis::SkinTone::LightAndMediumDark => SkinToneMode::LIGHT | SkinToneMode::MEDIUM_DARK,
            emojis::SkinTone::LightAndDark => SkinToneMode::LIGHT | SkinToneMode::DARK,
            emojis::SkinTone::MediumLightAndLight => {
                SkinToneMode::MEDIUM_LIGHT | SkinToneMode::LIGHT
            }
            emojis::SkinTone::MediumLightAndMedium => {
                SkinToneMode::MEDIUM_LIGHT | SkinToneMode::MEDIUM
            }
            emojis::SkinTone::MediumLightAndMediumDark => {
                SkinToneMode::MEDIUM_LIGHT | SkinToneMode::MEDIUM_DARK
            }
            emojis::SkinTone::MediumLightAndDark => SkinToneMode::MEDIUM_LIGHT | SkinToneMode::DARK,
            emojis::SkinTone::MediumAndLight => SkinToneMode::MEDIUM | SkinToneMode::LIGHT,
            emojis::SkinTone::MediumAndMediumLight => {
                SkinToneMode::MEDIUM | SkinToneMode::MEDIUM_LIGHT
            }
            emojis::SkinTone::MediumAndMediumDark => {
                SkinToneMode::MEDIUM | SkinToneMode::MEDIUM_DARK
            }
            emojis::SkinTone::MediumAndDark => SkinToneMode::MEDIUM | SkinToneMode::DARK,
            emojis::SkinTone::MediumDarkAndLight => SkinToneMode::MEDIUM_DARK | SkinToneMode::LIGHT,
            emojis::SkinTone::MediumDarkAndMediumLight => {
                SkinToneMode::MEDIUM_DARK | SkinToneMode::MEDIUM_LIGHT
            }
            emojis::SkinTone::MediumDarkAndMedium => {
                SkinToneMode::MEDIUM_DARK | SkinToneMode::MEDIUM
            }
            emojis::SkinTone::MediumDarkAndDark => SkinToneMode::MEDIUM_DARK | SkinToneMode::DARK,
            emojis::SkinTone::DarkAndLight => SkinToneMode::DARK | SkinToneMode::LIGHT,
            emojis::SkinTone::DarkAndMediumLight => SkinToneMode::DARK | SkinToneMode::MEDIUM_LIGHT,
            emojis::SkinTone::DarkAndMedium => SkinToneMode::DARK | SkinToneMode::MEDIUM,
            emojis::SkinTone::DarkAndMediumDark => SkinToneMode::DARK | SkinToneMode::MEDIUM_DARK,
            _ => SkinToneMode::OTHER,
        }
    }

    pub fn new_exact(skin_tone: emojis::SkinTone) -> Self {
        match skin_tone {
            emojis::SkinTone::Default => SkinToneMode::DEFAULT,
            emojis::SkinTone::Light => SkinToneMode::LIGHT,
            emojis::SkinTone::MediumLight => SkinToneMode::MEDIUM_LIGHT,
            emojis::SkinTone::Medium => SkinToneMode::MEDIUM,
            emojis::SkinTone::MediumDark => SkinToneMode::MEDIUM_DARK,
            emojis::SkinTone::Dark => SkinToneMode::DARK,
            emojis::SkinTone::LightAndMediumLight => SkinToneMode::LIGHT_AND_MEDIUM_LIGHT,
            emojis::SkinTone::LightAndMedium => SkinToneMode::LIGHT_AND_MEDIUM,
            emojis::SkinTone::LightAndMediumDark => SkinToneMode::LIGHT_AND_MEDIUM_DARK,
            emojis::SkinTone::LightAndDark => SkinToneMode::LIGHT_AND_DARK,
            emojis::SkinTone::MediumLightAndLight => SkinToneMode::MEDIUM_LIGHT_AND_LIGHT,
            emojis::SkinTone::MediumLightAndMedium => SkinToneMode::MEDIUM_LIGHT_AND_MEDIUM,
            emojis::SkinTone::MediumLightAndMediumDark => {
                SkinToneMode::MEDIUM_LIGHT_AND_MEDIUM_DARK
            }
            emojis::SkinTone::MediumLightAndDark => SkinToneMode::MEDIUM_LIGHT_AND_DARK,
            emojis::SkinTone::MediumAndLight => SkinToneMode::MEDIUM_AND_LIGHT,
            emojis::SkinTone::MediumAndMediumLight => SkinToneMode::MEDIUM_AND_MEDIUM_LIGHT,
            emojis::SkinTone::MediumAndMediumDark => SkinToneMode::MEDIUM_AND_MEDIUM_DARK,
            emojis::SkinTone::MediumAndDark => SkinToneMode::MEDIUM_AND_DARK,
            emojis::SkinTone::MediumDarkAndLight => SkinToneMode::MEDIUM_DARK_AND_LIGHT,
            emojis::SkinTone::MediumDarkAndMediumLight => {
                SkinToneMode::MEDIUM_DARK_AND_MEDIUM_LIGHT
            }
            emojis::SkinTone::MediumDarkAndMedium => SkinToneMode::MEDIUM_DARK_AND_MEDIUM,
            emojis::SkinTone::MediumDarkAndDark => SkinToneMode::MEDIUM_DARK_AND_DARK,
            emojis::SkinTone::DarkAndLight => SkinToneMode::DARK_AND_LIGHT,
            emojis::SkinTone::DarkAndMediumLight => SkinToneMode::DARK_AND_MEDIUM_LIGHT,
            emojis::SkinTone::DarkAndMedium => SkinToneMode::DARK_AND_MEDIUM,
            emojis::SkinTone::DarkAndMediumDark => SkinToneMode::DARK_AND_MEDIUM_DARK,
            _ => SkinToneMode::OTHER,
        }
    }
}
#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Copy, Clone)]

pub struct ColorButton {
    pub color: [f32; 4],
    pub skin_tone_mode: SkinToneMode,
    pub active: bool,
}
