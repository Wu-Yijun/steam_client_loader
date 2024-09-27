use std::{collections::HashMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};
type Name = String;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Achievement {
    pub earned: bool,
    pub earned_time: u32,
}

#[derive(Clone, Debug, Default)]
pub struct Achievements {
    pub achievements: HashMap<Name, Achievement>,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[allow(non_snake_case)]
pub struct AchievementRaw {
    pub hidden: String,
    pub displayName: HashMap<Name, String>,
    pub description: HashMap<Name, String>,
    pub icon: String,
    pub icon_gray: String,
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct AchievementsRaw {
    pub achievements: Vec<AchievementRaw>,
}

impl Achievements {
    pub fn new(app_id: String) -> Self {
        let mut path = dirs::config_dir().unwrap();
        // let mut path = PathBuf::from(".\\AppData");
        path.push("Goldberg SteamEmu Saves");
        path.push(app_id);
        path.push("achievements.json");
        println!("{:?}", path);
        let achievements = fs::read_to_string(path.clone()).unwrap();
        let achievements: HashMap<Name, Achievement> = serde_json::from_str(&achievements).unwrap();
        Self { achievements, path }
    }
    /// update the achievements from the file
    /// return the updated achievements name
    pub fn update(&mut self) -> Option<(Vec<Name>, Vec<Name>)> {
        let achievements = fs::read_to_string(self.path.clone()).ok()?;
        let achievements: HashMap<Name, Achievement> = serde_json::from_str(&achievements).ok()?;
        let mut updated = (vec![], vec![]);
        for (name, achievement) in &achievements {
            if let Some(ac) = self.achievements.get(name) {
                if !ac.earned && achievement.earned {
                    updated.0.push(name.clone());
                } else if ac.earned && !achievement.earned {
                    updated.1.push(name.clone());
                }
            }
        }
        self.achievements = achievements;
        Some(updated)
    }
}

impl AchievementsRaw {
    /// read achievements from ./steam_settings/achievements.json
    pub fn new() -> Self {
        let achievements = fs::read_to_string("./steam_settings/achievements.json").unwrap();
        let achievements: Vec<AchievementRaw> = serde_json::from_str(&achievements).unwrap();
        Self { achievements }
    }

    /// get the achievement by name
    pub fn get(&self, name: &str) -> Option<&AchievementRaw> {
        self.achievements
            .iter()
            .find(|achievement| achievement.name == name)
    }
}

impl AchievementRaw {
    const LANGUAGE_LIST: [&'static str; 6] = [
        "schinese", "tchinese", "chinese", "english", "japanese", "french",
    ];
    pub fn get_display_name(&self) -> String {
        for language in Self::LANGUAGE_LIST {
            if let Some(display_name) = self.displayName.get(language) {
                return display_name.clone();
            }
        }
        self.displayName.values().next().unwrap().clone()
    }
    pub fn get_description(&self) -> String {
        for language in Self::LANGUAGE_LIST {
            if let Some(description) = self.description.get(language) {
                return description.clone();
            }
        }
        self.description.values().next().unwrap().clone()
    }
    /// search path:
    /// 1. self.icon
    /// 2. ./steam_settings/achievement_images/{self.icon}
    /// 3. ./steam_settings/achievement_images/{self.name}
    /// Otherwise, return self.icon
    pub fn get_icon(&self) -> PathBuf {
        
        // 1. self.icon
        let path = PathBuf::from(&self.icon);
        if path.exists() {
            return std::path::absolute(path).unwrap();
        }
        // 2. ./steam_settings/achievement_images/{self.icon}
        let mut path = PathBuf::from("./steam_settings/achievement_images");
        path.push(&self.icon);
        if path.exists() {
            return std::path::absolute(path).unwrap();
        }
        // 3. ./steam_settings/achievement_images/{self.name}
        path.pop();
        path.push(&self.name);
        if path.exists() {
            return std::path::absolute(path).unwrap();
        }
        // Otherwise, return self.icon
        self.icon.clone().into()
    }

    /// search path:
    /// 1. self.icon_gray
    /// 2. ./steam_settings/achievement_images/{self.icon_gray}
    /// 3. ./steam_settings/achievement_images/{self.name}
    /// Otherwise, return self.icon_gray
    pub fn get_icon_gray(&self) -> PathBuf {
        // 1. self.icon
        let path = PathBuf::from(&self.icon_gray);
        if path.exists() {
            return std::path::absolute(path).unwrap();
        }
        // 2. ./steam_settings/achievement_images/{self.icon}
        let mut path = PathBuf::from("./steam_settings/achievement_images");
        path.push(&self.icon_gray);
        if path.exists() {
            return std::path::absolute(path).unwrap();
        }
        // 3. ./steam_settings/achievement_images/{self.name}
        path.pop();
        path.push(&self.name);
        if path.exists() {
            return std::path::absolute(path).unwrap();
        }
        // Otherwise, return self.icon
        self.icon_gray.clone().into()
    }
}
