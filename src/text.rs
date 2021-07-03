use anyhow::Result;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Write;
use xml::writer::{EventWriter, XmlEvent};

#[derive(Deserialize, Debug)]
pub struct TextElement {
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

fn write_text_start<W: Write>(
    writer: &mut EventWriter<W>,
    class: &str,
    x: usize,
    y: usize,
    fontsize: f64,
    align: &Option<Align>,
) -> Result<()> {
    let (x, y) = (format!("{}", x), format!("{}", y));
    let fontsize = format!("{}", fontsize);
    let start: XmlEvent = XmlEvent::start_element("text")
        .attr("class", class)
        .attr("x", &x)
        .attr("y", &y)
        .attr("font-size", &fontsize)
        .attr(
            "text-anchor",
            match align {
                Some(Align::Right) => "end",
                _ => "start",
            },
        )
        .into();
    writer.write(start)?;
    Ok(())
}

fn write_text_characters<W: Write>(
    writer: &mut EventWriter<W>,
    text: &str,
    dic: &HashMap<String, String>,
) -> Result<()> {
    let mut t = text.to_owned();
    let re = Regex::new(r"\{(\w+)\}")?;
    for cap in re.captures_iter(text) {
        let k = cap[1].to_owned();
        let v = dic.get(&k).map(String::from).unwrap_or_default();
        t = t.replace(&format!("{{{}}}", k), &v);
    }
    let cs: XmlEvent = XmlEvent::characters(&t).into();
    writer.write(cs)?;
    Ok(())
}

fn write_text_end<W: Write>(writer: &mut EventWriter<W>) -> Result<()> {
    let end: XmlEvent = XmlEvent::end_element().into();
    writer.write(end)?;
    Ok(())
}

pub fn write_text_element<W: Write>(
    writer: &mut EventWriter<W>,
    te: &TextElement,
    dic: &HashMap<String, String>,
) -> Result<()> {
    let (x, mut y) = te.pos;
    write_text_start(writer, &te.fontset, x, y, te.fontsize, &te.align)?;
    match &te.text {
        Text::Multi(vecstr) => {
            for text in vecstr {
                write_text_characters(writer, text, dic)?;
                write_text_end(writer)?;
                y = y + te.fontsize.ceil() as usize;
                write_text_start(writer, &te.fontset, x, y, te.fontsize, &te.align)?;
            }
        }
        Text::Single(text) => write_text_characters(writer, text, dic)?,
    }
    write_text_end(writer)
}
