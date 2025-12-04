use crate::hex_editor::editor::HexEditor;
use crate::hex_editor::theme::Theme;
use crate::schema::Field;
use eframe::egui;
use std::collections::HashSet;

pub struct LayoutInfo {
    pub char_width: f32,
    pub hex_start_x: f32,
    pub ascii_start_x: f32,
    pub start_pos: egui::Pos2,
}

impl LayoutInfo {
    pub fn calculate(
        theme: &Theme,
        painter: &egui::Painter,
        response: &egui::Response,
    ) -> Self {
        let char_width = painter
            .layout_no_wrap(
                "0".to_string(),
                theme.font_id(),
                egui::Color32::WHITE,
            )
            .rect
            .width();

        let start_pos = response.rect.left_top() + egui::vec2(theme.padding, theme.padding);
        let addr_width = char_width * theme.addr_width_chars;
        let hex_section_width = char_width * theme.char_spacing * theme.bytes_per_line as f32;
        let hex_start_x = start_pos.x + addr_width;
        let ascii_start_x = hex_start_x + hex_section_width + char_width * theme.separator_spacing;

        Self {
            char_width,
            hex_start_x,
            ascii_start_x,
            start_pos,
        }
    }

    pub fn byte_position(&self, theme: &Theme, byte_index: usize) -> (f32, f32) {
        let line = byte_index / theme.bytes_per_line;
        let col = byte_index % theme.bytes_per_line;
        let x = self.hex_start_x + col as f32 * (self.char_width * theme.char_spacing);
        let y = self.start_pos.y + line as f32 * theme.line_height;
        (x, y)
    }

    pub fn ascii_position(&self, theme: &Theme, byte_index: usize) -> (f32, f32) {
        let line = byte_index / theme.bytes_per_line;
        let col = byte_index % theme.bytes_per_line;
        let x = self.ascii_start_x + col as f32 * self.char_width;
        let y = self.start_pos.y + line as f32 * theme.line_height;
        (x, y)
    }
}

pub fn render_addresses(
    editor: &HexEditor,
    theme: &Theme,
    painter: &egui::Painter,
    layout: &LayoutInfo,
) {
    let num_lines = (editor.data.len() + theme.bytes_per_line - 1) / theme.bytes_per_line;

    for line in 0..num_lines {
        let y = layout.start_pos.y + line as f32 * theme.line_height;
        let addr = line * theme.bytes_per_line;

        painter.text(
            egui::pos2(layout.start_pos.x, y),
            egui::Align2::LEFT_TOP,
            format!("{:08X}:", addr),
            theme.font_id(),
            theme.color_text_address,
        );
    }
}

pub fn render_separator(
    editor: &HexEditor,
    theme: &Theme,
    painter: &egui::Painter,
    layout: &LayoutInfo,
) {
    let sep_x = layout.ascii_start_x - layout.char_width;
    let num_lines = (editor.data.len() + theme.bytes_per_line - 1) / theme.bytes_per_line;
    let start_y = layout.start_pos.y;
    let end_y = start_y + num_lines as f32 * theme.line_height;

    painter.vline(sep_x, start_y..=end_y, theme.separator_stroke());
}

pub fn render_selection_background(
    editor: &HexEditor,
    theme: &Theme,
    painter: &egui::Painter,
    layout: &LayoutInfo,
) {
    if let Some((start_byte, end_byte)) = editor.get_selected_range() {
        let start_line = start_byte / theme.bytes_per_line;
        let start_col = start_byte % theme.bytes_per_line;
        let end_line = end_byte / theme.bytes_per_line;
        let end_col = end_byte % theme.bytes_per_line;

        for line in start_line..=end_line {
            let line_start_col = if line == start_line { start_col } else { 0 };
            let line_end_col = if line == end_line {
                end_col
            } else {
                theme.bytes_per_line - 1
            };

            let y = layout.start_pos.y + line as f32 * theme.line_height;

            // Hex section
            let hex_start_x = layout.hex_start_x + line_start_col as f32 * (layout.char_width * theme.char_spacing);
            let hex_width = (line_end_col - line_start_col + 1) as f32 * (layout.char_width * theme.char_spacing);

            let hex_rect = egui::Rect::from_min_size(
                egui::pos2(hex_start_x, y),
                egui::vec2(hex_width, theme.line_height),
            );
            painter.rect_filled(hex_rect, 0.0, theme.color_selection_bg);

            // ASCII section
            let ascii_start_x = layout.ascii_start_x + line_start_col as f32 * layout.char_width;
            let ascii_width = (line_end_col - line_start_col + 1) as f32 * layout.char_width;

            let ascii_rect = egui::Rect::from_min_size(
                egui::pos2(ascii_start_x, y),
                egui::vec2(ascii_width, theme.line_height),
            );
            painter.rect_filled(ascii_rect, 0.0, theme.color_selection_bg);
        }

        // Bordure autour de la sélection ASCII
        let border_stroke = egui::Stroke::new(2.0, theme.color_selection_text);

        if start_line == end_line {
            // Une seule ligne - rectangle simple
            let start_x = layout.ascii_start_x + start_col as f32 * layout.char_width;
            let end_x = layout.ascii_start_x + (end_col + 1) as f32 * layout.char_width;
            let y = layout.start_pos.y + start_line as f32 * theme.line_height;

            let selection_rect = egui::Rect::from_min_max(
                egui::pos2(start_x, y),
                egui::pos2(end_x, y + theme.line_height),
            );
            painter.rect_stroke(selection_rect, 0.0, border_stroke);
        } else {
            // Plusieurs lignes - dessiner uniquement le contour extérieur
            let max_x = layout.ascii_start_x + theme.bytes_per_line as f32 * layout.char_width;

            // Starting line
            let start_y = layout.start_pos.y + start_line as f32 * theme.line_height;
            let start_x = layout.ascii_start_x + start_col as f32 * layout.char_width;

            // Top border de la première ligne
            painter.line_segment(
                [egui::pos2(start_x, start_y), egui::pos2(max_x, start_y)],
                border_stroke,
            );
            // Left border de la première ligne
            painter.line_segment(
                [egui::pos2(start_x, start_y), egui::pos2(start_x, start_y + theme.line_height)],
                border_stroke,
            );
            // Right border de la première ligne
            painter.line_segment(
                [egui::pos2(max_x, start_y), egui::pos2(max_x, start_y + theme.line_height)],
                border_stroke,
            );

            // Connexion entre la première ligne et la deuxième (si la sélection ne commence pas au début)
            if start_col > 0 {
                let second_line_y = layout.start_pos.y + (start_line + 1) as f32 * theme.line_height;
                // Horizontal de start_x vers ascii_start_x au bas de la première ligne
                painter.line_segment(
                    [egui::pos2(start_x, start_y + theme.line_height), egui::pos2(layout.ascii_start_x, start_y + theme.line_height)],
                    border_stroke,
                );
                // Vertical de bas de ligne 1 vers haut de ligne 2
                painter.line_segment(
                    [egui::pos2(layout.ascii_start_x, start_y + theme.line_height), egui::pos2(layout.ascii_start_x, second_line_y)],
                    border_stroke,
                );
            }

            // Intermediate lines (complete)
            for line in (start_line + 1)..end_line {
                let y = layout.start_pos.y + line as f32 * theme.line_height;

                // Continuous left and right borders
                painter.line_segment(
                    [egui::pos2(layout.ascii_start_x, y), egui::pos2(layout.ascii_start_x, y + theme.line_height)],
                    border_stroke,
                );
                painter.line_segment(
                    [egui::pos2(max_x, y), egui::pos2(max_x, y + theme.line_height)],
                    border_stroke,
                );
            }

            // Ending line
            let end_y = layout.start_pos.y + end_line as f32 * theme.line_height;
            let end_x = layout.ascii_start_x + (end_col + 1) as f32 * layout.char_width;

            // Connexion entre l'avant-dernière ligne et la dernière (si la sélection se termine avant la fin)
            if end_col < theme.bytes_per_line - 1 {
                // Horizontal de max_x vers end_x au haut de la dernière ligne
                painter.line_segment(
                    [egui::pos2(max_x, end_y), egui::pos2(end_x, end_y)],
                    border_stroke,
                );
            }

            // Left border de la dernière ligne
            painter.line_segment(
                [egui::pos2(layout.ascii_start_x, end_y), egui::pos2(layout.ascii_start_x, end_y + theme.line_height)],
                border_stroke,
            );
            // Bottom border de la dernière ligne
            painter.line_segment(
                [egui::pos2(layout.ascii_start_x, end_y + theme.line_height), egui::pos2(end_x, end_y + theme.line_height)],
                border_stroke,
            );
            // Right border de la dernière ligne
            painter.line_segment(
                [egui::pos2(end_x, end_y), egui::pos2(end_x, end_y + theme.line_height)],
                border_stroke,
            );
        }

        // Bordure autour de toute la sélection (hex) - suit le contour exact
        let border_stroke = egui::Stroke::new(2.0, theme.color_selection_text);

        if start_line == end_line {
            // Une seule ligne - rectangle simple
            let start_x = layout.hex_start_x + start_col as f32 * (layout.char_width * theme.char_spacing);
            let end_x = layout.hex_start_x + (end_col + 1) as f32 * (layout.char_width * theme.char_spacing);
            let y = layout.start_pos.y + start_line as f32 * theme.line_height;

            let selection_rect = egui::Rect::from_min_max(
                egui::pos2(start_x, y),
                egui::pos2(end_x, y + theme.line_height),
            );
            painter.rect_stroke(selection_rect, 0.0, border_stroke);
        } else {
            // Plusieurs lignes - dessiner uniquement le contour extérieur
            let max_x = layout.hex_start_x + theme.bytes_per_line as f32 * (layout.char_width * theme.char_spacing);

            // Starting line
            let start_y = layout.start_pos.y + start_line as f32 * theme.line_height;
            let start_x = layout.hex_start_x + start_col as f32 * (layout.char_width * theme.char_spacing);

            // Top border de la première ligne
            painter.line_segment(
                [egui::pos2(start_x, start_y), egui::pos2(max_x, start_y)],
                border_stroke,
            );
            // Left border de la première ligne
            painter.line_segment(
                [egui::pos2(start_x, start_y), egui::pos2(start_x, start_y + theme.line_height)],
                border_stroke,
            );
            // Right border de la première ligne
            painter.line_segment(
                [egui::pos2(max_x, start_y), egui::pos2(max_x, start_y + theme.line_height)],
                border_stroke,
            );

            // Connexion entre la première ligne et la deuxième (si la sélection ne commence pas au début)
            if start_col > 0 {
                let second_line_y = layout.start_pos.y + (start_line + 1) as f32 * theme.line_height;
                // Horizontal de start_x vers hex_start_x au bas de la première ligne
                painter.line_segment(
                    [egui::pos2(start_x, start_y + theme.line_height), egui::pos2(layout.hex_start_x, start_y + theme.line_height)],
                    border_stroke,
                );
                // Vertical de bas de ligne 1 vers haut de ligne 2
                painter.line_segment(
                    [egui::pos2(layout.hex_start_x, start_y + theme.line_height), egui::pos2(layout.hex_start_x, second_line_y)],
                    border_stroke,
                );
            }

            // Intermediate lines (complete)
            for line in (start_line + 1)..end_line {
                let y = layout.start_pos.y + line as f32 * theme.line_height;

                // Continuous left and right borders
                painter.line_segment(
                    [egui::pos2(layout.hex_start_x, y), egui::pos2(layout.hex_start_x, y + theme.line_height)],
                    border_stroke,
                );
                painter.line_segment(
                    [egui::pos2(max_x, y), egui::pos2(max_x, y + theme.line_height)],
                    border_stroke,
                );
            }

            // Ending line
            let end_y = layout.start_pos.y + end_line as f32 * theme.line_height;
            let end_x = layout.hex_start_x + (end_col + 1) as f32 * (layout.char_width * theme.char_spacing);

            // Connexion entre l'avant-dernière ligne et la dernière (si la sélection se termine avant la fin)
            if end_col < theme.bytes_per_line - 1 {
                // Horizontal de max_x vers end_x au haut de la dernière ligne
                painter.line_segment(
                    [egui::pos2(max_x, end_y), egui::pos2(end_x, end_y)],
                    border_stroke,
                );
            }

            // Left border de la dernière ligne
            painter.line_segment(
                [egui::pos2(layout.hex_start_x, end_y), egui::pos2(layout.hex_start_x, end_y + theme.line_height)],
                border_stroke,
            );
            // Bottom border de la dernière ligne
            painter.line_segment(
                [egui::pos2(layout.hex_start_x, end_y + theme.line_height), egui::pos2(end_x, end_y + theme.line_height)],
                border_stroke,
            );
            // Right border de la dernière ligne
            painter.line_segment(
                [egui::pos2(end_x, end_y), egui::pos2(end_x, end_y + theme.line_height)],
                border_stroke,
            );
        }
    }
}

pub fn render_hex_byte(
    editor: &HexEditor,
    theme: &Theme,
    painter: &egui::Painter,
    layout: &LayoutInfo,
    byte_index: usize,
    byte: u8,
    has_focus: bool,
    time: f64,
) {
    let (hex_x, y) = layout.byte_position(theme, byte_index);
    let hex_pos = egui::pos2(hex_x, y);

    let is_selected = editor.is_byte_selected(byte_index);
    let is_cursor = has_focus && byte_index == editor.cursor_byte;
    let is_editing_this = editor.is_editing && is_cursor && !editor.editing_in_ascii;

    // La sélection est déjà rendue globalement, pas besoin de la redessiner

    // Fond et bordure du mode édition
    if is_editing_this {
        let bg_rect = egui::Rect::from_min_size(
            hex_pos,
            egui::vec2(layout.char_width * 2.0, theme.line_height),
        );
        painter.rect_filled(bg_rect, theme.edit_border_radius, theme.color_edit_bg);
        painter.rect_stroke(bg_rect, theme.edit_border_radius, theme.edit_stroke());

        // Selection in edit buffer
        if let Some(sel_start) = editor.edit_selection_start {
            let min_pos = sel_start.min(editor.edit_cursor_pos);
            let max_pos = sel_start.max(editor.edit_cursor_pos);
            if min_pos != max_pos {
                let sel_rect = egui::Rect::from_min_size(
                    egui::pos2(hex_pos.x + min_pos as f32 * layout.char_width, y),
                    egui::vec2((max_pos - min_pos) as f32 * layout.char_width, theme.line_height),
                );
                painter.rect_filled(sel_rect, 0.0, theme.color_selection_bg);
            }
        }
    } else if is_cursor {
        // Bordure du curseur
        let bg_rect = egui::Rect::from_min_size(
            hex_pos,
            egui::vec2(layout.char_width * 2.0, theme.line_height),
        );
        painter.rect_stroke(bg_rect, 0.0, theme.cursor_stroke());
    }

    // Texte du byte
    let byte_str = if is_editing_this {
        format!("{:_<2}", editor.edit_buffer)
    } else {
        format!("{:02X}", byte)
    };

    let text_color = if is_editing_this {
        theme.color_edit_text
    } else if is_selected {
        theme.color_selection_text
    } else if is_cursor {
        theme.color_cursor_highlight
    } else {
        theme.color_text_normal
    };

    painter.text(hex_pos, egui::Align2::LEFT_TOP, byte_str, theme.font_id(), text_color);

    // Blinking cursor in edit mode
    if is_editing_this && theme.should_cursor_blink(time) {
        let cursor_x = hex_pos.x + editor.edit_cursor_pos as f32 * layout.char_width;
        painter.vline(
            cursor_x,
            y..=(y + theme.line_height),
            egui::Stroke::new(2.0, theme.color_edit_cursor),
        );
    }
}

pub fn render_ascii_byte(
    editor: &HexEditor,
    theme: &Theme,
    painter: &egui::Painter,
    layout: &LayoutInfo,
    byte_index: usize,
    byte: u8,
    has_focus: bool,
    time: f64,
) {
    let (ascii_x, y) = layout.ascii_position(theme, byte_index);
    let ascii_pos = egui::pos2(ascii_x, y);

    let is_selected = editor.is_byte_selected(byte_index);
    let is_cursor = has_focus && byte_index == editor.cursor_byte;
    let is_editing_ascii_this = editor.is_editing && is_cursor && editor.editing_in_ascii;

    // La sélection est déjà rendue globalement, pas besoin de la redessiner

    // Fond et bordure du mode édition
    if is_editing_ascii_this {
        let bg_rect = egui::Rect::from_min_size(
            ascii_pos,
            egui::vec2(layout.char_width, theme.line_height),
        );
        painter.rect_filled(bg_rect, theme.edit_border_radius, theme.color_edit_bg);
        painter.rect_stroke(bg_rect, theme.edit_border_radius, theme.edit_stroke());
    } else if is_cursor {
        // Bordure du curseur
        let bg_rect = egui::Rect::from_min_size(
            ascii_pos,
            egui::vec2(layout.char_width, theme.line_height),
        );
        painter.rect_stroke(bg_rect, 0.0, theme.cursor_stroke());
    }

    // ASCII character
    let ascii_char = if is_editing_ascii_this {
        editor.edit_buffer.chars().next().unwrap_or('_')
    } else if byte.is_ascii_graphic() || byte == b' ' {
        byte as char
    } else {
        '.'
    };

    let ascii_color = if is_editing_ascii_this {
        theme.color_edit_text
    } else if is_selected {
        theme.color_selection_text
    } else if is_cursor {
        theme.color_cursor_highlight
    } else {
        theme.color_text_ascii
    };

    painter.text(ascii_pos, egui::Align2::LEFT_TOP, ascii_char.to_string(), theme.font_id(), ascii_color);

    // Blinking cursor in edit mode
    if is_editing_ascii_this && theme.should_cursor_blink(time) {
        let cursor_x = if editor.edit_buffer.is_empty() {
            ascii_x
        } else {
            ascii_x + layout.char_width
        };
        painter.vline(
            cursor_x,
            y..=(y + theme.line_height),
            egui::Stroke::new(2.0, theme.color_edit_cursor),
        );
    }
}

// === Field Highlighting Support ===

/// Get the field that contains the given byte offset, if any
fn get_field_at_offset(fields: &[Field], offset: usize) -> Option<(usize, &Field)> {
    fields
        .iter()
        .enumerate()
        .find(|(_, field)| offset >= field.offset && offset < field.offset + field.size())
}

/// Generate a distinct color for each field
fn get_field_color(field_idx: usize) -> egui::Color32 {
    let colors = [
        egui::Color32::from_rgb(100, 150, 255), // Blue
        egui::Color32::from_rgb(255, 150, 100), // Orange
        egui::Color32::from_rgb(150, 255, 100), // Green
        egui::Color32::from_rgb(255, 100, 200), // Pink
        egui::Color32::from_rgb(200, 100, 255), // Purple
        egui::Color32::from_rgb(100, 255, 200), // Cyan
        egui::Color32::from_rgb(255, 255, 100), // Yellow
        egui::Color32::from_rgb(255, 150, 150), // Light red
    ];
    colors[field_idx % colors.len()]
}

/// Render field highlights for the visible bytes
pub fn render_field_highlights(
    fields: &[Field],
    selected_fields: &HashSet<usize>,
    theme: &Theme,
    painter: &egui::Painter,
    layout: &LayoutInfo,
    visible_start: usize,
    visible_end: usize,
) {
    // Group consecutive bytes by field for efficient rendering
    let mut current_field: Option<(usize, usize, usize)> = None; // (field_idx, start_byte, end_byte)

    for byte_offset in visible_start..visible_end {
        if let Some((field_idx, field)) = get_field_at_offset(fields, byte_offset) {
            // Check if this byte is within the field's range
            if byte_offset >= field.offset && byte_offset < field.offset + field.size() {
                match current_field {
                    Some((curr_field_idx, start, _)) if curr_field_idx == field_idx => {
                        // Same field, extend the range
                        current_field = Some((field_idx, start, byte_offset));
                    }
                    _ => {
                        // Draw previous field if any
                        if let Some((prev_field_idx, start, end)) = current_field {
                            draw_field_highlight(
                                painter,
                                layout,
                                theme,
                                start,
                                end,
                                prev_field_idx,
                                selected_fields,
                            );
                        }
                        // Start new field
                        current_field = Some((field_idx, byte_offset, byte_offset));
                    }
                }
            }
        } else {
            // No field, draw previous if any
            if let Some((prev_field_idx, start, end)) = current_field {
                draw_field_highlight(
                    painter,
                    layout,
                    theme,
                    start,
                    end,
                    prev_field_idx,
                    selected_fields,
                );
            }
            current_field = None;
        }
    }

    // Draw last field if any
    if let Some((prev_field_idx, start, end)) = current_field {
        draw_field_highlight(
            painter,
            layout,
            theme,
            start,
            end,
            prev_field_idx,
            selected_fields,
        );
    }
}

/// Draw highlight for a range of bytes belonging to the same field
fn draw_field_highlight(
    painter: &egui::Painter,
    layout: &LayoutInfo,
    theme: &Theme,
    start_byte: usize,
    end_byte: usize,
    field_idx: usize,
    selected_fields: &HashSet<usize>,
) {
    let is_selected = selected_fields.contains(&field_idx);
    let color = get_field_color(field_idx);

    let start_line = start_byte / theme.bytes_per_line;
    let end_line = end_byte / theme.bytes_per_line;

    // Render line by line for multi-line fields
    for line in start_line..=end_line {
        let line_start_byte = if line == start_line {
            start_byte
        } else {
            line * theme.bytes_per_line
        };

        let line_end_byte = if line == end_line {
            end_byte
        } else {
            (line + 1) * theme.bytes_per_line - 1
        };

        let start_col = line_start_byte % theme.bytes_per_line;
        let end_col = line_end_byte % theme.bytes_per_line;
        let num_bytes = end_col - start_col + 1;

        let y = layout.start_pos.y + line as f32 * theme.line_height;

        // Hex section highlight
        let hex_start_x = layout.hex_start_x + start_col as f32 * (layout.char_width * theme.char_spacing);
        let hex_width = num_bytes as f32 * 2.0 * layout.char_width + (num_bytes - 1) as f32 * layout.char_width;

        let half_space = layout.char_width * 0.5;
        let hex_rect = egui::Rect::from_min_max(
            egui::pos2(hex_start_x - half_space, y),
            egui::pos2(hex_start_x + hex_width + half_space, y + theme.line_height),
        );

        // ASCII section highlight
        let ascii_start_x = layout.ascii_start_x + start_col as f32 * layout.char_width;
        let ascii_width = num_bytes as f32 * layout.char_width;

        let ascii_rect = egui::Rect::from_min_max(
            egui::pos2(ascii_start_x - half_space, y),
            egui::pos2(ascii_start_x + ascii_width + half_space, y + theme.line_height),
        );

        // Draw rounded rectangles
        let rounding = 3.0;
        let stroke_width = if is_selected { 2.0 } else { 1.0 };
        let fill_alpha = if is_selected { 40 } else { 20 };

        // Hex column highlight
        painter.rect(
            hex_rect,
            rounding,
            egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), fill_alpha),
            egui::Stroke::new(stroke_width, color),
        );

        // ASCII column highlight
        painter.rect(
            ascii_rect,
            rounding,
            egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), fill_alpha),
            egui::Stroke::new(stroke_width, color),
        );
    }
}
