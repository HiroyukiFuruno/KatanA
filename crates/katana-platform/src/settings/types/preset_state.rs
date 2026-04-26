use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PresetSource {
    BuiltIn,
    User,
    Custom,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PresetReference {
    pub source: PresetSource,
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PresetState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current: Option<PresetReference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<PresetReference>,
    #[serde(default)]
    pub modified: bool,
    #[serde(default)]
    pub user_presets: Vec<PresetReference>,
}

impl PresetReference {
    pub fn built_in(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            source: PresetSource::BuiltIn,
            id: id.into(),
            label: label.into(),
        }
    }

    pub fn user(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            source: PresetSource::User,
            id: name.clone(),
            label: name,
        }
    }

    pub fn custom(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            source: PresetSource::Custom,
            id: name.clone(),
            label: name,
        }
    }
}

impl PresetState {
    pub fn built_in(id: impl Into<String>, label: impl Into<String>) -> Self {
        let reference = PresetReference::built_in(id, label);
        Self {
            current: Some(reference.clone()),
            base: Some(reference),
            modified: false,
            user_presets: Vec::new(),
        }
    }

    pub fn user(name: impl Into<String>) -> Self {
        let reference = PresetReference::user(name);
        Self {
            current: Some(reference.clone()),
            base: Some(reference),
            modified: false,
            user_presets: Vec::new(),
        }
    }

    pub fn with_modified(mut self, modified: bool) -> Self {
        self.modified = modified;
        self
    }

    pub fn with_user_presets(mut self, user_presets: Vec<PresetReference>) -> Self {
        self.user_presets = user_presets;
        self
    }
}
