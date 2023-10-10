/***
*Author: nicheface nicheface@outlook.com
*Date: 2023-09-27 11:00:01
*LastEditors: nicheface nicheface@outlook.com
*LastEditTime: 2023-10-10 09:20:24
*FilePath: \\autodark-egui-rr-test\\src\\main.rs
*/

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::egui;
use chrono::Local;
use winreg::{
    RegKey, 
    enums::{HKEY_CURRENT_USER, KEY_READ},
};
fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(500.0, 500.0)),
        ..Default::default()
    };
    eframe::run_native( 
        "深浅主题模式自动切换软件",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    )
}

struct MyApp {
    is_dark_mode: bool,
    is_system_dark_mode: bool,
    is_system_both_dark_mode: bool,
    auto_mode_change: bool, // 添加自动模式切换的标志
    auto_system_mode_change: bool,
    custom_night_start: chrono::NaiveTime, // 新增字段
    custom_night_end: chrono::NaiveTime,   // 新增字段
    custom_night_start_hh: u32,
    custom_night_start_mm: u32,
    custom_night_end_hh: u32,
    custom_night_end_mm: u32,
    custom_system_night_start: chrono::NaiveTime, // 新增字段
    custom_system_night_end: chrono::NaiveTime,   // 新增字段
    custom_system_night_start_hh: u32,
    custom_system_night_start_mm: u32,
    custom_system_night_end_hh: u32,
    custom_system_night_end_mm: u32,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        Self {
            is_dark_mode: is_dark_mode_enabled(),
            is_system_dark_mode: is_system_dark_mode_enabled(),
            auto_mode_change: false, // 默认禁用自动模式切换
            auto_system_mode_change: false,
            is_system_both_dark_mode: false,
            custom_night_start: chrono::NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            custom_night_end: chrono::NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            custom_night_start_hh: 18,
            custom_night_start_mm: 0,
            custom_night_end_hh: 6,
            custom_night_end_mm: 0,
            custom_system_night_start: chrono::NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            custom_system_night_end: chrono::NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            custom_system_night_start_hh: 18,
            custom_system_night_start_mm: 0,
            custom_system_night_end_hh: 6,
            custom_system_night_end_mm: 0,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 在每次更新时检查主题模式并更新界面
        self.is_dark_mode = is_dark_mode_enabled();
        self.is_system_dark_mode = is_system_dark_mode_enabled();
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("深浅主题模式自动切换软件");

            ui.group(|ui| {
                ui.horizontal(|ui| {
                    // ui.label("System Theme same with App theme:");
                    ui.checkbox(
                        &mut self.is_system_both_dark_mode,
                        "将默认windows模式(任务栏)和默认应用模式(常规应用)设为相同",
                    );
                });
            });

            ui.group(|ui| {
                // 在这里添加您的控件
                // 添加单选框来切换颜色模式
                ui.horizontal(|ui| {
                    ui.label("默认应用模式(常规应用):");
                    // 直接根据 is_dark_mode 决定单选框的状态
                    ui.radio_value(&mut self.is_dark_mode, false, "浅色模式");
                    ui.radio_value(&mut self.is_dark_mode, true, "深色模式");
                });
                ui.add_space(10.0);
                // 添加自动模式切换的控件
                ui.horizontal(|ui| {
                    ui.checkbox(
                        &mut self.auto_mode_change,
                        "根据时间自动切换默认应用模式(常规应用)",
                    );
                    if self.auto_mode_change {
                        ui.label("自动切换默认应用模式(常规应用)已启用");
                    }
                });

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("自定义深色模式开始时间"));
                    ui.add(egui::Slider::new(&mut self.custom_night_start_hh, 0..=23).text("小时"));
                    ui.add(egui::Slider::new(&mut self.custom_night_start_mm, 0..=59).text("分钟"));
                });
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("自定义深色模式结束时间"));
                    ui.add(egui::Slider::new(&mut self.custom_night_end_hh, 0..=23).text("小时"));
                    ui.add(egui::Slider::new(&mut self.custom_night_end_mm, 0..=59).text("分钟"));
                });
                self.custom_night_start = chrono::NaiveTime::from_hms_opt(
                    self.custom_night_start_hh,
                    self.custom_night_start_mm,
                    0,
                )
                    .unwrap();
                self.custom_night_end = chrono::NaiveTime::from_hms_opt(
                    self.custom_night_end_hh,
                    self.custom_night_end_mm,
                    0,
                )
                    .unwrap();

                // 如果启用了自动模式切换，并且当前时间在指定的范围内，则切换主题模式
                if self.auto_mode_change {
                    let current_time = Local::now().time();

                    if current_time >= self.custom_night_start
                        || current_time < self.custom_night_end
                    {
                        self.is_dark_mode = true;
                        set_dark_mode(true);
                    } else {
                        self.is_dark_mode = false;
                        set_dark_mode(false);
                    }
                }
                ui.add_space(10.0);
                ui.add(egui::Label::new(
                    "当前设定的默认应用模式(常规应用)深色模式时间范围是：".to_owned()
                        + &self.custom_night_start.to_string()
                        + " - "
                        + &self.custom_night_end.to_string(),
                ));
            });
            if !self.is_system_both_dark_mode {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("默认windows模式(任务栏)");
                        // 直接根据 is_dark_mode 决定单选框的状态
                        ui.radio_value(&mut self.is_system_dark_mode, false, "浅色模式");
                        ui.radio_value(&mut self.is_system_dark_mode, true, "深色模式");
                    });
                    ui.add_space(10.0);
                    // 添加自动模式切换的控件
                    ui.horizontal(|ui| {
                        ui.checkbox(
                            &mut self.auto_system_mode_change,
                            "根据时间自动切换默认windows模式(任务栏)",
                        );
                        if self.auto_system_mode_change {
                            ui.label("自动切换默认windows模式(任务栏)已启用");
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new("自定义深色模式开始时间"));
                        ui.add(
                            egui::Slider::new(&mut self.custom_system_night_start_hh, 0..=23)
                                .text("小时"),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.custom_system_night_start_mm, 0..=59)
                                .text("分钟"),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new("自定义深色模式结束时间"));
                        ui.add(
                            egui::Slider::new(&mut self.custom_system_night_end_hh, 0..=23)
                                .text("小时"),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.custom_system_night_end_mm, 0..=59)
                                .text("分钟"),
                        );
                    });
                    self.custom_system_night_start = chrono::NaiveTime::from_hms_opt(
                        self.custom_system_night_start_hh,
                        self.custom_system_night_start_mm,
                        0,
                    )
                        .unwrap();
                    self.custom_system_night_end = chrono::NaiveTime::from_hms_opt(
                        self.custom_system_night_end_hh,
                        self.custom_system_night_end_mm,
                        0,
                    )
                        .unwrap();
                    // 如果启用了自动模式切换，并且当前时间在指定的范围内，则切换主题模式
                    if self.auto_system_mode_change {
                        let current_time = Local::now().time();

                        if current_time >= self.custom_system_night_start
                            || current_time < self.custom_system_night_end
                        {
                            self.is_system_dark_mode = true;
                            set_system_dark_mode(true);
                        } else {
                            self.is_system_dark_mode = false;
                            set_system_dark_mode(false);
                        }
                    }
                    ui.add_space(10.0);
                    ui.add(egui::Label::new(
                        "当前设定的默认windows模式(任务栏)深色模式时间范围是：".to_owned()
                            + &self.custom_system_night_start.to_string()
                            + " - "
                            + &self.custom_system_night_end.to_string(),
                    ));
                });
            }

            // 如果用户改变了设置，则更新系统的主题模式
            set_dark_mode(self.is_dark_mode);
            set_system_dark_mode(self.is_system_dark_mode);
            if self.is_system_both_dark_mode {
                set_system_dark_mode(self.is_dark_mode);
            }
            // 显示主题信息
            ui.horizontal(|ui| {
                ui.label("当前默认应用模式(常规应用)为");
                if self.is_dark_mode {
                    ui.label("深色模式");
                } else {
                    ui.label("浅色模式");
                }
            });

            ui.horizontal(|ui| {
                ui.label("当前默认windows模式(任务栏)为");
                if self.is_system_dark_mode {
                    ui.label("深色模式");
                } else {
                    ui.label("浅色模式");
                }
            });
        });
    }
}
// 使用 Windows API 检查系统是否处于深色模式
fn is_dark_mode_enabled() -> bool {
    // let output = Command::new("reg")
    //     .args(&[
    //         "query",
    //         "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
    //         "/v",
    //         "AppsUseLightTheme",
    //     ])
    //     .output()
    //     .expect("Failed to execute command");
    
    // let stdout = String::from_utf8_lossy(&output.stdout);
    
    // stdout.contains("0x0")
    let hklm = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(subkey) = hklm.open_subkey_with_flags(
        r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
        KEY_READ,
    ) {
        if let Ok(value) = subkey.get_value::<u32, _>("AppsUseLightTheme") {
            return value == 0;
        }
    }
    // Return false by default if there was an error or the value doesn't exist
    false
}
fn is_system_dark_mode_enabled() -> bool {
    // let output = Command::new("reg")
    //     .args(&[
    //         "query",
    //         "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
    //         "/v",
    //         "SystemUsesLightTheme",
    //     ])
    //     .output()
    //     .expect("Failed to execute command");

    // let stdout = String::from_utf8_lossy(&output.stdout);

    // stdout.contains("0x0")
    let hklm = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(subkey) = hklm.open_subkey_with_flags(
        r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
        KEY_READ,
    ) {
        if let Ok(value) = subkey.get_value::<u32, _>("SystemUsesLightTheme") {
            return value == 0;
        }
    }
    // Return false by default if there was an error or the value doesn't exist
    false
}

// fn set_dark_mode(enabled: bool) {
//     let value = if enabled { "0" } else { "1" };
//     Command::new("reg")
//         .args(&[
//             "add",
//             "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
//             "/v",
//             "AppsUseLightTheme",
//             "/t",
//             "REG_DWORD",
//             "/d",
//             value,
//             "/f",
//         ])
//         .output()
//         .expect("Failed to execute command");
// }
fn set_dark_mode(enabled: bool){
    let hkcu = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
            winreg::enums::KEY_SET_VALUE,
        )
        .unwrap();

    let value = if enabled { 0u32 } else { 1u32 };
    hkcu.set_value("AppsUseLightTheme", &value).unwrap();
}

// fn set_system_dark_mode(enabled: bool) {
//     let value = if enabled { "0" } else { "1" };
//     Command::new("reg")
//         .args(&[
//             "add",
//             "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
//             "/v",
//             "SystemUsesLightTheme",
//             "/t",
//             "REG_DWORD",
//             "/d",
//             value,
//             "/f",
//         ])
//         .output()
//         .expect("Failed to execute command");
// }
fn set_system_dark_mode(enabled: bool) {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
            winreg::enums::KEY_SET_VALUE,
        )
        .unwrap();

    let value = if enabled { 0u32 } else { 1u32 };
    hkcu.set_value("SystemUsesLightTheme", &value).unwrap();
}
fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../resources/fonts/SmileySans-Oblique.ttf")),
    );

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "my_font".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}
