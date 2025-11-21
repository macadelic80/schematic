use crate::binary_data::BinaryData;
use crate::schema::{DataType, Field, Schema};
use crate::ui::{DataView, FieldAction, HexView};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// View focus state for keyboard shortcuts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewFocus {
    HexView,
    DataView,
}

/// Main application state
pub struct SchematicApp {
    /// Loaded binary data
    binary_data: BinaryData,
    /// Defined fields for interpreting the binary
    fields: Vec<Field>,
    /// Hex view widget
    hex_view: HexView,
    /// Data view widget
    data_view: DataView,
    /// UI state for adding new fields
    add_field_window_open: bool,
    new_field_name: String,
    new_field_offset: String,
    new_field_type_idx: usize,
    new_field_comment: String,
    /// UI state for editing fields
    edit_field_window_open: bool,
    edit_field_idx: Option<usize>,
    edit_field_name: String,
    edit_field_offset: String,
    edit_field_type_idx: usize,
    edit_field_comment: String,
    /// Currently selected fields for highlighting (supports multi-selection)
    selected_fields: HashSet<usize>,
    /// Last selected field index for shift-click range selection
    last_selected_field: Option<usize>,
    /// Current view focus (for keyboard shortcuts)
    view_focus: ViewFocus,
    /// Path to the current schema file (for save/save-as)
    schema_file_path: Option<PathBuf>,
}

impl Default for SchematicApp {
    fn default() -> Self {
        Self {
            binary_data: BinaryData::new(),
            fields: Vec::new(),
            hex_view: HexView::new(),
            data_view: DataView::new(),
            add_field_window_open: false,
            new_field_name: String::new(),
            new_field_offset: String::from("0"),
            new_field_type_idx: 0,
            new_field_comment: String::new(),
            edit_field_window_open: false,
            edit_field_idx: None,
            edit_field_name: String::new(),
            edit_field_offset: String::from("0"),
            edit_field_type_idx: 0,
            edit_field_comment: String::new(),
            selected_fields: HashSet::new(),
            last_selected_field: None,
            view_focus: ViewFocus::HexView,
            schema_file_path: None,
        }
    }
}

impl SchematicApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    /// Open a file dialog and load the selected binary file
    fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            if let Err(e) = self.binary_data.load_from_file(path.clone()) {
                eprintln!("Error loading file: {}", e);
            } else {
                println!("Loaded file: {:?}", path);
            }
        }
    }

    /// Render the top menu bar
    fn show_menu(&mut self, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open...").clicked() {
                    self.open_file();
                    ui.close_menu();
                }

                ui.separator();

                if ui.button("Quit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.menu_button("Schema", |ui| {
                if ui.button("Add Field...").clicked() {
                    self.add_field_window_open = true;
                    ui.close_menu();
                }

                ui.separator();

                if ui.button("Save Schema...").clicked() {
                    self.save_schema();
                    ui.close_menu();
                }

                if ui.button("Load Schema...").clicked() {
                    self.load_schema();
                    ui.close_menu();
                }

                ui.separator();

                if ui.button("Clear All Fields").clicked() {
                    self.fields.clear();
                    ui.close_menu();
                }
            });
        });
    }

    /// Show the "Add Field" dialog window
    fn show_add_field_window(&mut self, ctx: &egui::Context) {
        if !self.add_field_window_open {
            return;
        }

        let mut window_open = self.add_field_window_open;
        egui::Window::new("Add Field")
            .open(&mut window_open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.new_field_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Offset:");
                    ui.text_edit_singleline(&mut self.new_field_offset);
                    ui.label("(hex or decimal)");
                });

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::from_id_salt("field_type")
                        .selected_text(DataType::all()[self.new_field_type_idx].name())
                        .show_ui(ui, |ui| {
                            for (idx, dt) in DataType::all().iter().enumerate() {
                                ui.selectable_value(&mut self.new_field_type_idx, idx, dt.name());
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Comment:");
                    ui.text_edit_singleline(&mut self.new_field_comment);
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Add").clicked() {
                        if let Some(field) = self.create_field_from_input() {
                            self.fields.push(field);
                            self.reset_add_field_form();
                            self.add_field_window_open = false;
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        self.reset_add_field_form();
                        self.add_field_window_open = false;
                    }
                });
            });

        self.add_field_window_open = window_open;
    }

    /// Create a field from the current input values
    fn create_field_from_input(&self) -> Option<Field> {
        if self.new_field_name.is_empty() {
            return None;
        }

        // Parse offset (support both hex with 0x prefix and decimal)
        let offset = if let Some(hex_str) = self.new_field_offset.strip_prefix("0x") {
            usize::from_str_radix(hex_str, 16).ok()?
        } else {
            self.new_field_offset.parse::<usize>().ok()?
        };

        let data_type = DataType::all()[self.new_field_type_idx];

        let mut field = Field::new(self.new_field_name.clone(), offset, data_type);
        field.comment = self.new_field_comment.clone();

        Some(field)
    }

    /// Reset the add field form to default values
    fn reset_add_field_form(&mut self) {
        self.new_field_name.clear();
        self.new_field_offset = String::from("0");
        self.new_field_type_idx = 0;
        self.new_field_comment.clear();
    }

    /// Start editing a field by populating the edit form
    fn start_edit_field(&mut self, idx: usize) {
        if let Some(field) = self.fields.get(idx) {
            self.edit_field_idx = Some(idx);
            self.edit_field_name = field.name.clone();
            self.edit_field_offset = format!("0x{:X}", field.offset);
            self.edit_field_type_idx = DataType::all()
                .iter()
                .position(|&dt| dt == field.data_type)
                .unwrap_or(0);
            self.edit_field_comment = field.comment.clone();
            self.edit_field_window_open = true;
        }
    }

    /// Show the "Edit Field" dialog window
    fn show_edit_field_window(&mut self, ctx: &egui::Context) {
        if !self.edit_field_window_open {
            return;
        }

        let mut window_open = self.edit_field_window_open;
        egui::Window::new("Edit Field")
            .open(&mut window_open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.edit_field_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Offset:");
                    ui.text_edit_singleline(&mut self.edit_field_offset);
                    ui.label("(hex or decimal)");
                });

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::from_id_salt("edit_field_type")
                        .selected_text(DataType::all()[self.edit_field_type_idx].name())
                        .show_ui(ui, |ui| {
                            for (idx, dt) in DataType::all().iter().enumerate() {
                                ui.selectable_value(&mut self.edit_field_type_idx, idx, dt.name());
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Comment:");
                    ui.text_edit_singleline(&mut self.edit_field_comment);
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        if self.update_field_from_input() {
                            self.edit_field_window_open = false;
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        self.edit_field_window_open = false;
                    }
                });
            });

        self.edit_field_window_open = window_open;
    }

    /// Update the field being edited with the current input values
    fn update_field_from_input(&mut self) -> bool {
        if self.edit_field_name.is_empty() {
            return false;
        }

        let Some(idx) = self.edit_field_idx else {
            return false;
        };

        // Parse offset (support both hex with 0x prefix and decimal)
        let offset = if let Some(hex_str) = self.edit_field_offset.strip_prefix("0x") {
            if let Ok(val) = usize::from_str_radix(hex_str, 16) {
                val
            } else {
                return false;
            }
        } else {
            if let Ok(val) = self.edit_field_offset.parse::<usize>() {
                val
            } else {
                return false;
            }
        };

        let data_type = DataType::all()[self.edit_field_type_idx];

        let mut field = Field::new(self.edit_field_name.clone(), offset, data_type);
        field.comment = self.edit_field_comment.clone();

        // Update the field in the vector
        if let Some(existing_field) = self.fields.get_mut(idx) {
            *existing_field = field;
        }

        true
    }

    /// Show file information panel
    fn show_file_info(&self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("File:");
                if let Some(path) = self.binary_data.file_path() {
                    ui.label(path.display().to_string());
                } else {
                    ui.label("No file loaded");
                }
            });

            if self.binary_data.is_loaded() {
                ui.horizontal(|ui| {
                    ui.label("Size:");
                    ui.label(format!("{} bytes", self.binary_data.size()));
                });
            }
        });
    }

    /// Save the current schema to a TOML file
    fn save_schema(&mut self) {
        if self.fields.is_empty() {
            eprintln!("No fields to save");
            return;
        }

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("TOML Schema", &["toml"])
            .set_file_name("schema.toml")
            .save_file()
        {
            self.save_schema_to_path(path);
        }
    }

    /// Save schema to a specific path
    fn save_schema_to_path(&mut self, path: PathBuf) {
        let schema = Schema {
            fields: self.fields.clone(),
        };

        match toml::to_string_pretty(&schema) {
            Ok(toml_string) => {
                if let Err(e) = fs::write(&path, toml_string) {
                    eprintln!("Error saving schema: {}", e);
                } else {
                    println!("Schema saved to: {:?}", path);
                    self.schema_file_path = Some(path);
                }
            }
            Err(e) => {
                eprintln!("Error serializing schema: {}", e);
            }
        }
    }

    /// Save schema with save-as dialog (always prompt for location)
    fn save_schema_as(&mut self) {
        if self.fields.is_empty() {
            eprintln!("No fields to save");
            return;
        }

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("TOML Schema", &["toml"])
            .set_file_name("schema.toml")
            .save_file()
        {
            self.save_schema_to_path(path);
        }
    }

    /// Save schema (save-as if new, overwrite if existing)
    fn save_schema_smart(&mut self) {
        if self.fields.is_empty() {
            eprintln!("No fields to save");
            return;
        }

        if let Some(path) = self.schema_file_path.clone() {
            // Overwrite existing file
            self.save_schema_to_path(path);
        } else {
            // Prompt for new location
            self.save_schema_as();
        }
    }

    /// Load a schema from a TOML file
    fn load_schema(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("TOML Schema", &["toml"])
            .pick_file()
        {
            match fs::read_to_string(&path) {
                Ok(toml_string) => match toml::from_str::<Schema>(&toml_string) {
                    Ok(schema) => {
                        self.fields = schema.fields;
                        self.schema_file_path = Some(path.clone());
                        println!("Schema loaded from: {:?}", path);
                    }
                    Err(e) => {
                        eprintln!("Error parsing schema: {}", e);
                    }
                },
                Err(e) => {
                    eprintln!("Error reading schema file: {}", e);
                }
            }
        }
    }
}

impl eframe::App for SchematicApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard shortcuts
        ctx.input(|i| {
            // Focus switching
            if i.key_pressed(egui::Key::Num1) && i.modifiers.ctrl {
                self.view_focus = ViewFocus::HexView;
            }
            if i.key_pressed(egui::Key::Num2) && i.modifiers.ctrl {
                self.view_focus = ViewFocus::DataView;
            }

            // Ctrl+Q: Quit
            if i.key_pressed(egui::Key::Q) && i.modifiers.ctrl {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            // Ctrl+O: Context-aware open (file or schema)
            if i.key_pressed(egui::Key::O) && i.modifiers.ctrl {
                match self.view_focus {
                    ViewFocus::HexView => self.open_file(),
                    ViewFocus::DataView => self.load_schema(),
                }
            }

            // Ctrl+S: Save schema (smart save)
            if i.key_pressed(egui::Key::S) && i.modifiers.ctrl && !i.modifiers.shift {
                if self.view_focus == ViewFocus::DataView {
                    self.save_schema_smart();
                }
            }

            // Ctrl+Shift+S: Save schema as (always prompt)
            if i.key_pressed(egui::Key::S) && i.modifiers.ctrl && i.modifiers.shift {
                if self.view_focus == ViewFocus::DataView {
                    self.save_schema_as();
                }
            }

            // Ctrl+N: Add new field
            if i.key_pressed(egui::Key::N) && i.modifiers.ctrl {
                if self.view_focus == ViewFocus::DataView {
                    self.add_field_window_open = true;
                }
            }
        });

        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            self.show_menu(ui);
        });

        // File info panel
        egui::TopBottomPanel::top("file_info").show(ctx, |ui| {
            self.show_file_info(ui);
        });

        // Show add field window if open
        self.show_add_field_window(ctx);

        // Show edit field window if open
        self.show_edit_field_window(ctx);

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.binary_data.is_loaded() {
                ui.centered_and_justified(|ui| {
                    ui.label("Open a file to get started (File → Open...)");
                });
                return;
            }

            // Split view: hex on left, data on right
            let hex_focused = self.view_focus == ViewFocus::HexView;
            let data_focused = self.view_focus == ViewFocus::DataView;

            ui.columns(2, |columns| {
                // Hex View with focus indicator
                let hex_frame = if hex_focused {
                    egui::Frame::group(columns[0].style())
                        .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255)))
                } else {
                    egui::Frame::group(columns[0].style())
                };

                hex_frame.show(&mut columns[0], |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Hex View");
                        if hex_focused {
                            ui.label(egui::RichText::new("●").color(egui::Color32::from_rgb(100, 150, 255)));
                        }
                    });
                    ui.separator();
                    self.hex_view.show(
                        ui,
                        self.binary_data.bytes(),
                        &self.fields,
                        &self.selected_fields,
                    );
                });

                // Data View with focus indicator
                let data_frame = if data_focused {
                    egui::Frame::group(columns[1].style())
                        .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255)))
                } else {
                    egui::Frame::group(columns[1].style())
                };

                data_frame.show(&mut columns[1], |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Data View");
                        if data_focused {
                            ui.label(egui::RichText::new("●").color(egui::Color32::from_rgb(100, 150, 255)));
                        }
                    });
                    ui.separator();
                    if let Some(action) = self.data_view
                        .show(ui, &self.fields, self.binary_data.bytes(), &self.selected_fields)
                    {
                        match action {
                            FieldAction::Select(idx) => {
                                // Multi-selection with Ctrl/Shift support
                                let modifiers = ui.input(|i| i.modifiers);

                                if modifiers.ctrl {
                                    // Ctrl+Click: Toggle field in selection
                                    if self.selected_fields.contains(&idx) {
                                        self.selected_fields.remove(&idx);
                                    } else {
                                        self.selected_fields.insert(idx);
                                    }
                                    self.last_selected_field = Some(idx);
                                } else if modifiers.shift {
                                    // Shift+Click: Select range from last selected to clicked
                                    if let Some(last) = self.last_selected_field {
                                        let start = last.min(idx);
                                        let end = last.max(idx);
                                        for i in start..=end {
                                            self.selected_fields.insert(i);
                                        }
                                    } else {
                                        self.selected_fields.clear();
                                        self.selected_fields.insert(idx);
                                    }
                                    self.last_selected_field = Some(idx);
                                } else {
                                    // Normal click: Select only this field (clear others)
                                    if self.selected_fields.len() == 1 && self.selected_fields.contains(&idx) {
                                        // Toggle if already the only selected field
                                        self.selected_fields.clear();
                                        self.last_selected_field = None;
                                    } else {
                                        self.selected_fields.clear();
                                        self.selected_fields.insert(idx);
                                        self.last_selected_field = Some(idx);
                                    }
                                }
                            }
                            FieldAction::Edit(idx) => {
                                self.start_edit_field(idx);
                            }
                            FieldAction::Delete(idx) => {
                                self.fields.remove(idx);
                                // Remove deleted field from selection
                                self.selected_fields.remove(&idx);
                                // Adjust all remaining selection indices
                                let old_selections: Vec<usize> = self.selected_fields.iter().copied().collect();
                                self.selected_fields.clear();
                                for &field_idx in &old_selections {
                                    if field_idx > idx {
                                        self.selected_fields.insert(field_idx - 1);
                                    } else if field_idx < idx {
                                        self.selected_fields.insert(field_idx);
                                    }
                                    // field_idx == idx was already removed above
                                }
                                // Adjust last_selected_field
                                if let Some(last) = self.last_selected_field {
                                    if last == idx {
                                        self.last_selected_field = None;
                                    } else if last > idx {
                                        self.last_selected_field = Some(last - 1);
                                    }
                                }
                            }
                        }
                    }
                });
            });
        });
    }
}
