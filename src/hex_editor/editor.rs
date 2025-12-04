use std::path::PathBuf;

/// Hex editor state
pub struct HexEditor {
    pub data: Vec<u8>,
    pub file_path: Option<PathBuf>,
    pub cursor_byte: usize,
    pub selection_start: Option<usize>,
    pub editing_in_ascii: bool,
    pub is_editing: bool,
    pub edit_buffer: String,
    pub edit_cursor_pos: usize,
    pub edit_selection_start: Option<usize>,
    pub editing_in_selection_mode: bool,
}

impl Default for HexEditor {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            file_path: None,
            cursor_byte: 0,
            selection_start: None,
            editing_in_ascii: false,
            is_editing: false,
            edit_buffer: String::new(),
            edit_cursor_pos: 0,
            edit_selection_start: None,
            editing_in_selection_mode: false,
        }
    }
}

impl HexEditor {
    // === File management ===

    pub fn open_file(&mut self, path: PathBuf) -> Result<(), std::io::Error> {
        let data = std::fs::read(&path)?;
        self.data = data;
        self.file_path = Some(path);
        self.reset_cursor();
        Ok(())
    }

    pub fn save_file(&mut self) -> Result<(), std::io::Error> {
        if let Some(path) = &self.file_path {
            std::fs::write(path, &self.data)?;
        }
        Ok(())
    }

    pub fn save_file_as(&mut self, path: PathBuf) -> Result<(), std::io::Error> {
        std::fs::write(&path, &self.data)?;
        self.file_path = Some(path);
        Ok(())
    }

    // === Cursor navigation ===

    pub fn move_cursor_left(&mut self) {
        if self.cursor_byte > 0 {
            self.cursor_byte -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_byte < self.data.len().saturating_sub(1) {
            self.cursor_byte += 1;
        }
    }

    pub fn move_cursor_up(&mut self, bytes_per_line: usize) {
        if self.cursor_byte >= bytes_per_line {
            self.cursor_byte -= bytes_per_line;
        }
    }

    pub fn move_cursor_down(&mut self, bytes_per_line: usize) {
        if self.cursor_byte + bytes_per_line < self.data.len() {
            self.cursor_byte += bytes_per_line;
        }
    }

    fn reset_cursor(&mut self) {
        self.cursor_byte = 0;
        self.selection_start = None;
        self.editing_in_ascii = false;
        self.editing_in_selection_mode = false;
        self.cancel_editing();
    }

    // === Selection ===

    pub fn get_selected_range(&self) -> Option<(usize, usize)> {
        self.selection_start.map(|start| {
            if start <= self.cursor_byte {
                (start, self.cursor_byte)
            } else {
                (self.cursor_byte, start)
            }
        })
    }

    pub fn is_byte_selected(&self, byte_index: usize) -> bool {
        if let Some((start, end)) = self.get_selected_range() {
            byte_index >= start && byte_index <= end
        } else {
            false
        }
    }

    pub fn get_selected_text(&self) -> String {
        if let Some((start, end)) = self.get_selected_range() {
            self.data[start..=end]
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            String::new()
        }
    }

    // === Edit mode ===

    pub fn start_editing(&mut self) {
        if self.cursor_byte < self.data.len() {
            self.is_editing = true;

            // Si on a une sélection active, on entre en mode "édition dans sélection"
            self.editing_in_selection_mode = self.selection_start.is_some();

            let byte = self.data[self.cursor_byte];

            if self.editing_in_ascii {
                if byte.is_ascii_graphic() || byte == b' ' {
                    self.edit_buffer = (byte as char).to_string();
                } else {
                    self.edit_buffer.clear();
                }
            } else {
                self.edit_buffer = format!("{:02X}", byte);
            }

            self.edit_cursor_pos = self.edit_buffer.len();
            self.edit_selection_start = None;
        }
    }

    pub fn cancel_editing(&mut self) {
        self.is_editing = false;
        self.edit_buffer.clear();
        self.edit_cursor_pos = 0;
        self.edit_selection_start = None;

        // Si on annule l'édition en mode sélection, on garde la sélection mais on sort du mode
        self.editing_in_selection_mode = false;
    }

    pub fn validate_editing(&mut self) {
        if !self.is_editing || self.cursor_byte >= self.data.len() {
            return;
        }

        if self.editing_in_ascii {
            if let Some(c) = self.edit_buffer.chars().next() {
                if c.is_ascii() {
                    self.data[self.cursor_byte] = c as u8;
                }
            }
        } else if let Ok(value) = u8::from_str_radix(&self.edit_buffer, 16) {
            self.data[self.cursor_byte] = value;
        }

        let was_editing_in_selection = self.editing_in_selection_mode;
        self.cancel_editing();
        self.move_cursor_right();

        // Si on était en mode édition dans sélection, on garde la sélection pour continuer à éditer
        if was_editing_in_selection {
            self.editing_in_selection_mode = true;
        }
    }

    // === Buffer editing ===

    pub fn handle_edit_input(&mut self, c: char) {
        if self.edit_selection_start.is_some() {
            self.delete_edit_selection();
        }

        if self.editing_in_ascii {
            if c.is_ascii() && !c.is_control() {
                self.edit_buffer = c.to_string();
                self.edit_cursor_pos = 1;
            }
        } else if c.is_ascii_hexdigit() && self.edit_buffer.len() < 2 {
            self.edit_buffer.insert(self.edit_cursor_pos, c.to_ascii_uppercase());
            self.edit_cursor_pos = (self.edit_cursor_pos + 1).min(2);
        }
    }

    pub fn move_edit_cursor_left(&mut self) {
        if self.edit_cursor_pos > 0 {
            self.edit_cursor_pos -= 1;
        }
    }

    pub fn move_edit_cursor_right(&mut self) {
        let max_len = if self.editing_in_ascii { 1 } else { 2 };
        if self.edit_cursor_pos < self.edit_buffer.len().min(max_len) {
            self.edit_cursor_pos += 1;
        }
    }

    pub fn move_edit_cursor_home(&mut self) {
        self.edit_cursor_pos = 0;
    }

    pub fn move_edit_cursor_end(&mut self) {
        let max_len = if self.editing_in_ascii { 1 } else { 2 };
        self.edit_cursor_pos = self.edit_buffer.len().min(max_len);
    }

    fn delete_edit_selection(&mut self) {
        if let Some(start) = self.edit_selection_start {
            let min = start.min(self.edit_cursor_pos);
            let max = start.max(self.edit_cursor_pos);
            self.edit_buffer.replace_range(min..max, "");
            self.edit_cursor_pos = min;
            self.edit_selection_start = None;
        }
    }

    pub fn handle_edit_backspace(&mut self) {
        if self.edit_selection_start.is_some() {
            self.delete_edit_selection();
        } else if self.edit_cursor_pos > 0 {
            self.edit_buffer.remove(self.edit_cursor_pos - 1);
            self.edit_cursor_pos -= 1;
        }
    }

    pub fn handle_edit_delete(&mut self) {
        if self.edit_selection_start.is_some() {
            self.delete_edit_selection();
        } else if self.edit_cursor_pos < self.edit_buffer.len() {
            self.edit_buffer.remove(self.edit_cursor_pos);
        }
    }

    // === Copy/Paste ===

    pub fn paste_hex(&mut self, text: &str) {
        let hex_chars: String = text.chars().filter(|c| c.is_ascii_hexdigit()).collect();
        let mut byte_index = self.cursor_byte;
        let mut buffer = String::new();

        for c in hex_chars.chars() {
            if byte_index >= self.data.len() {
                break;
            }

            buffer.push(c);

            if buffer.len() == 2 {
                if let Ok(value) = u8::from_str_radix(&buffer, 16) {
                    self.data[byte_index] = value;
                    byte_index += 1;
                }
                buffer.clear();
            }
        }

        self.cursor_byte = byte_index.min(self.data.len().saturating_sub(1));
        self.selection_start = None;
    }

    pub fn paste_ascii(&mut self, text: &str) {
        let mut byte_index = self.cursor_byte;

        for c in text.chars() {
            if byte_index >= self.data.len() {
                break;
            }

            if c.is_ascii() {
                self.data[byte_index] = c as u8;
                byte_index += 1;
            }
        }

        self.cursor_byte = byte_index.min(self.data.len().saturating_sub(1));
        self.selection_start = None;
    }
}
