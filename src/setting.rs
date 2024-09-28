use core::arch;
use std::fs;

use serde::{Deserialize, Serialize};

use clap::Parser;

#[derive(Parser, Debug, Default)]
#[command(version = "0.1.1", about = "A tool for visiualization steam achievements.", long_about = None)]
struct Args {
    /// Appid of the game. If not provided, it will read from ColdClientLoader.ini or steam_settings/steam_appid.txt
    #[arg(short, long)]
    appid: Option<u32>,
    /// Directory of the achievements image. If not provided, it will use the default directory steam_settings/achievement_images/.
    #[arg(short, long)]
    imagedir: Option<String>,
    /// Path of the achievements information. If not provided, it will use the default directory steam_settings/achievements.json.
    #[arg(short, long)]
    datadir: Option<String>,
    /// Path of the achievements statistical data. If not provided, it will use the default directory %APPDATA%/Goldberg SteamEmu Saves/${AppId}/achievements.json.
    #[arg(short, long)]
    jsondir: Option<String>,
    /// Path of the setting file. If not provided, it will use the default directory %APPDATA%/Goldberg SteamEmu Saves/achievement_reminder_setting.json.
    #[arg(short, long)]
    settingpath: Option<String>,
    /// Default languages list of the achievements. If not provided, it will use the list ["schinese", "tchinese", "chinese", "english", "japanese", "french"].
    /// The display language will be the first language that can be found in the list.
    #[arg(short, long, value_parser, num_args = 1.., value_delimiter = ' ')]
    languages: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize)]
pub struct Setting {
    fonts: Option<Vec<String>>,
    languages: Option<Vec<String>>,
    app_data_path: Option<String>,
    setting_path: Option<String>,
    goldberg_path: Option<String>,
    image_dir: Option<String>,

    #[serde(skip)]
    args: Args,
}

impl Setting {
    const DEFAULT_FONTS: [&str; 13] = [
        "Segoe UI",
        "Segoe UI Emoji",
        "Segoe UI Symbol",
        "Microsoft Sans Serif",
        "Source Han Sans",
        "Consolas",
        "Sans Serif Collection",
        "Arial",
        "等线",
        "楷体",
        "思源黑体",
        "微软雅黑",
        "新宋体",
    ];

    const DEFAULT_LANGUAGES: [&str; 6] = [
        "schinese", "tchinese", "chinese", "english", "japanese", "french",
    ];

    const DEFAULT_APP_ID_PATH_1: &str = "ColdClientLoader.ini";
    const DEFAULT_IMAGE_DIR: &str = "steam_settings/achievement_images/";
    const DEFAULT_APP_ID_PATH_2: &str = "steam_settings/steam_appid.txt";
    const DEFAULT_ACHIEVEMENTS_DATA_PATH: &str = "steam_settings/achievements.json";
    // releative to goldberg_path/Appid
    const DEFAULT_GOLDBERG_NAME: &str = "Goldberg SteamEmu Saves/";
    const DEFAULT_SETTING_PATH: &str = "achievement_reminder_setting.json";
    const DEFAULT_ACHIEVEMENTS_PATH: &str = "achievements.json";

    fn get_default_app_data_path() -> String {
        dirs::data_dir()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string()
            + "/"
    }

    fn get_default_goldberg_path() -> String {
        Self::get_default_app_data_path() + Self::DEFAULT_GOLDBERG_NAME
    }

    fn get_default_setting_path() -> String {
        Self::get_default_goldberg_path() + Self::DEFAULT_SETTING_PATH
    }

    fn get_setting_path_args(args: &Args) -> String {
        if let Some(sp) = &args.settingpath {
            sp.clone()
        } else {
            Self::get_default_setting_path()
        }
    }
}

impl Default for Setting {
    fn default() -> Self {
        Self {
            fonts: Some(Self::DEFAULT_FONTS.iter().map(|s| s.to_string()).collect()),
            languages: Some(
                Self::DEFAULT_LANGUAGES
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            ),
            app_data_path: Some(Self::get_default_app_data_path()),
            goldberg_path: Some(Self::get_default_goldberg_path()),
            setting_path: Some(Self::get_default_setting_path()),
            image_dir: Some(Self::DEFAULT_IMAGE_DIR.to_string()),
            args: Default::default(),
        }
    }
}

impl Setting {
    pub fn new() -> Self {
        let args = Args::try_parse().unwrap_or_default();
        let slf = if let Ok(s) = std::fs::read_to_string(Self::get_setting_path_args(&args)) {
            serde_json::from_str(&s).unwrap_or_default()
        } else {
            Self::default()
        };
        slf.with_args(args)
    }

    fn with_args(mut self, args: Args) -> Self {
        self.args = args;
        self
    }

    pub fn get_fonts(&self) -> Vec<String> {
        if let Some(fonts) = &self.fonts {
            fonts.clone()
        } else {
            Self::DEFAULT_FONTS.iter().map(|s| s.to_string()).collect()
        }
    }

    pub fn get_languages(&self) -> Vec<String> {
        if let Some(languages) = &self.languages {
            languages.clone()
        } else {
            Self::DEFAULT_LANGUAGES
                .iter()
                .map(|s| s.to_string())
                .collect()
        }
    }

    pub fn get_app_data_path(&self) -> String {
        if let Some(path) = &self.app_data_path {
            path.clone()
        } else {
            Self::get_default_app_data_path()
        }
    }

    pub fn get_setting_path(&self) -> String {
        if let Some(path) = &self.app_data_path {
            path.clone()
        } else {
            Self::get_default_app_data_path()
        }
    }

    pub fn get_image_dir(&self) -> String {
        if let Some(path) = &self.image_dir {
            path.clone()
        } else {
            Self::DEFAULT_IMAGE_DIR.to_string()
        }
    }

    pub fn get_app_id(&self) -> u32 {
        if let Some(id) = self.args.appid {
            return id;
        }
        if let Ok(s) = fs::read_to_string(Self::DEFAULT_APP_ID_PATH_1) {
            if let Some(id) = ini::macro_safe_read(&s)
                .ok()
                .and_then(|f| f.get("steamclient")?.get("appid")?.to_owned()?.parse().ok())
            {
                return id;
            }
        }
        if let Ok(s) = fs::read_to_string(Self::DEFAULT_APP_ID_PATH_1) {
            if let Ok(id) = s.trim().to_owned().parse() {
                return id;
            }
        }
        panic!("Can not find app id from files or command line!")
    }

    pub fn get_goldberg_path(&self) -> String {
        if let Some(path) = &self.goldberg_path {
            path.clone()
        } else {
            self.get_app_data_path() + Self::DEFAULT_GOLDBERG_NAME
        }
    }

    pub fn get_achievement_data_path(&self) -> String {
        if let Some(path) = &self.args.datadir {
            path.clone()
        } else {
            format!(
                "{}{}/{}",
                self.get_goldberg_path(),
                self.get_app_id(),
                Self::DEFAULT_ACHIEVEMENTS_PATH
            )
        }
    }
    pub fn get_achievement_json_path(&self) -> String {
        if let Some(path) = &self.args.jsondir {
            path.clone()
        } else {
            Self::DEFAULT_ACHIEVEMENTS_DATA_PATH.to_string()
        }
    }
}
