use pyo3::prelude::*;
use anyhow::Result;


pub fn get_latest_64p_game_info() -> Result<(String, String)> {
    //Python interpreter
    pyo3::prepare_freethreaded_python();
    
    let gil = Python::acquire_gil();
    let py = gil.python();

    let module = PyModule::from_code(py, include_str!("get_np_games.py"), "np_games.py", "np_games")?;
    let function = module.getattr("get_latest_64p_game_info")?;
    let result = function.call0()?;
    let result = result.extract::<(String, String)>()?;

    Ok(result)
}