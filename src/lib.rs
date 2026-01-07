use std::f64::consts::PI;

// Rayonのプレリュード（並列化用ツール）をインポート
#[cfg(not(target_family = "wasm"))]
use rayon::prelude::*;

/// Core Algorithm
/// use_parallel: trueなら並列化（Nativeのみ有効）、falseなら直列
fn core_algorithm(iterations: u64, param: f64, use_parallel: bool) -> f64 {
    // 【Native (Python) かつ Parallel ON の場合】
    #[cfg(not(target_family = "wasm"))]
    if use_parallel {
        return (0..iterations)
            .into_par_iter() // <--- ここが魔法！ par_iterにするだけ
            .map(|i| {
                let x = (i as f64) * PI / 180.0;
                (x * param).sin() * (x * param).cos()
            })
            .sum();
    }

    // 【WASM または Parallel OFF の場合】
    (0..iterations)
        .into_iter() // 通常のイテレータ
        .map(|i| {
            let x = (i as f64) * PI / 180.0;
            (x * param).sin() * (x * param).cos()
        })
        .sum()
}

// -----------------------------------------------------------------------------
// Module: Python Interface (PyO3)
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pyfunction]
// 引数に parallel を追加
fn compute_metrics(iterations: u64, param: f64, parallel: bool) -> PyResult<f64> {
    Ok(core_algorithm(iterations, param, parallel))
}

#[cfg(feature = "python")]
#[pymodule]
fn nx_compute_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compute_metrics, m)?)?;
    Ok(())
}

// -----------------------------------------------------------------------------
// Module: WebAssembly Interface (wasm-bindgen)
// -----------------------------------------------------------------------------
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
// 引数に parallel を追加 (WASMでは内部で無視されるがIFは合わせる)
pub fn compute_metrics_js(iterations: u64, param: f64, parallel: bool) -> f64 {
    core_algorithm(iterations, param, parallel)
}