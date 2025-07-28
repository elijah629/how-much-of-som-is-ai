#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn predict(devlog: &str) -> JsValue {
    let prediction = sonai::predict(devlog);

    serde_wasm_bindgen::to_value(&prediction).unwrap()
}
