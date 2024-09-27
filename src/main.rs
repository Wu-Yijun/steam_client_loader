use std::{fs, path::Path, time};

use ini;
use notify::{RecursiveMode, Watcher};
use winrt_notification::{Sound, Toast};

mod achievement;

fn main() {
    let app_id = fs::read_to_string(Path::new("ColdClientLoader.ini")).unwrap();
    let app_id = ini::macro_safe_read(&app_id)
        .unwrap()
        .get("steamclient")
        .unwrap()
        .get("appid")
        .unwrap()
        .to_owned()
        .unwrap();
    println!("app_id: {:#?}", app_id);
    let achievements_raw: achievement::AchievementsRaw = achievement::AchievementsRaw::new();
    let mut achievements = achievement::Achievements::new(app_id);
    let path = achievements.path.clone();

    // Automatically select the best implementation for your platform.
    let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(event) => {
            println!("File modified: {:?}", event);
            if let Some(updated) = achievements.update() {
                // get achievement
                for name in updated.0 {
                    if let Some(achievement) = achievements_raw.get(&name) {
                        let title = achievement.get_display_name();
                        let text = achievement.get_description();
                        let image = achievement.get_icon().into_os_string();
                        Toast::new(Toast::POWERSHELL_APP_ID)
                            .title(&title)
                            .text1(&text)
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
                        let text1 = achievement.get_display_name();
                        let text2 = achievement.get_description();
                        let image = achievement.get_icon_gray();
                        Toast::new(Toast::POWERSHELL_APP_ID)
                            .title(">>>--- Lose achievement ---<<<")
                            .text1(&text1)
                            .text2(&text2)
                            .image(&image, "Image cannot find")
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
