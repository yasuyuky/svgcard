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

fn filltext(text: &str, dic: &HashMap<String, Text>) -> Vec<String> {
    let mut t = vec![text.to_owned()];
    let re = Regex::new(r"\{(\w+)\}").expect("compile regex for placeholder");
    for cap in re.captures_iter(text) {
        let k = cap[1].to_owned();
        t = match dic.get(&k).unwrap_or(&Text::single("")) {
            Text::Single(s) => replace_vecstr(&k, s, &t),
            Text::Multi(ss) => ss.iter().flat_map(|s| replace_vecstr(&k, s, &t)).collect(),
        };
    }
    t
}

fn write_text_characters<W: Write>(writer: &mut EventWriter<W>, text: &str) -> Result<()> {
    let cs: XmlEvent = XmlEvent::characters(&text);
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

fn fontsize(text: &Text, dic: &HashMap<String, Text>, col: &Option<usize>, fontsize: f64) -> f64 {
    let len = match text {
        Text::Multi(ss) => ss
            .iter()
            .map(|s| maxlen(&filltext(s, dic)))
            .max()
            .unwrap_or(1),
        Text::Single(s) => maxlen(&filltext(s, dic)),
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
    dic: &HashMap<String, Text>,
) -> Result<()> {
    let (x, mut y) = te.pos;
    let fontsize = fontsize(&te.text, dic, &te.column, te.fontsize);
    let (xspacing, yspacing) = te.space.unwrap_or_default();
    let lettersp = format!("{}", xspacing);
    write_text_start(writer, &te.fontset, x, y, fontsize, &lettersp, &te.align)?;
    let texts = match &te.text {
        Text::Multi(texts) => texts.iter().flat_map(|text| filltext(text, &dic)).collect(),
        Text::Single(text) => filltext(text, dic),
    };
    for t in texts {
        write_text_characters(writer, &t)?;
        write_text_end(writer)?;
        y = (y as f64 + te.fontsize + yspacing).round() as usize;
        write_text_start(writer, &te.fontset, x, y, fontsize, &lettersp, &te.align)?;
    }
    write_text_end(writer)
}
