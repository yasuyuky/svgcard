use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use xml::writer::{EmitterConfig, EventWriter, XmlEvent};

#[derive(Deserialize, Debug)]
struct CardTemplate {
    dimension: Dimension,
    fontset: FontSet,
    texts: HashMap<String, TextElement>,
}

impl CardTemplate {
    pub fn from_path(path: &Path) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        Ok(toml::from_str::<CardTemplate>(&buf)?)
    }
}

#[derive(Deserialize, Debug)]
struct Dimension {
    width: usize,
    height: usize,
    bezel: usize,
}

#[derive(Deserialize, Debug)]
struct FontSet {
    normal: Vec<String>,
    logo: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct TextElement {
    text: Text,
    fontset: String,
    fontsize: f64,
    align: Option<Align>,
    pos: (usize, usize),
    space: Option<(f64, f64)>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Text {
    Multi(Vec<String>),
    Single(String),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum Align {
    Left,
    Right,
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct Opt {
    template: PathBuf,
    values: PathBuf,
}

fn write_text<W: Write>(
    w: &mut EventWriter<W>,
    text: &str,
    dic: &HashMap<String, String>,
) -> Result<()> {
    let mut t = text.to_owned();
    for (k, v) in dic {
        t = t.replace(&format!("{{{}}}", k), v);
    }
    let cs: XmlEvent = XmlEvent::characters(&t).into();
    w.write(cs)?;
    Ok(())
}

fn write_te<W: Write>(
    w: &mut EventWriter<W>,
    te: &TextElement,
    dic: &HashMap<String, String>,
) -> Result<()> {
    let (x, y) = te.pos;
    let x = &format!("{}", x);
    let y = &format!("{}", y);
    let fontsize = &format!("{}", te.fontsize);
    let start: XmlEvent = XmlEvent::start_element("text")
        .attr("class", &te.fontset)
        .attr("x", x)
        .attr("y", y)
        .attr("font-size", fontsize)
        .into();
    w.write(start)?;

    match &te.text {
        Text::Multi(vecstr) => {
            for text in vecstr {
                write_text(w, text, dic)?
            }
        }
        Text::Single(text) => write_text(w, text, dic)?,
    }
    let end: XmlEvent = XmlEvent::end_element().into();
    w.write(end)?;
    Ok(())
}

fn load_values(path: &Path) -> Result<HashMap<String, String>> {
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(toml::from_str::<HashMap<String, String>>(&buf)?)
}

fn write_svg<W: Write>(
    writer: &mut EventWriter<W>,
    template: &CardTemplate,
    dic: &HashMap<String, String>,
) -> Result<()> {
    let svg_start: XmlEvent = XmlEvent::start_element("svg").into();
    writer.write(svg_start)?;
    for (_, te) in &template.texts {
        write_te(writer, &te, &dic)?;
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
