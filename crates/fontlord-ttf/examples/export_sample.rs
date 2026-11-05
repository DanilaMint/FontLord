//! Пример: собирает демонстрационный шрифт и сохраняет его в файл.
//!
//! ```bash
//! cargo run -p fontlord-ttf --example export_sample -- /tmp/sample.ttf
//! ```

use std::path::PathBuf;

use fontlord_core::brush::{BrushKind, BrushSettings};
use fontlord_core::model::{FontProject, Glyph, Stroke, StrokePoint};
use fontlord_ttf::build_truetype;

/// Строит шрифт с буквами A, B и C.
fn build_demo() -> FontProject {
    let mut project = FontProject::new("Fontlord Demo", "Regular");
    project.meta.designer = "Fontlord".into();
    project.meta.copyright = "© Fontlord".into();

    let mut a = Glyph::new(0x0041, "A", 620);
    let mut left = Stroke::new(
        1,
        BrushKind::Pen,
        BrushSettings {
            width: 80.0,
            ..BrushSettings::for_kind(BrushKind::Pen)
        },
    );
    left.points = vec![
        StrokePoint::new(120.0, 0.0, 0.9),
        StrokePoint::new(240.0, 380.0, 0.6),
        StrokePoint::new(360.0, 700.0, 0.9),
    ];
    let mut right = left.clone();
    right.id = 2;
    right.points = vec![
        StrokePoint::new(360.0, 700.0, 0.9),
        StrokePoint::new(480.0, 380.0, 0.6),
        StrokePoint::new(600.0, 0.0, 0.9),
    ];
    let mut bar = Stroke::new(
        3,
        BrushKind::Marker,
        BrushSettings::for_kind(BrushKind::Marker),
    );
    bar.points = vec![
        StrokePoint::new(200.0, 300.0, 0.8),
        StrokePoint::new(520.0, 300.0, 0.8),
    ];
    a.strokes.extend([left, right, bar]);
    project.glyphs.insert(0x0041, a);

    let mut b = Glyph::new(0x0042, "B", 600);
    let mut stem = Stroke::new(
        1,
        BrushKind::Pen,
        BrushSettings {
            width: 80.0,
            ..BrushSettings::for_kind(BrushKind::Pen)
        },
    );
    stem.points = vec![
        StrokePoint::new(150.0, 0.0, 0.9),
        StrokePoint::new(150.0, 350.0, 0.7),
        StrokePoint::new(150.0, 700.0, 0.9),
    ];
    let mut bowl = Stroke::new(
        2,
        BrushKind::Marker,
        BrushSettings::for_kind(BrushKind::Marker),
    );
    bowl.closed = true;
    bowl.points = vec![
        StrokePoint::new(150.0, 350.0, 0.8),
        StrokePoint::new(420.0, 350.0, 0.8),
        StrokePoint::new(420.0, 700.0, 0.8),
        StrokePoint::new(150.0, 700.0, 0.8),
    ];
    b.strokes.extend([stem, bowl]);
    project.glyphs.insert(0x0042, b);

    let mut c = Glyph::new(0x0043, "C", 600);
    let mut arc = Stroke::new(
        1,
        BrushKind::BroadNib,
        BrushSettings::for_kind(BrushKind::BroadNib),
    );
    arc.points = vec![
        StrokePoint::new(480.0, 560.0, 0.8),
        StrokePoint::new(320.0, 700.0, 0.8),
        StrokePoint::new(140.0, 560.0, 0.8),
        StrokePoint::new(140.0, 140.0, 0.8),
        StrokePoint::new(320.0, 0.0, 0.8),
        StrokePoint::new(480.0, 140.0, 0.8),
    ];
    c.strokes.push(arc);
    project.glyphs.insert(0x0043, c);

    project
}

fn main() {
    let path: PathBuf = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("fontlord-demo.ttf"));

    let project = build_demo();
    match build_truetype(&project) {
        Ok(bytes) => {
            std::fs::write(&path, &bytes).expect("не удалось записать шрифт");
            println!("Шрифт записан: {} ({} байт)", path.display(), bytes.len());
        }
        Err(error) => {
            eprintln!("Ошибка сборки шрифта: {error}");
            std::process::exit(1);
        }
    }
}
