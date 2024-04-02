use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::PyNone;

use quick_xml::name::QName;

use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;

trait QNameExt {
    fn qn(&self) -> Result<String>;
}

impl QNameExt for QName<'_> {
    /// Returns the qualified name of the element (prefix:local_name).
    fn qn(&self) -> Result<String> {
        Ok(match self.prefix() {
            None => std::str::from_utf8(self.local_name().as_ref())?.to_string(),
            Some(prefix) => {
                std::str::from_utf8(prefix.as_ref())?.to_string()
                    + ":"
                    + std::str::from_utf8(self.local_name().as_ref())?
            }
        })
    }
}

type JsonMapping = HashMap<String, Value>;

#[derive(Debug)]
enum Value {
    None,
    Text(String),
    Mapping(JsonMapping),
    List(Vec<Value>),
}

impl ToPyObject for Value {
    fn to_object(&self, py: Python) -> PyObject {
        match self {
            Value::None => PyNone::get_bound(py).to_object(py),
            Value::Text(s) => s.to_object(py),
            Value::Mapping(m) => m.to_object(py),
            Value::List(l) => l.to_object(py),
        }
    }
}

fn update_mapping(mapping: &mut JsonMapping, tag_name: String, value: Value) -> Result<()> {
    match mapping.entry(tag_name) {
        std::collections::hash_map::Entry::Vacant(e) => {
            e.insert(value);
        }
        std::collections::hash_map::Entry::Occupied(mut e) => match e.get_mut() {
            Value::List(l) => l.push(value),
            _ => {
                let old_value = std::mem::replace(e.get_mut(), Value::List(vec![]));
                if let Value::List(l) = e.into_mut() {
                    l.push(old_value);
                    l.push(value);
                }
            }
        },
    }

    Ok(())
}

fn _parse(xml: &str) -> Result<JsonMapping> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut mapping: JsonMapping = HashMap::new();
    loop {
        match reader.read_event() {
            Err(e) => return Err(e.into()),
            Ok(Event::Eof) => break,
            Ok(Event::Empty(e)) => {
                let value: Value;
                if e.attributes().count() == 0 {
                    value = Value::None;
                } else {
                    let mut attrs: JsonMapping = HashMap::new();
                    for attr in e.attributes() {
                        let attr = attr?;
                        // attrs.set_item(format!("@{}", attr.key.qn()?), attr.unescape_value()?)?;
                        attrs.insert(
                            "@".to_string() + &attr.key.qn()?,
                            Value::Text(attr.unescape_value()?.parse()?),
                        );
                    }
                    value = Value::Mapping(attrs);
                }
                update_mapping(&mut mapping, e.name().qn()?, value)?;
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape()?.to_string();
                mapping.insert("#text".to_string(), Value::Text(text));
            }
            Ok(Event::Start(e)) => {
                let mut sub_xml_mapping = Value::Mapping(HashMap::new());
                if e.attributes().count() > 0 {
                    for attr in e.attributes() {
                        let attr = attr?;
                        if let Value::Mapping(m) = &mut sub_xml_mapping {
                            m.insert(
                                "@".to_string() + &attr.key.qn()?,
                                Value::Text(attr.unescape_value()?.parse()?),
                            );
                        }
                    }
                }

                if let Value::Mapping(m) = &mut sub_xml_mapping {
                    m.extend(_parse(&(reader.read_text(e.name())?))?);
                }

                if let Value::Mapping(m) = &sub_xml_mapping {
                    if m.len() == 1 {
                        if let Some(text) = m.get("#text") {
                            if let Value::Text(text) = text {
                                sub_xml_mapping = Value::Text(text.clone());
                            }
                        }
                    }
                }

                update_mapping(&mut mapping, e.name().qn()?, sub_xml_mapping)?;
            }
            _ => (),
        }
    }

    Ok(mapping)
}

#[pyfunction]
fn parse(py: Python, xml: &str) -> PyResult<PyObject> {
    Ok(_parse(xml)?.to_object(py))
}

#[pymodule]
#[allow(deprecated)]
fn quick_xmltodict(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    Ok(())
}
