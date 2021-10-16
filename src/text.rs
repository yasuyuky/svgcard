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
    column: Option<usize>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Text {
    Multi(Vec<String>),
    Single(String),
}

impl Text {
    pub fn single(s: &str) -> Self {
        Self::Single(s.to_owned())
    }
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
    letterspacing: &str,
    align: &Option<Align>,
) -> Result<()> {
    let (x, y) = (format!("{}", x), format!("{}", y));
    let fontsize = format!("{}", fontsize);
    let start: XmlEvent = XmlEvent::start_element("text")
        .attr("class", class)
        .attr("x", &x)
        .attr("y", &y)
        .attr("font-size", &fontsize)
        .attr("letter-spacing", letterspacing)
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

fn filltext(text: &str, dic: &HashMap<String, String>) -> String {
    let mut t = text.to_owned();
    let re = Regex::new(r"\{(\w+)\}").expect("compile regex for placeholder");
    for cap in re.captures_iter(text) {
        let k = cap[1].to_owned();
        let v = dic.get(&k).map(String::from).unwrap_or_default();
        t = t.replace(&format!("{{{}}}", k), &v);
    }
    t
}

fn write_text_characters<W: Write>(
    writer: &mut EventWriter<W>,
    text: &str,
    dic: &HashMap<String, String>,
) -> Result<()> {
    let t = filltext(text, dic);
    let cs: XmlEvent = XmlEvent::characters(&t);
    writer.write(cs)?;
    Ok(())
}

fn replace_vecstr(k: &str, s: &str, t: &[String]) -> Vec<String> {
    t.iter()
        .map(|u| u.replace(&format!("{{{}}}", k), s))
        .collect()
}

fn write_text_end<W: Write>(writer: &mut EventWriter<W>) -> Result<()> {
    let end: XmlEvent = XmlEvent::end_element().into();
    writer.write(end)?;
    Ok(())
}

fn fontsize(text: &Text, dic: &HashMap<String, String>, col: &Option<usize>, fontsize: f64) -> f64 {
    let len = match text {
        Text::Multi(ss) => ss
            .iter()
            .map(|s| filltext(s, dic).chars().count())
            .max()
            .unwrap_or(1),
        Text::Single(s) => filltext(s, dic).chars().count(),
    };
    match col {
        Some(col) => fontsize * (*col as f64 / len as f64).min(1.0),
        _ => fontsize,
    }
}

fn maxlen(ss: &[String]) -> usize {
    ss.iter().map(|t| t.chars().count()).max().unwrap_or(1)
}

pub fn write_text_element<W: Write>(
    writer: &mut EventWriter<W>,
    te: &TextElement,
    dic: &HashMap<String, String>,
) -> Result<()> {
    let (x, mut y) = te.pos;
    let fontsize = fontsize(&te.text, dic, &te.column, te.fontsize);
    let (xspacing, yspacing) = te.space.unwrap_or_default();
    let lettersp = format!("{}", xspacing);
    write_text_start(writer, &te.fontset, x, y, fontsize, &lettersp, &te.align)?;
    match &te.text {
        Text::Multi(vecstr) => {
            for text in vecstr {
                write_text_characters(writer, text, dic)?;
                write_text_end(writer)?;
                y = (y as f64 + te.fontsize + yspacing).round() as usize;
                write_text_start(writer, &te.fontset, x, y, fontsize, &lettersp, &te.align)?;
            }
        }
        Text::Single(text) => write_text_characters(writer, text, dic)?,
    }
    write_text_end(writer)
}
