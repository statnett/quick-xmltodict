use anyhow::Result;
use pyo3::types::{PyBytes, PyNone, PyString};
use pyo3::{prelude::*, IntoPyObjectExt};

use quick_xml::name::QName;

use pyo3::exceptions::PyUnicodeDecodeError;
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
pub enum Value {
    None,
    Text(String),
    Mapping(JsonMapping),
    List(Vec<Value>),
}

impl<'py> IntoPyObject<'py> for &Value {
    type Target = pyo3::types::PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            Value::None => Ok(PyNone::get(py).to_owned().into_any()),
            Value::Text(s) => Ok(s.into_pyobject(py)?.into_any()),
            Value::Mapping(m) => Ok(m.into_pyobject(py)?.into_any()),
            Value::List(l) => Ok(l.into_pyobject(py)?.into_any()),
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
                let old_value = std::mem::replace(e.get_mut(), Value::List(Vec::with_capacity(2)));
                if let Value::List(l) = e.into_mut() {
                    l.push(old_value);
                    l.push(value);
                }
            }
        },
    }

    Ok(())
}

pub fn _parse(xml: &str) -> Result<JsonMapping> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut mapping: JsonMapping = HashMap::with_capacity(1);
    loop {
        match reader.read_event() {
            Err(e) => return Err(e.into()),
            Ok(Event::Eof) => break,
            Ok(Event::Empty(e)) => {
                let value: Value;
                if e.attributes().count() == 0 {
                    value = Value::None;
                } else {
                    let mut attrs: JsonMapping = HashMap::with_capacity(e.attributes().count());
                    for attr in e.attributes() {
                        let attr = attr?;
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
                let mut sub_xml_mapping = Value::Mapping(HashMap::with_capacity(1));
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

#[pyfunction(signature = (xml), text_signature = "(xml: str | bytes)")]
fn parse(py: Python<'_>, xml: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    let xml_str: &str = if let Ok(s) = xml.cast::<PyString>() {
        s.to_str()?
    } else if let Ok(b) = xml.cast::<PyBytes>() {
        let bytes = b.as_bytes();
        std::str::from_utf8(bytes).map_err(|e| PyUnicodeDecodeError::new_err_from_utf8(py, bytes, e))?
    } else {
        return Err(pyo3::exceptions::PyTypeError::new_err("Expected str or UTF-8 bytes"));
    };
    _parse(xml_str)?.into_py_any(py)
}

#[pymodule]
fn quick_xmltodict(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    Ok(())
}
