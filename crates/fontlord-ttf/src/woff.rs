//! Сборка WOFF (Web Open Font Format 1.0).
//!
//! WOFF — это контейнер, в котором таблицы шрифта хранятся сжатыми
//! алгоритмом zlib. Файл содержит заголовок, каталог таблиц и сами данные.

use std::io::Write;

use flate2::Compression;
use flate2::write::ZlibEncoder;
use fontlord_core::model::FontProject;

use crate::writer::{Writer, checksum};
use crate::{FontError, SFNT_TRUETYPE, assemble, truetype_tables};

/// Размер заголовка WOFF.
const HEADER_SIZE: usize = 44;

/// Размер записи каталога таблиц.
const ENTRY_SIZE: usize = 20;

/// Собирает шрифт в формате WOFF.
pub fn build_woff(project: &FontProject) -> Result<Vec<u8>, FontError> {
    let tables = truetype_tables(project)?;
    // Размер эквивалентного sfnt-файла без сжатия.
    let sfnt_size = assemble(SFNT_TRUETYPE, tables.clone()).len() as u32;

    let mut sorted = tables;
    sorted.sort_by(|a, b| a.0.cmp(b.0));

    let mut entries: Vec<(u32, u32, u32, u32)> = Vec::with_capacity(sorted.len());
    let mut payload = Writer::new();
    let mut offset = (HEADER_SIZE + sorted.len() * ENTRY_SIZE) as u32;

    for (_, content) in &sorted {
        let compressed = zlib_compress(content);
        let (data, length) = if compressed.len() < content.len() {
            let length = compressed.len();
            (compressed, length)
        } else {
            // Выигрыш от сжатия отсутствует — храним как есть.
            (content.clone(), content.len())
        };
        entries.push((
            offset,
            length as u32,
            content.len() as u32,
            checksum(content),
        ));
        payload.bytes(&data);
        offset += data.len() as u32;
        // Данные таблиц выравниваются по четырём байтам.
        while offset % 4 != 0 {
            payload.u8(0);
            offset += 1;
        }
    }

    let total_length = offset;

    let mut font = Writer::new();
    font.bytes(b"wOFF");
    font.u32(SFNT_TRUETYPE);
    font.u32(total_length);
    font.u16(sorted.len() as u16);
    font.u16(0); // зарезервировано
    font.u32(sfnt_size);
    font.u16(1); // majorVersion
    font.u16(0); // minorVersion
    font.u32(0); // metaOffset
    font.u32(0); // metaLength
    font.u32(0); // metaOrigLength
    font.u32(0); // privOffset
    font.u32(0); // privLength

    for (index, (tag, _)) in sorted.iter().enumerate() {
        font.bytes(&tag[..]);
        font.u32(entries[index].0);
        font.u32(entries[index].1);
        font.u32(entries[index].2);
        font.u32(entries[index].3);
    }

    font.bytes(&payload.bytes);
    Ok(font.bytes)
}

/// Сжимает данные таблицы алгоритмом zlib.
fn zlib_compress(data: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).expect("сжатие не должно падать");
    encoder.finish().expect("сжатие не должно падать")
}

#[cfg(test)]
mod tests {
    use super::*;
    use fontlord_core::brush::{BrushKind, BrushSettings};
    use fontlord_core::model::{FontProject, Glyph, Stroke, StrokePoint};

    fn sample_project() -> FontProject {
        let mut project = FontProject::new("Test Font", "Regular");
        let mut glyph = Glyph::new(0x0041, "A", 600);
        let mut stroke = Stroke::new(
            1,
            BrushKind::Marker,
            BrushSettings::for_kind(BrushKind::Marker),
        );
        stroke.closed = true;
        stroke.points = vec![
            StrokePoint::new(100.0, 0.0, 0.8),
            StrokePoint::new(500.0, 0.0, 0.8),
            StrokePoint::new(500.0, 700.0, 0.8),
            StrokePoint::new(100.0, 700.0, 0.8),
        ];
        glyph.strokes.push(stroke);
        project.glyphs.insert(0x0041, glyph);
        project
    }

    #[test]
    fn builds_woff_container() {
        let bytes = build_woff(&sample_project()).unwrap();
        assert_eq!(&bytes[0..4], b"wOFF");
        let length = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        assert_eq!(length as usize, bytes.len());
    }
}
