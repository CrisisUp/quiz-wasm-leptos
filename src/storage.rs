use serde::{de::DeserializeOwned, Serialize};
use web_sys::window;

pub fn salvar<T: Serialize>(chave: &str, valor: &T) {
    if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
        if let Ok(json) = serde_json::to_string(valor) {
            let _ = storage.set_item(chave, &json);
        }
    }
}

pub fn carregar<T: DeserializeOwned>(chave: &str) -> Option<T> {
    let storage = window().and_then(|w| w.local_storage().ok().flatten())?;
    let raw = storage.get_item(chave).ok().flatten()?;
    serde_json::from_str(&raw).ok()
}
