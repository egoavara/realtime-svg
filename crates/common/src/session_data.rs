use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{errors::ApiError, SvgFrame};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionData {
    pub template: String,
    pub args: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
}

impl SessionData {
    pub fn new(template: impl Into<String>, args: HashMap<String, serde_json::Value>) -> Self {
        Self {
            template: template.into(),
            args,
            owner: None,
        }
    }

    pub fn new_with_owner(
        template: impl Into<String>,
        args: HashMap<String, serde_json::Value>,
        owner: String,
    ) -> Self {
        Self {
            template: template.into(),
            args,
            owner: Some(owner),
        }
    }

    pub fn set_arg(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.args.insert(key.into(), value);
    }
    pub fn replace_args(&mut self, new_args: HashMap<String, serde_json::Value>) {
        self.args = new_args;
    }
    pub fn remove_arg(&mut self, key: &str) {
        self.args.remove(key);
    }

    pub fn clear_args(&mut self) {
        self.args.clear();
    }

    pub fn get_arg(&self, key: &str) -> Option<&serde_json::Value> {
        self.args.get(key)
    }

    pub fn get_all_args(&self) -> &HashMap<String, serde_json::Value> {
        &self.args
    }

    pub fn current_frame(&self) -> SvgFrame {
        let ctx =
            tera::Context::from_value(serde_json::to_value(self.args.clone()).unwrap()).unwrap();
        let rendered = tera::Tera::one_off(&self.template, &ctx, false)
            .unwrap_or_else(|_| self.template.clone());

        SvgFrame::new(rendered)
    }
}

impl TryFrom<&str> for SessionData {
    type Error = ApiError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(value).map_err(ApiError::from)
    }
}
