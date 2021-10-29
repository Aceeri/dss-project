use crate::{
    menu::prelude::*,
    renderer::{self, Renderer, TextId},
};

#[derive(Debug, Clone)]
pub struct Text {
    text_id: Option<TextId>,

    position: Position,
    text: String,
    font_size: f32,
    color: [f32; 4],
    update: bool,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text_id: None,

            position: Position::new(),
            text: "".to_owned(),
            font_size: 24.0,
            color: [1.0, 1.0, 1.0, 1.0],
            update: true,
        }
    }
}

impl Text {
    pub fn new(text: String) -> Self {
        Self {
            text,
            ..Default::default()
        }
    }

    pub fn set_update(&mut self) {
        self.update = true;
    }

    pub fn set_text(&mut self, text: String) {
        self.set_update();
        self.text = text;
    }

    pub fn set_font_size(&mut self, font_size: f32) {
        self.set_update();
        self.font_size = font_size;
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.set_update();
        self.color = color;
    }
}

impl PositionHierarchy for Text {
    fn position(&self) -> &Position {
        &self.position
    }
    fn position_mut(&mut self) -> &mut Position {
        self.set_update();
        &mut self.position
    }
}

impl Draw for Text {
    fn set_render_details(&mut self, renderer: &mut Renderer) {
        if self.update {
            match &self.text_id {
                Some(text_id) => {
                    let new_position = self.absolute_position();
                    renderer.text_pass.update_text(
                        text_id,
                        renderer::Text {
                            text: self.text.clone(),
                            font_size: self.font_size,
                            position: [new_position.x, new_position.y],
                            color: self.color,
                        },
                    )
                }
                None => {
                    let new_position = self.absolute_position();
                    let text_id = renderer.text_pass.add_text(renderer::Text {
                        text: self.text.clone(),
                        font_size: self.font_size,
                        position: [new_position.x, new_position.y],
                        color: self.color,
                    });

                    self.text_id = Some(text_id);
                }
            }

            self.update = false;
        }
    }
}
