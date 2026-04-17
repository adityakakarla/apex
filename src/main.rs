use eframe::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use std::sync::Arc;

use std::collections::HashSet;

use crate::files::{
    get_claude_command, get_course_section_contents, get_course_sections, get_courses,
    get_markdown, get_quiz, initialize_directory, is_initialized, load_progress, save_progress,
};
mod files;

const SIDEBAR_BG: egui::Color32 = egui::Color32::from_rgb(12, 10, 9); // stone-950
const MAIN_BG: egui::Color32 = egui::Color32::from_rgb(28, 25, 23); // stone-900
const CARD_BG: egui::Color32 = egui::Color32::from_rgb(41, 37, 36); // stone-800
const ELEMENT_BG: egui::Color32 = egui::Color32::from_rgb(68, 64, 60); // stone-700
const ACCENT: egui::Color32 = egui::Color32::from_rgb(87, 83, 78); // stone-600
const ACCENT_MUTED: egui::Color32 = egui::Color32::from_rgb(68, 64, 60); // stone-700
const BORDER: egui::Color32 = egui::Color32::from_rgb(68, 64, 60); // stone-700
const TEXT_WEAK: egui::Color32 = egui::Color32::from_rgb(168, 162, 158); // stone-400
const TEXT: egui::Color32 = egui::Color32::from_rgb(231, 229, 228); // stone-200
const GREEN: egui::Color32 = egui::Color32::from_rgb(52, 211, 153);
const RED: egui::Color32 = egui::Color32::from_rgb(248, 113, 113);
const AMBER: egui::Color32 = egui::Color32::from_rgb(251, 191, 36);

fn main() -> eframe::Result {
    let d = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"))
        .expect("The icon data must be valid");
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([640.0, 480.0]),
        ..Default::default()
    };
    options.viewport.icon = Some(Arc::new(d));
    eframe::run_native(
        "Apex",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            cc.egui_ctx.set_theme(egui::Theme::Dark);

            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = MAIN_BG;
            visuals.window_fill = MAIN_BG;
            visuals.extreme_bg_color = CARD_BG;
            visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, BORDER);
            visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT);
            visuals.selection.bg_fill = ACCENT_MUTED;
            visuals.selection.stroke = egui::Stroke::new(1.0, ACCENT);
            cc.egui_ctx.set_visuals(visuals);

            let mut style = (*cc.egui_ctx.global_style()).clone();
            style.spacing.item_spacing = egui::vec2(8.0, 4.0);
            style.spacing.button_padding = egui::vec2(10.0, 6.0);
            style.spacing.window_margin = egui::Margin::same(0);
            cc.egui_ctx.set_global_style(style);

            Ok(Box::<MyApp>::default())
        }),
    )
}

#[derive(PartialEq)]
enum QuizStep {
    Answering,
    Revealed,
    Results,
}

struct QuizState {
    questions: Vec<(String, String)>,
    index: usize,
    user_answer: String,
    step: QuizStep,
    correct: Vec<bool>,
    completion_saved: bool,
}

impl QuizState {
    fn new(questions: Vec<(String, String)>) -> Self {
        let len = questions.len();
        Self {
            questions,
            index: 0,
            user_answer: String::new(),
            step: QuizStep::Answering,
            correct: vec![false; len],
            completion_saved: false,
        }
    }

    fn current(&self) -> Option<&(String, String)> {
        self.questions.get(self.index)
    }

    fn score(&self) -> usize {
        self.correct.iter().filter(|&&c| c).count()
    }
}

struct MyApp {
    course: String,
    section: String,
    content: String,
    quiz: Option<QuizState>,
    copied_at: Option<std::time::Instant>,
    progress: HashSet<String>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            course: String::new(),
            section: String::new(),
            content: String::new(),
            quiz: None,
            copied_at: None,
            progress: HashSet::new(),
        }
    }
}

fn sidebar_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(SIDEBAR_BG)
        .inner_margin(egui::Margin {
            left: 10,
            right: 10,
            top: 16,
            bottom: 16,
        })
}

fn section_label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .size(10.0)
            .color(TEXT_WEAK)
            .strong(),
    );
    ui.add_space(4.0);
}

fn nav_item(ui: &mut egui::Ui, label: &str, selected: bool, completed: bool) -> bool {
    let fill = if selected {
        ACCENT_MUTED
    } else {
        egui::Color32::TRANSPARENT
    };
    let text_color = if selected {
        egui::Color32::WHITE
    } else if completed {
        GREEN
    } else {
        TEXT
    };

    let btn = egui::Button::new(egui::RichText::new(label).color(text_color).size(13.5))
        .fill(fill)
        .frame(true)
        .corner_radius(6.0)
        .min_size(egui::vec2(ui.available_width(), 30.0));

    ui.add(btn).clicked()
}

fn file_chip(ui: &mut egui::Ui, label: &str, selected: bool, completed: bool) -> bool {
    let fill = if selected { ACCENT } else { ELEMENT_BG };

    let display = label
        .rsplit_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(label);

    let text_color = if completed {
        GREEN
    } else {
        egui::Color32::WHITE
    };

    let btn = egui::Button::new(egui::RichText::new(display).color(text_color).size(12.5))
        .fill(fill)
        .corner_radius(16.0)
        .min_size(egui::vec2(0.0, 26.0));

    ui.add(btn).clicked()
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if !is_initialized() {
            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(200.0);
                        ui.label(egui::RichText::new("Apex").size(32.0).strong().color(TEXT));
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("No data directory found.")
                                .size(14.0)
                                .color(TEXT_WEAK),
                        );
                        ui.add_space(20.0);
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Initialize Apex")
                                        .color(egui::Color32::WHITE),
                                )
                                .fill(ACCENT)
                                .corner_radius(8.0)
                                .min_size(egui::vec2(160.0, 38.0)),
                            )
                            .clicked()
                        {
                            initialize_directory();
                        }
                    });
                });
            });
            return;
        }

        // Courses sidebar
        egui::Panel::left("app_panel")
            .exact_size(160.0)
            .resizable(false)
            .frame(sidebar_frame())
            .show_inside(ui, |ui| {
                section_label(ui, "COURSES");

                for course in get_courses() {
                    let selected = self.course == course;
                    if nav_item(ui, &course, selected, false) {
                        self.course = course.clone();
                        self.section.clear();
                        self.content.clear();
                        self.quiz = None;
                        self.progress = load_progress(&course);
                    }
                }
            });

        // Sections sidebar
        if !self.course.is_empty() {
            egui::Panel::left("course_panel")
                .exact_size(170.0)
                .resizable(false)
                .frame(
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgb(20, 18, 17))
                        .stroke(egui::Stroke::new(1.0, BORDER))
                        .inner_margin(egui::Margin {
                            left: 10,
                            right: 10,
                            top: 16,
                            bottom: 16,
                        }),
                )
                .show_inside(ui, |ui| {
                    section_label(ui, "SECTIONS");
                    for section in get_course_sections(self.course.clone()) {
                        let selected = self.section == section;
                        let contents =
                            get_course_section_contents(self.course.clone(), section.clone());
                        let section_complete = !contents.is_empty()
                            && contents
                                .iter()
                                .all(|f| self.progress.contains(&format!("{}/{}", section, f)));
                        if nav_item(ui, &section, selected, section_complete) {
                            self.section = section.clone();
                            self.content.clear();
                            self.quiz = None;
                        }
                    }
                });
        }

        // Main content
        egui::CentralPanel::default().show_inside(ui, |ui| {
            // Header
            egui::Frame::new()
                .inner_margin(egui::Margin {
                    left: 20,
                    right: 20,
                    top: 14,
                    bottom: 14,
                })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let title = if !self.section.is_empty() {
                            format!("{} / {}", self.course, self.section)
                        } else if !self.course.is_empty() {
                            self.course.clone()
                        } else {
                            "Apex".to_string()
                        };
                        ui.label(
                            egui::RichText::new(title)
                                .size(16.0)
                                .strong()
                                .color(egui::Color32::WHITE),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let just_copied = self
                                .copied_at
                                .map(|t| t.elapsed().as_secs_f32() < 2.0)
                                .unwrap_or(false);

                            let label = if just_copied {
                                "Copied!"
                            } else {
                                "Copy Command"
                            };
                            let fill = if just_copied {
                                egui::Color32::from_rgb(22, 101, 70)
                            } else {
                                ELEMENT_BG
                            };

                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(label)
                                            .size(12.0)
                                            .color(egui::Color32::WHITE),
                                    )
                                    .fill(fill)
                                    .corner_radius(6.0),
                                )
                                .clicked()
                            {
                                ui.ctx().copy_text(get_claude_command());
                                self.copied_at = Some(std::time::Instant::now());
                            }

                            if just_copied {
                                ui.ctx().request_repaint();
                            }
                        });
                    });
                });

            ui.add(egui::Separator::default().spacing(0.0));

            // File chips strip
            if !self.course.is_empty() && !self.section.is_empty() {
                let contents =
                    get_course_section_contents(self.course.clone(), self.section.clone());
                if !contents.is_empty() {
                    egui::Frame::new()
                        .inner_margin(egui::Margin {
                            left: 20,
                            right: 20,
                            top: 10,
                            bottom: 10,
                        })
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                for content in &contents {
                                    let selected = *content == self.content;
                                    let key = format!("{}/{}", self.section, content);
                                    let completed = self.progress.contains(&key);
                                    if file_chip(ui, content, selected, completed) {
                                        self.content = content.clone();
                                        if content.ends_with(".json") {
                                            let questions = get_quiz(
                                                self.course.clone(),
                                                self.section.clone(),
                                                content.clone(),
                                            );
                                            self.quiz = Some(QuizState::new(questions));
                                        } else {
                                            self.quiz = None;
                                        }
                                    }
                                }
                            });
                        });
                    ui.add(egui::Separator::default().spacing(0.0));
                }
            }

            // Content view
            let has_content =
                !self.course.is_empty() && !self.section.is_empty() && !self.content.is_empty();

            if has_content && self.content.ends_with(".md") {
                let mut cache = CommonMarkCache::default();
                let markdown = get_markdown(
                    self.course.clone(),
                    self.section.clone(),
                    self.content.clone(),
                );
                let key = format!("{}/{}", self.section, self.content);
                let is_complete = self.progress.contains(&key);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Frame::new()
                        .inner_margin(egui::Margin {
                            left: 32,
                            right: 32,
                            top: 20,
                            bottom: 20,
                        })
                        .show(ui, |ui| {
                            CommonMarkViewer::new().show(ui, &mut cache, markdown.as_str());
                            ui.add_space(16.0);
                            ui.separator();
                            ui.add_space(12.0);
                            let (label, fill) = if is_complete {
                                ("Completed", egui::Color32::from_rgb(22, 101, 70))
                            } else {
                                ("Mark Complete", ACCENT)
                            };
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(label).color(egui::Color32::WHITE),
                                    )
                                    .fill(fill)
                                    .corner_radius(8.0)
                                    .min_size(egui::vec2(140.0, 34.0)),
                                )
                                .clicked()
                            {
                                if is_complete {
                                    self.progress.remove(&key);
                                } else {
                                    self.progress.insert(key);
                                }
                                save_progress(&self.course, &self.progress);
                            }
                        });
                });
            } else if has_content && self.content.ends_with(".json") {
                // Auto-mark quiz complete when Results screen is reached
                if let Some(quiz) = &self.quiz {
                    if quiz.step == QuizStep::Results && !quiz.completion_saved {
                        let key = format!("{}/{}", self.section, self.content);
                        self.progress.insert(key);
                        save_progress(&self.course, &self.progress);
                    }
                }
                if let Some(quiz) = &mut self.quiz {
                    if quiz.step == QuizStep::Results {
                        quiz.completion_saved = true;
                    }
                }

                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Frame::new()
                        .inner_margin(egui::Margin {
                            left: 32,
                            right: 32,
                            top: 20,
                            bottom: 20,
                        })
                        .show(ui, |ui| {
                            show_quiz(ui, &mut self.quiz);
                        });
                });
            } else if has_content {
                // unsupported file type
            } else if self.course.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new("Select a course to get started.")
                            .size(15.0)
                            .color(TEXT_WEAK),
                    );
                });
            }
        });
    }
}

fn show_quiz(ui: &mut egui::Ui, quiz_opt: &mut Option<QuizState>) {
    let Some(quiz) = quiz_opt else {
        ui.label(egui::RichText::new("No quiz loaded.").color(TEXT_WEAK));
        return;
    };

    if quiz.questions.is_empty() {
        ui.label(egui::RichText::new("No questions found in this file.").color(TEXT_WEAK));
        return;
    }

    if quiz.step == QuizStep::Results {
        show_results(ui, quiz);
        return;
    }

    let total = quiz.questions.len();
    let idx = quiz.index;

    // Progress
    let progress = (idx + 1) as f32 / total as f32;
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("{} of {}", idx + 1, total))
                .size(12.0)
                .color(TEXT_WEAK),
        );
    });
    ui.add_space(4.0);
    ui.add(
        egui::ProgressBar::new(progress)
            .desired_height(4.0)
            .fill(TEXT_WEAK),
    );
    ui.add_space(20.0);

    let Some((question, answer)) = quiz.current() else {
        return;
    };
    let question = question.clone();
    let answer = answer.clone();

    // Question card
    egui::Frame::new()
        .fill(CARD_BG)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .corner_radius(10.0)
        .inner_margin(egui::Margin::same(20))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(
                egui::RichText::new(&question)
                    .size(18.0)
                    .strong()
                    .color(TEXT),
            );
        });

    ui.add_space(16.0);

    match quiz.step {
        QuizStep::Answering => {
            ui.label(
                egui::RichText::new("Your answer")
                    .size(12.0)
                    .color(TEXT_WEAK),
            );
            ui.add_space(4.0);

            let resp = ui.add(
                egui::TextEdit::singleline(&mut quiz.user_answer)
                    .hint_text("Type your answer and press Enter…")
                    .font(egui::TextStyle::Body)
                    .desired_width(f32::INFINITY),
            );

            let submitted = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            ui.add_space(12.0);

            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("Reveal Answer").color(egui::Color32::WHITE),
                    )
                    .fill(ACCENT)
                    .corner_radius(8.0)
                    .min_size(egui::vec2(140.0, 36.0)),
                )
                .clicked()
                || submitted
            {
                quiz.step = QuizStep::Revealed;
            }
        }
        QuizStep::Revealed => {
            if !quiz.user_answer.is_empty() {
                ui.label(
                    egui::RichText::new(format!("You wrote:  {}", quiz.user_answer))
                        .size(13.0)
                        .color(TEXT_WEAK),
                );
                ui.add_space(10.0);
            }

            egui::Frame::new()
                .fill(egui::Color32::from_rgb(20, 46, 36))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 90, 65)))
                .corner_radius(8.0)
                .inner_margin(egui::Margin::same(16))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.label(
                        egui::RichText::new("Correct answer")
                            .size(11.0)
                            .color(GREEN),
                    );
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(&answer).size(16.0).strong().color(TEXT));
                });

            ui.add_space(16.0);
            ui.label(
                egui::RichText::new("Did you get it right?")
                    .size(13.0)
                    .color(TEXT_WEAK),
            );
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("Got it").color(egui::Color32::WHITE),
                        )
                        .fill(egui::Color32::from_rgb(22, 101, 70))
                        .corner_radius(8.0)
                        .min_size(egui::vec2(100.0, 34.0)),
                    )
                    .clicked()
                {
                    quiz.correct[quiz.index] = true;
                    advance_quiz(quiz);
                }

                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("Missed it").color(egui::Color32::WHITE),
                        )
                        .fill(egui::Color32::from_rgb(120, 40, 40))
                        .corner_radius(8.0)
                        .min_size(egui::vec2(100.0, 34.0)),
                    )
                    .clicked()
                {
                    quiz.correct[quiz.index] = false;
                    advance_quiz(quiz);
                }
            });
        }
        QuizStep::Results => unreachable!(),
    }
}

fn advance_quiz(quiz: &mut QuizState) {
    if quiz.index + 1 >= quiz.questions.len() {
        quiz.step = QuizStep::Results;
    } else {
        quiz.index += 1;
        quiz.user_answer.clear();
        quiz.step = QuizStep::Answering;
    }
}

fn show_results(ui: &mut egui::Ui, quiz: &mut QuizState) {
    let total = quiz.questions.len();
    let score = quiz.score();
    let pct = score as f32 / total as f32;

    let score_color = if pct >= 0.8 {
        GREEN
    } else if pct >= 0.5 {
        AMBER
    } else {
        RED
    };

    egui::Frame::new()
        .fill(CARD_BG)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .corner_radius(12.0)
        .inner_margin(egui::Margin::same(28))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("Quiz Complete")
                        .size(22.0)
                        .strong()
                        .color(TEXT),
                );
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new(format!("{} / {}", score, total))
                        .size(52.0)
                        .strong()
                        .color(score_color),
                );
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new(format!("{:.0}%", pct * 100.0))
                        .size(18.0)
                        .color(score_color),
                );
                ui.add_space(18.0);
                ui.add(
                    egui::ProgressBar::new(pct)
                        .desired_width(280.0)
                        .desired_height(6.0)
                        .fill(score_color),
                );
                ui.add_space(22.0);
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("Retry Quiz").color(egui::Color32::WHITE),
                        )
                        .fill(ACCENT)
                        .corner_radius(8.0)
                        .min_size(egui::vec2(130.0, 36.0)),
                    )
                    .clicked()
                {
                    let questions = std::mem::take(&mut quiz.questions);
                    *quiz = QuizState::new(questions);
                }
            });
        });

    ui.add_space(24.0);
    ui.label(
        egui::RichText::new("Review")
            .size(14.0)
            .strong()
            .color(TEXT),
    );
    ui.add_space(8.0);

    for (i, (q, a)) in quiz.questions.iter().enumerate() {
        let got_it = quiz.correct.get(i).copied().unwrap_or(false);
        let (bg, border, marker_color, marker) = if got_it {
            (
                egui::Color32::from_rgb(18, 40, 30),
                egui::Color32::from_rgb(36, 80, 56),
                GREEN,
                "✓",
            )
        } else {
            (
                egui::Color32::from_rgb(40, 18, 18),
                egui::Color32::from_rgb(80, 36, 36),
                RED,
                "✗",
            )
        };

        egui::Frame::new()
            .fill(bg)
            .stroke(egui::Stroke::new(1.0, border))
            .corner_radius(8.0)
            .inner_margin(egui::Margin {
                left: 14,
                right: 14,
                top: 10,
                bottom: 10,
            })
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(marker).strong().color(marker_color));
                    ui.label(egui::RichText::new(q).strong().color(TEXT));
                });
                ui.label(egui::RichText::new(a).size(12.5).color(TEXT_WEAK));
            });
        ui.add_space(5.0);
    }
}
