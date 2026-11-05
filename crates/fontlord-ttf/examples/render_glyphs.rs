//! Пример: растеризация глифов собранного шрифта в PPM.
//!
//! ```bash
//! cargo run -p fontlord-ttf --example render_glyphs -- demo.ttf out.ppm
//! ```

use std::io::Write;

use fontlord_core::brush::{BrushKind, BrushSettings};
use fontlord_core::model::{FontProject, Glyph, Stroke, StrokePoint};
use fontlord_ttf::build_truetype;

/// Строит демонстрационный проект с буквами A, B и C.
fn demo_project() -> FontProject {
    let mut project = FontProject::new("Fontlord Demo", "Regular");

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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let font_path = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "demo.ttf".to_string());
    let output_path = args
        .get(2)
        .cloned()
        .unwrap_or_else(|| "glyphs.ppm".to_string());

    let bytes = build_truetype(&demo_project()).expect("шрифт собран");
    std::fs::write(&font_path, &bytes).expect("шрифт записан");

    let font =
        fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).expect("шрифт загружен");

    const SIZE: usize = 96;
    const GLYPHS: [char; 3] = ['A', 'B', 'C'];
    let width = SIZE * GLYPHS.len();
    let mut canvas = vec![255u8; width * SIZE];

    for (index, character) in GLYPHS.iter().enumerate() {
        let (metrics, pixels) = font.rasterize(*character, SIZE as f32);
        // Растровое изображение глифа выравнивается по нижней границе.
        let bottom = SIZE as i32 - 12;
        for y in 0..metrics.height {
            for x in 0..metrics.width {
                let value = pixels[y * metrics.width + x];
                let target_x = index * SIZE + x + 8;
                let target_y = (bottom - (metrics.height as i32 - y as i32)) as usize;
                if target_x < width && target_y < SIZE {
                    let position = target_y * width + target_x;
                    canvas[position] = canvas[position].min(value);
                }
            }
        }
    }

    let mut file = std::fs::File::create(&output_path).expect("файл создан");
    write!(file, "P5\n{} {}\n255\n", width, SIZE).expect("заголовок записан");
    file.write_all(&canvas).expect("данные записаны");
    println!("Растеризация сохранена: {output_path}");
}
