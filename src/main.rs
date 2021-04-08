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

fn write_te<W: Write>(
    w: &mut EventWriter<W>,
    te: &TextElement,
    dic: &HashMap<String, String>,
) -> Result<()> {
    let start: XmlEvent = XmlEvent::start_element("text")
        .attr("class", &te.fontset)
        .into();
    w.write(start)?;

    match &te.text {
        Text::Multi(vecstr) => {
            for text in vecstr {
                let mut t = text.clone();
                for (k, v) in dic {
                    t = t.replace(&format!("{{{}}}", k), v);
                }
                let cs: XmlEvent = XmlEvent::characters(&text).into();
                w.write(cs)?;
            }
        }
        Text::Single(text) => {
            let mut t = text.clone();
            for (k, v) in dic {
                t = t.replace(&format!("{{{}}}", k), v);
            }
            let cs: XmlEvent = XmlEvent::characters(&text).into();
            w.write(cs)?;
        }
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

fn main() -> Result<()> {
    let mut file = File::create("output.svg").unwrap();
    let opt = Opt::from_args();
    let template = CardTemplate::from_path(&opt.template)?;
    let dic = load_values(&opt.values)?;
    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .create_writer(&mut file);
    for (key, te) in template.texts {
        println!("{}", key);
        write_te(&mut writer, &te, &dic)?;
    }
    Ok(())
}
