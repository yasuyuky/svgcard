use crate::CardTemplate;
use anyhow::Result;
use std::io::Write;
use xml::writer::{EventWriter, XmlEvent};

pub fn write_style<W: Write>(writer: &mut EventWriter<W>, template: &CardTemplate) -> Result<()> {
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
