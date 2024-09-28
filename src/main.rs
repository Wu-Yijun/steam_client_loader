use std::{
    fs,
    path::{Path, PathBuf},
    time,
};

use clap::Parser;
use ini;
use notify::{RecursiveMode, Watcher};
use winrt_notification::{Sound, Toast};

mod achievement;

#[derive(Parser, Debug)]
#[command(version = "0.1.1", about = "A tool for visiualization steam achievements.", long_about = None)]
struct Args {
    /// Appid of the game. If not provided, it will read from ColdClientLoader.ini or steam_settings/steam_appid.txt
    #[arg(short, long)]
    appid: Option<u32>,
    /// Directory of the achievements image. If not provided, it will use the default directory steam_settings/achievement_images/.
    #[arg(short, long)]
    imagedir: Option<PathBuf>,
    /// Path of the achievements information. If not provided, it will use the default directory steam_settings/achievements.json.
    #[arg(short, long)]
    datadir: Option<PathBuf>,
    /// Path of the achievements statistical data. If not provided, it will use the default directory %APPDATA%/Goldberg SteamEmu Saves/${AppId}/achievements.json.
    #[arg(short, long)]
    jsondir: Option<PathBuf>,
    /// Default languages list of the achievements. If not provided, it will use the list ["schinese", "tchinese", "chinese", "english", "japanese", "french"].
    /// The display language will be the first language that can be found in the list.
    #[arg(short, long, value_parser, num_args = 1.., value_delimiter = ' ')]
    languages: Option<Vec<String>>,
}

fn main() {
    let args = Args::parse();
    println!("Command Line {:#?}", args);
    let app_id = if let Some(app_id) = args.appid {
        app_id.to_string()
    } else if fs::exists(Path::new("ColdClientLoader.ini")).unwrap() {
        let app_id = fs::read_to_string(Path::new("ColdClientLoader.ini")).unwrap();
        let app_id = ini::macro_safe_read(&app_id)
            .unwrap()
            .get("steamclient")
            .unwrap()
            .get("appid")
            .unwrap()
            .to_owned()
            .unwrap();
        app_id
    } else if fs::exists(Path::new("steam_settings/steam_appid.txt")).unwrap() {
        let app_id = fs::read_to_string(Path::new("steam_settings/steam_appid.txt")).unwrap();
        app_id.trim().to_owned()
    } else {
        panic!("Appid not found")
    };
    println!("app_id: {:#?}", app_id);
    let achievements_raw: achievement::AchievementsRaw = achievement::AchievementsRaw::new(
        args.datadir
            .unwrap_or(PathBuf::from("./steam_settings/achievements.json")),
        args.imagedir
            .unwrap_or(PathBuf::from("./steam_settings/achievement_images/")),
        args.languages.unwrap_or_default(),
    );
    let mut achievements = if let Some(path) = args.jsondir {
        achievement::Achievements::new(path)
    } else {
        achievement::Achievements::from(app_id)
    };
    let path = achievements.path.clone();

    // Automatically select the best implementation for your platform.
    let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(_) => {
            println!("File modified!");
            if let Some(updated) = achievements.update() {
                // get achievement
                for name in updated.0 {
                    if let Some(achievement) = achievements_raw.get(&name) {
                        let title = achievements_raw.get_display_name(achievement);
                        let text = achievements_raw.get_description(achievement);
                        let image = achievements_raw.get_icon(achievement).into_os_string();
                        let time = achievements.get_time(&name).unwrap();
                        let datetime: chrono::DateTime<chrono::offset::Local> = time.into();
                        let time = datetime.format("%Y-%m-%d %T").to_string();
                        println!("Achievement get: {:#?}", (&title, &text, &time, &image));
                        Toast::new(Toast::POWERSHELL_APP_ID)
                            .title(&title)
                            .text1(&text)
                            .text2(&time)
                            .image(Path::new(&image), "Image cannot find")
                            .sound(Some(Sound::Reminder))
                            .duration(winrt_notification::Duration::Short)
                            .show()
                            .expect("unable to toast");
                    }
                }
                // lose achievement
                for name in updated.1 {
                    if let Some(achievement) = achievements_raw.get(&name) {
                        let text1 = achievements_raw.get_display_name(achievement);
                        let text2 = achievements_raw.get_description(achievement);
                        let image = achievements_raw.get_icon_gray(achievement).into_os_string();
                        let time = achievements.get_time(&name).unwrap();
                        let datetime: chrono::DateTime<chrono::offset::Local> = time.into();
                        let time = datetime.format("%Y-%m-%d %T").to_string();
                        println!("Achievement lose: {:#?}", (&text1, &text2, &time, &image));
                        Toast::new(Toast::POWERSHELL_APP_ID)
                            .title(">>>--- Lose achievement ---<<<")
                            .text1(&text1)
                            .text2(&format!("{}\n{}", &text2, &time))
                            .image(Path::new(&image), "Image cannot find")
                            .sound(Some(Sound::SMS))
                            .duration(winrt_notification::Duration::Short)
                            .show()
                            .expect("unable to toast");
                    }
                }
            }
        }
        Err(e) => println!("watch error: {:?}", e),
    })
    .unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(&path, RecursiveMode::Recursive).unwrap();

    loop {
        std::thread::sleep(time::Duration::from_secs(60));
    }
}
