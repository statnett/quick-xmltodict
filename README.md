# quick-xmltodict

Efficient XML-to-dict conversion backed by Rust.

```python
>>> from quick_xmltodict import parse

>>> xml = """
... <movies>
...     <movie>
...         <title>Her</title>
...         <director>Spike Jonze</director>
...         <year>2013</year>
...         <genre>Science Fiction, Drama, Romance</genre>
...     </movie>
...     <movie>
...         <title>Encanto</title>
...         <director>Byron Howard, Jared Bush</director>
...         <year>2021</year>
...         <genre>Animation, Family, Fantasy</genre>
...     </movie>
... </movies>
... """

>>> parse(xml)

{'movies': {'movie': [{'director': 'Spike Jonze',
                       'genre': 'Science Fiction, Drama, Romance',
                       'title': 'Her',
                       'year': '2013'},
                      {'director': 'Byron Howard, Jared Bush',
                       'genre': 'Animation, Family, Fantasy',
                       'title': 'Encanto',
                       'year': '2021'}]}}
```

## Features

`quick-xmltodict` is a Rust-backed XML-to-dict conversion package designed to be fast and efficient.
It has a single function, `parse`, that takes an XML string and returns a Python dictionary.
You should be able to use this function as a drop-in replacement for the `xmltodict.parse` function from the original `xmltodict` package (used without any extra arguments).
Like `xmltodict`, `quick-xmltodict` follows [this](https://www.xml.com/pub/a/2006/05/31/converting-between-xml-and-json.html) schema for converting XML to JSON.

`quick-xmltodict` currently does not support namespace expansion, or the reverse operation (dict-to-XML conversion). For these features, use the original `xmltodict` package.

## Performance

Since `xmltodict` uses the non-validating C-based [expat](https://docs.python.org/3/library/pyexpat.html) parser from Python's standard library, it is already very fast.
`quick-xmltodict` is nonetheless about 2-5 times faster than `xmltodict`.

## Contributing

PRs are very welcome! Please make sure to run the tests before submitting a PR.

## Development

This project uses [uv](https://docs.astral.sh/uv/) to manage the environment and Python dependencies,
so you'll need to have it installed in addition to Python and Rust.

To install the development environment and run the test suite:
```bash
uv sync
uv run maturin develop
uv run pytest
```

Be sure to run `uv run maturin develop --uv` after making changes to the Rust code.
Add the `-r` flag for a release build (for example, if you want to run benchmarks).

It's recommended to install the pre-commit hooks:
```bash
uv run pre-commit install
```

This ensures that linting and formatting are run automatically on every commit.
