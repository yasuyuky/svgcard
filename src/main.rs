use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use xml::writer::{EmitterConfig, EventWriter, XmlEvent};

mod import;
mod style;
mod text;

#[derive(Deserialize, Debug)]
pub struct CardTemplate {
    dimension: Dimension,
    fontset: HashMap<String, Vec<String>>,
    fontweight: Option<HashMap<String, usize>>,
    imports: Option<Vec<String>>,
    texts: HashMap<String, text::TextElement>,
    svgs: Option<HashMap<String, SvgElement>>,
}

impl CardTemplate {
    pub fn from_path(path: &Path) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        Ok(toml::from_str::<CardTemplate>(&buf)?)
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct SvgElement {
    path: PathBuf,
    scale: f64,
    pos: (f64, f64),
}

#[derive(Clone, Deserialize, Debug)]
struct Dimension {
    width: usize,
    height: usize,
    bezel: usize,
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct Opt {
    template: PathBuf,
    values: PathBuf,
}

fn load_values(path: &Path) -> Result<HashMap<String, String>> {
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(toml::from_str::<HashMap<String, String>>(&buf)?)
}

pub fn comment<W: Write>(writer: &mut EventWriter<W>, s: &str) -> Result<()> {
    let comment: XmlEvent = XmlEvent::comment(s);
    writer.write(comment)?;
    Ok(())
}

fn write_svg<W: Write>(
    writer: &mut EventWriter<W>,
    path: &Path,
    template: &CardTemplate,
    dic: &HashMap<String, String>,
) -> Result<()> {
    let dim = template.dimension.clone();
    let (ws, hs) = (format!("{}mm", dim.width), format!("{}mm", dim.height));
    let vb = format!("0 0 {} {}", dim.width, dim.height);
    let svg_start: XmlEvent = XmlEvent::start_element("svg")
        .default_ns("http://www.w3.org/2000/svg")
        .attr("viewBox", &vb)
        .attr("width", &ws)
        .attr("height", &hs)
        .into();
    writer.write(svg_start)?;
    style::write_style(writer, template)?;
    for (_, te) in &template.texts {
        text::write_text_element(writer, &te, &dic)?;
    }

    for (_, se) in &template.svgs.clone().unwrap_or_default() {
        let path = path.parent().unwrap().join(&se.path);
        comment(writer, se.path.to_str().unwrap_or_default())?;
        import::import_svg(writer, &path, se.scale)?;
    }

    let svg_end: XmlEvent = XmlEvent::end_element().into();
    writer.write(svg_end)?;
    Ok(())
}

fn main() -> Result<()> {
    let stdout = std::io::stdout();
    let opt = Opt::from_args();
    let template = CardTemplate::from_path(&opt.template)?;
    let dic = load_values(&opt.values)?;
    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .create_writer(stdout);
    write_svg(&mut writer, &opt.template, &template, &dic)
}
