use serde_json::{Result, Value};

pub(crate) fn get_json_field(json_string: &str, field: &str) -> Result<Value> {
    let json: Value = serde_json::from_str(json_string)?;
    Ok(json[field].clone())
}