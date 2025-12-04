use crate::binary_data::BinaryData;
use crate::hex_editor::ui as hex_ui;
use crate::hex_editor::{HexEditor, Theme};
use crate::schema::{DataType, Field, Schema};
use crate::ui::{DataView, FieldAction};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

// === UI Constants ===

/// Focus indicator color (RGB)
const FOCUS_COLOR: egui::Color32 = egui::Color32::from_rgb(100, 150, 255);

/// Focus indicator stroke width
const FOCUS_BORDER_WIDTH: f32 = 2.0;

/// Default offset input value
const DEFAULT_OFFSET_INPUT: &str = "0";

// === Dialog Text Constants ===

const DIALOG_JUMP_TITLE: &str = "Jump to Address";
const DIALOG_JUMP_PROMPT: &str = "Enter address (hex):";
const DIALOG_JUMP_BTN_OK: &str = "OK";
const DIALOG_JUMP_BTN_CANCEL: &str = "Cancel";

const DIALOG_SEARCH_TITLE: &str = "Search";
const DIALOG_SEARCH_PROMPT: &str = "Search:";
const DIALOG_SEARCH_OPTION_HEX: &str = "Hex";
const DIALOG_SEARCH_OPTION_ASCII: &str = "ASCII";
const DIALOG_SEARCH_BTN_SEARCH: &str = "Search";
const DIALOG_SEARCH_BTN_CLOSE: &str = "Close";

const DIALOG_ADD_FIELD_TITLE: &str = "Add Field";
const DIALOG_ADD_FIELD_LABEL_NAME: &str = "Name:";
const DIALOG_ADD_FIELD_LABEL_OFFSET: &str = "Offset:";
const DIALOG_ADD_FIELD_HINT_OFFSET: &str = "(hex or decimal)";
const DIALOG_ADD_FIELD_LABEL_TYPE: &str = "Type:";
const DIALOG_ADD_FIELD_LABEL_COMMENT: &str = "Comment:";
const DIALOG_ADD_FIELD_BTN_ADD: &str = "Add";
const DIALOG_ADD_FIELD_BTN_CANCEL: &str = "Cancel";

const DIALOG_EDIT_FIELD_TITLE: &str = "Edit Field";
const DIALOG_EDIT_FIELD_BTN_SAVE: &str = "Save";
const DIALOG_EDIT_FIELD_BTN_CANCEL: &str = "Cancel";

// === Menu Text Constants ===

const MENU_FILE: &str = "File";
const MENU_FILE_OPEN: &str = "Open...";
const MENU_FILE_QUIT: &str = "Quit";

const MENU_SCHEMA: &str = "Schema";
const MENU_SCHEMA_ADD_FIELD: &str = "Add Field...";
const MENU_SCHEMA_SAVE: &str = "Save Schema...";
const MENU_SCHEMA_LOAD: &str = "Load Schema...";
const MENU_SCHEMA_CLEAR: &str = "Clear All Fields";

// === Status Messages ===

const MSG_NO_FILE_LOADED: &str = "Open a file to get started (File → Open...)";
const MSG_NO_FILE_IN_INFO: &str = "No file loaded";

// === View Labels ===

const LABEL_HEX_VIEW: &str = "Hex View";
const LABEL_DATA_VIEW: &str = "Data View";
const LABEL_FILE: &str = "File:";
const LABEL_SIZE: &str = "Size:";

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
    /// Hex editor
    hex_editor: HexEditor,
    /// Hex view theme
    hex_theme: Theme,
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
    /// Jump to address dialog state
    show_jump_dialog: bool,
    jump_address_input: String,
    /// Search dialog state
    show_search_dialog: bool,
    search_input: String,
    search_in_ascii: bool,
}

impl Default for SchematicApp {
    fn default() -> Self {
        Self {
            binary_data: BinaryData::new(),
            fields: Vec::new(),
            hex_editor: HexEditor::default(),
            hex_theme: Theme::default(),
            data_view: DataView::new(),
            add_field_window_open: false,
            new_field_name: String::new(),
            new_field_offset: String::from(DEFAULT_OFFSET_INPUT),
            new_field_type_idx: 0,
            new_field_comment: String::new(),
            edit_field_window_open: false,
            edit_field_idx: None,
            edit_field_name: String::new(),
            edit_field_offset: String::from(DEFAULT_OFFSET_INPUT),
            edit_field_type_idx: 0,
            edit_field_comment: String::new(),
            selected_fields: HashSet::new(),
            last_selected_field: None,
            view_focus: ViewFocus::HexView,
            schema_file_path: None,
            show_jump_dialog: false,
            jump_address_input: String::new(),
            show_search_dialog: false,
            search_input: String::new(),
            search_in_ascii: false,
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
                // Sync hex_editor with binary_data
                let _ = self.hex_editor.open_file(path);
            }
        }
    }

    /// Render the top menu bar
    fn show_menu(&mut self, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button(MENU_FILE, |ui| {
                if ui.button(MENU_FILE_OPEN).clicked() {
                    self.open_file();
                    ui.close_menu();
                }

                ui.separator();

                if ui.button(MENU_FILE_QUIT).clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.menu_button(MENU_SCHEMA, |ui| {
                if ui.button(MENU_SCHEMA_ADD_FIELD).clicked() {
                    self.add_field_window_open = true;
                    ui.close_menu();
                }

                ui.separator();

                if ui.button(MENU_SCHEMA_SAVE).clicked() {
                    self.save_schema();
                    ui.close_menu();
                }

                if ui.button(MENU_SCHEMA_LOAD).clicked() {
                    self.load_schema();
                    ui.close_menu();
                }

                ui.separator();

                if ui.button(MENU_SCHEMA_CLEAR).clicked() {
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
        egui::Window::new(DIALOG_ADD_FIELD_TITLE)
            .open(&mut window_open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(DIALOG_ADD_FIELD_LABEL_NAME);
                    ui.text_edit_singleline(&mut self.new_field_name);
                });

                ui.horizontal(|ui| {
                    ui.label(DIALOG_ADD_FIELD_LABEL_OFFSET);
                    ui.text_edit_singleline(&mut self.new_field_offset);
                    ui.label(DIALOG_ADD_FIELD_HINT_OFFSET);
                });

                ui.horizontal(|ui| {
                    ui.label(DIALOG_ADD_FIELD_LABEL_TYPE);
                    egui::ComboBox::from_id_salt("field_type")
                        .selected_text(DataType::all()[self.new_field_type_idx].name())
                        .show_ui(ui, |ui| {
                            for (idx, dt) in DataType::all().iter().enumerate() {
                                ui.selectable_value(&mut self.new_field_type_idx, idx, dt.name());
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label(DIALOG_ADD_FIELD_LABEL_COMMENT);
                    ui.text_edit_singleline(&mut self.new_field_comment);
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button(DIALOG_ADD_FIELD_BTN_ADD).clicked() {
                        if let Some(field) = self.create_field_from_input() {
                            self.fields.push(field);
                            self.reset_add_field_form();
                            self.add_field_window_open = false;
                        }
                    }

                    if ui.button(DIALOG_ADD_FIELD_BTN_CANCEL).clicked() {
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
        egui::Window::new(DIALOG_EDIT_FIELD_TITLE)
            .open(&mut window_open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(DIALOG_ADD_FIELD_LABEL_NAME);
                    ui.text_edit_singleline(&mut self.edit_field_name);
                });

                ui.horizontal(|ui| {
                    ui.label(DIALOG_ADD_FIELD_LABEL_OFFSET);
                    ui.text_edit_singleline(&mut self.edit_field_offset);
                    ui.label(DIALOG_ADD_FIELD_HINT_OFFSET);
                });

                ui.horizontal(|ui| {
                    ui.label(DIALOG_ADD_FIELD_LABEL_TYPE);
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
                    if ui.button(DIALOG_EDIT_FIELD_BTN_SAVE).clicked() {
                        if self.update_field_from_input() {
                            self.edit_field_window_open = false;
                        }
                    }

                    if ui.button(DIALOG_ADD_FIELD_BTN_CANCEL).clicked() {
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
                ui.label(LABEL_FILE);
                if let Some(path) = self.binary_data.file_path() {
                    ui.label(path.display().to_string());
                } else {
                    ui.label(MSG_NO_FILE_IN_INFO);
                }
            });

            if self.binary_data.is_loaded() {
                ui.horizontal(|ui| {
                    ui.label(LABEL_SIZE);
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

    /// Handle keyboard navigation in hex view
    fn handle_keyboard_navigation(&mut self, ui: &egui::Ui) {
        if self.hex_editor.is_editing {
            return; // Skip navigation when editing (not implemented yet)
        }

        let shift_pressed = ui.input(|i| i.modifiers.shift);

        // Manage selection with Shift
        let handle_selection = |editor: &mut HexEditor| {
            if shift_pressed {
                if editor.selection_start.is_none() {
                    editor.selection_start = Some(editor.cursor_byte);
                }
            } else {
                if !editor.editing_in_selection_mode {
                    editor.selection_start = None;
                }
            }
        };

        // Arrow navigation
        if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
            handle_selection(&mut self.hex_editor);
            self.hex_editor.move_cursor_left();
        }

        if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
            handle_selection(&mut self.hex_editor);
            self.hex_editor.move_cursor_right();
        }

        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            handle_selection(&mut self.hex_editor);
            self.hex_editor
                .move_cursor_up(self.hex_theme.bytes_per_line);
        }

        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            handle_selection(&mut self.hex_editor);
            self.hex_editor
                .move_cursor_down(self.hex_theme.bytes_per_line);
        }

        // Copy selection
        if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::C)) {
            let text = self.hex_editor.get_selected_text();
            if !text.is_empty() {
                ui.output_mut(|o| o.copied_text = text);
            }
        }
    }

    /// Get byte at mouse position
    fn get_byte_at_pos(
        &self,
        pos: egui::Pos2,
        layout: &hex_ui::LayoutInfo,
    ) -> Option<(usize, bool)> {
        let clicked_x_from_hex = pos.x - layout.hex_start_x;
        let clicked_x_from_ascii = pos.x - layout.ascii_start_x;
        let clicked_y = pos.y - layout.start_pos.y;

        if clicked_y < 0.0 {
            return None;
        }

        let line = (clicked_y / self.hex_theme.line_height) as usize;
        let hex_section_width =
            layout.char_width * self.hex_theme.char_spacing * self.hex_theme.bytes_per_line as f32;

        // Click in ASCII zone
        if clicked_x_from_ascii >= 0.0 {
            let col = (clicked_x_from_ascii / layout.char_width) as usize;
            let byte_index = line * self.hex_theme.bytes_per_line + col;
            if byte_index < self.hex_editor.data.len() {
                return Some((byte_index, true));
            }
        }
        // Click in hex zone
        else if clicked_x_from_hex >= 0.0 && clicked_x_from_hex < hex_section_width {
            let hex_col_width = layout.char_width * self.hex_theme.char_spacing;
            let col = (clicked_x_from_hex / hex_col_width) as usize;
            let byte_index = line * self.hex_theme.bytes_per_line + col;
            if byte_index < self.hex_editor.data.len() {
                return Some((byte_index, false));
            }
        }

        None
    }

    /// Handle mouse click in hex view
    fn handle_mouse_click(
        &mut self,
        _ui: &egui::Ui,
        response: &egui::Response,
        layout: &hex_ui::LayoutInfo,
    ) {
        if !response.clicked() || self.hex_editor.is_editing {
            return;
        }

        if let Some(pos) = response.interact_pointer_pos() {
            if let Some((byte_index, is_ascii)) = self.get_byte_at_pos(pos, layout) {
                self.hex_editor.cursor_byte = byte_index;
                self.hex_editor.selection_start = None;
                self.hex_editor.editing_in_ascii = is_ascii;
                self.hex_editor.editing_in_selection_mode = false;
            }
        }
    }

    /// Handle mouse drag in hex view
    fn handle_mouse_drag(&mut self, response: &egui::Response, layout: &hex_ui::LayoutInfo) {
        if self.hex_editor.is_editing {
            return;
        }

        if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                if let Some((byte_index, is_ascii)) = self.get_byte_at_pos(pos, layout) {
                    self.hex_editor.cursor_byte = byte_index;
                    self.hex_editor.selection_start = Some(byte_index);
                    self.hex_editor.editing_in_ascii = is_ascii;
                    self.hex_editor.editing_in_selection_mode = false;
                }
            }
        }

        if response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                if let Some((byte_index, _)) = self.get_byte_at_pos(pos, layout) {
                    self.hex_editor.cursor_byte = byte_index;
                }
            }
        }
    }

    /// Show jump to address dialog (Ctrl+G)
    fn show_jump_to_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Window::new(DIALOG_JUMP_TITLE)
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.show_jump_dialog = false;
                    self.jump_address_input.clear();
                    return;
                }

                ui.label(DIALOG_JUMP_PROMPT);
                let response = ui.text_edit_singleline(&mut self.jump_address_input);
                response.request_focus();

                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Ok(address) = usize::from_str_radix(
                        &self.jump_address_input.trim().trim_start_matches("0x"),
                        16,
                    ) {
                        if address < self.hex_editor.data.len() {
                            self.hex_editor.cursor_byte = address;
                            self.hex_editor.selection_start = None;
                            self.hex_editor.editing_in_selection_mode = false;
                            self.show_jump_dialog = false;
                            self.jump_address_input.clear();
                        }
                    }
                }

                ui.horizontal(|ui| {
                    if ui.button(DIALOG_JUMP_BTN_OK).clicked() {
                        if let Ok(address) = usize::from_str_radix(
                            &self.jump_address_input.trim().trim_start_matches("0x"),
                            16,
                        ) {
                            if address < self.hex_editor.data.len() {
                                self.hex_editor.cursor_byte = address;
                                self.hex_editor.selection_start = None;
                                self.hex_editor.editing_in_selection_mode = false;
                                self.show_jump_dialog = false;
                                self.jump_address_input.clear();
                            }
                        }
                    }

                    if ui.button(DIALOG_ADD_FIELD_BTN_CANCEL).clicked() {
                        self.show_jump_dialog = false;
                        self.jump_address_input.clear();
                    }
                });
            });

        if !open {
            self.show_jump_dialog = false;
            self.jump_address_input.clear();
        }
    }

    /// Show search dialog (Ctrl+F)
    fn show_search_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Window::new(DIALOG_SEARCH_TITLE)
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.show_search_dialog = false;
                    return;
                }

                ui.horizontal(|ui| {
                    ui.label(DIALOG_SEARCH_PROMPT);
                    let response = ui.text_edit_singleline(&mut self.search_input);
                    response.request_focus();
                });

                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.search_in_ascii, false, DIALOG_SEARCH_OPTION_HEX);
                    ui.radio_value(&mut self.search_in_ascii, true, DIALOG_SEARCH_OPTION_ASCII);
                });

                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.perform_search();
                }

                ui.horizontal(|ui| {
                    if ui.button(DIALOG_SEARCH_BTN_SEARCH).clicked() {
                        self.perform_search();
                    }

                    if ui.button(DIALOG_SEARCH_BTN_CLOSE).clicked() {
                        self.show_search_dialog = false;
                    }
                });
            });

        if !open {
            self.show_search_dialog = false;
        }
    }

    /// Perform search operation
    fn perform_search(&mut self) {
        if self.search_input.is_empty() {
            return;
        }

        let start_pos = if self.hex_editor.selection_start.is_some() {
            (self.hex_editor.cursor_byte + 1).min(self.hex_editor.data.len())
        } else {
            0
        };

        if self.search_in_ascii {
            // ASCII search
            let search_bytes: Vec<u8> = self.search_input.bytes().collect();
            if let Some(pos) = self.find_bytes(&search_bytes, start_pos) {
                self.hex_editor.selection_start = Some(pos);
                self.hex_editor.cursor_byte = pos + search_bytes.len() - 1;
                self.hex_editor.editing_in_selection_mode = false;
            }
        } else {
            // Hex search
            let hex_chars: String = self
                .search_input
                .chars()
                .filter(|c| c.is_ascii_hexdigit())
                .collect();
            if hex_chars.len() >= 2 && hex_chars.len() % 2 == 0 {
                let mut search_bytes = Vec::new();
                for i in (0..hex_chars.len()).step_by(2) {
                    if let Ok(byte) = u8::from_str_radix(&hex_chars[i..i + 2], 16) {
                        search_bytes.push(byte);
                    }
                }
                if let Some(pos) = self.find_bytes(&search_bytes, start_pos) {
                    self.hex_editor.selection_start = Some(pos);
                    self.hex_editor.cursor_byte = pos + search_bytes.len() - 1;
                    self.hex_editor.editing_in_selection_mode = false;
                }
            }
        }
    }

    /// Find bytes in data starting from a position
    fn find_bytes(&self, pattern: &[u8], start_pos: usize) -> Option<usize> {
        if pattern.is_empty() || start_pos >= self.hex_editor.data.len() {
            return None;
        }

        self.hex_editor.data[start_pos..]
            .windows(pattern.len())
            .position(|window| window == pattern)
            .map(|pos| pos + start_pos)
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

            // Ctrl+G: Jump to address (HexView only)
            if i.key_pressed(egui::Key::G) && i.modifiers.ctrl {
                if self.view_focus == ViewFocus::HexView
                    && !self.show_jump_dialog
                    && !self.show_search_dialog
                {
                    self.show_jump_dialog = true;
                }
            }

            // Ctrl+F: Search (HexView only)
            if i.key_pressed(egui::Key::F) && i.modifiers.ctrl {
                if self.view_focus == ViewFocus::HexView
                    && !self.show_jump_dialog
                    && !self.show_search_dialog
                {
                    self.show_search_dialog = true;
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

        // Show jump dialog if open
        if self.show_jump_dialog {
            self.show_jump_to_dialog(ctx);
        }

        // Show search dialog if open
        if self.show_search_dialog {
            self.show_search_dialog(ctx);
        }

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
                        .stroke(egui::Stroke::new(2.0, FOCUS_COLOR))
                } else {
                    egui::Frame::group(columns[0].style())
                };

                hex_frame.show(&mut columns[0], |ui| {
                    ui.horizontal(|ui| {
                        ui.heading(LABEL_HEX_VIEW);
                        if hex_focused {
                            ui.label(egui::RichText::new("●").color(FOCUS_COLOR));
                        }
                    });
                    ui.separator();

                    // Hex editor view
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let num_lines =
                            (self.hex_editor.data.len() + self.hex_theme.bytes_per_line - 1)
                                / self.hex_theme.bytes_per_line;
                        let content_height = self.hex_theme.padding * 2.0
                            + num_lines as f32 * self.hex_theme.line_height;

                        let response = ui.allocate_response(
                            egui::vec2(ui.available_width(), content_height.max(400.0)),
                            egui::Sense::click_and_drag(),
                        );

                        if response.clicked() {
                            ui.memory_mut(|mem| mem.request_focus(response.id));
                        }

                        let has_focus = response.has_focus() && hex_focused;

                        if has_focus {
                            ui.ctx().memory_mut(|mem| {
                                mem.set_focus_lock_filter(
                                    response.id,
                                    egui::EventFilter {
                                        tab: true,
                                        horizontal_arrows: true,
                                        vertical_arrows: true,
                                        escape: true,
                                    },
                                )
                            });

                            // Handle keyboard navigation
                            self.handle_keyboard_navigation(ui);
                        }

                        let painter = ui.painter();
                        let layout =
                            hex_ui::LayoutInfo::calculate(&self.hex_theme, painter, &response);

                        // Handle mouse interactions
                        self.handle_mouse_click(ui, &response, &layout);
                        self.handle_mouse_drag(&response, &layout);

                        // Render hex editor components
                        hex_ui::render_addresses(
                            &self.hex_editor,
                            &self.hex_theme,
                            painter,
                            &layout,
                        );
                        hex_ui::render_separator(
                            &self.hex_editor,
                            &self.hex_theme,
                            painter,
                            &layout,
                        );
                        hex_ui::render_selection_background(
                            &self.hex_editor,
                            &self.hex_theme,
                            painter,
                            &layout,
                        );

                        // Render field highlights BEFORE rendering bytes
                        if !self.fields.is_empty() {
                            hex_ui::render_field_highlights(
                                &self.fields,
                                &self.selected_fields,
                                &self.hex_theme,
                                painter,
                                &layout,
                                0,
                                self.hex_editor.data.len(),
                            );
                        }

                        let time = ui.input(|i| i.time);

                        // Render bytes
                        for (i, &byte) in self.hex_editor.data.iter().enumerate() {
                            hex_ui::render_hex_byte(
                                &self.hex_editor,
                                &self.hex_theme,
                                painter,
                                &layout,
                                i,
                                byte,
                                has_focus,
                                time,
                            );

                            hex_ui::render_ascii_byte(
                                &self.hex_editor,
                                &self.hex_theme,
                                painter,
                                &layout,
                                i,
                                byte,
                                has_focus,
                                time,
                            );
                        }
                    });
                });

                // Data View with focus indicator
                let data_frame = if data_focused {
                    egui::Frame::group(columns[1].style())
                        .stroke(egui::Stroke::new(2.0, FOCUS_COLOR))
                } else {
                    egui::Frame::group(columns[1].style())
                };

                data_frame.show(&mut columns[1], |ui| {
                    ui.horizontal(|ui| {
                        ui.heading(LABEL_DATA_VIEW);
                        if data_focused {
                            ui.label(egui::RichText::new("●").color(FOCUS_COLOR));
                        }
                    });
                    ui.separator();
                    if let Some(action) = self.data_view.show(
                        ui,
                        &self.fields,
                        self.binary_data.bytes(),
                        &self.selected_fields,
                    ) {
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
                                    if self.selected_fields.len() == 1
                                        && self.selected_fields.contains(&idx)
                                    {
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
                                let old_selections: Vec<usize> =
                                    self.selected_fields.iter().copied().collect();
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
