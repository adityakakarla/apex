use eframe::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use crate::files::{
    get_claude_command, get_course_section_contents, get_course_sections, get_courses,
    get_markdown, initialize_directory, is_initialized,
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
    content: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            course: String::new(),
            section: String::new(),
            content: String::new(),
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
                        self.section = String::new();
                        self.content = String::new();
                    }
                }
            });

            if self.course.len() > 0 {
                egui::Panel::left("course_panel").show_inside(ui, |ui| {
                    ui.add_space(10.0);
                    for section in get_course_sections(self.course.clone()) {
                        if ui.add(egui::Button::new(section.clone())).clicked() {
                            self.section = section.clone();
                            self.content = String::new();
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
                });

                if self.course.len() > 0 && self.section.len() > 0 {
                    for content in
                        get_course_section_contents(self.course.clone(), self.section.clone())
                    {
                        if ui.button(content.clone()).clicked() {
                            self.content = content.clone()
                        }
                    }
                }

                if self.course.len() > 0
                    && self.section.len() > 0
                    && self.content.len() > 0
                    && self.content.ends_with(".md")
                {
                    let mut cache = CommonMarkCache::default();
                    let markdown = get_markdown(
                        self.course.clone(),
                        self.section.clone(),
                        self.content.clone(),
                    );

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        CommonMarkViewer::new().show(ui, &mut cache, markdown.as_str());
                    });
                }
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
