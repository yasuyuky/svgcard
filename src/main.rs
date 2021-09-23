use anyhow::Result;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::ops::Neg;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use xml::writer::{EmitterConfig, EventWriter, XmlEvent};

mod import;
mod style;
mod template;
mod text;

use template::CardTemplate;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct Opt {
    template: PathBuf,
    values: PathBuf,
    #[structopt(short = "s", long = "style", default_value = ".svgcard.css")]
    style: PathBuf,
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
    style: &Path,
) -> Result<()> {
    let dim = template.dimension.clone();
    let (xoffset, yoffset) = (dim.offset.0.neg(), dim.offset.1.neg());
    let unit = dim.unit.clone();
    let ws = format!("{}{}", dim.width, unit);
    let hs = format!("{}{}", dim.height, unit);
    let vb = format!("{} {} {} {}", xoffset, yoffset, dim.width, dim.height);
    let svg_start: XmlEvent = XmlEvent::start_element("svg")
        .default_ns("http://www.w3.org/2000/svg")
        .attr("viewBox", &vb)
        .attr("width", &ws)
        .attr("height", &hs)
        .into();
    writer.write(svg_start)?;
    if style.exists() {
        style::import_style(writer, style)?
    } else {
        style::write_style(writer, template)?;
    }
    for (_, te) in &template.texts {
        text::write_text_element(writer, &te, &dic)?;
    }

    for (_, se) in &template.svgs.clone().unwrap_or_default() {
        let path = path.parent().unwrap().join(&se.path);
        comment(writer, se.path.to_str().unwrap_or_default())?;
        import::import_svg(writer, &path, se.pos, se.scale)?;
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
    write_svg(&mut writer, &opt.template, &template, &dic, &opt.style)
}
