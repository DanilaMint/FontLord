//! # Fontlord Core
//!
//! Доменное ядро редактора шрифтов: модель проекта, движок кистей (растр ->
//! векторные контуры), индекс Unicode и формат файла проекта.
//!
//! Крейт не зависит от Tauri и может использоваться как библиотека:
//!
//! ```no_run
//! use fontlord_core::prelude::*;
//!
//! let mut project = FontProject::new("My Font", "Regular");
//! project.glyph_mut_or_create(0x0041, "A").advance_width = 600;
//! let contours = project.glyph_outline(0x0041).unwrap();
//! assert!(contours.iter().all(|c| c.points.len() >= 3));
//! ```

pub mod brush;
pub mod error;
pub mod features;
pub mod geometry;
pub mod model;
pub mod outline;
pub mod project;
pub mod stroke;
pub mod unicode;

pub use error::{CoreError, CoreResult};
pub use model::{
    Anchor, BrushSettings, Contour, ContourPoint, FeatureRule, FontMeta, FontProject, Glyph,
    KernPair, Ligature, MarkSpec, Stroke, StrokePoint,
};
pub use outline::{build_glyph_outline, OutlineSettings};
pub use project::{ProjectDocument, PROJECT_FORMAT_VERSION};

/// Наиболее часто используемые типы ядра.
pub mod prelude {
    pub use crate::brush::{BrushKind, BrushPreset};
    pub use crate::error::{CoreError, CoreResult};
    pub use crate::features::ligatures::LigatureBuilder;
    pub use crate::features::marks::MarkAttacher;
    pub use crate::model::*;
    pub use crate::outline::{build_glyph_outline, OutlineSettings};
    pub use crate::project::ProjectDocument;
    pub use crate::unicode::{BlockInfo, CodepointInfo, UnicodeIndex};
}

/// Версия формата данных ядра (инкрементируется при несовместимых изменениях).
pub const CORE_VERSION: u32 = 1;
