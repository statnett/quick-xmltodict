import json
from pathlib import Path
from xml.parsers.expat import ExpatError

import pytest
from quick_xmltodict import parse as rustparse
from xmltodict import parse as pyparse

pytestmark = pytest.mark.parametrize("parse", [pyparse, rustparse])


def test_empty(parse):
    xml = "<a/>"
    target = {"a": None}
    assert parse(xml) == target


def test_empty_with_attributes(parse):
    xml = '<e name="value" />'
    target = {"e": {"@name": "value"}}
    assert parse(xml) == target


def test_multiple_empty(parse):
    xml = """
        <root>
            <a a_attr_1="a_value_1" />
            <b/>
            <c c_attr_1="c_value_1" c_attr_2="c_value_2" />
        </root>
    """
    target = {
        "root": {
            "a": {"@a_attr_1": "a_value_1"},
            "b": None,
            "c": {"@c_attr_1": "c_value_1", "@c_attr_2": "c_value_2"},
        },
    }
    assert parse(xml) == target


def test_text(parse):
    xml = "<a>text</a>"
    target = {"a": "text"}
    assert parse(xml) == target


def test_text_whitespace(parse):
    xml = "<a>  text  </a>"
    target = {"a": "text"}
    assert parse(xml) == target


def test_text_line_breaks(parse):
    xml = "<a>\ntext\n</a>"
    target = {"a": "text"}
    assert parse(xml) == target


def test_text_and_attributes(parse):
    xml = '<e name="value">text</e>'
    target = {"e": {"@name": "value", "#text": "text"}}
    assert parse(xml) == target


def test_with_children(parse):
    xml = "<e><a>1</a><b>2</b></e>"
    target = {"e": {"a": "1", "b": "2"}}
    assert parse(xml) == target


def test_identically_named_children(parse):
    xml = "<e><a>1</a><a>2</a></e>"
    target = {"e": {"a": ["1", "2"]}}
    assert parse(xml) == target


def test_more_identically_named_children(parse):
    xml = "<e><a>1</a><a>2</a><a>3</a></e>"
    target = {"e": {"a": ["1", "2", "3"]}}
    assert parse(xml) == target


def test_mixed_type_identically_named_children(parse):
    xml = """
        <e>
            <a/>
            <a attr="attr-1" />
            <a>2</a>
            <a attr="attr-3">3</a>
        </e>
    """
    target = {"e": {"a": [None, {"@attr": "attr-1"}, "2", {"@attr": "attr-3", "#text": "3"}]}}
    assert parse(xml) == target


def test_elements_and_text(parse):
    xml = "<e>1<a>2</a></e>"
    target = {"e": {"#text": "1", "a": "2"}}
    assert parse(xml) == target


def test_namespace_prefixed(parse):
    xml = """
        <a xmlns:ns="http://example.com">
            <ns:b>text</ns:b>
        </a>"""
    target = {"a": {"@xmlns:ns": "http://example.com", "ns:b": "text"}}
    assert parse(xml) == target


def test_namespace_prefixed_attr(parse):
    xml = """
        <a xmlns:ns="http://example.com">
            <ns:b ns:name="value">text</ns:b>
        </a>"""
    target = {"a": {"@xmlns:ns": "http://example.com", "ns:b": {"@ns:name": "value", "#text": "text"}}}
    assert parse(xml) == target


def test_error_missing_closing_tag(parse):
    with pytest.raises((RuntimeError, ExpatError)):
        parse("<a>")


def test_error_missing_opening_tag(parse):
    with pytest.raises((RuntimeError, ExpatError)):
        parse("</a>")


def test_error_malformed_tag(parse):
    with pytest.raises((RuntimeError, ExpatError)):
        parse("<a")


@pytest.fixture
def data_dir():
    return Path(__file__).parent / "data"


@pytest.fixture
def forecast_xml(data_dir):
    return (data_dir / "forecast.xml").read_text()


@pytest.fixture
def forecast_target(data_dir):
    return json.loads((data_dir / "forecast.json").read_text())


def test_forecast(parse, forecast_xml, forecast_target):
    assert parse(forecast_xml) == forecast_target


@pytest.fixture
def simple_xml(data_dir):
    return (data_dir / "simple.xml").read_text()


@pytest.fixture
def simple_target(data_dir):
    return json.loads((data_dir / "simple.json").read_text())


def test_simple(parse, simple_xml, simple_target):
    assert parse(simple_xml) == simple_target


@pytest.fixture
def time_series_xml(data_dir):
    return (data_dir / "time-series.xml").read_text()


@pytest.fixture
def time_series_target(data_dir):
    return json.loads((data_dir / "time-series.json").read_text())


def test_time_series(parse, time_series_xml, time_series_target):
    assert parse(time_series_xml) == time_series_target
