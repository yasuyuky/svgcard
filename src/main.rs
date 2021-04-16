use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use xml::writer::{EmitterConfig, EventWriter, XmlEvent};

mod text;

#[derive(Deserialize, Debug)]
struct CardTemplate {
    dimension: Dimension,
    fontset: HashMap<String, Vec<String>>,
    fontweight: Option<HashMap<String, usize>>,
    imports: Option<Vec<String>>,
    texts: HashMap<String, text::TextElement>,
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

fn write_style<W: Write>(writer: &mut EventWriter<W>, template: &CardTemplate) -> Result<()> {
    let start: XmlEvent = XmlEvent::start_element("style").into();
    writer.write(start)?;
    for url in &template.imports.clone().unwrap_or_default() {
        let s = format!("@import url('{}');\n", url);
        let cs: XmlEvent = XmlEvent::characters(&s).into();
        writer.write(cs)?;
    }
    for (key, fonts) in &template.fontset {
        let fonts = fonts.join(",");
        let s = format!(".{} {{ font-family: {}; }}\n", key, fonts);
        let cs: XmlEvent = XmlEvent::characters(&s).into();
        writer.write(cs)?;
    }
    for (key, weight) in &template.fontweight.clone().unwrap_or_default() {
        let s = format!(".{} {{ font-weight: {}; }}\n", key, weight);
        let cs: XmlEvent = XmlEvent::characters(&s).into();
        writer.write(cs)?;
    }
    let end: XmlEvent = XmlEvent::end_element().into();
    writer.write(end)?;
    Ok(())
}

fn write_svg<W: Write>(
    writer: &mut EventWriter<W>,
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
    write_style(writer, template)?;
    for (_, te) in &template.texts {
        text::write_te(writer, &te, &dic)?;
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
    write_svg(&mut writer, &template, &dic)
}
