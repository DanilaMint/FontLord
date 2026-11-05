//! Кисти: типы, параметры и пресеты.
//!
//! Пользователь рисует растр, но каждая кисть имеет собственную математическую
//! модель. Движок [`crate::stroke`] превращает штрих в векторный контур согласно
//! модели кисти.

use serde::{Deserialize, Serialize};

/// Тип кисти.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BrushKind {
    /// Перо: ширина следует за нажимом, концы сужаются.
    #[default]
    Pen,
    /// Маркер: почти постоянная ширина, скруглённые концы.
    Marker,
    /// Каллиграфия: широкое перо под углом, края строго параллельны.
    BroadNib,
    /// Мягкая кисть: плавный переход ширины, «тушь».
    Soft,
    /// Ластик: удаляет целые штрихи, которых касается.
    Eraser,
}

impl BrushKind {
    /// Все доступные типы кистей.
    pub const ALL: [BrushKind; 5] = [
        BrushKind::Pen,
        BrushKind::Marker,
        BrushKind::BroadNib,
        BrushKind::Soft,
        BrushKind::Eraser,
    ];

    /// Идентификатор типа (используется во фронтенде как ключ).
    pub const fn id(self) -> &'static str {
        match self {
            BrushKind::Pen => "pen",
            BrushKind::Marker => "marker",
            BrushKind::BroadNib => "broadNib",
            BrushKind::Soft => "soft",
            BrushKind::Eraser => "eraser",
        }
    }

    /// Человекочитаемое название кисти.
    pub const fn label(self) -> &'static str {
        match self {
            BrushKind::Pen => "Перо",
            BrushKind::Marker => "Маркер",
            BrushKind::BroadNib => "Каллиграфия",
            BrushKind::Soft => "Мягкая кисть",
            BrushKind::Eraser => "Ластик",
        }
    }

    /// Краткое описание поведения кисти.
    pub const fn description(self) -> &'static str {
        match self {
            BrushKind::Pen => "Ширина линии следует за нажимом, концы заостряются.",
            BrushKind::Marker => "Постоянная ширина линии, скруглённые концы.",
            BrushKind::BroadNib => "Перо фиксированной ширины под заданным углом.",
            BrushKind::Soft => "Плавный переход толщины, имитация туши или аэрографа.",
            BrushKind::Eraser => "Удаляет штрихи, которых касается.",
        }
    }

    /// Признак того, что кисть вычитает форму (ластик).
    pub const fn is_eraser(self) -> bool {
        matches!(self, BrushKind::Eraser)
    }
}

/// Параметры кисти.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrushSettings {
    /// Номинальная ширина мазка в шрифтовых единицах.
    pub width: f32,
    /// Угол наклона пера в радианах (для каллиграфии).
    pub angle: f32,
    /// Минимальная доля ширины при нулевом нажиме.
    pub min_width_ratio: f32,
    /// Экспонента отклика на нажим (больше — контрастнее).
    pub pressure_gamma: f32,
    /// Доля длины штриха, на которую идёт сужение в начале (`0..1`).
    pub taper_in: f32,
    /// Доля длины штриха, на которую идёт сужение в конце (`0..1`).
    pub taper_out: f32,
    /// Жёсткость края (`0` — мягкий, `1` — резкий).
    pub hardness: f32,
}

impl Default for BrushSettings {
    fn default() -> Self {
        Self {
            width: 60.0,
            angle: std::f32::consts::FRAC_PI_4,
            min_width_ratio: 0.35,
            pressure_gamma: 1.0,
            taper_in: 0.0,
            taper_out: 0.0,
            hardness: 0.8,
        }
    }
}

impl BrushSettings {
    /// Параметры по умолчанию для указанной кисти.
    pub fn for_kind(kind: BrushKind) -> Self {
        let mut settings = Self::default();
        match kind {
            BrushKind::Pen => {
                settings.width = 52.0;
                settings.min_width_ratio = 0.25;
                settings.taper_in = 0.12;
                settings.taper_out = 0.12;
            }
            BrushKind::Marker => {
                settings.width = 70.0;
                settings.min_width_ratio = 0.9;
                settings.taper_in = 0.0;
                settings.taper_out = 0.0;
                settings.hardness = 1.0;
            }
            BrushKind::BroadNib => {
                settings.width = 90.0;
                settings.min_width_ratio = 1.0;
                settings.hardness = 1.0;
            }
            BrushKind::Soft => {
                settings.width = 110.0;
                settings.min_width_ratio = 0.45;
                settings.pressure_gamma = 0.7;
            }
            BrushKind::Eraser => {
                settings.width = 90.0;
                settings.min_width_ratio = 1.0;
            }
        }
        settings
    }
}

/// Пресет кисти — готовая комбинация типа и параметров.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrushPreset {
    /// Идентификатор пресета.
    pub id: String,
    /// Тип кисти.
    pub kind: BrushKind,
    /// Параметры кисти.
    pub settings: BrushSettings,
    /// Название пресета.
    pub name: String,
    /// Описание пресета.
    pub description: String,
}

/// Возвращает список пресетов кистей.
pub fn presets() -> Vec<BrushPreset> {
    vec![
        BrushPreset {
            id: "pen-fine".into(),
            kind: BrushKind::Pen,
            settings: BrushSettings {
                width: 36.0,
                min_width_ratio: 0.2,
                taper_in: 0.15,
                taper_out: 0.15,
                ..BrushSettings::default()
            },
            name: "Тонкое перо".into(),
            description: "Контрастная линия с острыми концами.".into(),
        },
        BrushPreset {
            id: "pen-bold".into(),
            kind: BrushKind::Pen,
            settings: BrushSettings {
                width: 96.0,
                min_width_ratio: 0.45,
                taper_in: 0.08,
                taper_out: 0.08,
                ..BrushSettings::default()
            },
            name: "Жирное перо".into(),
            description: "Плотный мазок с умеренным сужением.".into(),
        },
        BrushPreset {
            id: "marker".into(),
            kind: BrushKind::Marker,
            settings: BrushSettings::for_kind(BrushKind::Marker),
            name: "Маркер".into(),
            description: "Ровная линия постоянной толщины.".into(),
        },
        BrushPreset {
            id: "calligraphy-45".into(),
            kind: BrushKind::BroadNib,
            settings: BrushSettings {
                width: 110.0,
                angle: std::f32::consts::FRAC_PI_4,
                ..BrushSettings::for_kind(BrushKind::BroadNib)
            },
            name: "Каллиграфия 45°".into(),
            description: "Перо под углом 45° — классические штрихи.".into(),
        },
        BrushPreset {
            id: "calligraphy-30".into(),
            kind: BrushKind::BroadNib,
            settings: BrushSettings {
                width: 130.0,
                angle: std::f32::consts::FRAC_PI_6,
                ..BrushSettings::for_kind(BrushKind::BroadNib)
            },
            name: "Каллиграфия 30°".into(),
            description: "Пологий наклон пера для «рукописных» форм.".into(),
        },
        BrushPreset {
            id: "soft-ink".into(),
            kind: BrushKind::Soft,
            settings: BrushSettings::for_kind(BrushKind::Soft),
            name: "Тушь".into(),
            description: "Мягкий переход толщины, имитация перьевой ручки.".into(),
        },
        BrushPreset {
            id: "eraser".into(),
            kind: BrushKind::Eraser,
            settings: BrushSettings::for_kind(BrushKind::Eraser),
            name: "Ластик".into(),
            description: "Удаляет затронутые штрихи целиком.".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_are_unique_and_valid() {
        let presets = presets();
        let mut ids: Vec<&str> = presets.iter().map(|p| p.id.as_str()).collect();
        ids.sort_unstable();
        let count = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), count);
        for preset in presets {
            assert!(preset.settings.width > 0.0);
            assert!(preset.settings.min_width_ratio >= 0.0);
            assert!(preset.settings.min_width_ratio <= 1.0);
        }
    }
}
