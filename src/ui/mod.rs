use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use std::fs;
use std::path::Path;
use crate::design::loader::LoadRobotRequest;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
           .init_resource::<AvailableModels>()
           .add_systems(Startup, (scan_models_directory, configure_ui_font)) // <--- フォント設定を追加
           .add_systems(Update, ui_system);
    }
}

#[derive(Resource, Default)]
struct AvailableModels {
    models: Vec<String>,
}

fn configure_ui_font(mut contexts: EguiContexts) {
    let font_path = Path::new("assets/fonts/NotoSansJP-Medium.ttf");
    
    if !font_path.exists() {
        warn!("Font not found at {:?}. Japanese characters may not display correctly.", font_path);
        return;
    }

    if let Ok(font_data) = fs::read(font_path) {
        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "NotoSansJP".to_owned(),
            egui::FontData::from_owned(font_data),
        );

        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap()
            .insert(0, "NotoSansJP".to_owned());
        
        fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap()
            .insert(0, "NotoSansJP".to_owned());

        contexts.ctx_mut().set_fonts(fonts);
    } else {
        error!("Failed to read font file: {:?}", font_path);
    }
}

fn scan_models_directory(mut available_models: ResMut<AvailableModels>) {
    let models_dir = Path::new("assets/models");
    if !models_dir.exists() {
        return;
    }

    if let Ok(entries) = fs::read_dir(models_dir) {
        available_models.models = entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|name| name.ends_with("_description"))
            .collect();
    }
}

fn ui_system(
    mut contexts: EguiContexts,
    available_models: Res<AvailableModels>,
    mut load_event_writer: EventWriter<LoadRobotRequest>,
) {
    egui::TopBottomPanel::top("top_panel").show(contexts.ctx_mut(), |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("ファイル", |ui| {
                ui.menu_button("開く...", |ui| {
                    if available_models.models.is_empty() {
                        ui.label("利用可能なモデルがありません");
                    } else {
                        for model_name in &available_models.models {
                            if ui.button(model_name).clicked() {
                                load_event_writer.send(LoadRobotRequest {
                                    model_name: model_name.clone(),
                                });
                                ui.close_menu();
                            }
                        }
                    }
                });
                
                if ui.button("終了").clicked() {
                    std::process::exit(0);
                }
            });
        });
    });
}
