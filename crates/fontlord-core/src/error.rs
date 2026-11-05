//! Тип ошибок ядра.

use std::path::PathBuf;

/// Ошибка доменного ядра.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// Некорректный ввод пользователя (значения вне диапазона и т.п.).
    #[error("некорректные данные: {0}")]
    Invalid(String),

    /// Символ с таким кодом отсутствует в проекте.
    #[error("символ U+{0:04X} не найден в проекте")]
    GlyphNotFound(u32),

    /// Ошибка чтения/записи файла проекта.
    #[error("ошибка файла проекта: {0}")]
    Io(#[from] std::io::Error),

    /// Файл проекта повреждён или имеет неподдерживаемую версию.
    #[error("файл проекта повреждён: {0}")]
    Malformed(String),

    /// Ошибка сериализации/десериализации JSON.
    #[error("ошибка сериализации: {0}")]
    Json(#[from] serde_json::Error),
}

/// Результат операции ядра.
pub type CoreResult<T> = Result<T, CoreError>;

impl CoreError {
    /// Конструктор ошибки невалидных данных.
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::Invalid(message.into())
    }

    /// Путь к файлу, с которым связана ошибка (если есть).
    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            Self::Io(_) => None,
            _ => None,
        }
    }
}
