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

pub struct FontSource {
    channel: u8,
    metrics: Vec<GlyphMetricsSource>,
    kerning_pairs: Vec<(char, char, f32)>,
}

impl FontSource {
    fn new(channel: u8) -> Self {
        FontSource {
            channel,
            metrics: Vec::new(),
            kerning_pairs: Vec::new(),
        }
    }
        
    fn write<W: std::io::Write>(&mut self, writer: &mut W)
        -> std::io::Result<()>
    {
        self.metrics.sort_unstable_by_key(|c| c.0);
        self.kerning_pairs.sort_unstable_by_key(|v| (v.0, v.1));
        let metrics_ref: &[GlyphMetricsSource] = &self.metrics;
        let kerning_ref: &[(char, char, f32)] = &self.kerning_pairs;
        write!(
            writer,
            "({}, &{:?}, &{:?})",
            self.channel,
            metrics_ref,
            kerning_ref,
        )?;
        Ok(())
    }

    fn write_opt<W: std::io::Write>(
        opt: &mut Option<Self>,
        writer: &mut W,
    ) -> std::io::Result<()> {
        match opt.as_mut() {
            Some(source) => {
                write!(writer, "Some(")?;
                source.write(writer)?;
                write!(writer, ")")?;
            },
            None => write!(writer, "None")?,
        }
        Ok(())
    }
}

pub(super) struct FontOutput {
    width: usize,
    height: usize,
    channels: usize,
    buffer: Vec<u8>,
    normal: FontSource,
    bold: Option<FontSource>,
    italic: Option<FontSource>,
    bold_italic: Option<FontSource>,
}

impl FontOutput {
    pub fn new(
        width: usize,
        height: usize,
        channels: usize,
        bold_channel: Option<u8>,
        italic_channel: Option<u8>,
        bold_italic_channel: Option<u8>,
    ) -> Self {
        let bufsize = width * height * channels;
        Self {
            width,
            height,
            channels,
            buffer: vec![0; bufsize],
            normal: FontSource::new(0),
            bold: bold_channel.map(FontSource::new),
            italic: italic_channel.map(FontSource::new),
            bold_italic: bold_italic_channel.map(FontSource::new),
        }
    }

    pub fn buffer(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    pub fn add_metric(&mut self, channel: u8, metric: GlyphMetric) {
        let source = if channel == 0 {
            &mut self.normal
        } else {
            let Self { bold, italic, bold_italic, .. } = self;
            let bold = bold.as_mut();
            let italic = italic.as_mut();
            let bold_italic = bold_italic.as_mut();
            bold.filter(|fs| fs.channel == channel)
                .or_else(|| italic.filter(|fs| fs.channel == channel))
                .or_else(|| bold_italic.filter(|fs| fs.channel == channel))
                .expect("Cannot add metric to invalid channel")
        };
        source.metrics.push((
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
        let mut data_filename = path.as_ref().to_path_buf();
        let mut ext = data_filename.extension()
            .unwrap_or_default()
            .to_os_string();
        ext.push(".texdata");
        data_filename.set_extension(ext);
        let data_filename_str = data_filename.to_str()
            .expect("Sorry, output filenames must be valid unicode");
        {
            let mut texfile = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&data_filename)?;
            texfile.write_all(&self.buffer)?;
        }
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        writeln!(
            &file,
            "use suzy::platform::opengl::FontFamilySource;",
        )?;
        write!(
            &file,
            "pub const FONT: FontFamilySource = FontFamilySource {{
                image_width: {},
                image_height: {},
                image_channels: {},
                atlas_image: include_bytes!({:?}),
            ",
            self.width,
            self.height,
            self.channels,
            data_filename_str,
        )?;
        write!(&file, "normal: ")?;
        self.normal.write(&mut file)?;
        write!(&file, ", bold: ")?;
        FontSource::write_opt(&mut self.bold, &mut file)?;
        write!(&file, ", italic: ")?;
        FontSource::write_opt(&mut self.italic, &mut file)?;
        write!(&file, ", bold_italic: ")?;
        FontSource::write_opt(&mut self.bold_italic, &mut file)?;
        writeln!(&file, "}};")?;
        Ok(())
    }
}
