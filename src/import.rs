use anyhow::Result;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;
use xml::{reader as r, writer as w};

pub fn import_svg<W: Write>(writer: &mut w::EventWriter<W>, path: &Path, scale: f64) -> Result<()> {
    let file = File::open(path).unwrap();
    let file = BufReader::new(file);

    let parser = r::EventReader::new(file);
    let mut inside_svg = false;
    for e in parser {
        match &e {
            Ok(r::XmlEvent::StartDocument { .. }) => {}
            Ok(r::XmlEvent::EndDocument { .. }) => {}
            Ok(r::XmlEvent::StartElement { name, .. }) => {
                if name.local_name == "svg" {
                    inside_svg = true;
                    let tr = transform(scale);
                    let g: w::XmlEvent = w::XmlEvent::start_element("g")
                        .attr("transform", &tr)
                        .into();
                    writer.write(g)?;
                } else if inside_svg {
                    writer.write(e.unwrap().as_writer_event().unwrap())?;
                }
            }
            Ok(r::XmlEvent::EndElement { .. }) => {
                let e: w::XmlEvent = w::XmlEvent::end_element().into();
                writer.write(e)?;
            }
            Ok(ev) => {
                writer.write(ev.as_writer_event().unwrap())?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn transform(scale: f64) -> String {
    format!("scale({})", scale)
}
