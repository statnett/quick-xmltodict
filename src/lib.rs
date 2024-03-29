use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyNone};
use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::reader::Reader;

trait QNameExt {
    fn qn(&self) -> Result<String>;
}

impl QNameExt for QName<'_> {
    /// Returns the qualified name of the element (prefix:local_name).
    fn qn(&self) -> Result<String> {
        let mut name = std::str::from_utf8(self.local_name().as_ref())?.to_string();
        if let Some(prefix) = self.prefix() {
            name = format!("{}:{}", std::str::from_utf8(prefix.as_ref())?, name);
        }
        Ok(name)
    }
}

#[derive(Debug)]
enum Value<'py> {
    None,
    Text(String),
    Dict(&'py PyDict),
}

impl ToPyObject for Value<'_> {
    fn to_object(&self, py: Python) -> PyObject {
        match self {
            Value::None => PyNone::get(py).into(),
            Value::Text(s) => s.to_object(py),
            Value::Dict(d) => d.to_object(py),
        }
    }
}

fn _update_dict<'a>(py: Python<'a>, d: &'a PyDict, tag_name: &str, value: &'a PyObject) -> Result<()> {
    match d.get_item(tag_name)? {
        None => {
            d.set_item(tag_name, value)?;
        }
        Some(existing_val) => {
            let list: &PyList;
            if existing_val.is_instance_of::<PyList>() {
                list = existing_val.extract::<&PyList>()?;
            } else {
                list = PyList::new(py, vec![existing_val]);
            }

            list.append(value)?;
            d.set_item(tag_name, list)?;
        }
    }
    Ok(())
}

fn _parse<'a>(py: Python<'a>, xml: &'a str) -> Result<&'a PyDict> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let d = PyDict::new(py);
    loop {
        match reader.read_event() {
            Err(e) => return Err(e.into()),
            Ok(Event::Eof) => break,
            Ok(Event::Empty(e)) => {
                let tag_name = e.name().qn()?;

                let value: Value;
                if e.attributes().count() == 0 {
                    value = Value::None;
                } else {
                    let attrs = PyDict::new(py);
                    for attr in e.attributes() {
                        let attr = attr?;
                        attrs.set_item(format!("@{}", attr.key.qn()?), attr.unescape_value()?)?;
                    }
                    value = Value::Dict(attrs);
                }
                _update_dict(py, d, &tag_name, &value.to_object(py))?;
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape()?;
                d.set_item("#text".to_string(), text)?;
            }
            Ok(Event::Start(e)) => {
                let tag_name = e.name().qn()?;

                let mut value = Value::Dict(PyDict::new(py));
                if e.attributes().count() > 0 {
                    for attr in e.attributes() {
                        let attr = attr?;
                        match value {
                            Value::Dict(d) => {
                                d.set_item(format!("@{}", attr.key.qn()?), attr.unescape_value()?)?;
                            }
                            _ => unreachable!(),
                        }
                    }
                }

                match value {
                    Value::Dict(d) => {
                        d.update(_parse(py, &(reader.read_text(e.name())?))?.as_mapping())?;
                    }
                    _ => unreachable!(),
                }

                match value {
                    Value::Dict(d) if d.len() == 1 => {
                        if let Some(text) = d.get_item("#text")? {
                            value = Value::Text(text.extract::<String>()?);
                        }
                    }
                    _ => (),
                }

                _update_dict(py, d, &tag_name, &value.to_object(py))?;
            }
            _ => (),
        }
    }

    Ok(d)
}

#[pyfunction]
fn parse(py: Python, xml: &str) -> PyResult<PyObject> {
    Ok(_parse(py, xml)?.into())
}

#[pymodule]
fn quick_xmltodict(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    Ok(())
}
