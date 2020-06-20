use std::io::Write;
use std::path::Path;

pub(crate) struct GlyphMetric {
    pub ch: char,
    pub u_offset: f32,
    pub v_offset: f32,
    pub uv_width: f32,
    pub uv_height: f32,
    pub bb_min_x: f32,
    pub bb_max_x: f32,
    pub bb_min_y: f32,
    pub bb_max_y: f32,
    pub advance_width: f32,
}

// 0: char
// 1,2: u,v
// 3,4: uv width,height
// 5-8: relative bb
// 9: relative advance width
type GlyphMetricsSource = (char, f32, f32, f32, f32, f32, f32, f32, f32, f32);

pub(super) struct FontOutput {
    width: usize,
    height: usize,
    channels: usize,
    font_size: f64,
    padding_ratio: f64,
    buffer: Vec<u8>,
    metrics: Vec<GlyphMetricsSource>,
    kerning_pairs: Vec<(char, char, f32)>,
}

impl FontOutput {
    pub fn new(
        width: usize,
        height: usize,
        channels: usize,
        font_size: f64,
        padding_ratio: f64,
    ) -> Self {
        let bufsize = width * height * channels;
        Self {
            width,
            height,
            channels,
            font_size,
            padding_ratio,
            buffer: vec![0; bufsize],
            metrics: vec![],
            kerning_pairs: vec![],
        }
    }

    pub fn buffer(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    pub fn add_metric(&mut self, metric: GlyphMetric) {
        self.metrics.push((
            metric.ch,
            metric.u_offset,
            metric.v_offset,
            metric.uv_width,
            metric.uv_height,
            metric.bb_min_x,
            metric.bb_max_x,
            metric.bb_min_y,
            metric.bb_max_y,
            metric.advance_width,
        ));
    }

    pub fn write<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        self.metrics.sort_unstable_by_key(|c| c.0);
        self.kerning_pairs.sort_unstable_by_key(|v| (v.0, v.1));
        let pixels_ref: &[u8] = &self.buffer;
        let metrics_ref: &[GlyphMetricsSource] = &self.metrics;
        let kerning_ref: &[(char, char, f32)] = &self.kerning_pairs;
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        writeln!(&file, "use suzy::platform::opengl::text::FontSource;")?;
        writeln!(
            &file,
            "pub const FONT: FontSource = FontSource {{
                font_size: {:.1},
                image_width: {},
                image_height: {},
                atlas_image: &{:?},
                coords: &{:?},
                padding_ratio: {:.1},
                kerning_pairs: &{:?},
            }};",
            self.font_size,
            self.width,
            self.height,
            pixels_ref,
            metrics_ref,
            self.padding_ratio,
            kerning_ref,
        )?;
        Ok(())
    }
}
