//! Точка входа приложения Fontlord.

pub mod commands;
pub mod license;
pub mod state;

use state::AppState;

/// Запускает графическое приложение.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::unicode_blocks,
            commands::unicode_search,
            commands::unicode_range,
            commands::brush_presets,
            commands::project_state_cmd,
            commands::project_new,
            commands::project_open,
            commands::project_save,
            commands::project_set_meta,
            commands::project_undo,
            commands::project_redo,
            commands::glyph_get,
            commands::glyph_list,
            commands::glyph_upsert_stroke,
            commands::glyph_remove_stroke,
            commands::glyph_erase,
            commands::glyph_set_metrics,
            commands::glyph_delete,
            commands::glyph_outline,
            commands::font_export,
            commands::font_preview_ttf,
            commands::project_open_dialog,
            commands::project_save_dialog,
            commands::font_export_dialog,
            commands::set_window_title,
            commands::app_exit,
            commands::app_license,
        ])
        .run(tauri::generate_context!())
        .expect("не удалось запустить приложение Fontlord");
}
