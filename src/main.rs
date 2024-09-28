/*
use std::{
    fs,
    path::{Path, PathBuf},
    time::{self, Duration},
};

use clap::Parser;
use notify::{RecursiveMode, Watcher};

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

fn test() {
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
                        // Toast::new(Toast::POWERSHELL_APP_ID)
                        //     .title(&title)
                        //     .text1(&text)
                        //     .text2(&time)
                        //     .image(Path::new(&image), "Image cannot find")
                        //     .sound(Some(Sound::Reminder))
                        //     .duration(winrt_notification::Duration::Short)
                        //     .show()
                        //     .expect("unable to toast");
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
                        // Toast::new(Toast::POWERSHELL_APP_ID)
                        //     .title(">>>--- Lose achievement ---<<<")
                        //     .text1(&text1)
                        //     .text2(&format!("{}\n{}", &text2, &time))
                        //     .image(Path::new(&image), "Image cannot find")
                        //     .sound(Some(Sound::SMS))
                        //     .duration(winrt_notification::Duration::Short)
                        //     .show()
                        //     .expect("unable to toast");
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
*/

use eframe::egui;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Steam Achievements Reminder",
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
    .unwrap();
}

enum AppWindow {
    Main,
    Achievement,
}

struct MyApp {
    app: AppWindow,
    window_pos: egui::Pos2,
    window_size: egui::Vec2,
    title_bar: f32,

    showed: AppShownAchievement,

    acheivement_align: egui::Align2,

    sfx: SoundEffects,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.app {
            AppWindow::Main => self.main_window(ctx),
            AppWindow::Achievement => self.achievement_window(ctx),
        }
    }
}

impl MyApp {
    const ACHIEVEMENT_WINDOW_SIZE: (f32, f32) = (500.0, 150.0);
    fn new(cc: &eframe::CreationContext) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Self {
            app: AppWindow::Main,
            window_pos: [0.0, 0.0].into(),
            window_size: [600.0, 400.0].into(),
            title_bar: 50.0,
            showed: AppShownAchievement::test(),
            acheivement_align: egui::Align2::RIGHT_BOTTOM,
            sfx: SoundEffects::new(),
        }
    }
    fn main_window(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello main_window!");
            ui.label("This is a demo app!");
            if ui.button("Top").clicked() {}
            if ui.button("Buttom").clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                    egui::WindowLevel::AlwaysOnBottom,
                ));
            }
            if ui.button("Achievement").clicked() {
                self.to_achievement_window(ctx);
            }
        });
    }
    fn to_achievement_window(&mut self, ctx: &egui::Context) {
        println!("--- Convert to achievement window! ---");
        self.app = AppWindow::Achievement;
        let moniter_size = ctx.input(|i| i.viewport().monitor_size.unwrap());
        let window_pos = ctx.input(|i| i.viewport().outer_rect.unwrap().min);
        let window_size = ctx.input(|i| i.viewport().inner_rect.unwrap().size());
        let title_bar = ctx.input(|i| i.viewport().outer_rect.unwrap().height()) - window_size.y;
        let new_pos = moniter_size - Self::ACHIEVEMENT_WINDOW_SIZE.into();
        println!("moniter_size: {:?}", moniter_size);
        println!("window_pos: {:?}", window_pos);
        println!("window_size: {:?}", window_size);
        println!("title_bar: {:?}", title_bar);
        println!("new_pos: {:?}", new_pos);
        {
            let x = match self.acheivement_align.0[0] {
                egui::Align::Min => Self::ACHIEVEMENT_WINDOW_SIZE.1 * 0.1,
                egui::Align::Center => moniter_size.x * 0.5 - Self::ACHIEVEMENT_WINDOW_SIZE.0 * 0.5,
                egui::Align::Max => {
                    moniter_size.x
                        - Self::ACHIEVEMENT_WINDOW_SIZE.0
                        - Self::ACHIEVEMENT_WINDOW_SIZE.1 * 0.1
                }
            };
            let y = match self.acheivement_align.0[1] {
                egui::Align::Min => Self::ACHIEVEMENT_WINDOW_SIZE.1 * 0.1,
                egui::Align::Center => moniter_size.y * 0.5 - Self::ACHIEVEMENT_WINDOW_SIZE.1 * 0.5,
                egui::Align::Max => {
                    moniter_size.y - Self::ACHIEVEMENT_WINDOW_SIZE.1 * 1.1 - title_bar
                }
            };
            // println!("Align: {:?}, pos: ({x}, {y})", self.acheivement_align.0);
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
                Self::ACHIEVEMENT_WINDOW_SIZE.into(),
            ));
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition((x, y).into()));
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
            egui::WindowLevel::AlwaysOnTop,
        ));
        self.window_pos = window_pos;
        self.window_size = window_size;
        self.title_bar = title_bar;
    }
    fn achievement_window(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // let rect = ui.allocate_rect(
            //     ui.max_rect(),
            //     egui::Sense {
            //         click: true,
            //         drag: false,
            //         focusable: false,
            //     },
            // );
            let height = ui.max_rect().height();
            ui.horizontal(|ui| {
                ui.add(
                    egui::Image::new(&format!("file://{}", self.showed.image))
                        .fit_to_exact_size([height, height / 2.0].into()),
                );
                ui.vertical(|ui| {
                    ui.heading(&self.showed.header);
                    ui.label(&self.showed.text_header);
                    ui.label(&self.showed.text);
                    ui.label(&self.showed.note);
                });
            });
            if ui.input(|r| r.pointer.primary_clicked()) {
                self.to_main_window(ctx);
            }
        });
    }
    fn to_main_window(&mut self, ctx: &egui::Context) {
        println!("--- Convert to main window! ---");
        self.app = AppWindow::Main;
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(self.window_pos));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(self.window_size));
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
            egui::WindowLevel::Normal,
        ));
    }
    fn add_achievement(&mut self, ctx: &egui::Context) {
        match self.app {
            AppWindow::Main => todo!(),
            AppWindow::Achievement => (),
        }
    }
}

struct AppShownAchievement {
    pub header: String,
    pub text_header: String,
    pub text: String,
    pub note: String,
    pub image: String,
}

impl AppShownAchievement {
    fn test() -> Self {
        Self {
            header: "Achievement Get!".to_string(),
            text_header: "Achievement 1".to_string(),
            text: "This is a test achievement!".to_string(),
            note: "Date: 2021-09-01 12:00:00".to_string(),
            image:
                "./steam_settings/achievement_images/0cc8155984cfe0580672966b15c475f14458138c.jpg"
                    .to_string(),
        }
    }
}

struct SoundEffects {
    stream: rodio::OutputStream,
    handle: rodio::OutputStreamHandle,
    sink: rodio::Sink,
}
impl SoundEffects {
    const BYTES: [&[u8]; 6] = [
        include_bytes!("../assets/sound1.mp3"),
        include_bytes!("../assets/sound2.mp3"),
        include_bytes!("../assets/sound3.mp3"),
        include_bytes!("../assets/sound4.mp3"),
        include_bytes!("../assets/sound5.mp3"),
        include_bytes!("../assets/sound6.mp3"),
    ];
    fn new() -> Self {
        let (stream, handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&handle).unwrap();
        Self {
            stream,
            handle,
            sink,
        }
    }
    fn play_get(&self) {
        let source1 = rodio::Decoder::new(std::io::Cursor::new(Self::BYTES[2])).unwrap();
        let source2 = rodio::Decoder::new(std::io::Cursor::new(Self::BYTES[1])).unwrap();

        self.sink.append(source1);
        self.sink.append(source2);
    }
}
