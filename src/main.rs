use eframe::egui;

use crate::files::{
    get_claude_command, get_course_sections, get_courses, initialize_directory, is_initialized,
};
mod files;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };
    eframe::run_native(
        "Apex",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            cc.egui_ctx.set_theme(egui::Theme::Dark);

            Ok(Box::<MyApp>::default())
        }),
    )
}

struct MyApp {
    course: String,
    section: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            course: String::new(),
            section: String::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if is_initialized() {
            egui::Panel::left("app_panel").show_inside(ui, |ui| {
                ui.add_space(10.0);

                for course in get_courses() {
                    if ui.add(egui::Button::new(course.clone())).clicked() {
                        self.course = course.clone();
                    }
                }
            });

            if self.course.len() > 0 {
                ui.add_space(10.0);
                egui::Panel::left("course_panel").show_inside(ui, |ui| {
                    for section in get_course_sections(self.course.clone()) {
                        if ui.add(egui::Button::new(section.clone())).clicked() {
                            self.section = section.clone();
                        }
                    }
                });
            }
            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Apex");
                    if ui.button("Copy Command").clicked() {
                        ui.ctx().copy_text(get_claude_command());
                    }
                })
            });
        } else {
            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.heading("Apex not initialized");
                ui.add_space(10.0);
                if ui.add(egui::Button::new("Initialize")).clicked() {
                    initialize_directory();
                }
            });
        }
    }
}
