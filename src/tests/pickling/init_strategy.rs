use pyo3::{prelude::*, types::PyDict};

use crate::{pyany_serde_impl::InitStrategy, tests::run_python_test_file};

#[pyfunction]
fn validate_eq<'py>(actual: InitStrategy, expected: InitStrategy) -> PyResult<()> {
    match actual.clone() {
        InitStrategy::ALL {} => {
            let InitStrategy::ALL {} = expected.clone() else {
                panic!("Expected {actual} to be InitStrategy::ALL {{}}");
            };
        }
        InitStrategy::SOME {
            kwargs: actual_kwargs,
        } => {
            let InitStrategy::SOME {
                kwargs: expected_kwargs,
            } = expected.clone()
            else {
                panic!("Expected {actual} to be InitStrategy::SOME {{..}}");
            };
            assert_eq!(actual_kwargs, expected_kwargs);
        }
        InitStrategy::NONE {} => {
            let InitStrategy::NONE {} = expected.clone() else {
                panic!("Expected {actual} to be InitStrategy::NONE {{}}");
            };
        }
    };
    Ok(())
}

fn tests_submod<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyModule>> {
    let sub = PyModule::new(py, "pickling_init_strategy_tests")?;
    sub.add_function(wrap_pyfunction!(validate_eq, py)?)?;
    Ok(sub)
}

#[test]
fn run_pickling_tests() -> PyResult<()> {
    println!("VIRTUAL_ENV: {}", std::env::var("VIRTUAL_ENV").unwrap());
    println!("PATH: {}", std::env::var("PATH").unwrap());
    Python::initialize();
    Python::attach(|py| {
        let os = py.import("os")?;
        let environ = os.getattr("environ")?;
        let environ = environ.cast::<PyDict>()?;

        println!(
            "PYTHONHOME={:?}",
            environ
                .get_item("PYTHONHOME")?
                .map(|v| v.extract::<String>())
                .transpose()?
        );
        println!(
            "PYTHONPATH={:?}",
            environ
                .get_item("PYTHONPATH")?
                .map(|v| v.extract::<String>())
                .transpose()?
        );

        let sys = py.import("sys").unwrap();

        println!(
            "sys.executable = {:?}",
            sys.getattr("executable")
                .unwrap()
                .extract::<String>()
                .unwrap()
        );
        println!(
            "sys.prefix = {:?}",
            sys.getattr("prefix").unwrap().extract::<String>().unwrap()
        );
        println!(
            "sys.base_prefix = {:?}",
            sys.getattr("base_prefix")
                .unwrap()
                .extract::<String>()
                .unwrap()
        );
        println!(
            "sys.path = {:?}",
            sys.getattr("path").unwrap().repr().unwrap()
        );
        Ok::<_, PyErr>(())
    })?;
    Python::attach(|py| {
        run_python_test_file(
            py,
            "python/tests/pickling/init_strategy.py",
            tests_submod(py)?,
        )
    })
}
