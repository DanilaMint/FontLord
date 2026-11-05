//! Построение лигатур из компонентов.
//!
//! Лигатура собирается из глифов-компонентов: каждый компонент сдвигается по X
//! на ширину предыдущих. Итоговая ширина берётся из самого правого компонента
//! с учётом его боковых полей.

use crate::error::{CoreError, CoreResult};
use crate::model::{FontProject, Glyph, Ligature};

/// Размещение компонента лигатуры.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComponentPlacement {
    /// Код символа компонента.
    pub codepoint: u32,
    /// Смещение по X.
    pub dx: f32,
    /// Смещение по Y.
    pub dy: f32,
}

/// Расчётная геометрия лигатуры.
#[derive(Debug, Clone, PartialEq)]
pub struct LigatureLayout {
    /// Ширина продвижения результата.
    pub advance_width: u16,
    /// Позиции компонентов.
    pub components: Vec<ComponentPlacement>,
}

/// Строитель лигатур.
pub struct LigatureBuilder<'a> {
    project: &'a FontProject,
}

impl<'a> LigatureBuilder<'a> {
    /// Создаёт строитель для проекта.
    pub fn new(project: &'a FontProject) -> Self {
        Self { project }
    }

    /// Рассчитывает раскладку лигатуры из компонентов.
    pub fn layout(&self, components: &[u32]) -> CoreResult<LigatureLayout> {
        if components.len() < 2 {
            return Err(CoreError::invalid(
                "лигатура должна состоять минимум из двух компонентов",
            ));
        }

        let mut placements = Vec::with_capacity(components.len());
        let mut pen = 0.0f32;
        let mut last_advance = 0.0f32;

        for (index, codepoint) in components.iter().enumerate() {
            let glyph = self.project.glyph(*codepoint).ok_or_else(|| {
                CoreError::invalid(format!(
                    "компонент U+{codepoint:04X} отсутствует в проекте"
                ))
            })?;
            if index > 0 {
                placements.push(ComponentPlacement {
                    codepoint: *codepoint,
                    dx: pen,
                    dy: 0.0,
                });
            } else {
                placements.push(ComponentPlacement {
                    codepoint: *codepoint,
                    dx: 0.0,
                    dy: 0.0,
                });
            }
            pen += glyph.advance_width as f32;
            last_advance = glyph.advance_width as f32;
        }

        Ok(LigatureLayout {
            advance_width: last_advance.round().clamp(0.0, u16::MAX as f32) as u16,
            components: placements,
        })
    }

    /// Создаёт глиф-лигатуру в проекте (заготовка операции).
    pub fn build(&self, ligature: &Ligature) -> CoreResult<Glyph> {
        let layout = self.layout(&ligature.components)?;
        let result_codepoint = ligature
            .result
            .ok_or_else(|| CoreError::invalid("у лигатуры не задан символ результата"))?;

        let mut glyph = Glyph::new(result_codepoint, ligature.name.clone(), layout.advance_width);
        glyph.components = layout
            .components
            .iter()
            .map(|c| crate::model::GlyphComponent {
                codepoint: c.codepoint,
                dx: c.dx,
                dy: c.dy,
                scale: 1.0,
            })
            .collect();
        glyph.notes = "Лигатура собрана из компонентов.".to_string();
        Ok(glyph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project() -> FontProject {
        let mut project = FontProject::new("Test", "Regular");
        project.glyphs.insert(0x0041, Glyph::new(0x0041, "A", 600));
        project.glyphs.insert(0x0066, Glyph::new(0x0066, "f", 300));
        project
    }

    #[test]
    fn layout_places_components_sequentially() {
        let project = project();
        let builder = LigatureBuilder::new(&project);
        let layout = builder.layout(&[0x0066, 0x0066]).unwrap();
        assert_eq!(layout.components.len(), 2);
        assert_eq!(layout.components[0].dx, 0.0);
        assert_eq!(layout.components[1].dx, 300.0);
        assert_eq!(layout.advance_width, 300);
    }

    #[test]
    fn layout_requires_two_components() {
        let project = project();
        let builder = LigatureBuilder::new(&project);
        assert!(builder.layout(&[0x0041]).is_err());
    }
}
