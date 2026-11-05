//! Модель данных проекта шрифта.
//!
//! Модель разделена на две части:
//!
//! * **Источник истины** — штрихи ([`Stroke`]): то, что рисует пользователь.
//! * **Производные данные** — контуры ([`Contour`]): результат работы движка кистей.
//!
//! Производные контуры пересчитываются всегда, поэтому проект можно безопасно
//! редактировать в разных версиях приложения.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::brush::BrushKind;
pub use crate::brush::BrushSettings;
use crate::outline::{OutlineSettings, build_glyph_outline};

/// Точка штриха с параметром нажима.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrokePoint {
    /// Положение в шрифтовых единицах.
    pub x: f32,
    /// Положение по вертикали (ось Y направлена вверх).
    pub y: f32,
    /// Нажим в диапазоне `0.0..=1.0`.
    pub pressure: f32,
}

impl StrokePoint {
    /// Создаёт точку штриха.
    pub const fn new(x: f32, y: f32, pressure: f32) -> Self {
        Self { x, y, pressure }
    }
}

/// Штрих — непрерывная линия, нарисованная одной кистью.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stroke {
    /// Уникальный идентификатор штриха внутри глифа.
    pub id: u32,
    /// Тип кисти, которой нарисован штрих.
    pub kind: BrushKind,
    /// Параметры кисти на момент рисования (фиксируются в штрихе).
    pub settings: BrushSettings,
    /// Опорные точки штриха.
    pub points: Vec<StrokePoint>,
    /// Признак замкнутого контура (кольцо, «O», «0»).
    #[serde(default)]
    pub closed: bool,
}

impl Stroke {
    /// Создаёт пустой штрих с настройками по умолчанию.
    pub fn new(id: u32, kind: BrushKind, settings: BrushSettings) -> Self {
        Self {
            id,
            kind,
            settings,
            points: Vec::new(),
            closed: false,
        }
    }
}

/// Точка контура. `on_curve == false` означает управляющую точку квадратичной кривой.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContourPoint {
    /// Координата X.
    pub x: f32,
    /// Координата Y.
    pub y: f32,
    /// Признак точки, лежащей на контуре.
    pub on_curve: bool,
}

impl ContourPoint {
    /// Создаёт точку на контуре.
    pub const fn on_curve(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            on_curve: true,
        }
    }

    /// Создаёт управляющую точку.
    pub const fn off_curve(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            on_curve: false,
        }
    }
}

/// Замкнутый контур глифа.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contour {
    /// Точки контура; первая точка всегда лежит на кривой.
    pub points: Vec<ContourPoint>,
}

impl Contour {
    /// Создаёт контур из точек.
    pub fn new(points: Vec<ContourPoint>) -> Self {
        Self { points }
    }

    /// Ориентированная площадь контура (для правила ненулевой намотки).
    pub fn signed_area(&self) -> f32 {
        let n = self.points.len();
        if n < 3 {
            return 0.0;
        }
        let mut sum = 0.0f64;
        for i in 0..n {
            let a = self.points[i];
            let b = self.points[(i + 1) % n];
            sum += (a.x as f64) * (b.y as f64) - (b.x as f64) * (a.y as f64);
        }
        (sum * 0.5) as f32
    }
}

/// Якорь глифа (точка привязки для диакритики и лигатур).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Anchor {
    /// Имя якоря: `top`, `bottom`, `ogonek`, `topleft` и т.п.
    pub name: String,
    /// Положение по X.
    pub x: f32,
    /// Положение по Y.
    pub y: f32,
}

/// Глиф проекта.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Glyph {
    /// Имя глифа (PostScript-подобное).
    pub name: String,
    /// Код символа Unicode (`None` для глифов без символа).
    #[serde(default)]
    pub codepoint: Option<u32>,
    /// Ширина продвижения в шрифтовых единицах.
    pub advance_width: u16,
    /// Штрихи — источник истины.
    #[serde(default)]
    pub strokes: Vec<Stroke>,
    /// Контуры — производные значения (пересчитываются движком кистей).
    #[serde(default)]
    pub contours: Vec<Contour>,
    /// Якоря глифа.
    #[serde(default)]
    pub anchors: Vec<Anchor>,
    /// Ссылки на другие глифы (компонентные сборки, заготовки под лигатуры).
    #[serde(default)]
    pub components: Vec<GlyphComponent>,
    /// Пользовательские заметки.
    #[serde(default)]
    pub notes: String,
}

/// Ссылка на другой глиф (основа для лигатур и диакритики).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlyphComponent {
    /// Код символа компонента.
    pub codepoint: u32,
    /// Смещение компонента по X.
    #[serde(default)]
    pub dx: f32,
    /// Смещение компонента по Y.
    #[serde(default)]
    pub dy: f32,
    /// Масштаб компонента.
    #[serde(default)]
    pub scale: f32,
}

impl Glyph {
    /// Создаёт глиф по умолчанию.
    pub fn new(codepoint: u32, name: impl Into<String>, advance_width: u16) -> Self {
        Self {
            name: name.into(),
            codepoint: Some(codepoint),
            advance_width,
            strokes: Vec::new(),
            contours: Vec::new(),
            anchors: Vec::new(),
            components: Vec::new(),
            notes: String::new(),
        }
    }

    /// Пересчитывает контуры глифа по его штрихам.
    pub fn rebuild_outline(&mut self, settings: &OutlineSettings) {
        self.contours = build_glyph_outline(self, settings);
    }
}

/// Пара кернинга.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KernPair {
    /// Первый символ.
    pub left: u32,
    /// Второй символ.
    pub right: u32,
    /// Значение кернинга в шрифтовых единицах.
    pub value: i16,
}

/// Описание лигатуры (заготовка под будущую функциональность).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ligature {
    /// Идентификатор записи.
    pub id: u32,
    /// Имя лигатуры.
    pub name: String,
    /// Коды символов-компонентов.
    pub components: Vec<u32>,
    /// Код символа результата (если задан).
    #[serde(default)]
    pub result: Option<u32>,
    /// Признак активности правила.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

const fn default_true() -> bool {
    true
}

/// Описание диакритического знака (заготовка под будущую функциональность).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkSpec {
    /// Идентификатор записи.
    pub id: u32,
    /// Имя знака.
    pub name: String,
    /// Код символа базовой буквы.
    #[serde(default)]
    pub base: Option<u32>,
    /// Код символа самого знака.
    pub mark: u32,
    /// Якоря знака.
    #[serde(default)]
    pub anchors: Vec<Anchor>,
}

/// Правило OpenType-фичи (заготовка под будущую функциональность).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureRule {
    /// Тег фичи (`liga`, `kern`, `ccmp` и т.п.).
    pub tag: String,
    /// Описание правила.
    pub name: String,
    /// Признак активности.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Метаданные шрифта.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontMeta {
    /// Имя семейства.
    pub family_name: String,
    /// Имя начертания.
    pub style_name: String,
    /// Версия шрифта.
    pub version: String,
    /// Единицы на em.
    pub units_per_em: u16,
    /// Высота верхнего выносного элемента.
    pub ascender: i16,
    /// Глубина нижнего выносного элемента.
    pub descender: i16,
    /// Межстрочный интервал.
    pub line_gap: i16,
    /// Высота строчных.
    pub x_height: u16,
    /// Высота заглавных.
    pub cap_height: u16,
    /// Автор.
    #[serde(default)]
    pub designer: String,
    /// Copyright.
    #[serde(default)]
    pub copyright: String,
    /// Лицензия.
    #[serde(default)]
    pub license: String,
}

impl Default for FontMeta {
    fn default() -> Self {
        Self {
            family_name: "Untitled Font".into(),
            style_name: "Regular".into(),
            version: "1.000".into(),
            units_per_em: 1000,
            ascender: 800,
            descender: -200,
            line_gap: 0,
            x_height: 500,
            cap_height: 700,
            designer: String::new(),
            copyright: String::new(),
            license: String::new(),
        }
    }
}

/// Проект шрифта.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontProject {
    /// Метаданные.
    pub meta: FontMeta,
    /// Глифы по кодам символов.
    pub glyphs: BTreeMap<u32, Glyph>,
    /// Кернинг.
    #[serde(default)]
    pub kerning: Vec<KernPair>,
    /// Лигатуры (заготовка).
    #[serde(default)]
    pub ligatures: Vec<Ligature>,
    /// Диакритика (заготовка).
    #[serde(default)]
    pub marks: Vec<MarkSpec>,
    /// OpenType-фичи (заготовка).
    #[serde(default)]
    pub features: Vec<FeatureRule>,
}

impl Default for FontProject {
    fn default() -> Self {
        Self::new("Untitled Font", "Regular")
    }
}

impl FontProject {
    /// Создаёт пустой проект с базовыми метриками.
    pub fn new(family_name: impl Into<String>, style_name: impl Into<String>) -> Self {
        Self {
            meta: FontMeta {
                family_name: family_name.into(),
                style_name: style_name.into(),
                ..FontMeta::default()
            },
            glyphs: BTreeMap::new(),
            kerning: Vec::new(),
            ligatures: Vec::new(),
            marks: Vec::new(),
            features: vec![FeatureRule {
                tag: "liga".into(),
                name: "Стандартные лигатуры".into(),
                enabled: true,
            }],
        }
    }

    /// Возвращает глиф по коду символа.
    pub fn glyph(&self, codepoint: u32) -> Option<&Glyph> {
        self.glyphs.get(&codepoint)
    }

    /// Возвращает изменяемый глиф по коду символа.
    pub fn glyph_mut(&mut self, codepoint: u32) -> Option<&mut Glyph> {
        self.glyphs.get_mut(&codepoint)
    }

    /// Создаёт глиф, если его ещё нет, и возвращает изменяемую ссылку.
    pub fn glyph_mut_or_create(&mut self, codepoint: u32, name: impl Into<String>) -> &mut Glyph {
        self.glyphs
            .entry(codepoint)
            .or_insert_with(|| Glyph::new(codepoint, name, self.meta.units_per_em / 2))
    }

    /// Удаляет глиф.
    pub fn remove_glyph(&mut self, codepoint: u32) -> Option<Glyph> {
        self.glyphs.remove(&codepoint)
    }

    /// Пересчитывает контуры всех глифов проекта.
    pub fn rebuild_all_outlines(&mut self, settings: &OutlineSettings) {
        for glyph in self.glyphs.values_mut() {
            glyph.rebuild_outline(settings);
        }
    }

    /// Контуры глифа (с пересчётом, если глиф существует).
    pub fn glyph_outline(&self, codepoint: u32) -> Option<Vec<Contour>> {
        self.glyph(codepoint)
            .map(|g| build_glyph_outline(g, &OutlineSettings::default()))
    }

    /// Коды символов глифов проекта.
    pub fn glyph_codepoints(&self) -> Vec<u32> {
        self.glyphs.keys().copied().collect()
    }
}
