use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use minijinja::Value;
use minijinja::value::{Enumerator, Object};

#[derive(Debug)]
pub struct DynamicEnvironment {
    vars: HashMap<&'static str, String>,
    keys: &'static [&'static str],
}

impl DynamicEnvironment {
    pub fn new() -> Self {
        let vars: HashMap<&'static str, String> = env::vars_os()
            .filter_map(|(k, v)| {
                if let (Ok(k), Ok(v)) = (k.into_string(), v.into_string()) {
                    let k: &'static str = k.leak();
                    Some((k, v))
                } else {
                    None
                }
            })
            .collect();
        let keys: Vec<&'static str> = vars.keys().map(|k| *k).collect();

        DynamicEnvironment {
            vars,
            keys: keys.leak(),
        }
    }
}

impl Object for DynamicEnvironment {
    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        self.vars.get(key.as_str()?).map(Value::from)
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::Str(self.keys)
    }
}
