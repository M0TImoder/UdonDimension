use bevy::prelude::*;
use bevy::render::view::screenshot::ScreenshotManager;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContexts};
use std::time::SystemTime;
use chrono::{DateTime, Local};
use arboard::Clipboard;
use std::path::Path;

#[derive(Resource, Default)]
pub struct ScreenshotNotification {
    pub message: Option<String>,
    pub timer: Timer,
}

pub struct ScreenshotPlugin;

impl Plugin for ScreenshotPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScreenshotNotification>()
           .add_systems(Update, (screenshot_input, screenshot_notification_ui));
    }
}

fn screenshot_input(
    input: Res<ButtonInput<KeyCode>>,
    main_window: Query<Entity, With<PrimaryWindow>>,
    mut screenshot_manager: ResMut<ScreenshotManager>,
    mut notification: ResMut<ScreenshotNotification>,
) {
    if input.just_pressed(KeyCode::F3) {
        if let Ok(window) = main_window.get_single() {
            let now: DateTime<Local> = SystemTime::now().into();
            let dir = Path::new("screenshots");
            if !dir.exists() {
                if let Err(e) = std::fs::create_dir(dir) {
                    error!("Failed to create screenshots directory: {}", e);
                    return;
                }
            }
            
            let filename = format!("screenshot-{}.png", now.format("%Y-%m-%d-%H-%M-%S"));
            let path = dir.join(&filename);
            
            match screenshot_manager.save_screenshot_to_disk(window, &path) {
                Ok(_) => {
                    let path_buf = path.to_path_buf();
                    let filename_clone = filename.clone();

                    // 別スレッドで画像を読み込んでクリップボードにコピー
                    std::thread::spawn(move || {
                        let start = std::time::Instant::now();
                        while start.elapsed().as_secs() < 5 {
                            if path_buf.exists() {
                                // 書き込み完了待ちのために少し待機
                                std::thread::sleep(std::time::Duration::from_millis(500));
                                
                                if let Ok(img) = image::open(&path_buf) {
                                    let rgba = img.to_rgba8();
                                    let (w, h) = rgba.dimensions();
                                    let image_data = arboard::ImageData {
                                        width: w as usize,
                                        height: h as usize,
                                        bytes: std::borrow::Cow::Borrowed(&rgba),
                                    };
                                    
                                    if let Ok(mut clipboard) = Clipboard::new() {
                                        if let Err(e) = clipboard.set_image(image_data) {
                                            eprintln!("Failed to copy to clipboard: {}", e);
                                        }
                                    }
                                    break;
                                }
                            }
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                    });

                    notification.message = Some(format!("スクリーンショット - {}をクリップボードに保存しました。", filename_clone));
                    notification.timer = Timer::from_seconds(3.0, TimerMode::Once);
                }
                Err(e) => {
                    error!("Failed to take screenshot: {}", e);
                }
            }
        }
    }
}

fn screenshot_notification_ui(
    mut contexts: EguiContexts,
    mut notification: ResMut<ScreenshotNotification>,
    time: Res<Time>,
) {
    let msg = match &notification.message {
        Some(m) => m.clone(),
        None => return,
    };

    notification.timer.tick(time.delta());
    if notification.timer.finished() {
        notification.message = None;
        return;
    }

    let ctx = contexts.ctx_mut();
    
    egui::Window::new("Screenshot Notification")
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-10.0, -10.0))
        .title_bar(false)
        .resizable(false)
        .frame(egui::Frame::popup(ctx.style().as_ref()).fill(egui::Color32::from_black_alpha(200)))
        .show(ctx, |ui| {
            ui.label(egui::RichText::new(msg).color(egui::Color32::WHITE).size(16.0));
        });
}
