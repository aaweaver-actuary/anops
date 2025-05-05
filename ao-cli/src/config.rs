use serde::Deserialize;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::collections::HashMap; // Added HashMap

/// Represents the overall configuration loaded from ao.toml
#[derive(Deserialize, Debug, PartialEq, Default)]
pub struct Config {
    pub project: ProjectConfig,
    #[serde(default)]
    pub check: CheckConfig,
    #[serde(default)] // Use default HashMap if missing
    pub tasks: HashMap<String, Vec<String>>,
}

/// Represents the [project] table in ao.toml
#[derive(Deserialize, Debug, PartialEq, Default)] // Added Default
pub struct ProjectConfig {
    pub name: String,
}

/// Represents the [check] table in ao.toml
#[derive(Deserialize, Debug, PartialEq, Default)]
pub struct CheckConfig {
    #[serde(default)]
    pub linters: Vec<String>,
    #[serde(default)]
    pub testers: Vec<String>,
}


/// Loads the configuration from the ao.toml file in the project root.
///
/// # Arguments
///
/// * `project_root` - The path to the project root directory (containing ao.toml).
///
/// # Errors
///
/// Returns an error if the config file cannot be read or parsed.
pub fn load_config(project_root: &Path) -> Result<Config> {
    let config_path = project_root.join("ao.toml");
    println!("Loading config from: {:?}", config_path);

    if !config_path.exists() {
        anyhow::bail!("Configuration file not found: {}", config_path.display());
    }

    let config_content = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

    let config: Config = toml::from_str(&config_content)
        .with_context(|| format!("Failed to parse TOML config file: {}", config_path.display()))?;

    println!("Config loaded successfully: {:?}", config);
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    use std::path::PathBuf;

    // Helper to create a dummy ao.toml
    fn create_dummy_config(dir: &Path, content: &str) -> PathBuf {
        let config_path = dir.join("ao.toml");
        fs::write(&config_path, content).unwrap();
        config_path
    }

    #[test]
    fn load_config_succeeds_with_valid_file_no_extras() {
        let tmp_dir = tempdir().unwrap();
        let project_name = "test-project";
        let config_content = format!("[project]\nname = \"{}\"", project_name);
        create_dummy_config(tmp_dir.path(), &config_content);

        let config = load_config(tmp_dir.path()).unwrap();

        assert_eq!(config.project.name, project_name);
        assert_eq!(config.check, CheckConfig::default());
        assert!(config.tasks.is_empty()); // Default tasks is empty map
    }

    #[test]
    fn load_config_succeeds_with_tasks_section() {
        let tmp_dir = tempdir().unwrap();
        let project_name = "test-project-with-tasks";
        let config_content = format!("[project]\nname = \"{}\"\n\n[tasks]\nbuild = [\"echo building...\", \"mkdir dist\"]\ndeploy = [\"echo deploying...\"]", project_name);
        create_dummy_config(tmp_dir.path(), &config_content);

        let config = load_config(tmp_dir.path()).unwrap();

        assert_eq!(config.project.name, project_name);
        assert_eq!(config.check, CheckConfig::default());
        assert_eq!(config.tasks.len(), 2);
        assert_eq!(config.tasks.get("build").unwrap(), &vec!["echo building...", "mkdir dist"]);
        assert_eq!(config.tasks.get("deploy").unwrap(), &vec!["echo deploying..."]);
    }

    #[test]
    fn load_config_succeeds_with_empty_tasks_section() {
        let tmp_dir = tempdir().unwrap();
        let project_name = "test-project-empty-tasks-section";
        let config_content = format!("[project]\nname = \"{}\"\n\n[tasks] # Empty tasks table", project_name);
        create_dummy_config(tmp_dir.path(), &config_content);

        let config = load_config(tmp_dir.path()).unwrap();

        assert_eq!(config.project.name, project_name);
        assert!(config.tasks.is_empty());
    }

    #[test]
    fn load_config_succeeds_with_all_sections() {
        let tmp_dir = tempdir().unwrap();
        let project_name = "test-project-all-sections";
        let config_content = format!("[project]\nname = \"{}\"\n\n[check]\nlinters = [\"lint1\"]\n\n[tasks]\nbuild = [\"build1\"]", project_name);
        create_dummy_config(tmp_dir.path(), &config_content);

        let config = load_config(tmp_dir.path()).unwrap();

        assert_eq!(config.project.name, project_name);
        assert_eq!(config.check.linters, vec!["lint1"]);
        assert!(config.check.testers.is_empty());
        assert_eq!(config.tasks.len(), 1);
        assert_eq!(config.tasks.get("build").unwrap(), &vec!["build1"]);
    }

    #[test]
    fn load_config_fails_with_malformed_tasks() {
        let tmp_dir = tempdir().unwrap();
        let project_name = "test_project";
        // Task steps should be an array of strings
        let config_content = format!("[project]\nname = \"{}\"\n\n[tasks]\nbuild = \"not-an-array\"", project_name);
        create_dummy_config(tmp_dir.path(), &config_content);
        let result = load_config(tmp_dir.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid type") && err.contains("not-an-array"));
    }

    #[test]
    fn load_config_fails_with_malformed_task_steps() {
        let tmp_dir = tempdir().unwrap();
        let project_name = "test_project";
        let config_content = format!("[project]\nname = \"{}\"\n\n[tasks]\nbuild = [1, 2, 3] # Numbers instead of strings", project_name);
        create_dummy_config(tmp_dir.path(), &config_content);
        let result = load_config(tmp_dir.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid type") && err.contains("integer"));
    }

    #[test]
    fn load_config_fails_if_file_missing() {
        let tmp_dir = tempdir().unwrap();
        let result = load_config(tmp_dir.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Configuration file not found") || err.contains("No such file"));
    }

    #[test]
    fn load_config_fails_if_file_malformed() {
        let tmp_dir = tempdir().unwrap();
        let malformed_content = "[project]name=";
        create_dummy_config(tmp_dir.path(), malformed_content);
        let result = load_config(tmp_dir.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Failed to parse TOML config file") || err.contains("expected"));
    }

    #[test]
    fn load_config_fails_if_missing_project_table() {
        let tmp_dir = tempdir().unwrap();
        let content = "[tasks]\nbuild=['a']"; // Missing [project]
        create_dummy_config(tmp_dir.path(), content);
        let result = load_config(tmp_dir.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("missing field") && err.contains("project"));
    }

    #[test]
    fn load_config_fails_if_missing_project_name() {
        let tmp_dir = tempdir().unwrap();
        let content = "[project]\n# name intentionally missing";
        create_dummy_config(tmp_dir.path(), content);
        let result = load_config(tmp_dir.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("missing field") && err.contains("name"));
    }

    #[test]
    fn load_config_handles_incorrect_types_in_check() {
        let tmp_dir = tempdir().unwrap();
        let project_name = "test_project";
        let config_content = format!("[project]\nname = \"{}\"\n\n[check]\nlinters = \"not-an-array\" # Incorrect type", project_name);
        create_dummy_config(tmp_dir.path(), &config_content);
        let result = load_config(tmp_dir.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid type") && err.contains("not-an-array"));
    }
}
