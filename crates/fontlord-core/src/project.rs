//! Формат файла проекта: сохранение и загрузка.
//!
//! Файл проекта — JSON-документ с заголовком версии. Запись атомарная:
//! сначала во временный файл, затем переименование.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{CoreError, CoreResult};
use crate::model::FontProject;

/// Идентификатор формата файла проекта.
pub const PROJECT_FORMAT: &str = "fontlord-project";

/// Текущая версия формата файла проекта.
pub const PROJECT_FORMAT_VERSION: u32 = 1;

/// Документ проекта (содержимое файла `.fontlord`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDocument {
    /// Идентификатор формата.
    pub format: String,
    /// Версия формата.
    pub version: u32,
    /// Имя программы-генератора.
    pub generator: String,
    /// Время сохранения (секунды Unix).
    pub saved_at: u64,
    /// Данные проекта.
    pub project: FontProject,
}

impl ProjectDocument {
    /// Создаёт документ из проекта.
    pub fn new(project: FontProject) -> Self {
        Self {
            format: PROJECT_FORMAT.to_string(),
            version: PROJECT_FORMAT_VERSION,
            generator: format!("Fontlord {}", env!("CARGO_PKG_VERSION")),
            saved_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or_default(),
            project,
        }
    }

    /// Проверяет совместимость документа.
    pub fn validate(&self) -> CoreResult<()> {
        if self.format != PROJECT_FORMAT {
            return Err(CoreError::Malformed(format!(
                "ожидался формат «{PROJECT_FORMAT}», получен «{}»",
                self.format
            )));
        }
        if self.version > PROJECT_FORMAT_VERSION {
            return Err(CoreError::Malformed(format!(
                "файл создан более новой версией приложения (формат {}, поддерживается {})",
                self.version, PROJECT_FORMAT_VERSION
            )));
        }
        Ok(())
    }
}

/// Текущее время в секундах Unix.
pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

/// Сохраняет проект в файл (атомарно).
pub fn save(project: &FontProject, path: &Path) -> CoreResult<()> {
    let document = ProjectDocument::new(project.clone());
    let json = serde_json::to_vec_pretty(&document)?;

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let temp: PathBuf = path.with_extension("tmp-save");
    {
        let mut file = fs::File::create(&temp)?;
        file.write_all(&json)?;
        file.flush()?;
        file.sync_all()?;
    }
    fs::rename(&temp, path)?;
    Ok(())
}

/// Загружает проект из файла.
pub fn load(path: &Path) -> CoreResult<FontProject> {
    let bytes = fs::read(path)?;
    let document: ProjectDocument = serde_json::from_slice(&bytes)?;
    document.validate()?;
    Ok(document.project)
}

/// Читает проект из строки JSON.
pub fn from_json(json: &str) -> CoreResult<FontProject> {
    let document: ProjectDocument = serde_json::from_str(json)?;
    document.validate()?;
    Ok(document.project)
}

/// Сериализует проект в строку JSON.
pub fn to_json(project: &FontProject) -> CoreResult<String> {
    let document = ProjectDocument::new(project.clone());
    Ok(serde_json::to_string_pretty(&document)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Glyph;

    #[test]
    fn roundtrip_through_json() {
        let mut project = FontProject::new("Test", "Regular");
        project.glyphs.insert(
            0x0041,
            Glyph::new(0x0041, "A", 600),
        );
        let json = to_json(&project).unwrap();
        let restored = from_json(&json).unwrap();
        assert_eq!(restored.meta.family_name, "Test");
        assert_eq!(restored.glyphs.len(), 1);
        assert_eq!(restored.glyphs[&0x0041].advance_width, 600);
    }

    #[test]
    fn rejects_foreign_format() {
        let json = r#"{"format":"other","version":1,"generator":"x","savedAt":0,"project":{"meta":{},"glyphs":{}}}"#;
        assert!(from_json(json).is_err());
    }
}
