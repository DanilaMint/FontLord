//! Сборка WOFF2 (Web Open Font Format 2.0).
//!
//! В отличие от WOFF 1.0, все таблицы шрифта хранятся в одном потоке,
//! сжатом алгоритмом Brotli, а каталог таблиц записывается компактным
//! форматом.
//!
//! Таблицы `glyf` и `loca` записываются с преобразованием версии 3, то есть
//! без предварительной обработки: их данные попадают в поток как есть.
//! Такой выбор описан в спецификации WOFF2 и поддерживается всеми
//! распространёнными декодерами.

use std::io::Write;

use brotli::CompressorWriter;
use fontlord_core::model::FontProject;

use crate::writer::Writer;
use crate::{FontError, SFNT_TRUETYPE, assemble, truetype_tables};

/// Размер заголовка WOFF2.
const HEADER_SIZE: usize = 48;

/// Известные теги таблиц WOFF2 (индекс в списке — значение младших битов
/// флага записи каталога).
const KNOWN_TAGS: [&[u8; 4]; 63] = [
    b"cmap", b"head", b"hhea", b"hmtx", b"maxp", b"name", b"OS/2", b"post", b"cvt ", b"fpgm",
    b"glyf", b"loca", b"prep", b"CFF ", b"VORG", b"EBDT", b"EBLC", b"gasp", b"hdmx", b"kern",
    b"LTSH", b"PCLT", b"VDMX", b"vhea", b"vmtx", b"BASE", b"GDEF", b"GPOS", b"GSUB", b"EBSC",
    b"JSTF", b"MATH", b"CBDT", b"CBLC", b"COLR", b"CPAL", b"SVG ", b"sbix", b"acnt", b"avar",
    b"bdat", b"bloc", b"bsln", b"cvar", b"fdsc", b"feat", b"fmtx", b"fvar", b"gvar", b"hsty",
    b"just", b"lcar", b"mort", b"morx", b"opbd", b"prop", b"trak", b"Zapf", b"Silf", b"Glat",
    b"Gloc", b"Feat", b"Sill",
];

/// Собирает шрифт в формате WOFF2.
pub fn build_woff2(project: &FontProject) -> Result<Vec<u8>, FontError> {
    let tables = truetype_tables(project)?;
    // Размер эквивалентного sfnt-файла без сжатия.
    let sfnt_size = assemble(SFNT_TRUETYPE, tables.clone()).len() as u32;

    let mut sorted = tables;
    sorted.sort_by(|a, b| a.0.cmp(b.0));

    // Таблица `loca` должна идти сразу за `glyf`.
    let mut ordered: Vec<(&'static [u8; 4], Vec<u8>)> = Vec::with_capacity(sorted.len());
    let mut glyf = None;
    let mut loca = None;
    let mut rest: Vec<(&'static [u8; 4], Vec<u8>)> = Vec::with_capacity(sorted.len());
    for table in sorted {
        if table.0 == b"glyf" {
            glyf = Some(table);
        } else if table.0 == b"loca" {
            loca = Some(table);
        } else {
            rest.push(table);
        }
    }
    if let Some(table) = glyf {
        ordered.push(table);
    }
    if let Some(table) = loca {
        ordered.push(table);
    }
    ordered.extend(rest);

    // Каталог таблиц и поток данных.
    let mut directory = Writer::new();
    let mut stream = Writer::new();
    for (tag, content) in &ordered {
        let transform_version: u8 = if *tag == b"glyf" || *tag == b"loca" {
            3
        } else {
            0
        };
        match KNOWN_TAGS.iter().position(|known| known == tag) {
            Some(index) => {
                directory.u8(index as u8 | (transform_version << 6));
            }
            None => {
                directory.u8(63 | (transform_version << 6));
                directory.bytes(&tag[..]);
            }
        }
        write_base128(&mut directory, content.len() as u32);
        // Длина преобразованной таблицы не указывается: преобразование
        // нулевое, данные попадают в поток без изменений.
        stream.bytes(content);
    }

    let compressed = brotli_compress(&stream.bytes);
    let total_length = (HEADER_SIZE + directory.bytes.len() + compressed.len()) as u32;

    let mut font = Writer::new();
    font.bytes(b"wOF2");
    font.u32(SFNT_TRUETYPE);
    font.u32(total_length);
    font.u16(ordered.len() as u16);
    font.u16(0); // зарезервировано
    font.u32(sfnt_size);
    font.u32(compressed.len() as u32);
    font.u16(1); // majorVersion
    font.u16(0); // minorVersion
    font.u32(0); // metaOffset
    font.u32(0); // metaLength
    font.u32(0); // metaOrigLength
    font.u32(0); // privOffset
    font.u32(0); // privLength
    font.bytes(&directory.bytes);
    font.bytes(&compressed);
    Ok(font.bytes)
}

/// Записывает число в формате UIntBase128.
fn write_base128(writer: &mut Writer, value: u32) {
    let mut bytes = Vec::new();
    let mut value = value;
    loop {
        bytes.push((value & 0x7F) as u8);
        value >>= 7;
        if value == 0 {
            break;
        }
    }
    // Байты выводятся от старшего к младшему, continuation-бит (0x80)
    // ставится у всех, кроме последнего.
    for (index, byte) in bytes.iter().enumerate().rev() {
        if index == 0 {
            writer.u8(*byte);
        } else {
            writer.u8(byte | 0x80);
        }
    }
}

/// Сжимает данные алгоритмом Brotli.
fn brotli_compress(data: &[u8]) -> Vec<u8> {
    let mut encoder = CompressorWriter::new(Vec::new(), 4096, 11, 22);
    encoder.write_all(data).expect("сжатие не должно падать");
    encoder.into_inner()
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
    fn builds_woff2_container() {
        let bytes = build_woff2(&sample_project()).unwrap();
        assert_eq!(&bytes[0..4], b"wOF2");
        let length = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        assert_eq!(length as usize, bytes.len());
    }

    #[test]
    fn encodes_base128() {
        let mut writer = Writer::new();
        write_base128(&mut writer, 0);
        assert_eq!(writer.bytes, vec![0]);

        let mut writer = Writer::new();
        write_base128(&mut writer, 127);
        assert_eq!(writer.bytes, vec![127]);

        let mut writer = Writer::new();
        write_base128(&mut writer, 128);
        assert_eq!(writer.bytes, vec![0x81, 0x00]);

        let mut writer = Writer::new();
        write_base128(&mut writer, 300);
        assert_eq!(writer.bytes, vec![0x82, 0x2C]);
    }
}
