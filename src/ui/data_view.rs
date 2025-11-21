use crate::schema::Field;
use egui::{Color32, RichText, ScrollArea};
use egui_extras::{Column, TableBuilder};
use std::collections::HashSet;

/// Action to perform on a field
#[derive(Debug, Clone, Copy)]
pub enum FieldAction {
    Select(usize),
    Edit(usize),
    Delete(usize),
}

/// Data view widget showing interpreted fields
pub struct DataView;

impl DataView {
    pub fn new() -> Self {
        Self
    }

    /// Render the data view for the given fields and binary data
    /// Returns an optional action to perform on a field
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        fields: &[Field],
        data: &[u8],
        selected_fields: &HashSet<usize>,
    ) -> Option<FieldAction> {
        let mut action = None;
        if fields.is_empty() {
            ui.label("No fields defined. Add fields to interpret the binary data.");
            return None;
        }

        ScrollArea::vertical()
            .id_salt("data_view_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                TableBuilder::new(ui)
                    .striped(true)
                    .column(Column::exact(80.0)) // Offset
                    .column(Column::exact(150.0)) // Name
                    .column(Column::exact(80.0)) // Type
                    .column(Column::exact(120.0)) // Value
                    .column(Column::remainder().at_least(100.0)) // Comment
                    .column(Column::exact(120.0)) // Actions
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("Offset");
                        });
                        header.col(|ui| {
                            ui.heading("Name");
                        });
                        header.col(|ui| {
                            ui.heading("Type");
                        });
                        header.col(|ui| {
                            ui.heading("Value");
                        });
                        header.col(|ui| {
                            ui.heading("Comment");
                        });
                        header.col(|ui| {
                            ui.heading("Actions");
                        });
                    })
                    .body(|mut body| {
                        for (idx, field) in fields.iter().enumerate() {
                            let is_selected = selected_fields.contains(&idx);

                            body.row(18.0, |mut row| {
                                // Offset - clickable to select row
                                row.col(|ui| {
                                    let mut text = RichText::new(format!("0x{:08X}", field.offset))
                                        .color(Color32::from_rgb(100, 100, 100));
                                    if is_selected {
                                        text = text.strong();
                                    }
                                    if ui.selectable_label(is_selected, text).clicked() {
                                        action = Some(FieldAction::Select(idx));
                                    }
                                });

                                // Name
                                row.col(|ui| {
                                    let mut text = RichText::new(&field.name);
                                    if is_selected {
                                        text = text.strong();
                                    }
                                    ui.label(text);
                                });

                                // Type
                                row.col(|ui| {
                                    let mut text = RichText::new(field.data_type.name())
                                        .color(Color32::from_rgb(80, 150, 200));
                                    if is_selected {
                                        text = text.strong();
                                    }
                                    ui.label(text);
                                });

                                // Value
                                row.col(|ui| {
                                    let mut text = if let Some(value) = field.read_value(data) {
                                        RichText::new(value)
                                    } else {
                                        RichText::new("(out of bounds)")
                                            .color(Color32::from_rgb(200, 80, 80))
                                    };
                                    if is_selected {
                                        text = text.strong();
                                    }
                                    ui.label(text);
                                });

                                // Comment
                                row.col(|ui| {
                                    let text_str = if !field.comment.is_empty() {
                                        &field.comment
                                    } else {
                                        ""
                                    };
                                    let mut text = RichText::new(text_str)
                                        .color(Color32::from_rgb(120, 120, 120))
                                        .italics();
                                    if is_selected {
                                        text = text.strong();
                                    }
                                    ui.label(text);
                                });

                                // Actions
                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        if ui.button("Edit").clicked() {
                                            action = Some(FieldAction::Edit(idx));
                                        }
                                        if ui.button("Delete").clicked() {
                                            action = Some(FieldAction::Delete(idx));
                                        }
                                    });
                                });
                            });
                        }
                    });
            });

        action
    }
}

impl Default for DataView {
    fn default() -> Self {
        Self::new()
    }
}
