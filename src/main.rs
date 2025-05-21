use eframe::egui;
use rfd::FileDialog;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use syntect::easy::HighlightLines;
use syntect::highlighting::{ThemeSet, Style};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

#[derive(Default)]
struct FileTab {
    path: Option<PathBuf>,
    title: String,
    content: String,
    syntax: Option<String>,
    last_find: Option<usize>,
}

pub struct TextEditorApp {
    tabs: HashMap<String, FileTab>,
    open_order: Vec<String>,
    active_tab: Option<String>,

    folder_path: Option<PathBuf>,
    file_list: Vec<PathBuf>,

    syntax_set: SyntaxSet,
    theme: syntect::highlighting::Theme,

    new_file_counter: usize,

    show_rename: bool,
    rename_input: String,

    show_find: bool,
    find_input: String,
    found_count: usize,

    show_replace: bool,
    replace_find_input: String,
    replace_with_input: String,
    
    // Added: theme state
    dark_mode: bool,
    
    // Added: sidebar width state
    sidebar_width: f32,
}

impl Default for TextEditorApp {
    fn default() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme = ThemeSet::load_defaults().themes["InspiredGitHub"].clone();
        Self {
            tabs: HashMap::new(),
            open_order: Vec::new(),
            active_tab: None,
            folder_path: None,
            file_list: Vec::new(),
            syntax_set,
            theme,
            new_file_counter: 1,
            show_rename: false,
            rename_input: String::new(),
            show_find: false,
            find_input: String::new(),
            found_count: 0,
            show_replace: false,
            replace_find_input: String::new(),
            replace_with_input: String::new(),
            dark_mode: false, // Default to light mode
            sidebar_width: 200.0, // Default sidebar width
        }
    }
}

impl TextEditorApp {
    fn open_file(&mut self, path: &Path) {
        if let Ok(content) = fs::read_to_string(path) {
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            let syntax = self
                .syntax_set
                .find_syntax_for_file(path)
                .ok()
                .flatten()
                .map(|s| s.name.clone());

            let tab = FileTab {
                path: Some(path.to_path_buf()),
                title: file_name.clone(),
                content,
                syntax,
                last_find: None,
            };
            self.tabs.insert(file_name.clone(), tab);
            self.open_order.push(file_name.clone());
            self.active_tab = Some(file_name);
        }
    }

    fn create_new_file(&mut self) {
        let title = format!("Untitled {}", self.new_file_counter);
        self.new_file_counter += 1;
        let tab = FileTab {
            path: None,
            title: title.clone(),
            content: String::new(),
            syntax: None,
            last_find: None,
        };
        self.tabs.insert(title.clone(), tab);
        self.open_order.push(title.clone());
        self.active_tab = Some(title);
    }

    fn save_active(&mut self) {
        if let Some(tab_name) = &self.active_tab {
            if let Some(tab) = self.tabs.get_mut(tab_name) {
                let target_path = if let Some(ref path) = tab.path {
                    Some(path.clone())
                } else {
                    FileDialog::new().set_file_name(&tab.title).save_file()
                };

                if let Some(path) = target_path {
                    if fs::write(&path, &tab.content).is_ok() {
                        tab.path = Some(path);
                    }
                }
            }
        }
    }
    
    // New method to toggle theme
    fn toggle_theme(&mut self, ctx: &egui::Context) {
        self.dark_mode = !self.dark_mode;
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
    }
}

impl eframe::App for TextEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set visuals based on current theme
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
        
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("New File").clicked() {
                    self.create_new_file();
                }
                if ui.button("Open File").clicked() {
                    if let Some(path) = FileDialog::new().pick_file() {
                        self.open_file(&path);
                    }
                }
                if ui.button("Open Folder").clicked() {
                    if let Some(folder) = FileDialog::new().pick_folder() {
                        self.folder_path = Some(folder.clone());
                        self.file_list = fs::read_dir(&folder)
                            .unwrap()
                            .filter_map(Result::ok)
                            .map(|e| e.path())
                            .filter(|p| p.is_file())
                            .collect();
                    }
                }
                if ui.button("Save").clicked() {
                    self.save_active();
                }
                if ui.button("Rename").clicked() {
                    if let Some(tab_name) = &self.active_tab {
                        if let Some(tab) = self.tabs.get(tab_name) {
                            self.rename_input = tab.title.clone();
                            self.show_rename = true;
                        }
                    }
                }
                if ui.button("Find").clicked() {
                    self.show_find = true;
                }
                if ui.button("Replace").clicked() {
                    self.show_replace = true;
                }
                
                // Add theme toggle button
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let theme_text = if self.dark_mode { "Light Theme" } else { "Dark Theme" };
                    if ui.button(theme_text).clicked() {
                        self.toggle_theme(ctx);
                    }
                });
            });
        });
        
        // Add sidebar width control panel
        egui::TopBottomPanel::top("sidebar_controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Sidebar Width:");
                if ui.button("Small (150px)").clicked() {
                    self.sidebar_width = 150.0;
                }
                if ui.button("Medium (250px)").clicked() {
                    self.sidebar_width = 250.0;
                }
                if ui.button("Large (350px)").clicked() {
                    self.sidebar_width = 350.0;
                }
            });
        });

        // Side panel with fixed width based on sidebar_width
        egui::SidePanel::left("file_browser")
            .exact_width(self.sidebar_width) // Use exact width from current sidebar_width
            .show(ctx, |ui| {
                ui.heading("Files");
                
                if let Some(folder) = &self.folder_path {
                    ui.label(folder.display().to_string());
                    ui.separator();
                    
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for path in self.file_list.clone() {
                            if let Some(name) = path.file_name().map(|n| n.to_string_lossy().to_string()) {
                                if ui.button(&name).clicked() {
                                    self.open_file(&path);
                                }
                            }
                        }
                    });
                } else {
                    ui.label("No folder opened");
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::TopBottomPanel::top("tabs").show_inside(ui, |ui| {
                let mut tab_to_close: Option<String> = None;
                ui.horizontal_wrapped(|ui| {
                    for tab_name in &self.open_order {
                        let is_active = Some(tab_name) == self.active_tab.as_ref();
                        ui.horizontal(|ui| {
                            if ui.selectable_label(is_active, tab_name).clicked() {
                                self.active_tab = Some(tab_name.clone());
                            }
                            if ui.button("×").clicked() {
                                tab_to_close = Some(tab_name.clone());
                            }
                        });
                    }
                });
                if let Some(to_close) = tab_to_close {
                    self.tabs.remove(&to_close);
                    self.open_order.retain(|n| n != &to_close);
                    if self.active_tab.as_ref() == Some(&to_close) {
                        self.active_tab = self.open_order.last().cloned();
                    }
                }
            });

            if let Some(tab_name) = &self.active_tab {
                if let Some(tab) = self.tabs.get_mut(tab_name) {
                    ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::multiline(&mut tab.content)
                            .font(egui::TextStyle::Monospace)
                            .code_editor(),
                    );
                }
            } else {
                ui.label("No file opened");
            }
        });

        let mut show_rename = self.show_rename;
        if show_rename {
            egui::Window::new("Rename File")
                .collapsible(false)
                .resizable(false)
                .default_size((300.0, 120.0))
                .open(&mut show_rename)
                .show(ctx, |ui| {
                    ui.label("New name:");
                    ui.text_edit_singleline(&mut self.rename_input);
                    ui.horizontal(|ui| {
                        if ui.button("OK").clicked() {
                            if let Some(tab_name) = &self.active_tab {
                                if let Some(tab) = self.tabs.get_mut(tab_name) {
                                    let new_title = self.rename_input.trim();
                                    if !new_title.is_empty() {
                                        if let Some(old_path) = &tab.path {
                                            let new_path = old_path.with_file_name(new_title);
                                            if fs::rename(old_path, &new_path).is_ok() {
                                                tab.path = Some(new_path);
                                            }
                                        }
                                        let old_key = tab_name.clone();
                                        let mut updated_tab = self.tabs.remove(&old_key).unwrap();
                                        updated_tab.title = new_title.to_string();
                                        self.tabs.insert(new_title.to_string(), updated_tab);
                                        for name in &mut self.open_order {
                                            if name == &old_key {
                                                *name = new_title.to_string();
                                            }
                                        }
                                        self.active_tab = Some(new_title.to_string());
                                    }
                                }
                            }
                            self.show_rename = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_rename = false;
                        }
                    });
                });
            self.show_rename = show_rename;
        }

        if self.show_find {
            egui::Window::new("Find")
                .collapsible(false)
                .resizable(false)
                .default_size((300.0, 120.0))
                .open(&mut self.show_find)
                .show(ctx, |ui| {
                    ui.label("Find:");
                    ui.text_edit_singleline(&mut self.find_input);
                    if ui.button("Count occurrences").clicked() {
                        if let Some(tab_name) = &self.active_tab {
                            if let Some(tab) = self.tabs.get(tab_name) {
                                self.found_count = tab.content.matches(&self.find_input).count();
                            }
                        }
                    }
                    ui.label(format!("Found: {}", self.found_count));
                });
        }

        let mut show_replace = self.show_replace;
        if show_replace {
            egui::Window::new("Find & Replace")
                .collapsible(false)
                .resizable(false)
                .default_size((350.0, 160.0))
                .open(&mut show_replace)
                .show(ctx, |ui| {
                    ui.label("Find:");
                    ui.text_edit_singleline(&mut self.replace_find_input);
                    ui.label("Replace with:");
                    ui.text_edit_singleline(&mut self.replace_with_input);
                    ui.horizontal(|ui| {
                        if ui.button("Replace All").clicked() {
                            if let Some(tab_name) = &self.active_tab {
                                if let Some(tab) = self.tabs.get_mut(tab_name) {
                                    tab.content = tab
                                        .content
                                        .replace(&self.replace_find_input, &self.replace_with_input);
                                }
                            }
                        }
                        if ui.button("Close").clicked() {
                            self.show_replace = false;
                        }
                    });
                });
            self.show_replace = show_replace;
        }
    }


}

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Rust Text Editor",
        options,
        Box::new(|cc| {
            let mut app = TextEditorApp::default();
            // Apply initial theme
            if app.dark_mode {
                cc.egui_ctx.set_visuals(egui::Visuals::dark());
            } else {
                cc.egui_ctx.set_visuals(egui::Visuals::light());
            }
            Box::new(app)
        }),
    );
}

