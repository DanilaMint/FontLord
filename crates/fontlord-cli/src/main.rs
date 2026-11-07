//! Консольные утилиты Fontlord.
//!
//! Позволяют работать с проектами без графического приложения:
//!
//! ```bash
//! fontlord-cli info project.fontlord          # сводка по проекту
//! fontlord-cli export project.fontlord out.ttf # сборка шрифта
//! ```

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use fontlord_core::brush::{BrushKind, BrushSettings};
use fontlord_core::model::{FontProject, Glyph, Stroke, StrokePoint};
use fontlord_core::outline::build_glyph_outline;

/// Утилиты Fontlord.
#[derive(Parser, Debug)]
#[command(name = "fontlord-cli", version, about = "Консольные утилиты Fontlord")]
struct Cli {
    /// Подкоманда.
    #[command(subcommand)]
    command: Command,
}

/// Подкоманды.
#[derive(Subcommand, Debug)]
enum Command {
    /// Показать сводку по проекту.
    Info {
        /// Файл проекта.
        project: PathBuf,
    },
    /// Собрать шрифт из проекта.
    Export {
        /// Файл проекта.
        project: PathBuf,
        /// Куда сохранить шрифт.
        output: PathBuf,
        /// Формат шрифта: ttf, otf, woff или woff2.
        ///
        /// По умолчанию определяется по расширению выходного файла.
        #[arg(long, value_parser = ["ttf", "otf", "woff", "woff2"])]
        format: Option<String>,
    },
    /// Вывести список глифов проекта.
    Glyphs {
        /// Файл проекта.
        project: PathBuf,
    },
    /// Создать демонстрационный проект с буквами A, B и C.
    Demo {
        /// Куда сохранить проект.
        output: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Info { project } => {
            let project = load(&project)?;
            print_info(&project);
            Ok(())
        }
        Command::Export {
            project,
            output,
            format,
        } => {
            let project = load(&project)?;
            let format = resolve_format(format.as_deref(), &output)?;
            let bytes =
                fontlord_ttf::build_font(&project, format).context("не удалось собрать шрифт")?;
            std::fs::write(&output, &bytes)
                .with_context(|| format!("не удалось записать {}", output.display()))?;
            println!(
                "Шрифт записан: {} ({} байт, {} глифов, формат {})",
                output.display(),
                bytes.len(),
                project.glyphs.len(),
                format.label()
            );
            Ok(())
        }
        Command::Demo { output } => {
            let project = demo_project();
            fontlord_core::project::save(&project, &output)
                .with_context(|| format!("не удалось записать проект {}", output.display()))?;
            println!(
                "Демонстрационный проект записан: {} ({} глифов)",
                output.display(),
                project.glyphs.len()
            );
            Ok(())
        }
        Command::Glyphs { project } => {
            let project = load(&project)?;
            for (codepoint, glyph) in &project.glyphs {
                let contours = build_glyph_outline(glyph, &Default::default());
                let points: usize = contours.iter().map(|c| c.points.len()).sum();
                println!(
                    "U+{:04X} {:<40} штрихов: {:<3} контуров: {:<3} точек: {:<5} ширина: {}",
                    codepoint,
                    truncate(&glyph.name, 40),
                    glyph.strokes.len(),
                    contours.len(),
                    points,
                    glyph.advance_width
                );
            }
            Ok(())
        }
    }
}

/// Загружает проект из файла.
fn load(path: &Path) -> Result<FontProject> {
    fontlord_core::project::load(path)
        .with_context(|| format!("не удалось прочитать проект {}", path.display()))
}

/// Определяет формат шрифта: по аргументу `--format` или по расширению файла.
fn resolve_format(requested: Option<&str>, output: &Path) -> Result<fontlord_ttf::FontFormat> {
    if let Some(value) = requested {
        let format = fontlord_ttf::FontFormat::from_extension(value)
            .ok_or_else(|| anyhow::anyhow!("неизвестный формат: {value}"))?;
        return Ok(format);
    }
    let extension = output
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    fontlord_ttf::FontFormat::from_extension(extension).ok_or_else(|| {
        anyhow::anyhow!(
            "не удалось определить формат по расширению «{extension}», укажите --format"
        )
    })
}

/// Печатает сводку по проекту.
fn print_info(project: &FontProject) {
    let meta = &project.meta;
    println!("Семейство:      {} {}", meta.family_name, meta.style_name);
    println!("Версия:        {}", meta.version);
    println!("Единиц на em:  {}", meta.units_per_em);
    println!(
        "Вертикальные:  ascender {} / descender {} / line gap {}",
        meta.ascender, meta.descender, meta.line_gap
    );
    println!(
        "Высота букв:   x-height {} / cap-height {}",
        meta.x_height, meta.cap_height
    );
    println!("Глифов:        {}", project.glyphs.len());
    println!("Кернингов:     {}", project.kerning.len());
    println!("Лигатур:       {}", project.ligatures.len());
    println!("Меток:         {}", project.marks.len());
}

/// Строит демонстрационный проект с буквами A, B и C.
fn demo_project() -> FontProject {
    let mut project = FontProject::new("Fontlord Demo", "Regular");
    project.meta.designer = "Fontlord".into();
    project.meta.copyright = "© Fontlord".into();

    // «A» — две диагонали и перекладина.
    let mut a = Glyph::new(0x0041, "A", 620);
    a.strokes.push(stroke(
        1,
        BrushKind::Pen,
        &[(120.0, 0.0), (240.0, 380.0), (360.0, 700.0)],
    ));
    a.strokes.push(stroke(
        2,
        BrushKind::Pen,
        &[(360.0, 700.0), (480.0, 380.0), (600.0, 0.0)],
    ));
    a.strokes.push(stroke(
        3,
        BrushKind::Marker,
        &[(200.0, 300.0), (520.0, 300.0)],
    ));
    project.glyphs.insert(0x0041, a);

    // «B» — вертикальная стойка и две bowls.
    let mut b = Glyph::new(0x0042, "B", 600);
    b.strokes.push(stroke(
        1,
        BrushKind::Pen,
        &[(150.0, 0.0), (150.0, 350.0), (150.0, 700.0)],
    ));
    b.strokes.push(stroke(
        2,
        BrushKind::Marker,
        &[
            (150.0, 350.0),
            (420.0, 350.0),
            (420.0, 700.0),
            (150.0, 700.0),
        ],
    ));
    project.glyphs.insert(0x0042, b);

    // «C» — широкая дуга.
    let mut c = Glyph::new(0x0043, "C", 600);
    c.strokes.push(stroke(
        1,
        BrushKind::BroadNib,
        &[
            (480.0, 560.0),
            (320.0, 700.0),
            (140.0, 560.0),
            (140.0, 140.0),
            (320.0, 0.0),
            (480.0, 140.0),
        ],
    ));
    project.glyphs.insert(0x0043, c);

    project
}

/// Создаёт штрих по списку точек.
fn stroke(id: u32, kind: BrushKind, points: &[(f32, f32)]) -> Stroke {
    let mut stroke = Stroke::new(id, kind, BrushSettings::for_kind(kind));
    stroke.points = points
        .iter()
        .map(|(x, y)| StrokePoint::new(*x, *y, 0.8))
        .collect();
    stroke
}

/// Обрезает строку до указанной длины.
fn truncate(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let mut result: String = text.chars().take(limit - 1).collect();
    result.push('…');
    result
}
