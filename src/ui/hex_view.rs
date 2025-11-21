use crate::schema::Field;
use egui::{Color32, RichText, ScrollArea, TextStyle};
use std::collections::HashSet;

/// Hexadecimal viewer widget
pub struct HexView {
    bytes_per_row: usize,
}

impl Default for HexView {
    fn default() -> Self {
        Self { bytes_per_row: 16 }
    }
}

impl HexView {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the field that contains the given byte offset, if any
    fn get_field_at_offset<'a>(fields: &'a [Field], offset: usize) -> Option<(usize, &'a Field)> {
        fields
            .iter()
            .enumerate()
            .find(|(_, field)| offset >= field.offset && offset < field.offset + field.size())
    }

    /// Generate a distinct color for each field
    fn get_field_color(field_idx: usize) -> Color32 {
        let colors = [
            Color32::from_rgb(100, 150, 255), // Blue
            Color32::from_rgb(255, 150, 100), // Orange
            Color32::from_rgb(150, 255, 100), // Green
            Color32::from_rgb(255, 100, 200), // Pink
            Color32::from_rgb(200, 100, 255), // Purple
            Color32::from_rgb(100, 255, 200), // Cyan
            Color32::from_rgb(255, 255, 100), // Yellow
            Color32::from_rgb(255, 150, 150), // Light red
        ];
        colors[field_idx % colors.len()]
    }

    /// Draw fancy rounded border highlight for a field's bytes
    #[allow(clippy::too_many_arguments)]
    fn draw_field_highlight(
        painter: &egui::Painter,
        hex_rect: &egui::Rect,
        ascii_rect: &egui::Rect,
        start_byte: usize,
        end_byte: usize,
        field_idx: usize,
        selected_fields: &HashSet<usize>,
        char_width: f32,
        line_height: f32,
    ) {
        let is_selected = selected_fields.contains(&field_idx);
        let color = Self::get_field_color(field_idx);

        // Calculate rects for hex column
        // Each byte is "XX" (2 chars) + space (1 char) except the last one
        // Format: "XX XX XX" - spaces between bytes but not after
        let num_bytes = end_byte - start_byte + 1;
        let hex_start_x = hex_rect.left() + (start_byte as f32 * 3.0 * char_width);
        // Width = num_bytes * 2 chars + (num_bytes - 1) spaces = num_bytes * 3 - 1
        let hex_width = (num_bytes as f32 * 2.0 + (num_bytes - 1) as f32) * char_width;

        // Symmetric padding - half space on each side so consecutive fields share the space
        let half_space = char_width * 0.5; // Half of a space character for symmetric borders
        let hex_highlight_rect = egui::Rect::from_min_max(
            egui::pos2(hex_start_x - half_space, hex_rect.top()),
            egui::pos2(hex_start_x + hex_width + half_space, hex_rect.bottom()),
        );

        // Calculate rects for ASCII column (each byte is 1 char)
        let ascii_start_x = ascii_rect.left() + (start_byte as f32 * char_width);
        let ascii_width = num_bytes as f32 * char_width;
        let ascii_highlight_rect = egui::Rect::from_min_max(
            egui::pos2(ascii_start_x - half_space, ascii_rect.top()),
            egui::pos2(ascii_start_x + ascii_width + half_space, ascii_rect.bottom()),
        );

        // Draw rounded rectangles
        let rounding = 3.0;
        let stroke_width = if is_selected { 2.0 } else { 1.0 };
        let fill_alpha = if is_selected { 40 } else { 20 };

        // Hex column highlight
        painter.rect(
            hex_highlight_rect,
            rounding,
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), fill_alpha),
            egui::Stroke::new(stroke_width, color),
        );

        // ASCII column highlight
        painter.rect(
            ascii_highlight_rect,
            rounding,
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), fill_alpha),
            egui::Stroke::new(stroke_width, color),
        );
    }

    /// Render the hex view for the given binary data
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        data: &[u8],
        fields: &[Field],
        selected_fields: &HashSet<usize>,
    ) {
        if data.is_empty() {
            ui.label("No file loaded");
            return;
        }

        ScrollArea::vertical()
            .id_salt("hex_view_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Use monospace font for better alignment
                    ui.style_mut().override_text_style = Some(TextStyle::Monospace);

                    ui.vertical(|ui| {
                        // Calculate character width for monospace font
                        let char_width = ui.fonts(|f| f.glyph_width(&egui::TextStyle::Monospace.resolve(ui.style()), '0'));

                        // Render each row
                        for (row_idx, chunk) in data.chunks(self.bytes_per_row).enumerate() {
                            let row_response = ui.horizontal(|ui| {
                                let offset = row_idx * self.bytes_per_row;

                                // Offset column - selectable label
                                ui.label(
                                    RichText::new(format!("{:08X}", offset))
                                        .color(Color32::from_rgb(100, 100, 100))
                                );

                                ui.label("│");

                                // Hex bytes column - selectable label
                                let hex_string: String = chunk
                                    .iter()
                                    .map(|b| format!("{:02X}", b))
                                    .collect::<Vec<_>>()
                                    .join(" ");
                                let padding = if chunk.len() < self.bytes_per_row {
                                    "   ".repeat(self.bytes_per_row - chunk.len())
                                } else {
                                    String::new()
                                };

                                let hex_response = ui.label(RichText::new(format!("{}{}", hex_string, padding)));

                                ui.label("│");

                                // ASCII column - selectable label
                                let ascii_string: String = chunk
                                    .iter()
                                    .map(|&b| {
                                        if b.is_ascii_graphic() || b == b' ' {
                                            b as char
                                        } else {
                                            '.'
                                        }
                                    })
                                    .collect();

                                let ascii_response = ui.label(
                                    RichText::new(ascii_string)
                                        .color(Color32::from_rgb(150, 150, 150))
                                );

                                // Get painter after all UI rendering
                                let painter = ui.painter().clone();

                                // Draw field highlights using painter
                                let line_height = hex_response.rect.height();

                                // Group consecutive bytes by field for rounded borders
                                let mut current_field: Option<(usize, usize, usize)> = None; // (field_idx, start_byte, end_byte)

                                for (byte_idx, _) in chunk.iter().enumerate() {
                                    let byte_offset = offset + byte_idx;

                                    if let Some((field_idx, _field)) = Self::get_field_at_offset(fields, byte_offset) {
                                        match current_field {
                                            Some((curr_field_idx, start, _)) if curr_field_idx == field_idx => {
                                                // Same field, extend the range
                                                current_field = Some((field_idx, start, byte_idx));
                                            }
                                            _ => {
                                                // Draw previous field if any
                                                if let Some((prev_field_idx, start, end)) = current_field {
                                                    Self::draw_field_highlight(
                                                        &painter,
                                                        &hex_response.rect,
                                                        &ascii_response.rect,
                                                        start,
                                                        end,
                                                        prev_field_idx,
                                                        selected_fields,
                                                        char_width,
                                                        line_height,
                                                    );
                                                }
                                                // Start new field
                                                current_field = Some((field_idx, byte_idx, byte_idx));
                                            }
                                        }
                                    } else {
                                        // No field, draw previous if any
                                        if let Some((prev_field_idx, start, end)) = current_field {
                                            Self::draw_field_highlight(
                                                &painter,
                                                &hex_response.rect,
                                                &ascii_response.rect,
                                                start,
                                                end,
                                                prev_field_idx,
                                                selected_fields,
                                                char_width,
                                                line_height,
                                            );
                                        }
                                        current_field = None;
                                    }
                                }

                                // Draw last field if any
                                if let Some((prev_field_idx, start, end)) = current_field {
                                    Self::draw_field_highlight(
                                        &painter,
                                        &hex_response.rect,
                                        &ascii_response.rect,
                                        start,
                                        end,
                                        prev_field_idx,
                                        selected_fields,
                                        char_width,
                                        line_height,
                                    );
                                }
                            });
                        }
                    });
                });
            });
    }
}
