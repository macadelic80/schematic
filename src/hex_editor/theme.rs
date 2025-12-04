use eframe::egui;

/// Visual configuration de l'éditeur hexadécimal
pub struct Theme {
    // Sizes and spacing
    pub bytes_per_line: usize,
    pub line_height: f32,
    pub font_size: f32,
    pub char_spacing: f32,
    pub addr_width_chars: f32,
    pub separator_spacing: f32,
    pub padding: f32,

    // Animation speeds
    pub cursor_blink_speed: f32,

    // Colors - Navigation
    pub color_text_normal: egui::Color32,
    pub color_text_address: egui::Color32,
    pub color_text_ascii: egui::Color32,
    pub color_cursor_highlight: egui::Color32,
    pub color_cursor_border: egui::Color32,

    // Couleurs - Selection
    pub color_selection_bg: egui::Color32,
    pub color_selection_text: egui::Color32,

    // Couleurs - Edit mode
    pub color_edit_bg: egui::Color32,
    pub color_edit_border: egui::Color32,
    pub color_edit_text: egui::Color32,
    pub color_edit_cursor: egui::Color32,

    // Borders
    pub cursor_border_width: f32,
    pub edit_border_width: f32,
    pub edit_border_radius: f32,
    pub separator_width: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            // Sizes and spacing
            bytes_per_line: 16,
            line_height: 20.0,
            font_size: 14.0,
            char_spacing: 3.0,
            addr_width_chars: 10.0,
            separator_spacing: 2.0,
            padding: 10.0,

            // Animation speeds
            cursor_blink_speed: 2.0,

            // Colors - Navigation
            color_text_normal: egui::Color32::LIGHT_GRAY,
            color_text_address: egui::Color32::DARK_GRAY,
            color_text_ascii: egui::Color32::GRAY,
            color_cursor_highlight: egui::Color32::from_rgb(255, 255, 100),
            color_cursor_border: egui::Color32::YELLOW,

            // Couleurs - Selection
            color_selection_bg: egui::Color32::from_rgb(60, 120, 180),
            color_selection_text: egui::Color32::WHITE,

            // Couleurs - Edit mode
            color_edit_bg: egui::Color32::from_rgb(80, 80, 80),
            color_edit_border: egui::Color32::from_rgb(255, 200, 0),
            color_edit_text: egui::Color32::WHITE,
            color_edit_cursor: egui::Color32::WHITE,

            // Borders
            cursor_border_width: 2.0,
            edit_border_width: 2.0,
            edit_border_radius: 2.0,
            separator_width: 1.0,
        }
    }
}

impl Theme {
    pub fn font_id(&self) -> egui::FontId {
        egui::FontId::monospace(self.font_size)
    }

    pub fn cursor_stroke(&self) -> egui::Stroke {
        egui::Stroke::new(self.cursor_border_width, self.color_cursor_border)
    }

    pub fn edit_stroke(&self) -> egui::Stroke {
        egui::Stroke::new(self.edit_border_width, self.color_edit_border)
    }

    pub fn separator_stroke(&self) -> egui::Stroke {
        egui::Stroke::new(self.separator_width, self.color_text_address)
    }

    pub fn should_cursor_blink(&self, time: f64) -> bool {
        ((time * self.cursor_blink_speed as f64) as i32 % 2) == 0
    }
}
