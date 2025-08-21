"""Simple benchmarking script.

Important! Remember to build the Rust extension with the -r flag to enable release mode.
Without this, the Rust extension will be compiled in debug mode, which is significantly slower.

uv run maturin develop --uv -r
"""

import timeit
from pathlib import Path

from quick_xmltodict import parse as quickparse  # noqa: F401
from xmltodict import parse as pyparse  # noqa: F401

DATA_DIR = Path(__file__).parent / "tests/data"

DATA = {
    "simple": (DATA_DIR / "simple.xml").read_text(),
    "time_series": (DATA_DIR / "time-series.xml").read_text(),
    "forecast": (DATA_DIR / "forecast.xml").read_text(),
    "eic_codes": (DATA_DIR / "eic-codes.xml").read_text(),
}

DATA_BYTES = {
    "simple": (DATA_DIR / "simple.xml").read_bytes(),
    "time_series": (DATA_DIR / "time-series.xml").read_bytes(),
    "forecast": (DATA_DIR / "forecast.xml").read_bytes(),
    "eic_codes": (DATA_DIR / "eic-codes.xml").read_bytes(),
}

results = {}
for name, xml in DATA.items():  # noqa: B007, PERF102
    print(f"Running benchmarks for {name}...")
    quick_time = timeit.timeit("quickparse(xml)", globals=globals(), number=3)
    py_time = timeit.timeit("pyparse(xml)", globals=globals(), number=3)
    results[name] = {"quick_xmltodict": quick_time, "xmltodict": py_time, "ratio": py_time / quick_time}

print("Relative performance of quick-xmltodict vs xmltodict:")
for name, result in results.items():
    print(f"{name}: {result['ratio']:.2f}")

results = {}
for name, xml in DATA_BYTES.items():  # noqa: B007, PERF102
    quick_time = timeit.timeit("quickparse(xml)", globals=globals(), number=10)
    conv_time = timeit.timeit("quickparse(xml.decode())", globals=globals(), number=10)
    results[name] = {
        "quick_xmltodict": quick_time,
        "quick_xmltodict_with_decode": py_time,
        "ratio": conv_time / quick_time,
    }

print("Relative performance of UTF-8 validation in Rust vs decode() overhead in Python:")
for name, result in results.items():
    print(f"{name}: {result['ratio']:.2f}")
