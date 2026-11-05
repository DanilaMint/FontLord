struct Builder {
    points: Vec<(f32, f32)>,
    contours: usize,
}

impl ttf_parser::OutlineBuilder for Builder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.points.push((x, y));
        self.contours += 1;
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.points.push((x, y));
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.points.push((x1, y1));
        self.points.push((x, y));
    }
    fn curve_to(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, x: f32, y: f32) {
        self.points.push((x, y));
    }
    fn close(&mut self) {}
}

#[test]
fn debug_outline() {
    let bytes = std::fs::read("/tmp/demo.ttf").unwrap();
    let face = ttf_parser::Face::parse(&bytes, 0).unwrap();
    let gid = face.glyph_index('A').unwrap();
    let mut builder = Builder { points: Vec::new(), contours: 0 };
    face.outline_glyph(gid, &mut builder);
    println!("contours={} points={}", builder.contours, builder.points.len());
    println!("first points: {:?}", &builder.points[..builder.points.len().min(12)]);
    println!("horiz advance {:?}", face.glyph_hor_advance(gid));

    let font = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).unwrap();
    println!("fontdue metrics A: {:?}", font.metrics('A', 64.0));
}
