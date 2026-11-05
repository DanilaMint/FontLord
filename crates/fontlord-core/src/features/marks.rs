//! Привязка диакритических знаков к базовым глифам.
//!
//! Знак позиционируется относительно якоря базовой буквы: якорь знака
//! совмещается с якорем базы. Если якорь базы отсутствует, используется
//! значение по умолчанию (верхняя точка над базой).

use crate::error::{CoreError, CoreResult};
use crate::model::{Anchor, FontProject, Glyph};

/// Имя якоря по умолчанию (для знаков, прикрепляемых сверху).
pub const DEFAULT_ANCHOR: &str = "top";

/// Позиция прикреплённого знака.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarkPlacement {
    /// Код символа базовой буквы.
    pub base: u32,
    /// Код символа знака.
    pub mark: u32,
    /// Смещение знака по X относительно базы.
    pub dx: f32,
    /// Смещение знака по Y относительно базы.
    pub dy: f32,
}

/// Строитель привязки диакритики.
pub struct MarkAttacher<'a> {
    project: &'a FontProject,
}

impl<'a> MarkAttacher<'a> {
    /// Создаёт строитель для проекта.
    pub fn new(project: &'a FontProject) -> Self {
        Self { project }
    }

    /// Находит якорь глифа по имени.
    pub fn anchor_of(glyph: &Glyph, name: &str) -> Option<Anchor> {
        glyph.anchors.iter().find(|a| a.name == name).cloned()
    }

    /// Рассчитывает смещение знака относительно базовой буквы.
    pub fn place(&self, base: u32, mark: u32, anchor_name: &str) -> CoreResult<MarkPlacement> {
        let base_glyph = self.project.glyph(base).ok_or_else(|| {
            CoreError::invalid(format!("база U+{base:04X} отсутствует в проекте"))
        })?;
        let mark_glyph = self.project.glyph(mark).ok_or_else(|| {
            CoreError::invalid(format!("знак U+{mark:04X} отсутствует в проекте"))
        })?;

        let base_anchor = Self::anchor_of(base_glyph, anchor_name).unwrap_or(Anchor {
            name: anchor_name.to_string(),
            x: base_glyph.advance_width as f32 * 0.5,
            y: self.project.meta.x_height as f32,
        });

        let mark_anchor = Self::anchor_of(mark_glyph, anchor_name).unwrap_or(Anchor {
            name: anchor_name.to_string(),
            x: 0.0,
            y: 0.0,
        });

        Ok(MarkPlacement {
            base,
            mark,
            dx: base_anchor.x - mark_anchor.x,
            dy: base_anchor.y - mark_anchor.y,
        })
    }

    /// Создаёт глиф со знаком (заготовка операции).
    pub fn compose(&self, base: u32, mark: u32, anchor_name: &str) -> CoreResult<Glyph> {
        let placement = self.place(base, mark, anchor_name)?;
        let base_glyph = self.project.glyph(base).ok_or_else(|| {
            CoreError::invalid(format!("база U+{base:04X} отсутствует в проекте"))
        })?;

        let mut glyph = base_glyph.clone();
        glyph.name = format!(
            "{}_{}",
            base_glyph.name,
            self.project
                .glyph(mark)
                .map(|g| g.name.clone())
                .unwrap_or_else(|| format!("u{mark:04x}"))
        );
        glyph.codepoint = None;
        glyph.components.push(crate::model::GlyphComponent {
            codepoint: mark,
            dx: placement.dx,
            dy: placement.dy,
            scale: 1.0,
        });
        Ok(glyph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project() -> FontProject {
        let mut project = FontProject::new("Test", "Regular");
        let mut base = Glyph::new(0x0041, "A", 600);
        base.anchors.push(Anchor {
            name: "top".into(),
            x: 300.0,
            y: 500.0,
        });
        project.glyphs.insert(0x0041, base);

        let mut mark = Glyph::new(0x0301, "acutecomb", 0);
        mark.anchors.push(Anchor {
            name: "top".into(),
            x: 10.0,
            y: 20.0,
        });
        project.glyphs.insert(0x0301, mark);
        project
    }

    #[test]
    fn place_aligns_anchors() {
        let project = project();
        let attacher = MarkAttacher::new(&project);
        let placement = attacher.place(0x0041, 0x0301, "top").unwrap();
        assert_eq!(placement.dx, 290.0);
        assert_eq!(placement.dy, 480.0);
    }

    #[test]
    fn compose_creates_component_glyph() {
        let project = project();
        let attacher = MarkAttacher::new(&project);
        let glyph = attacher.compose(0x0041, 0x0301, "top").unwrap();
        assert_eq!(glyph.components.len(), 1);
        assert_eq!(glyph.components[0].codepoint, 0x0301);
        assert_eq!(glyph.codepoint, None);
    }
}
