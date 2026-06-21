This project defines a crate which is a solution to a the following scenario:
- You want a user in Python to construct an instance of a serde for Python data
- You want this serde to be used in Rust without any unnecessary Python overhead
- You have a need for speed

This is a sticky problem - we want a trait object to be able to call the serde functions in Rust code for ergonomics, but a Rust trait object with a lifetime specifier can't be used as a Python class. The PyAnySerde trait defined by this crate defines serde functions without use of the 'py lifetime, and defines the PyAnySerdeType pyclass which can be used from Python to declare serdes that will be converted into Box<dyn PyAnySerde> when crossing the boundary into Rust. This crate defines serde structs implementing PyAnySerde for all common Python data types, including:
- Lists, sets (of a type serializable using a PyAnySerde impl)
- Dictionaries (with keys and values of types that are serializable using PyAnySerde impls)
- Numpy arrays (with dtypes int8-int64, uint8-uint64, float32, float64)
- Typed Dictionaries (with string keys and values with individually defined types serializable using PyAnySerde impls)
- Tuples (with items with individually defined types serializable using PyAnySerde impls)
- Options (with a value serializable using a PyAnySerde impl)
- Pickleable data (just relies on pickle internally to do serialization/deserialization - slow but generic)
- Basic types (bytes, complex, float, int, string)
- Dynamic (uses exact instance checks internally, uses a basic type or numpy array serde if it can but will fall back to pickling)
- Dataclass (essentially equivalent to a typed dictionary, but with configuration for constructor parameters)
- Custom (see below)

Unfortunately, custom implementations of the PyAnySerde trait cannot be declared from Python using the PyAnySerdeType class because the complex enum defining PyAnySerdeType is hard-coded into this crate. The intended workaround is to either:
1. Provide a custom method to pre/post process your data to/from numpy and use the numpy array serde (which has optional pre/post processors)
2. Implement the PythonSerde class (see `python/pyany_serde/python_serde.py`) - there is a PythonSerde PyAnySerdeType which will use your custom PythonSerde. Unfortunately this means there is an additional layer of Python indirection, even if you implement your PythonSerde in Rust.

The PyAnySerde trait includes methods for serializing directly into a memory buffer or for serializing to a Vec<u8> and returning it. For pure speed, serializing directly into a memory buffer should be preferred; however this is not always possible due to alignment constraints or whatever.

The PyAnySerdeType class and the other Python classes defined by this crate are fully pickleable and usable with pydantic, which is a nice bonus.

In order to use this crate, add it to your dependencies with `cargo add pyany-serde` and then expose its classes when defining your Python module. View `src/tests/common.rs#run_python_test_file` for an example of what it looks like to export the classes defined by this crate into a module. Note in particular that it is necessary to set the `__module__` attribute manually because this value depends on whatever module you are defining in your code. It is recommended to expose this crate's classes in a submodule so that you can directly copy the stubs and python_serde class from `python/pyany_serde` into your code base.

The dependencies in the pyproject.toml (and uv.lock) are for testing. Test by running `uv sync` to create the virtual environment and then run `uv run cargo test` to run the tests. The test files are largely Python code, which can be viewed in `python/tests` for examples of what it looks like to use the PyAnySerdeType class from Python.
