//! IPC-команды приложения.
//!
//! Все команды работают через [`crate::state::AppState`] и возвращают
//! сериализуемые структуры, которые фронтенд получает как обычные объекты.

use std::path::{Path, PathBuf};

use fontlord_core::brush::{BrushPreset, presets};
use fontlord_core::geometry::BBox;
use fontlord_core::model::{Contour, FontMeta, FontProject, Glyph, Stroke};
use fontlord_core::outline::build_glyph_outline;
use fontlord_core::stroke::erase_strokes;
use fontlord_core::unicode::{self, BlockInfo, CodepointInfo, SearchResult};

use tauri_plugin_dialog::DialogExt;

use crate::state::AppState;

/// Состояние проекта для интерфейса.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectState {
    /// Метаданные шрифта.
    pub meta: FontMeta,
    /// Количество глифов.
    pub glyph_count: usize,
    /// Коды символов глифов.
    pub glyphs: Vec<u32>,
    /// Путь к файлу проекта (если сохранён).
    pub path: Option<String>,
    /// Признак несохранённых изменений.
    pub dirty: bool,
    /// Доступна ли отмена.
    pub can_undo: bool,
    /// Доступен ли повтор.
    pub can_redo: bool,
}

/// Результат построения контура глифа.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlineResult {
    /// Контуры глифа.
    pub contours: Vec<Contour>,
    /// Границы контуров.
    pub bbox: BBox,
}

/// Отчёт об экспорте шрифта.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportReport {
    /// Путь к сохранённому файлу.
    pub path: String,
    /// Размер файла в байтах.
    pub bytes: usize,
    /// Количество глифов в шрифте.
    pub glyph_count: usize,
    /// Единицы на em.
    pub units_per_em: u16,
    /// Предупреждения сборщика.
    pub warnings: Vec<String>,
}

fn project_state(state: &AppState) -> ProjectState {
    state.with(|inner| ProjectState {
        meta: inner.project.meta.clone(),
        glyph_count: inner.project.glyphs.len(),
        glyphs: inner.project.glyph_codepoints(),
        path: inner
            .path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string()),
        dirty: inner.dirty,
        can_undo: inner.can_undo(),
        can_redo: inner.can_redo(),
    })
}

/// Возвращает список блоков Unicode.
#[tauri::command]
pub fn unicode_blocks() -> Vec<BlockInfo> {
    unicode::blocks().to_vec()
}

/// Ищет символы по имени, коду или символу.
#[tauri::command]
pub fn unicode_search(query: String, offset: usize, limit: usize) -> SearchResult {
    unicode::index().search(&query, offset, limit.clamp(1, 2000))
}

/// Возвращает символы диапазона кодов.
#[tauri::command]
pub fn unicode_range(start: u32, end: u32, offset: usize, limit: usize) -> Vec<CodepointInfo> {
    unicode::index().range(start, end, offset, limit.clamp(1, 5000))
}

/// Возвращает пресеты кистей.
#[tauri::command]
pub fn brush_presets() -> Vec<BrushPreset> {
    presets()
}

/// Возвращает текущее состояние проекта.
#[tauri::command]
pub fn project_state_cmd(state: tauri::State<'_, AppState>) -> ProjectState {
    project_state(&state)
}

/// Создаёт новый проект.
#[tauri::command]
pub fn project_new(
    family_name: String,
    style_name: String,
    state: tauri::State<'_, AppState>,
) -> ProjectState {
    let project = FontProject::new(family_name, style_name);
    state.replace_project(project, None);
    project_state(&state)
}

/// Открывает проект из файла.
#[tauri::command]
pub fn project_open(path: String, state: tauri::State<'_, AppState>) -> Result<ProjectState, String> {
    let path = PathBuf::from(path);
    let project = fontlord_core::project::load(&path).map_err(|e| e.to_string())?;
    state.replace_project(project, Some(path));
    Ok(project_state(&state))
}

/// Сохраняет проект (в указанный файл или в текущий).
#[tauri::command]
pub fn project_save(
    path: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<ProjectState, String> {
    let target: PathBuf = match path {
        Some(p) => PathBuf::from(p),
        None => state.with(|inner| inner.path.clone()).ok_or(
            "Проект ещё ни разу не сохранён — укажите имя файла.",
        )?,
    };

    let project = state.project();
    fontlord_core::project::save(&project, &target).map_err(|e| e.to_string())?;
    state.with(|inner| {
        inner.path = Some(target.clone());
        inner.dirty = false;
    });
    Ok(project_state(&state))
}

/// Обновляет метаданные шрифта.
#[tauri::command]
pub fn project_set_meta(
    meta: FontMeta,
    state: tauri::State<'_, AppState>,
) -> Result<ProjectState, String> {
    state.with(|inner| {
        inner.push_history();
        inner.project.meta = meta;
    });
    Ok(project_state(&state))
}

/// Отменяет последнее изменение.
#[tauri::command]
pub fn project_undo(state: tauri::State<'_, AppState>) -> ProjectState {
    state.with(|inner| {
        inner.undo();
    });
    project_state(&state)
}

/// Повторяет отменённое изменение.
#[tauri::command]
pub fn project_redo(state: tauri::State<'_, AppState>) -> ProjectState {
    state.with(|inner| {
        inner.redo();
    });
    project_state(&state)
}

/// Возвращает глиф по коду символа.
#[tauri::command]
pub fn glyph_get(codepoint: u32, state: tauri::State<'_, AppState>) -> Option<Glyph> {
    state.with(|inner| inner.project.glyph(codepoint).cloned())
}

/// Возвращает список кодов символов глифов.
#[tauri::command]
pub fn glyph_list(state: tauri::State<'_, AppState>) -> Vec<u32> {
    state.with(|inner| inner.project.glyph_codepoints())
}

/// Добавляет или обновляет штрих глифа.
#[tauri::command]
pub fn glyph_upsert_stroke(
    codepoint: u32,
    stroke: Stroke,
    state: tauri::State<'_, AppState>,
) -> Result<Glyph, String> {
    let glyph = state.with(|inner| {
        inner.push_history();
        let settings = inner.outline_settings;
        let project = &mut inner.project;
        let meta = project.meta.clone();
        let glyph = project.glyph_mut_or_create(codepoint, glyph_name(codepoint));
        match glyph.strokes.iter_mut().find(|s| s.id == stroke.id) {
            Some(existing) => *existing = stroke,
            None => glyph.strokes.push(stroke),
        }
        glyph.rebuild_outline(&settings);
        let _ = meta;
        glyph.clone()
    });
    Ok(glyph)
}

/// Удаляет штрих глифа.
#[tauri::command]
pub fn glyph_remove_stroke(
    codepoint: u32,
    stroke_id: u32,
    state: tauri::State<'_, AppState>,
) -> Result<Glyph, String> {
    state.with(|inner| {
        inner.push_history();
        let settings = inner.outline_settings;
        let project = &mut inner.project;
        let glyph = project
            .glyph_mut(codepoint)
            .ok_or_else(|| format!("Символ U+{codepoint:04X} не найден"))?;
        glyph.strokes.retain(|s| s.id != stroke_id);
        glyph.rebuild_outline(&settings);
        Ok(glyph.clone())
    })
}

/// Удаляет штрихи, которых касается путь ластика.
#[tauri::command]
pub fn glyph_erase(
    codepoint: u32,
    points: Vec<[f32; 2]>,
    radius: f32,
    state: tauri::State<'_, AppState>,
) -> Result<Glyph, String> {
    let path: Vec<fontlord_core::geometry::Point> = points
        .iter()
        .map(|p| fontlord_core::geometry::Point::new(p[0], p[1]))
        .collect();

    state.with(|inner| {
        inner.push_history();
        let settings = inner.outline_settings;
        let project = &mut inner.project;
        let glyph = project
            .glyph_mut(codepoint)
            .ok_or_else(|| format!("Символ U+{codepoint:04X} не найден"))?;
        erase_strokes(&mut glyph.strokes, &path, radius);
        glyph.rebuild_outline(&settings);
        Ok(glyph.clone())
    })
}

/// Изменяет метрики глифа.
#[tauri::command]
pub fn glyph_set_metrics(
    codepoint: u32,
    advance_width: Option<u16>,
    name: Option<String>,
    notes: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<Glyph, String> {
    state.with(|inner| {
        inner.push_history();
        let settings = inner.outline_settings;
        let project = &mut inner.project;
        let glyph = project.glyph_mut(codepoint).ok_or_else(|| {
            format!("Символ U+{codepoint:04X} не найден")
        })?;
        if let Some(width) = advance_width {
            glyph.advance_width = width;
        }
        if let Some(name) = name {
            glyph.name = name;
        }
        if let Some(notes) = notes {
            glyph.notes = notes;
        }
        glyph.rebuild_outline(&settings);
        Ok(glyph.clone())
    })
}

/// Удаляет глиф.
#[tauri::command]
pub fn glyph_delete(codepoint: u32, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.with(|inner| {
        inner.push_history();
        inner.project.remove_glyph(codepoint);
    });
    Ok(())
}

/// Возвращает контуры глифа (пересчитанные по штрихам).
#[tauri::command]
pub fn glyph_outline(
    codepoint: u32,
    state: tauri::State<'_, AppState>,
) -> Option<OutlineResult> {
    state.with(|inner| {
        let glyph = inner.project.glyph(codepoint)?;
        let contours = build_glyph_outline(glyph, &inner.outline_settings);
        let mut bbox = BBox::EMPTY;
        for contour in &contours {
            for point in &contour.points {
                bbox.extend(fontlord_core::geometry::Point::new(point.x, point.y));
            }
        }
        Some(OutlineResult { contours, bbox })
    })
}

/// Экспортирует проект в файл TrueType.
#[tauri::command]
pub fn font_export_ttf(
    path: String,
    state: tauri::State<'_, AppState>,
) -> Result<ExportReport, String> {
    let project = state.project();
    let bytes = fontlord_ttf::build_truetype(&project).map_err(|e| e.to_string())?;
    let target = Path::new(&path);
    std::fs::write(target, &bytes).map_err(|e| e.to_string())?;
    Ok(ExportReport {
        path,
        bytes: bytes.len(),
        glyph_count: project.glyphs.len(),
        units_per_em: project.meta.units_per_em,
        warnings: Vec::new(),
    })
}

/// Возвращает байты шрифта для предпросмотра в интерфейсе.
#[tauri::command]
pub fn font_preview_ttf(state: tauri::State<'_, AppState>) -> Result<Vec<u8>, String> {
    let project = state.project();
    fontlord_ttf::build_truetype(&project).map_err(|e| e.to_string())
}

/// Открывает проект через системный диалог.
#[tauri::command]
pub async fn project_open_dialog(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<Option<ProjectState>, String> {
    let path = app
        .dialog()
        .file()
        .set_title("Открыть проект Fontlord")
        .add_filter("Проект Fontlord", &["fontlord"])
        .add_filter("Все файлы", &["*"])
        .blocking_pick_file()
        .and_then(|file| file.into_path().ok());

    let Some(path) = path else {
        return Ok(None);
    };

    let project = fontlord_core::project::load(&path).map_err(|e| e.to_string())?;
    state.replace_project(project, Some(path));
    Ok(Some(project_state(&state)))
}

/// Сохраняет проект через системный диалог.
#[tauri::command]
pub async fn project_save_dialog(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<Option<ProjectState>, String> {
    let current = state.with(|inner| inner.path.clone());
    let default_name = current
        .as_ref()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "project.fontlord".to_string());

    let path = app
        .dialog()
        .file()
        .set_title("Сохранить проект")
        .set_file_name(&default_name)
        .add_filter("Проект Fontlord", &["fontlord"])
        .blocking_save_file()
        .and_then(|file| file.into_path().ok());

    let Some(path) = path else {
        return Ok(None);
    };

    let project = state.project();
    fontlord_core::project::save(&project, &path).map_err(|e| e.to_string())?;
    state.with(|inner| {
        inner.path = Some(path.clone());
        inner.dirty = false;
    });
    Ok(Some(project_state(&state)))
}

/// Экспортирует шрифт TrueType через системный диалог.
#[tauri::command]
pub async fn font_export_dialog(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<Option<ExportReport>, String> {
    let family = state.with(|inner| inner.project.meta.family_name.clone());
    let default_name = format!("{}.ttf", family.replace(' ', ""));

    let path = app
        .dialog()
        .file()
        .set_title("Экспортировать шрифт")
        .set_file_name(&default_name)
        .add_filter("Шрифт TrueType", &["ttf"])
        .blocking_save_file()
        .and_then(|file| file.into_path().ok());

    let Some(path) = path else {
        return Ok(None);
    };

    let project = state.project();
    let bytes = fontlord_ttf::build_truetype(&project).map_err(|e| e.to_string())?;
    std::fs::write(&path, &bytes).map_err(|e| e.to_string())?;

    Ok(Some(ExportReport {
        path: path.to_string_lossy().to_string(),
        bytes: bytes.len(),
        glyph_count: project.glyphs.len(),
        units_per_em: project.meta.units_per_em,
        warnings: Vec::new(),
    }))
}

/// Имя глифа по умолчанию для кода символа.
fn glyph_name(codepoint: u32) -> String {
    match unicode::index().info(codepoint) {
        Some(info) => info.name.clone(),
        None => format!("uni{codepoint:04X}"),
    }
}
