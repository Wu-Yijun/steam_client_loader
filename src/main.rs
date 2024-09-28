use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use notify::{RecursiveMode, Watcher};
use setting::Setting;

mod achievement;
mod fonts;
mod setting;

use std::sync::mpsc;

use eframe::egui::{self, RichText};

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Steam Achievements Reminder",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
    .unwrap();
    println!("Terminate successfully!");
}

#[derive(PartialEq)]
enum AppWindow {
    Main,
    Achievement,
}

enum AppCmd {
    AddAchievements(AppShownAchievement),
    UpdateAppAchievement(Vec<achievement::AppAchievement>),
    Close,
}

struct MyApp {
    app: AppWindow,
    window_pos: egui::Pos2,
    window_size: egui::Vec2,
    title_bar: f32,
    visiblilty: bool,

    achievements: Vec<AppShownAchievement>,
    achievement: Option<AppShownAchievement>,
    time_left: f32,
    start_time: std::time::Instant,
    acheivement_align: egui::Align2,

    sfx: SoundEffects,

    sender: mpsc::Sender<AppCmd>,
    receiver: mpsc::Receiver<AppCmd>,
    watcher: Option<notify::ReadDirectoryChangesWatcher>,
    app_achievenemt: Vec<achievement::AppAchievement>,
    send_app_achievenemt: Arc<Mutex<bool>>,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after_secs(1.0);
        // println!("12345");
        while let Ok(recv) = self.receiver.try_recv() {
            match recv {
                AppCmd::AddAchievements(achievement) => self.add_achievement(ctx, achievement),
                AppCmd::Close => self.close(ctx),
                AppCmd::UpdateAppAchievement(vec) => {
                    self.app_achievenemt = vec;
                    *self.send_app_achievenemt.lock().unwrap() = false;
                }
            }
        }
        match self.app {
            AppWindow::Main => self.main_window(ctx),
            AppWindow::Achievement if self.visiblilty => {
                if self.time_left <= 0.001 {
                    self.achievement = self.achievements.pop();
                    self.time_left = 10.0;
                    if !self.achievement.is_none() {
                        if self.achievement.as_ref().unwrap().get {
                            self.sfx.play_get();
                        } else {
                            self.sfx.play_lose();
                        }
                        self.start_time = std::time::Instant::now();
                    }
                }
                if self.achievement.is_none() {
                    self.time_left = 0.0;
                    self.hide(ctx);
                    return;
                }
                self.achievement_window(ctx);
                let end_time = std::time::Instant::now();
                self.time_left -= (end_time - self.start_time).as_secs_f32();
                self.start_time = end_time;
            }
            AppWindow::Achievement => {
                if !self.achievements.is_empty() || self.achievement.is_some() {
                    self.show(ctx);
                    return;
                }
                // println!("Frames:");
            }
        }
    }
}

impl MyApp {
    const ACHIEVEMENT_WINDOW_SIZE: (f32, f32) = (500.0, 150.0);
    fn new(cc: &eframe::CreationContext) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        // MyApp::load_fonts(&cc.egui_ctx);
        let setting = setting::Setting::new();
        fonts::load_system_font(&cc.egui_ctx, &setting);
        let (sender, receiver) = mpsc::channel();
        let send_app_achievenemt = Arc::new(Mutex::new(false));
        let watcher =
            Self::file_monitor_start(sender.clone(), Arc::clone(&send_app_achievenemt), &setting);
        Self {
            app: AppWindow::Main,
            window_pos: [0.0, 0.0].into(),
            window_size: [600.0, 400.0].into(),
            title_bar: 50.0,
            visiblilty: true,
            achievements: vec![],
            achievement: None,
            time_left: 0.0,
            start_time: std::time::Instant::now(),
            acheivement_align: egui::Align2::RIGHT_BOTTOM,
            sfx: SoundEffects::new(),
            sender,
            receiver,
            watcher,
            app_achievenemt: vec![],
            send_app_achievenemt,
        }
    }
    fn file_monitor_start(
        sender: mpsc::Sender<AppCmd>,
        send_app_achievenemt: Arc<Mutex<bool>>,
        setting: &Setting,
    ) -> Option<notify::ReadDirectoryChangesWatcher> {
        let achievements_raw: achievement::AchievementsRaw =
            achievement::AchievementsRaw::new(setting);
        let mut achievements = achievement::Achievements::new(setting);

        sender
            .send(AppCmd::UpdateAppAchievement(
                achievements_raw.get_achievements(&achievements),
            ))
            .unwrap();

        // Automatically select the best implementation for your platform.
        let mut watcher = notify::recommended_watcher(move |res| match res {
            Ok(_) => {
                if let Some(updated) = achievements.update() {
                    // get achievement
                    for name in updated.0 {
                        if let Some(achievement) = achievements_raw.get(&name) {
                            let title = achievements_raw.get_display_name(achievement);
                            let text = achievements_raw.get_description(achievement);
                            let image = achievements_raw.get_icon(achievement);
                            let time = achievements.get_time(&name).unwrap();
                            println!("Achievement get: {:#?}", (&title, &text, &time, &image));
                            sender
                                .send(AppCmd::AddAchievements(AppShownAchievement {
                                    get: true,
                                    header: "Achievement get".into(),
                                    text_header: title,
                                    text,
                                    note: time,
                                    image: image.as_os_str().to_str().unwrap().into(),
                                }))
                                .unwrap();
                        }
                    }
                    // lose achievement
                    for name in updated.1 {
                        if let Some(achievement) = achievements_raw.get(&name) {
                            let title = achievements_raw.get_display_name(achievement);
                            let text = achievements_raw.get_description(achievement);
                            let image = achievements_raw.get_icon_gray(achievement);
                            let time = achievements.get_time(&name).unwrap();
                            println!("Achievement lose: {:#?}", (&title, &text, &time, &image));
                            sender
                                .send(AppCmd::AddAchievements(AppShownAchievement {
                                    get: false,
                                    header: "Achievement lose".into(),
                                    text_header: title,
                                    text,
                                    note: time,
                                    image: image.as_os_str().to_str().unwrap().into(),
                                }))
                                .unwrap();
                        }
                    }
                }
                if *send_app_achievenemt.lock().unwrap() {
                    sender
                        .send(AppCmd::UpdateAppAchievement(
                            achievements_raw.get_achievements(&achievements),
                        ))
                        .unwrap();
                    *send_app_achievenemt.lock().unwrap() = true;
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        })
        .unwrap();

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher
            .watch(
                &PathBuf::from(setting.get_achievement_json_path()),
                RecursiveMode::Recursive,
            )
            .unwrap();
        Some(watcher)
    }
    fn main_window(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("Steam Achievements Reminder")
            .min_height(60.0)
            .show(ctx, |ui| {
                // ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                //     // 按钮代码
                //     let btn_run = RichText::new("⬤ Run Manager!")
                //     .color(egui::Color32::DARK_GREEN)
                //     .size(30.0);
                // if ui.button(btn_run).clicked() {
                //     self.to_achievement_window(ctx);
                // }
                // let btn_exit = RichText::new("⬤ Close!")
                //     .color(egui::Color32::RED)
                //     .size(30.0);
                // if ui.button(btn_exit).clicked() {
                //     self.sender.send(AppCmd::Close).unwrap();
                // }

                // });
                ui.allocate_space([10.0, 10.0].into());
                ui.horizontal(|ui| {
                    ui.allocate_space([10.0, 10.0].into());
                    let btn_run = RichText::new("⬤ Run Manager!")
                        .color(egui::Color32::DARK_GREEN)
                        .size(30.0);
                    if ui.button(btn_run).clicked() {
                        self.to_achievement_window(ctx);
                    }
                    ui.allocate_space([20.0, 10.0].into());
                    let btn_exit = RichText::new("⬤ Close!")
                        .color(egui::Color32::RED)
                        .size(30.0);
                    if ui.button(btn_exit).clicked() {
                        self.sender.send(AppCmd::Close).unwrap();
                    }
                });
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_table(ui);
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
            let height = ui.max_rect().height();
            ui.horizontal(|ui| {
                let ac = self.achievement.as_ref().unwrap();
                ui.add(
                    egui::Image::new(&format!("file://{}", ac.image))
                        .fit_to_exact_size([height, height].into())
                        .rounding(height / 10.0),
                );
                ui.vertical_centered(|ui| {
                    ui.heading(&ac.header);
                    ui.label(&ac.text_header);
                    ui.label(&ac.text);
                    ui.label(&ac.note);
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
    fn add_achievement(&mut self, ctx: &egui::Context, achievement: AppShownAchievement) {
        println!("Add achievement: {}", achievement.text_header);
        self.achievements.push(achievement);
        if self.app == AppWindow::Achievement && !self.visiblilty {
            self.show(ctx);
        }
    }
    fn hide(&mut self, ctx: &egui::Context) {
        println!("Hide view");
        // ctx.send_viewport_cmd(egui::ViewportCommand::Transparent(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize([0.0, 0.0].into()));
        ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(true));

        self.visiblilty = false;
    }
    fn show(&mut self, ctx: &egui::Context) {
        println!("Show view");
        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
            Self::ACHIEVEMENT_WINDOW_SIZE.into(),
        ));
        ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(false));
        self.visiblilty = true;
    }
    fn close(&mut self, ctx: &egui::Context) {
        drop(self.watcher.take());
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    fn draw_table(&self, ui: &mut egui::Ui) {
        let available_height = ui.available_height();
        let table = egui_extras::TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(egui_extras::Column::auto())
            .column(egui_extras::Column::auto())
            .column(egui_extras::Column::auto())
            .column(egui_extras::Column::auto().clip(true).at_least(60.0))
            .column(egui_extras::Column::auto())
            .column(egui_extras::Column::auto().clip(true).at_least(60.0))
            .column(egui_extras::Column::remainder().clip(true).at_least(60.0))
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height)
            .sense(egui::Sense::click());
        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Id");
                });
                header.col(|ui| {
                    ui.strong("Icon");
                });
                header.col(|ui| {
                    ui.strong("State");
                });
                header.col(|ui| {
                    ui.strong("Date");
                });
                header.col(|ui| {
                    ui.strong("Visibility");
                });
                header.col(|ui| {
                    ui.strong("Title");
                });
                header.col(|ui| {
                    ui.strong("Description");
                });
            })
            .body(|mut body| {
                for ac in &self.app_achievenemt {
                    body.row(45.0, |mut row| {
                        // row.set_selected(self.selection.contains(&row_index));
                        row.col(|ui| {
                            ui.label(egui::RichText::new(&ac.id).size(16.0));
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Image::new(&format!("file://{}", ac.icon))
                                    .fit_to_exact_size([40.0, 40.0].into())
                                    .rounding(5.0),
                            );
                        });
                        row.col(|ui| {
                            if ac.state {
                                ui.label(
                                    egui::RichText::new("Achieved!")
                                        .size(16.0)
                                        .color(egui::Color32::DARK_GREEN),
                                );
                            } else {
                                ui.add(egui::Separator::default().horizontal());
                            }
                        });
                        row.col(|ui| {
                            if ac.date.is_empty() {
                                ui.add(egui::Separator::default().horizontal());
                            } else {
                                ui.label(&ac.date);
                            }
                        });
                        row.col(|ui| {
                            if ac.visiblity {
                                ui.label(
                                    egui::RichText::new("✅")
                                        .size(16.0)
                                        .color(egui::Color32::GREEN),
                                );
                            } else {
                                ui.label(
                                    egui::RichText::new("❌")
                                        .size(16.0)
                                        .color(egui::Color32::RED),
                                );
                            }
                        });
                        row.col(|ui| {
                            ui.label(egui::RichText::new(&ac.title).heading().size(18.0));
                        });
                        row.col(|ui| {
                            // NOTE: `Label` overrides some of the wrapping settings, e.g. wrap width
                            if ac.visiblity {
                                ui.label(
                                    egui::RichText::new(&ac.description)
                                        .size(16.0)
                                        .color(egui::Color32::DARK_GRAY),
                                );
                            } else {
                                ui.label(
                                    egui::RichText::new(&ac.description)
                                        .size(16.0)
                                        .color(egui::Color32::GRAY),
                                );
                            }
                        });
                    });
                }
            });
    }
}

struct AppShownAchievement {
    pub get: bool,
    pub header: String,
    pub text_header: String,
    pub text: String,
    pub note: String,
    pub image: String,
}

struct SoundEffects {
    _stream: rodio::OutputStream,
    _handle: rodio::OutputStreamHandle,
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
        let (_stream, _handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&_handle).unwrap();
        Self {
            _stream,
            _handle,
            sink,
        }
    }
    fn play_get(&self) {
        let source1 = rodio::Decoder::new(std::io::Cursor::new(Self::BYTES[2])).unwrap();
        let source2 = rodio::Decoder::new(std::io::Cursor::new(Self::BYTES[1])).unwrap();

        self.sink.append(source1);
        self.sink.append(source2);
    }
    fn play_lose(&self) {
        let source1 = rodio::Decoder::new(std::io::Cursor::new(Self::BYTES[0])).unwrap();
        let source2 = rodio::Decoder::new(std::io::Cursor::new(Self::BYTES[3])).unwrap();

        self.sink.append(source1);
        self.sink.append(source2);
    }
}
