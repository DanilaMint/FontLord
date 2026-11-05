//! Состояние приложения: текущий проект, история изменений и параметры.
//!
//! Все изменения проекта проходят через [`AppState`], поэтому история
//! (undo/redo) остаётся согласованной с данными на диске.

use std::path::PathBuf;
use std::sync::Mutex;

use fontlord_core::model::FontProject;
use fontlord_core::outline::OutlineSettings;

/// Максимальное количество шагов истории.
const HISTORY_LIMIT: usize = 64;

/// Внутреннее состояние (защищено мьютексом).
#[derive(Debug)]
pub struct Inner {
    /// Текущий проект.
    pub project: FontProject,
    /// Параметры аппроксимации контуров.
    pub outline_settings: OutlineSettings,
    /// Снимки состояния для отмены изменений.
    pub history: Vec<FontProject>,
    /// Снимки состояния для повтора изменений.
    pub redo_stack: Vec<FontProject>,
    /// Путь к файлу проекта.
    pub path: Option<PathBuf>,
    /// Признак несохранённых изменений.
    pub dirty: bool,
}

/// Состояние приложения.
#[derive(Debug)]
pub struct AppState {
    inner: Mutex<Inner>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    /// Создаёт пустое состояние с новым проектом.
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner {
                project: FontProject::default(),
                outline_settings: OutlineSettings::default(),
                history: Vec::new(),
                redo_stack: Vec::new(),
                path: None,
                dirty: false,
            }),
        }
    }

    /// Выполняет операцию над состоянием под блокировкой.
    pub fn with<R>(&self, f: impl FnOnce(&mut Inner) -> R) -> R {
        let mut guard = self.inner.lock().expect("состояние приложения заблокировано");
        f(&mut guard)
    }

    /// Текущий проект (копия).
    pub fn project(&self) -> FontProject {
        self.with(|inner| inner.project.clone())
    }

    /// Заменяет проект целиком (используется при открытии файла).
    pub fn replace_project(&self, project: FontProject, path: Option<PathBuf>) {
        self.with(|inner| {
            inner.history.clear();
            inner.redo_stack.clear();
            inner.project = project;
            inner.path = path;
            inner.dirty = false;
        });
    }
}

impl Inner {
    /// Сохраняет снимок текущего состояния в историю.
    pub fn push_history(&mut self) {
        self.history.push(self.project.clone());
        if self.history.len() > HISTORY_LIMIT {
            self.history.remove(0);
        }
        self.redo_stack.clear();
        self.dirty = true;
    }

    /// Отменяет последнее изменение.
    pub fn undo(&mut self) -> bool {
        match self.history.pop() {
            Some(previous) => {
                self.redo_stack.push(std::mem::replace(&mut self.project, previous));
                true
            }
            None => false,
        }
    }

    /// Повторяет отменённое изменение.
    pub fn redo(&mut self) -> bool {
        match self.redo_stack.pop() {
            Some(next) => {
                self.history.push(std::mem::replace(&mut self.project, next));
                true
            }
            None => false,
        }
    }

    /// Можно ли отменить изменение.
    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    /// Можно ли повторить изменение.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}
