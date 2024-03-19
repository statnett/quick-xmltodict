use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyNone};
use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::reader::Reader;

trait QNameExt {
    fn qn(&self) -> String;
}

impl QNameExt for QName<'_> {
    /// Returns the qualified name of the element (prefix:local_name).
    fn qn(&self) -> String {
        let mut name = std::str::from_utf8(self.local_name().as_ref()).unwrap().to_string();
        if let Some(prefix) = self.prefix() {
            name = format!("{}:{}", std::str::from_utf8(prefix.as_ref()).unwrap(), name);
        }
        name
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

fn _update_dict<'a>(py: Python<'a>, d: &'a PyDict, tag_name: &str, value: &'a PyObject) {
    if d.contains(tag_name).unwrap() {
        let existing_val = d.get_item(tag_name).unwrap();
        let list: &PyList;
        if existing_val.unwrap().is_instance_of::<PyList>() {
            list = existing_val.unwrap().downcast::<PyList>().unwrap();
        } else {
            list = PyList::new(py, existing_val);
        }

        list.append(value).unwrap();
        d.set_item(tag_name, list).unwrap();
    } else {
        d.set_item(tag_name, value).unwrap();
    }
}

fn _parse<'a>(py: Python<'a>, xml: &'a str) -> &'a PyDict {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let d = PyDict::new(py);
    loop {
        match reader.read_event() {
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            Ok(Event::Eof) => break,
            Ok(Event::Empty(e)) => {
                let tag_name = e.name().qn();

                let value: Value;
                if e.attributes().count() == 0 {
                    value = Value::None;
                } else {
                    let attrs = PyDict::new(py);
                    for attr in e.attributes() {
                        let attr = attr.unwrap();
                        attrs
                            .set_item(format!("@{}", attr.key.qn()), attr.unescape_value().unwrap())
                            .unwrap();
                    }
                    value = Value::Dict(attrs);
                }
                _update_dict(py, d, &tag_name, &value.to_object(py));
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap();
                d.set_item("#text".to_string(), text).unwrap();
            }
            Ok(Event::Start(e)) => {
                let tag_name = e.name().qn();

                let mut value = Value::Dict(PyDict::new(py));
                if e.attributes().count() > 0 {
                    for attr in e.attributes() {
                        let attr = attr.unwrap();
                        match value {
                            Value::Dict(d) => {
                                d.set_item(format!("@{}", attr.key.qn()), attr.unescape_value().unwrap())
                                    .unwrap();
                            }
                            _ => unreachable!(),
                        }
                    }
                }

                let content = reader.read_text(e.name()).unwrap();
                match value {
                    Value::Dict(d) => {
                        d.update(_parse(py, &content).as_mapping()).unwrap();
                    }
                    _ => unreachable!(),
                }

                match value {
                    Value::Dict(d) if d.len() == 1 && d.contains("#text").unwrap() => {
                        value = Value::Text(d.get_item("#text").unwrap().unwrap().extract::<String>().unwrap());
                    }
                    _ => (),
                }

                _update_dict(py, d, &tag_name, &value.to_object(py));
            }
            _ => (),
        }
    }

    d
}

#[pyfunction]
fn parse(py: Python, xml: &str) -> PyResult<PyObject> {
    let d = _parse(py, xml);
    Ok(d.into())
}

#[pymodule]
fn quick_xmltodict(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    Ok(())
}
