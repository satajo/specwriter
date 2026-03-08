use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub claude_command: String,
    pub model: Option<String>,
    pub spec_filename: String,
    pub web_search: bool,
    pub web_fetch: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            claude_command: "claude".into(),
            model: None,
            spec_filename: "SPEC.md".into(),
            web_search: false,
            web_fetch: false,
        }
    }
}

impl Settings {
    /// Load settings from a directory. Returns settings and an optional error message.
    /// If the file doesn't exist, returns defaults with no error.
    /// If the file is malformed, returns best-effort settings with an error message.
    pub fn load_from(dir: &Path) -> (Self, Option<String>) {
        let path = dir.join("settings.json");
        if !path.exists() {
            return (Self::default(), None);
        }
        match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<Settings>(&content) {
                Ok(settings) => (settings, None),
                Err(e) => (
                    Self::default(),
                    Some(format!("Error loading settings: {e}")),
                ),
            },
            Err(e) => (
                Self::default(),
                Some(format!("Error reading settings: {e}")),
            ),
        }
    }

    /// Save settings to a directory, creating parent dirs if needed.
    pub fn save_to(&self, dir: &Path) -> Result<(), String> {
        std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create config dir: {e}"))?;
        let path = dir.join("settings.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {e}"))?;
        std::fs::write(&path, json).map_err(|e| format!("Failed to write settings: {e}"))?;
        Ok(())
    }

    /// Default config directory: $XDG_CONFIG_HOME/specwriter or ~/.config/specwriter
    pub fn default_config_dir() -> PathBuf {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(xdg).join("specwriter")
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config").join("specwriter")
        } else {
            PathBuf::from(".config").join("specwriter")
        }
    }

    /// Number of settings fields
    pub const COUNT: usize = 5;

    /// Get the label for a settings field by index
    pub fn label(index: usize) -> &'static str {
        match index {
            0 => "Claude Command",
            1 => "Model",
            2 => "Spec Filename",
            3 => "WebSearch",
            4 => "WebFetch",
            _ => "",
        }
    }

    /// Get the display value for a settings field by index
    pub fn display_value(&self, index: usize) -> String {
        match index {
            0 => self.claude_command.clone(),
            1 => self.model.as_deref().unwrap_or("(not set)").to_string(),
            2 => self.spec_filename.clone(),
            3 => if self.web_search { "on" } else { "off" }.to_string(),
            4 => if self.web_fetch { "on" } else { "off" }.to_string(),
            _ => String::new(),
        }
    }

    /// Whether a field is a boolean toggle (vs text edit)
    pub fn is_boolean(index: usize) -> bool {
        matches!(index, 3 | 4)
    }

    /// Get the current text value for editing (for text fields)
    pub fn edit_value(&self, index: usize) -> String {
        match index {
            0 => self.claude_command.clone(),
            1 => self.model.clone().unwrap_or_default(),
            2 => self.spec_filename.clone(),
            _ => String::new(),
        }
    }

    /// Set a text field value from the editor
    pub fn set_value(&mut self, index: usize, value: String) {
        match index {
            0 => {
                self.claude_command = if value.is_empty() {
                    "claude".into()
                } else {
                    value
                }
            }
            1 => self.model = if value.is_empty() { None } else { Some(value) },
            2 => {
                self.spec_filename = if value.is_empty() {
                    "SPEC.md".into()
                } else {
                    value
                }
            }
            _ => {}
        }
    }

    /// Toggle a boolean field
    pub fn toggle(&mut self, index: usize) {
        match index {
            3 => self.web_search = !self.web_search,
            4 => self.web_fetch = !self.web_fetch,
            _ => {}
        }
    }
}
