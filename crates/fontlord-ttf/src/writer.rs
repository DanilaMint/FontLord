//! Потоковая запись данных в формате big-endian.

/// Простой писатель байтов с сериализацией в big-endian.
#[derive(Debug, Default)]
pub struct Writer {
    /// Накопленные байты.
    pub bytes: Vec<u8>,
}

impl Writer {
    /// Создаёт пустой писатель.
    pub fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    /// Записывает один байт.
    pub fn u8(&mut self, value: u8) -> &mut Self {
        self.bytes.push(value);
        self
    }

    /// Записывает 16-битное число.
    pub fn u16(&mut self, value: u16) -> &mut Self {
        self.bytes.extend_from_slice(&value.to_be_bytes());
        self
    }

    /// Записывает знаковое 16-битное число.
    pub fn i16(&mut self, value: i16) -> &mut Self {
        self.bytes.extend_from_slice(&value.to_be_bytes());
        self
    }

    /// Записывает 32-битное число.
    pub fn u32(&mut self, value: u32) -> &mut Self {
        self.bytes.extend_from_slice(&value.to_be_bytes());
        self
    }

    /// Записывает знаковое 32-битное число.
    pub fn i32(&mut self, value: i32) -> &mut Self {
        self.bytes.extend_from_slice(&value.to_be_bytes());
        self
    }

    /// Записывает 64-битное число.
    pub fn u64(&mut self, value: u64) -> &mut Self {
        self.bytes.extend_from_slice(&value.to_be_bytes());
        self
    }

    /// Записывает последовательность байт.
    pub fn bytes(&mut self, data: &[u8]) -> &mut Self {
        self.bytes.extend_from_slice(data);
        self
    }

    /// Записывает строку в кодировке UTF-16BE без префикса длины.
    ///
    /// Так строки хранятся в таблице `name` для платформы Windows.
    pub fn utf16be_string(&mut self, text: &str) -> &mut Self {
        for unit in text.encode_utf16() {
            self.u16(unit);
        }
        self
    }

    /// Текущая длина.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Признак пустого буфера.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Дописывает нули до выравнивания по 4 байта.
    pub fn pad_to_4(&mut self) {
        while self.bytes.len() % 4 != 0 {
            self.bytes.push(0);
        }
    }
}

/// Контрольная сумма sfnt-таблицы.
pub fn checksum(data: &[u8]) -> u32 {
    let mut sum: u32 = 0;
    let mut chunks = data.chunks_exact(4);
    for chunk in &mut chunks {
        sum = sum.wrapping_add(u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }
    let remainder = chunks.remainder();
    if !remainder.is_empty() {
        let mut buffer = [0u8; 4];
        buffer[..remainder.len()].copy_from_slice(remainder);
        sum = sum.wrapping_add(u32::from_be_bytes(buffer));
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_matches_known_value() {
        // Контрольная сумма слова "abcd" в big-endian.
        assert_eq!(checksum(b"abcd"), 0x61626364);
    }

    #[test]
    fn writer_pads_to_four_bytes() {
        let mut writer = Writer::new();
        writer.u8(1).u8(2).u8(3);
        writer.pad_to_4();
        assert_eq!(writer.len(), 4);
    }
}
