use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};

use crate::setting::Setting;
type Name = String;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Achievement {
    pub earned: bool,
    pub earned_time: u64,
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
    pub image_dir: PathBuf,
    pub languages: Vec<String>,
}

impl Achievement {
    pub fn get_time(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(self.earned_time)
    }
    pub fn get_time_string(&self) -> String {
        let time = self.get_time();
        let datetime: chrono::DateTime<chrono::offset::Local> = time.into();
        datetime.format("%Y-%m-%d %T").to_string()
    }
}
impl Achievements {
    /// Create a new Achievements from the path
    pub fn new(setting: &Setting) -> Self {
        let path = PathBuf::from(setting.get_achievement_data_path());
        let achievements = fs::read_to_string(&path).unwrap();
        let achievements: HashMap<Name, Achievement> = serde_json::from_str(&achievements).unwrap();
        Self { achievements, path }
    }
    /// update the achievements from the file
    /// return the updated achievements name
    pub fn update(&mut self) -> Option<(Vec<Name>, Vec<Name>)> {
        let achievements = fs::read_to_string(&self.path).ok()?;
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

    pub fn get_time(&self, name: &str) -> Option<String> {
        self.achievements
            .get(name)
            .map(|achievement| achievement.get_time_string())
    }
}

impl AchievementsRaw {
    /// read achievements from path(./steam_settings/achievements.json)
    pub fn new(setting: &Setting) -> Self {
        let achievements = fs::read_to_string(setting.get_achievement_json_path()).unwrap();
        let achievements: Vec<AchievementRaw> = serde_json::from_str(&achievements).unwrap();
        Self {
            achievements,
            image_dir: PathBuf::from(setting.get_image_dir()),
            languages: setting.get_languages(),
        }
    }

    /// get the achievement by name
    pub fn get(&self, name: &str) -> Option<&AchievementRaw> {
        self.achievements
            .iter()
            .find(|achievement| achievement.name == name)
    }

    const LANGUAGE_LIST: [&'static str; 6] = [
        "schinese", "tchinese", "chinese", "english", "japanese", "french",
    ];
    pub fn get_display_name(&self, achievement: &AchievementRaw) -> String {
        for language in &self.languages {
            if let Some(display_name) = achievement.displayName.get(language) {
                return display_name.clone();
            }
        }
        for language in Self::LANGUAGE_LIST {
            if let Some(display_name) = achievement.displayName.get(language) {
                return display_name.clone();
            }
        }
        achievement.displayName.values().next().unwrap().clone()
    }
    pub fn get_description(&self, achievement: &AchievementRaw) -> String {
        for language in &self.languages {
            if let Some(description) = achievement.description.get(language) {
                return description.clone();
            }
        }
        for language in Self::LANGUAGE_LIST {
            if let Some(description) = achievement.description.get(language) {
                return description.clone();
            }
        }
        achievement.description.values().next().unwrap().clone()
    }

    /// search path:
    /// 1. achievement.icon
    /// 2. ./steam_settings/achievement_images/{achievement.icon}
    /// 3. ./steam_settings/achievement_images/{achievement.name}
    /// Otherwise, return achievement.icon
    pub fn get_icon(&self, achievement: &AchievementRaw) -> PathBuf {
        // 1. achievement.icon
        let path = PathBuf::from(&achievement.icon);
        if path.exists() {
            return std::path::absolute(path).unwrap();
        }
        // 2. ./steam_settings/achievement_images/{achievement.icon}
        let mut path = self.image_dir.to_owned();
        path.push(&achievement.icon);
        if path.exists() {
            return std::path::absolute(path).unwrap();
        }
        // 3. ./steam_settings/achievement_images/{achievement.name}
        path.pop();
        path.push(&achievement.name);
        if path.exists() {
            return std::path::absolute(path).unwrap();
        }
        // Otherwise, return achievement.icon
        achievement.icon.clone().into()
    }

    /// search path:
    /// 1. achievement.icon_gray
    /// 2. ./steam_settings/achievement_images/{achievement.icon_gray}
    /// 3. ./steam_settings/achievement_images/{achievement.name}
    /// Otherwise, return achievement.icon_gray
    pub fn get_icon_gray(&self, achievement: &AchievementRaw) -> PathBuf {
        // 1. achievement.icon
        let path = PathBuf::from(&achievement.icon_gray);
        if path.exists() {
            return std::path::absolute(path).unwrap();
        }
        // 2. ./steam_settings/achievement_images/{achievement.icon}
        let mut path = self.image_dir.to_owned();
        path.push(&achievement.icon_gray);
        if path.exists() {
            return std::path::absolute(path).unwrap();
        }
        // 3. ./steam_settings/achievement_images/{achievement.name}
        path.pop();
        path.push(&achievement.name);
        if path.exists() {
            return std::path::absolute(path).unwrap();
        }
        // Otherwise, return achievement.icon
        achievement.icon_gray.clone().into()
    }

    pub fn get_achievements(&self, achievements: &Achievements) -> Vec<AppAchievement> {
        let mut res = vec![];
        for a in &self.achievements {
            let (state, date) = achievements
                .achievements
                .get(&a.name)
                .and_then(|ac| {
                    if ac.earned {
                        Some((ac.earned, ac.get_time_string()))
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            let icon = self.get_icon(a).as_os_str().to_str().unwrap().to_string();
            let aa = AppAchievement {
                id: a.name.clone(),
                icon: icon,
                state: state,
                date: date,
                title: self.get_display_name(a),
                description: self.get_description(a),
                visiblity: a.hidden == "0",
            };
            res.push(aa);
        }
        res
    }
}

pub struct AppAchievement {
    pub id: String,
    pub icon: String,
    pub state: bool,
    pub date: String,
    pub title: String,
    pub description: String,
    pub visiblity: bool,
}
