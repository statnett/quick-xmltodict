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

`quick-xmltodict` is a Rust-backed XML-to-dict conversion package that is designed to be fast and efficient.
It has a single function, `parse`, that takes an XML string and returns a Python dictionary.
You should be able to use this function as a drop-in replacement for the `xmltodict.parse` function from the original `xmltodict` package (used without any extra arguments).
Like `xmltodict`, `quick-xmltodict` follows [this](https://www.xml.com/pub/a/2006/05/31/converting-between-xml-and-json.html) schema for converting XML to JSON.

`quick-xmltodict` currently does not support namespace expansion, or the reverse operation (dict-to-XML conversion). For these features, use the original `xmltodict` package.

## Performance

`quick-xmltodict` is currently about 2-5 times faster than `xmltodict`.
There are performance improvements to be made, so this difference is expected to increase.

## Contributing

PRs are very welcome! Please make sure to run the tests before submitting a PR.

## Development

This project uses [Poetry](https://python-poetry.org/) to manage the environment and Python dependencies,
so you'll need to have it installed in addition to Python and Rust.

To install the development environment and run the test suite:
```bash
poetry install
poetry run maturin develop
poetry run pytest
```

Be sure to run `poetry run maturin develop` after making changes to the Rust code.
Add the `-r` flag for a release build (for example if you want to run benchmarks).

It's recommended to install the pre-commit hooks:
```bash
poetry run pre-commit install
```

This ensures that linting and formatting is run automatically on every commit.
