"""Simple benchmarking script.

Important! Remember to build the Rust extension with the -r flag to enable release mode.
Without this, the Rust extension will be compiled in debug mode, which is significantly slower.

poetry run maturin develop -r
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

results = {}
for name, xml in DATA.items():  # noqa: B007, PERF102
    print(f"Running benchmarks for {name}...")
    quick_time = timeit.timeit("quickparse(xml)", globals=globals(), number=3)
    py_time = timeit.timeit("pyparse(xml)", globals=globals(), number=3)
    results[name] = {"quick_xmltodict": quick_time, "xmltodict": py_time, "ratio": py_time / quick_time}

print("Relative performance:")
for name, result in results.items():
    print(f"{name}: {result['ratio']:.2f}")
