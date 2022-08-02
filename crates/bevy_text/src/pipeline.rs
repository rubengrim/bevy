use std::hash::Hash;

use ab_glyph::{PxScale, ScaleFont};
use bevy_asset::{Assets, Handle, HandleId};
use bevy_ecs::system::Resource;
use bevy_math::Vec2;
use bevy_render::texture::Image;
use bevy_sprite::TextureAtlas;
use bevy_utils::HashMap;

use glyph_brush_layout::{FontId, SectionText};

use crate::{
    error::TextError, glyph_brush::GlyphBrush, scale_value, Font, FontAtlasSet, PositionedGlyph,
    TextAlignment, TextSection,
};

#[derive(Resource)]
pub struct TextPipeline<ID> {
    brush: GlyphBrush,
    glyph_map: HashMap<ID, TextLayoutInfo>,
    map_font_id: HashMap<HandleId, FontId>,
}

impl<ID> Default for TextPipeline<ID> {
    fn default() -> Self {
        TextPipeline {
            brush: GlyphBrush::default(),
            glyph_map: Default::default(),
            map_font_id: Default::default(),
        }
    }
}

pub struct TextLayoutInfo {
    pub glyphs: Vec<PositionedGlyph>,
    pub size: Vec2,
}

impl<ID: Hash + Eq> TextPipeline<ID> {
    pub fn get_or_insert_font_id(&mut self, handle: &Handle<Font>, font: &Font) -> FontId {
        let brush = &mut self.brush;
        *self
            .map_font_id
            .entry(handle.id)
            .or_insert_with(|| brush.add_font(handle.clone(), font.font.clone()))
    }

    pub fn get_glyphs(&self, id: &ID) -> Option<&TextLayoutInfo> {
        self.glyph_map.get(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn queue_text(
        &mut self,
        id: ID,
        fonts: &Assets<Font>,
        sections: &[TextSection],
        scale_factor: f64,
        text_alignment: TextAlignment,
        bounds: Vec2,
        font_atlas_set_storage: &mut Assets<FontAtlasSet>,
        texture_atlases: &mut Assets<TextureAtlas>,
        textures: &mut Assets<Image>,
    ) -> Result<(), TextError> {
        let mut scaled_fonts = Vec::new();
        let sections = sections
            .iter()
            .map(|section| {
                let handle = match section.style.font.clone() {
                    crate::FontRef::Handle(handle) => handle,
                    _ => {
                        return Err(TextError::FontNotLoaded);
                    }
                };

                let font = fonts.get(&handle).ok_or(TextError::NoSuchFont)?;
                let font_id = self.get_or_insert_font_id(&handle, font);
                let font_size = scale_value(section.style.font_size, scale_factor);

                scaled_fonts.push(ab_glyph::Font::as_scaled(&font.font, font_size));

                let section = SectionText {
                    font_id,
                    scale: PxScale::from(font_size),
                    text: &section.value,
                };

                Ok(section)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let section_glyphs = self
            .brush
            .compute_glyphs(&sections, bounds, text_alignment)?;

        if section_glyphs.is_empty() {
            self.glyph_map.insert(
                id,
                TextLayoutInfo {
                    glyphs: Vec::new(),
                    size: Vec2::ZERO,
                },
            );
            return Ok(());
        }

        let mut min_x: f32 = std::f32::MAX;
        let mut min_y: f32 = std::f32::MAX;
        let mut max_x: f32 = std::f32::MIN;
        let mut max_y: f32 = std::f32::MIN;

        for sg in &section_glyphs {
            let scaled_font = scaled_fonts[sg.section_index];
            let glyph = &sg.glyph;
            min_x = min_x.min(glyph.position.x);
            min_y = min_y.min(glyph.position.y - scaled_font.ascent());
            max_x = max_x.max(glyph.position.x + scaled_font.h_advance(glyph.id));
            max_y = max_y.max(glyph.position.y - scaled_font.descent());
        }

        let size = Vec2::new(max_x - min_x, max_y - min_y);

        let glyphs = self.brush.process_glyphs(
            section_glyphs,
            &sections,
            font_atlas_set_storage,
            fonts,
            texture_atlases,
            textures,
        )?;

        self.glyph_map.insert(id, TextLayoutInfo { glyphs, size });

        Ok(())
    }
}
