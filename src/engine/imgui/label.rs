use super::instance::ImguiState;
use super::widgets;
use super::widgets::Widget;
use super::{Metric, TextAlign};

use engine::MeshData;

struct BitmapFontData {
    hidpi: f32,
    screen_size: (u32, u32),
    texture_size: (u32, u32),
    font_size: (u32, u32),
}

impl BitmapFontData {
    fn texture_space_glyph_size(&self) -> (f32, f32) {
        let icw = (self.font_size.0 as f32) / (self.texture_size.0 as f32);
        let ich = (self.font_size.1 as f32) / (self.texture_size.1 as f32);

        (icw, ich)
    }

    fn n_glyph_per_row(&self) -> u8 {
        (self.texture_size.0 / self.font_size.0) as u8
    }

    fn ndc_glyph_size(&self) -> (f32, f32) {
        (
            ((self.font_size.0 as f32) / self.screen_size.0 as f32) * 2.0 * self.hidpi,
            ((self.font_size.1 as f32) / self.screen_size.1 as f32) * 2.0 * self.hidpi,
        )
    }
}

struct TextData {
    s: String,
    align: TextAlign,
    font_data: BitmapFontData,
}

fn make_text_mesh_data(text_data: TextData) -> MeshData {
    let mut vertices = vec![];
    let mut uvs = vec![];
    let mut indices = vec![];

    let bfont = &text_data.font_data;

    let (icw, ich) = bfont.texture_space_glyph_size();
    let (gw, gh) = bfont.ndc_glyph_size();

    let nrow = bfont.n_glyph_per_row();
    let mut base_y = 0.0;

    let lines: Vec<&str> = text_data.s.split('\n').collect();

    let max_len = lines.iter().fold(0, |acc, line| acc.max(line.len()));

    let mut i = 0;
    for line in lines.into_iter() {
        let x_offset = match text_data.align {
            TextAlign::Left => 0.0,
            TextAlign::Right => (max_len - line.len()) as f32 * gw,
            TextAlign::Center => (max_len - line.len()) as f32 * gw * 0.5,
        };

        for (cidx, c) in line.chars().enumerate() {
            let mut c: u8 = c as u8;

            // only support up to ascii 128
            c = c.min(128);

            let g_row = (c / nrow) as f32;
            let g_col = (c % nrow) as f32;

            let gx = (cidx as f32) * gw + x_offset;

            vertices.append(&mut vec![
                gx + 0.0, // 0
                base_y,
                0.0,
                gx + 0.0, // 1
                base_y - gh,
                0.0,
                gx + gw, // 2
                base_y - gh,
                0.0,
                gx + gw, // 3
                base_y,
                0.0,
            ]);

            uvs.append(&mut vec![
                g_col * icw + 0.0, // 0
                g_row * ich,
                g_col * icw + 0.0, // 1
                g_row * ich + ich,
                g_col * icw + icw, // 2
                g_row * ich + ich,
                g_col * icw + icw, // 3
                g_row * ich,
            ]);

            indices.append(&mut vec![
                i * 4,
                i * 4 + 1,
                i * 4 + 2,
                i * 4 + 0,
                i * 4 + 2,
                i * 4 + 3, // Top face
            ]);

            i += 1;
        }

        base_y -= gh * 2.0;
    }

    MeshData {
        vertices: vertices,
        uvs: Some(uvs),
        normals: None,
        indices: indices,
        tangents: None,
        bitangents: None,
    }
}

#[derive(Debug, PartialEq)]
pub struct Label {
    id: u32,
    pub pos: Metric,
    pub state: ImguiState,
    s: String,
}

impl Label {
    pub fn new(id: u32, pos: Metric, state: ImguiState, s: String) -> Widget {
        Widget::Label(Self {
            id: id,
            pos: pos,
            state,
            s: s,
        })
    }

    pub fn bind(&self, ssize: (u32, u32), hidpi: f32) -> MeshData {
        // Mesh Data
        let meshdata = {
            make_text_mesh_data(TextData {
                s: self.s.clone(),
                align: self.state.text_align,
                font_data: BitmapFontData {
                    hidpi,
                    screen_size: ssize,
                    texture_size: (128, 64),
                    font_size: (8, 8),
                },
            })
        };

        return meshdata;
    }
}

impl widgets::WidgetBinder for Label {
    fn id(&self) -> u32 {
        self.id
    }

    fn is_same(&self, other: &Widget) -> bool {
        match other {
            &Widget::Label(ref lbl) => lbl == self,
            _ => false,
        }
    }
}
