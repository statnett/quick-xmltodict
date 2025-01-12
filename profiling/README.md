This profiling directory contains a binary crate that is used to profile the performance of the Rust code.
The profiling is done using the `perf` tool on Linux. On Ubuntu, `perf` can be installed with:

```bash
apt-get install linux-tools-common linux-tools-generic linux-tools-`uname -r`
```

On WSL, this will fail because there is no version of `perf` that is compatible with the WSL kernel (output of `uname -r`).
Instead, install only the generic version of `perf`:

```bash
apt-get install linux-tools-common linux-tools-generic
```

This generic version is found at some path like `/usr/lib/linux-tools/5.15.0-101-generic/perf`.
When using flamegraph, the `PERF` environment variable must be set to this path.

```bash
export PERF=/usr/lib/linux-tools/5.15.0-101-generic/perf
```

Flamegraph can be installed with:
```bash
cargo install flamegraph
```

And run with:
```bash
uv run cargo flamegraph --bin main
```

This will generate a `flamegraph.svg` file, which can be opened in a browser to visualize the performance of the code.

If you see something like "error while loading shared libraries: libpython3.12.so.1.0: cannot open shared object file: No such file or directory", you need to tell the executable where to find libpython, which can be done by adding the containing directory to the `LD_LIBRARY_PATH` environment variable.
For example:

```bash
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/usr/local/lib
```
