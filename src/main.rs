#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use chrono::{Local, NaiveTime};
use eframe::egui;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::{env, fmt};

// 在文件开头添加引用

use winreg::enums::KEY_SET_VALUE;
use winreg::{
    enums::{HKEY_CURRENT_USER, KEY_READ},
    RegKey,
};

#[derive(Default)]
struct NaiveTimeWrapper(NaiveTime);
impl PartialEq for NaiveTimeWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for NaiveTimeWrapper {}

impl PartialOrd for NaiveTimeWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl Ord for NaiveTimeWrapper {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}
impl fmt::Display for NaiveTimeWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format("%H:%M:%S"))
    }
}
impl Serialize for NaiveTimeWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.format("%H:%M:%S").to_string())
    }
}
impl fmt::Debug for NaiveTimeWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl<'de> Deserialize<'de> for NaiveTimeWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let time_str = String::deserialize(deserializer)?;
        let naive_time =
            NaiveTime::parse_from_str(&time_str, "%H:%M:%S").map_err(serde::de::Error::custom)?;
        Ok(NaiveTimeWrapper(naive_time))
    }
}
fn save_config_to_toml(config: &MyApp, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let toml_string = toml::to_string_pretty(config)?;
    let mut file = File::create(filename)?;
    file.write_all(toml_string.as_bytes())?;
    Ok(())
}
fn load_config_from_toml(filename: &str) -> Result<MyApp, Box<dyn std::error::Error>> {
    let mut file = File::open(filename)?;
    let mut toml_string = String::new();
    file.read_to_string(&mut toml_string)?;
    let config: MyApp = toml::from_str(&toml_string)?;
    Ok(config)
}
// fn on_config_change(config: &MyApp) {
//     // 其他处理...
//     // 保存配置到文件
//     if let Err(e) = save_config_to_toml(config, "config.toml") {
//         eprintln!("Failed to save config: {}", e);
//     }
// }
fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 500.0]),
        ..Default::default()
    };
    let config_filename = "config.toml";
    let config = load_config_from_toml(config_filename).unwrap_or_else(|e| {
        eprintln!("Failed to load config: {}", e);

        // 使用默认配置或者其他处理方式
        // 注意：这里的示例代码中使用了 `Default::default()`，你可能需要根据你的实际情况修改这部分逻辑
        Default::default()
    });
    eframe::run_native(
        "深浅主题模式自动切换软件",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc, config))), // 传递配置到 MyApp::new
    )
}
#[derive(Default, Debug, Serialize, Deserialize)]
struct MyApp {
    is_dark_mode: bool,
    is_system_dark_mode: bool,
    is_system_both_dark_mode: bool,
    auto_mode_change: bool, // 添加自动模式切换的标志
    auto_system_mode_change: bool,
    custom_night_start: NaiveTimeWrapper, // 新增字段
    custom_night_end: NaiveTimeWrapper,   // 新增字段
    custom_night_start_hh: u32,
    custom_night_start_mm: u32,
    custom_night_end_hh: u32,
    custom_night_end_mm: u32,
    custom_system_night_start: NaiveTimeWrapper, // 新增字段
    custom_system_night_end: NaiveTimeWrapper,   // 新增字段
    custom_system_night_start_hh: u32,
    custom_system_night_start_mm: u32,
    custom_system_night_end_hh: u32,
    custom_system_night_end_mm: u32,
    is_autostart: bool,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>, config: MyApp) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        Self {
            is_dark_mode: config.is_dark_mode,
            is_system_dark_mode: config.is_system_dark_mode,
            auto_mode_change: config.auto_mode_change,
            auto_system_mode_change: config.auto_system_mode_change,
            is_system_both_dark_mode: config.is_system_both_dark_mode,
            custom_night_start: config.custom_night_start,
            custom_night_end: config.custom_night_end,
            custom_night_start_hh: config.custom_night_start_hh,
            custom_night_start_mm: config.custom_night_start_mm,
            custom_night_end_hh: config.custom_night_end_hh,
            custom_night_end_mm: config.custom_night_end_mm,
            custom_system_night_start: config.custom_system_night_start,
            custom_system_night_end: config.custom_system_night_end,
            custom_system_night_start_hh: config.custom_system_night_start_hh,
            custom_system_night_start_mm: config.custom_system_night_start_mm,
            custom_system_night_end_hh: config.custom_system_night_end_hh,
            custom_system_night_end_mm: config.custom_system_night_end_mm,
            is_autostart: config.is_autostart,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // if self.config_changed {
        //     self.on_config_change();
        //     // 可以在这里重置标志，表示已经处理了配置更改
        //     self.config_changed = false;
        // }
        save_config_to_toml(self, "config.toml").expect("TODO: panic message");
        // 在每次更新时检查主题模式并更新界面
        self.is_dark_mode = is_dark_mode_enabled();
        self.is_system_dark_mode = is_system_dark_mode_enabled();
        let mut tempautostart = self.is_autostart;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("深浅主题模式自动切换软件");
            ui.group(|ui| ui.horizontal(|ui| ui.checkbox(&mut self.is_autostart, "是否开机启动")));
            if self.is_autostart != tempautostart {
                if self.is_autostart {
                    option_env!("CARGO_PKG_NAME").map(|app_name| {
                        set_autostart(app_name, &env::current_exe().unwrap().to_str().unwrap())
                    });
                } else {
                    // 取消开机启动项
                    option_env!("CARGO_PKG_NAME").map(|app_name| remove_startup_entry(app_name));
                }
            }

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
                self.custom_night_start = NaiveTimeWrapper(
                    NaiveTime::from_hms_opt(
                        self.custom_night_start_hh,
                        self.custom_night_start_mm,
                        0,
                    )
                    .unwrap(),
                );
                self.custom_night_end = NaiveTimeWrapper(
                    NaiveTime::from_hms_opt(self.custom_night_end_hh, self.custom_night_end_mm, 0)
                        .unwrap(),
                );

                // 如果启用了自动模式切换，并且当前时间在指定的范围内，则切换主题模式
                if self.auto_mode_change {
                    let current_time = Local::now().time();

                    if current_time >= self.custom_night_start.0
                        || current_time < self.custom_night_end.0
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
                    self.custom_system_night_start = NaiveTimeWrapper(
                        NaiveTime::from_hms_opt(
                            self.custom_system_night_start_hh,
                            self.custom_system_night_start_mm,
                            0,
                        )
                        .unwrap(),
                    );

                    self.custom_system_night_end = NaiveTimeWrapper(
                        NaiveTime::from_hms_opt(
                            self.custom_system_night_end_hh,
                            self.custom_system_night_end_mm,
                            0,
                        )
                        .unwrap(),
                    );

                    // 如果启用了自动模式切换，并且当前时间在指定的范围内，则切换主题模式
                    if self.auto_system_mode_change {
                        let current_time = Local::now().time();

                        if current_time >= self.custom_system_night_start.0
                            || current_time < self.custom_system_night_end.0
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
    // stdout.contains("0x0")
    let helm = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(sub_key) = helm.open_subkey_with_flags(
        r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
        KEY_READ,
    ) {
        if let Ok(value) = sub_key.get_value::<u32, _>("AppsUseLightTheme") {
            return value == 0;
        }
    }
    // Return false by default if there was an error or the value doesn't exist
    false
}
fn is_system_dark_mode_enabled() -> bool {
    // stdout.contains("0x0")
    let helm = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(sub_key) = helm.open_subkey_with_flags(
        r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
        KEY_READ,
    ) {
        if let Ok(value) = sub_key.get_value::<u32, _>("SystemUsesLightTheme") {
            return value == 0;
        }
    }
    // Return false by default if there was an error or the value doesn't exist
    false
}
fn set_dark_mode(enabled: bool) {
    let check = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
            winreg::enums::KEY_SET_VALUE,
        )
        .unwrap();

    let value = if enabled { 0u32 } else { 1u32 };
    check.set_value("AppsUseLightTheme", &value).unwrap();
}

fn set_system_dark_mode(enabled: bool) {
    let check = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
            winreg::enums::KEY_SET_VALUE,
        )
        .unwrap();

    let value = if enabled { 0u32 } else { 1u32 };
    check.set_value("SystemUsesLightTheme", &value).unwrap();
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

    ctx.set_fonts(fonts);
}

fn set_autostart(app_name: &str, app_path: &str) -> Result<(), Box<dyn Error>> {
    // 打开注册表项
    let hklm = RegKey::predef(HKEY_CURRENT_USER);
    let key_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

    // 创建或打开注册表项
    let (key, _) = hklm.create_subkey_with_flags(key_path, KEY_SET_VALUE)?;

    // 设置注册表项值
    key.set_value(app_name, &app_path)?;

    // println!("成功添加 {} 到开机启动项。", app_name);

    Ok(())
}
fn remove_startup_entry(app_name: &str) -> Result<(), Box<dyn Error>> {
    // 打开注册表项
    let hklm = RegKey::predef(HKEY_CURRENT_USER);
    let key_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

    // 打开注册表项
    let key = hklm.open_subkey(key_path)?;

    // 删除注册表项值
    if let Err(err) = key.delete_value(app_name) {
        // eprintln!("删除注册表项值时出错：{}", err);
    } else {
        // println!("成功取消开机启动项 {}。", app_name);
    }

    Ok(())
}
