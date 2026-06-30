use std::fs;
use std::path::Path;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct ProfileSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_line_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_comments: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minify_json: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_only: Option<bool>,
}

impl Default for ProfileSettings {
    fn default() -> Self {
        Self {
            max_line_length: None,
            remove_comments: Some(false),
            minify_json: Some(false),
            json_only: Some(false),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct RegexFilterRule {
    pub pattern: String,
    pub action: String, // "strip" or "collapse"
}

/// The configuration structure loaded from global/local JSON files.
/// Includes a list of denied shell commands, custom Data Loss Prevention patterns, and savings profiles.
#[derive(Debug, Clone)]
pub struct UserConfig {
    /// Command regex patterns that should be denied/blocked.
    pub denied_commands: Vec<String>,
    /// Custom Data Loss Prevention regex patterns to scrub sensitive data.
    pub custom_dlp_patterns: Vec<String>,

    // P1 savings profiles
    pub output_profiles: std::collections::HashMap<String, ProfileSettings>,
    pub default_profile: String,
    pub overrides: std::collections::HashMap<String, String>,
    /// Custom config-driven regex filtering rules.
    pub regex_filters: Vec<RegexFilterRule>,
    /// When true, chained shell commands are blocked (exit 2) instead of passthrough.
    pub strict_chained: bool,
}

impl Default for UserConfig {
    fn default() -> Self {
        let mut output_profiles = std::collections::HashMap::new();
        output_profiles.insert(
            "strict".to_string(),
            ProfileSettings {
                max_line_length: Some(80),
                remove_comments: Some(true),
                minify_json: Some(true),
                json_only: Some(false),
            },
        );
        output_profiles.insert(
            "balanced".to_string(),
            ProfileSettings {
                max_line_length: Some(100),
                remove_comments: Some(true),
                minify_json: Some(false),
                json_only: Some(false),
            },
        );
        output_profiles.insert(
            "developer".to_string(),
            ProfileSettings {
                max_line_length: Some(120),
                remove_comments: Some(false),
                minify_json: Some(false),
                json_only: Some(false),
            },
        );
        output_profiles.insert(
            "audit".to_string(),
            ProfileSettings {
                max_line_length: None,
                remove_comments: Some(false),
                minify_json: Some(false),
                json_only: Some(false),
            },
        );
        output_profiles.insert(
            "json-only".to_string(),
            ProfileSettings {
                max_line_length: None,
                remove_comments: Some(false),
                minify_json: Some(false),
                json_only: Some(true),
            },
        );

        Self {
            denied_commands: vec![
                "git push.*--force".to_string(),
                "git reset --hard".to_string(),
            ],
            custom_dlp_patterns: Vec::new(),
            output_profiles,
            default_profile: "strict".to_string(),
            overrides: std::collections::HashMap::new(),
            regex_filters: Vec::new(),
            strict_chained: false,
        }
    }
}

impl UserConfig {
    /// Load the configuration from global (`~/.config/rtk/config.json`) and local (`./.rtk.json`) paths.
    /// Merge settings prioritizing local definitions.
    pub fn load() -> Self {
        let mut config = UserConfig::default();

        // 1. Try to load from user global home folder: ~/.config/rtk/config.json
        if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
            let global_path = Path::new(&home).join(".config/rtk/config.json");
            if let Ok(content) = fs::read_to_string(&global_path) {
                let _ = config.merge_from_str(&content);
            }
        }

        // 2. Try to load from local directory: ./.rtk.json
        let local_path = Path::new(".rtk.json");
        if let Ok(content) = fs::read_to_string(local_path) {
            let _ = config.merge_from_str(&content);
        }

        config
    }

    /// Merge config keys from a JSON string representation.
    pub fn merge_from_str(&mut self, content: &str) -> Result<(), serde_json::Error> {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(arr) = val.get("denied_commands").and_then(|v| v.as_array()) {
                self.denied_commands.clear();
                self.denied_commands.extend(
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .filter(|s| regex::Regex::new(s).is_ok())
                        .map(String::from),
                );
            }
            if let Some(arr) = val
                .pointer("/dlp/custom_patterns")
                .and_then(|v| v.as_array())
            {
                self.custom_dlp_patterns.clear();
                self.custom_dlp_patterns.extend(
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .filter(|s| regex::Regex::new(s).is_ok())
                        .map(String::from),
                );
            }

            // P1 profiles
            if let Some(profiles_val) = val.get("output_profiles").and_then(|v| v.as_object()) {
                for (k, v_obj) in profiles_val {
                    if let Ok(settings) = serde_json::from_value::<ProfileSettings>(v_obj.clone()) {
                        self.output_profiles.insert(k.clone(), settings);
                    }
                }
            }

            if let Some(def) = val.get("default_profile").and_then(|v| v.as_str()) {
                self.default_profile = def.to_string();
            }

            if let Some(overrides_val) = val.get("overrides").and_then(|v| v.as_object()) {
                for (k, v_str) in overrides_val {
                    if let Some(profile_name) = v_str.as_str() {
                        self.overrides.insert(k.clone(), profile_name.to_string());
                    }
                }
            }

            if let Some(arr) = val.get("regex_filters").and_then(|v| v.as_array()) {
                self.regex_filters.clear();
                for item in arr {
                    if let Ok(rule) = serde_json::from_value::<RegexFilterRule>(item.clone()) {
                        if regex::Regex::new(&rule.pattern).is_ok() {
                            self.regex_filters.push(rule);
                        }
                    }
                }
            }
            if let Some(strict) = val.get("strict_chained").and_then(|v| v.as_bool()) {
                self.strict_chained = strict;
            }
        }
        Ok(())
    }

    /// Retrieve the savings profile settings for a specific command using configured overrides or fallback to default.
    pub fn get_profile_for_cmd(&self, cmd: &str) -> ProfileSettings {
        for (pattern, profile_name) in &self.overrides {
            if cmd.contains(pattern) {
                if let Some(settings) = self.output_profiles.get(profile_name) {
                    return settings.clone();
                }
            }
        }

        if let Some(settings) = self.output_profiles.get(&self.default_profile) {
            settings.clone()
        } else {
            ProfileSettings::default()
        }
    }
}

/// Get the loaded UserConfig (global and local configuration merged).
pub fn get_config() -> UserConfig {
    UserConfig::load()
}

/// Create a default config.json file in `~/.config/rtk/` if it does not already exist.
const DEFAULT_CONFIG_JSON: &str = r#"{
  "denied_commands": [
    "git push.*--force",
    "git reset --hard"
  ],
  "dlp": {
    "custom_patterns": [
      "MY_PROJECT_SECRET_[a-zA-Z0-9]{12}"
    ]
  }
}"#;

/// Write the default config at `path` if it does not already exist.
fn ensure_default_config_at(path: &Path) -> Result<(), std::io::Error> {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, DEFAULT_CONFIG_JSON)?;
    }
    Ok(())
}

pub fn create_default_config() -> Result<(), std::io::Error> {
    if let Some(path) = global_config_path() {
        ensure_default_config_at(&path)?;
    }
    Ok(())
}

fn global_config_path() -> Option<std::path::PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(|home| Path::new(&home).join(".config/rtk/config.json"))
}

fn modify_config<F>(f: F) -> anyhow::Result<()>
where
    F: FnOnce(&mut serde_json::Map<String, serde_json::Value>),
{
    // Resolve the path ONCE, then create + read at that exact path. Resolving
    // via env inside create_default_config separately from here opened a race
    // window (parallel tests mutate HOME/USERPROFILE) where create and read
    // could target different dirs — flaky "file not found" on Windows CI.
    let path = global_config_path()
        .ok_or_else(|| anyhow::anyhow!("could not determine global config directory"))?;

    ensure_default_config_at(&path)
        .map_err(|e| anyhow::anyhow!("failed to create default config: {e}"))?;

    let content = fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("failed to read global config: {e}"))?;

    let mut val: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("failed to parse global config JSON: {e}"))?;

    if let Some(obj) = val.as_object_mut() {
        f(obj);
    } else {
        let mut obj = serde_json::Map::new();
        f(&mut obj);
        val = serde_json::Value::Object(obj);
    }

    let updated = serde_json::to_string_pretty(&val)
        .map_err(|e| anyhow::anyhow!("failed to serialize updated config: {e}"))?;

    fs::write(&path, updated).map_err(|e| anyhow::anyhow!("failed to write global config: {e}"))?;

    Ok(())
}

/// Display the current active merged configuration as pretty-printed JSON.
pub fn config_show() -> anyhow::Result<()> {
    let config = get_config();
    let merged_json = serde_json::json!({
        "denied_commands": config.denied_commands,
        "dlp": {
            "custom_patterns": config.custom_dlp_patterns
        },
        "output_profiles": config.output_profiles,
        "default_profile": config.default_profile,
        "overrides": config.overrides,
        "regex_filters": config.regex_filters
    });
    println!("{}", serde_json::to_string_pretty(&merged_json)?);
    Ok(())
}

/// Export the global configuration file to stdout.
pub fn config_export() -> anyhow::Result<()> {
    let path = global_config_path()
        .ok_or_else(|| anyhow::anyhow!("could not determine global config directory"))?;

    let content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        let default_val = serde_json::json!({
            "denied_commands": [],
            "dlp": {
                "custom_patterns": []
            },
            "default_profile": "balanced"
        });
        serde_json::to_string_pretty(&default_val)?
    };

    println!("{}", content);
    Ok(())
}

/// Import configuration from a path (or stdin if None) and overwrite the global config file.
pub fn config_import(import_path: Option<&str>) -> anyhow::Result<()> {
    let path = global_config_path()
        .ok_or_else(|| anyhow::anyhow!("could not determine global config directory"))?;

    let input_content = match import_path {
        Some(p) => fs::read_to_string(p)
            .map_err(|e| anyhow::anyhow!("failed to read import file {p}: {e}"))?,
        None => {
            use std::io::Read;
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };

    let val: serde_json::Value = serde_json::from_str(&input_content)
        .map_err(|e| anyhow::anyhow!("invalid JSON configuration: {e}"))?;

    if !val.is_object() {
        return Err(anyhow::anyhow!("configuration must be a JSON object"));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let pretty_str = serde_json::to_string_pretty(&val)?;
    fs::write(&path, pretty_str)?;
    Ok(())
}

/// Append a regex pattern to the list of denied commands in the global config.
pub fn config_deny_add(pattern: &str) -> anyhow::Result<()> {
    regex::Regex::new(pattern).map_err(|e| anyhow::anyhow!("invalid regex pattern: {e}"))?;
    modify_config(|obj| {
        let denied = obj
            .entry("denied_commands")
            .or_insert_with(|| serde_json::Value::Array(Vec::new()));
        if let Some(arr) = denied.as_array_mut() {
            let item = serde_json::Value::String(pattern.to_string());
            if !arr.contains(&item) {
                arr.push(item);
            }
        }
    })
}

/// Append a custom Data Loss Prevention regex pattern to the global config.
pub fn config_dlp_add(pattern: &str) -> anyhow::Result<()> {
    regex::Regex::new(pattern).map_err(|e| anyhow::anyhow!("invalid regex pattern: {e}"))?;
    modify_config(|obj| {
        let dlp = obj
            .entry("dlp")
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
        if let Some(dlp_obj) = dlp.as_object_mut() {
            let custom_patterns = dlp_obj
                .entry("custom_patterns")
                .or_insert_with(|| serde_json::Value::Array(Vec::new()));
            if let Some(arr) = custom_patterns.as_array_mut() {
                let item = serde_json::Value::String(pattern.to_string());
                if !arr.contains(&item) {
                    arr.push(item);
                }
            }
        }
    })
}

/// Set the default active savings profile in global config.
pub fn config_profile_set(name: &str) -> anyhow::Result<()> {
    let valid_profiles = ["strict", "balanced", "developer", "audit", "json-only"];
    if !valid_profiles.contains(&name) {
        return Err(anyhow::anyhow!(
            "Invalid profile '{}'. Supported profiles: {}",
            name,
            valid_profiles.join(", ")
        ));
    }
    modify_config(|obj| {
        obj.insert(
            "default_profile".to_string(),
            serde_json::Value::String(name.to_string()),
        );
    })
}

/// Append a custom regex filter pattern and action to the global config.
pub fn config_filter_add(pattern: &str, action: &str) -> anyhow::Result<()> {
    regex::Regex::new(pattern).map_err(|e| anyhow::anyhow!("invalid regex pattern: {e}"))?;
    if action != "strip" && action != "collapse" {
        return Err(anyhow::anyhow!(
            "invalid filter action: {action}. Must be 'strip' or 'collapse'"
        ));
    }
    modify_config(|obj| {
        let filters = obj
            .entry("regex_filters")
            .or_insert_with(|| serde_json::Value::Array(Vec::new()));
        if let Some(arr) = filters.as_array_mut() {
            let item = serde_json::json!({
                "pattern": pattern.to_string(),
                "action": action.to_string()
            });
            let mut exists = false;
            for val in arr.iter() {
                if let Some(p) = val.get("pattern").and_then(|p| p.as_str()) {
                    if p == pattern {
                        exists = true;
                        break;
                    }
                }
            }
            if !exists {
                arr.push(item);
            } else {
                for val in arr.iter_mut() {
                    if let Some(p) = val.get("pattern").and_then(|p| p.as_str()) {
                        if p == pattern {
                            if let Some(o) = val.as_object_mut() {
                                o.insert(
                                    "action".to_string(),
                                    serde_json::Value::String(action.to_string()),
                                );
                            }
                        }
                    }
                }
            }
        }
    })
}

/// Applies all configured regex filters to the given output.
pub fn apply_regex_filters(input: &str) -> String {
    let config = get_config();
    let mut current = input.to_string();
    for rule in &config.regex_filters {
        if let Ok(re) = regex::Regex::new(&rule.pattern) {
            match rule.action.as_str() {
                "strip" => {
                    current = re.replace_all(&current, "").to_string();
                }
                "collapse" => {
                    current = re.replace_all(&current, "[collapsed]").to_string();
                }
                _ => {}
            }
        }
    }
    current
}

/// Apply savings profile settings (line cap, comments, JSON) after command filters.
pub fn apply_profile_settings(input: &str, profile: &ProfileSettings) -> String {
    let mut lines: Vec<String> = input.lines().map(String::from).collect();

    if profile.json_only == Some(true) {
        lines.retain(|l| {
            let t = l.trim();
            t.starts_with('{')
                || t.starts_with('[')
                || t.starts_with('"')
                || t.contains(": {")
                || t.contains(": [")
        });
    }

    if profile.remove_comments == Some(true) {
        lines = lines
            .into_iter()
            .map(|l| {
                if l.trim_start().starts_with('#') {
                    return String::new();
                }
                if let Some(idx) = l.find("//") {
                    l[..idx].trim_end().to_string()
                } else {
                    l
                }
            })
            .filter(|l| !l.is_empty())
            .collect();
    }

    if let Some(max) = profile.max_line_length {
        lines = lines
            .into_iter()
            .map(|l| {
                if l.len() > max {
                    format!("{}…", &l[..max.saturating_sub(1)])
                } else {
                    l
                }
            })
            .collect();
    }

    let mut out = lines.join("\n");
    if input.ends_with('\n') && !out.is_empty() {
        out.push('\n');
    }

    if profile.minify_json == Some(true) {
        out = out
            .lines()
            .map(|line| {
                let t = line.trim();
                if (t.starts_with('{') && t.ends_with('}'))
                    || (t.starts_with('[') && t.ends_with(']'))
                {
                    serde_json::from_str::<serde_json::Value>(t)
                        .map(|v| v.to_string())
                        .unwrap_or_else(|_| line.to_string())
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
    }

    out
}

/// Return human-readable errors for invalid regex rules in config.
pub fn validate_regex_config() -> Vec<String> {
    let config = get_config();
    let mut errors = Vec::new();
    for rule in &config.regex_filters {
        if regex::Regex::new(&rule.pattern).is_err() {
            errors.push(format!("regex_filters pattern invalid: '{}'", rule.pattern));
        }
    }
    for pat in &config.denied_commands {
        if regex::Regex::new(pat).is_err() {
            errors.push(format!("denied_commands pattern invalid: '{pat}'"));
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    static CONFIG_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn test_temp_dir(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "rtk_{label}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn restore_home_env(
        original_home: Option<std::ffi::OsString>,
        original_userprofile: Option<std::ffi::OsString>,
    ) {
        if let Some(h) = original_home {
            std::env::set_var("HOME", h);
        } else {
            std::env::remove_var("HOME");
        }
        if let Some(up) = original_userprofile {
            std::env::set_var("USERPROFILE", up);
        } else {
            std::env::remove_var("USERPROFILE");
        }
    }

    /// ponytail: Windows CI can race on temp dir delete; retry briefly.
    fn cleanup_temp_dir(path: &std::path::Path) {
        if !path.exists() {
            return;
        }
        for attempt in 0..8 {
            match fs::remove_dir_all(path) {
                Ok(()) => return,
                Err(_) if attempt + 1 < 8 => {
                    std::thread::sleep(std::time::Duration::from_millis(25 * (attempt as u64 + 1)));
                }
                Err(e) => panic!("cleanup {}: {e}", path.display()),
            }
        }
    }

    #[test]
    fn merge_strict_chained() {
        let json = r#"{ "strict_chained": true }"#;
        let mut config = UserConfig::default();
        config.merge_from_str(json).unwrap();
        assert!(config.strict_chained);
    }

    #[test]
    fn test_merge_from_str() {
        let mut config = UserConfig::default();
        let json = r#"{
            "denied_commands": ["rm -rf", "git push --force"],
            "dlp": {
                "custom_patterns": ["SECRET_[0-9]+"]
            }
        }"#;
        config.merge_from_str(json).unwrap();
        assert_eq!(config.denied_commands.len(), 2);
        assert_eq!(config.denied_commands[0], "rm -rf");
        assert_eq!(config.denied_commands[1], "git push --force");
        assert_eq!(config.custom_dlp_patterns.len(), 1);
        assert_eq!(config.custom_dlp_patterns[0], "SECRET_[0-9]+");
    }

    #[test]
    fn test_merge_missing_keys() {
        let mut config = UserConfig::default();
        let json = r#"{
            "denied_commands": ["rm -rf"]
        }"#;
        config.merge_from_str(json).unwrap();
        assert_eq!(config.denied_commands.len(), 1);
        assert_eq!(config.custom_dlp_patterns.len(), 0);
    }

    #[test]
    fn test_modify_config() {
        let _lock = CONFIG_TEST_LOCK.lock().unwrap();
        let temp_dir = test_temp_dir("config_modify_test");
        fs::create_dir_all(&temp_dir).unwrap();

        // Temporarily override HOME and USERPROFILE env vars
        let original_home = std::env::var_os("HOME");
        let original_userprofile = std::env::var_os("USERPROFILE");
        std::env::set_var("HOME", &temp_dir);
        std::env::set_var("USERPROFILE", &temp_dir);

        // Add to deny
        config_deny_add("forbidden_cmd").unwrap();
        // Add to dlp
        config_dlp_add("PAT_[a-z]+").unwrap();

        // Read file and check
        let path = temp_dir.join(".config/rtk/config.json");
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        let val: serde_json::Value = serde_json::from_str(&content).unwrap();

        let denied = val["denied_commands"].as_array().unwrap();
        assert!(denied.contains(&serde_json::Value::String("forbidden_cmd".to_string())));
        assert!(denied.contains(&serde_json::Value::String("git reset --hard".to_string()))); // from default template

        let custom_patterns = val["dlp"]["custom_patterns"].as_array().unwrap();
        assert!(custom_patterns.contains(&serde_json::Value::String("PAT_[a-z]+".to_string())));

        // Test config_show does not error
        assert!(config_show().is_ok());

        restore_home_env(original_home, original_userprofile);
        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_profile_merging() {
        let mut config = UserConfig::default();
        let json = r#"{
            "default_profile": "developer",
            "overrides": {
                "git diff": "strict",
                "cargo test": "developer"
            },
            "output_profiles": {
                "custom": {
                    "max_line_length": 50,
                    "remove_comments": true,
                    "minify_json": true
                }
            }
        }"#;
        config.merge_from_str(json).unwrap();
        assert_eq!(config.default_profile, "developer");
        assert_eq!(config.overrides.get("git diff").unwrap(), "strict");
        assert_eq!(config.overrides.get("cargo test").unwrap(), "developer");

        let custom_profile = config.output_profiles.get("custom").unwrap();
        assert_eq!(custom_profile.max_line_length, Some(50));
        assert_eq!(custom_profile.remove_comments, Some(true));
        assert_eq!(custom_profile.minify_json, Some(true));

        // Test get_profile_for_cmd
        let p_git = config.get_profile_for_cmd("git diff --name-only");
        assert_eq!(p_git.max_line_length, Some(80)); // from default strict

        let p_cargo = config.get_profile_for_cmd("cargo test --all");
        assert_eq!(p_cargo.max_line_length, Some(120)); // from default developer

        let p_fallback = config.get_profile_for_cmd("npm run dev");
        assert_eq!(p_fallback.max_line_length, Some(120)); // fallbacks to default_profile which is developer
    }

    #[test]
    fn test_regex_filtering_rules() {
        let mut config = UserConfig::default();
        let json = r#"{
            "regex_filters": [
                {
                    "pattern": "secret_token=[a-zA-Z0-9]+",
                    "action": "strip"
                },
                {
                    "pattern": "db_password=.+",
                    "action": "collapse"
                }
            ]
        }"#;
        config.merge_from_str(json).unwrap();
        assert_eq!(config.regex_filters.len(), 2);
        assert_eq!(config.regex_filters[0].pattern, "secret_token=[a-zA-Z0-9]+");
        assert_eq!(config.regex_filters[0].action, "strip");
        assert_eq!(config.regex_filters[1].pattern, "db_password=.+");
        assert_eq!(config.regex_filters[1].action, "collapse");

        let re1 = regex::Regex::new(&config.regex_filters[0].pattern).unwrap();
        let re2 = regex::Regex::new(&config.regex_filters[1].pattern).unwrap();
        let input = "url: http://api.com?secret_token=abc123xyz\ndb_password=mysecretpassword123";
        let step1 = re1.replace_all(input, "").to_string();
        let step2 = re2.replace_all(&step1, "[collapsed]").to_string();
        assert_eq!(step2, "url: http://api.com?\n[collapsed]");
    }

    #[test]
    fn test_config_export_import() {
        let _lock = CONFIG_TEST_LOCK.lock().unwrap();
        let temp_dir = test_temp_dir("config_export_import_test");
        fs::create_dir_all(&temp_dir).unwrap();

        let original_home = std::env::var_os("HOME");
        let original_userprofile = std::env::var_os("USERPROFILE");
        std::env::set_var("HOME", &temp_dir);
        std::env::set_var("USERPROFILE", &temp_dir);

        create_default_config().unwrap();
        config_deny_add("test_deny_from_export").unwrap();

        let path = global_config_path().unwrap();
        assert!(path.exists());

        let import_data = serde_json::json!({
            "denied_commands": ["forbidden_1", "forbidden_2"],
            "dlp": {
                "custom_patterns": ["PAT_[0-9]+"]
            },
            "default_profile": "developer"
        });
        let import_file = temp_dir.join("new_config.json");
        fs::write(
            &import_file,
            serde_json::to_string_pretty(&import_data).unwrap(),
        )
        .unwrap();

        config_import(Some(import_file.to_str().unwrap())).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let val: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(val["default_profile"], "developer");
        assert_eq!(val["denied_commands"].as_array().unwrap().len(), 2);
        assert!(val["denied_commands"]
            .as_array()
            .unwrap()
            .contains(&serde_json::Value::String("forbidden_1".to_string())));
        assert_eq!(val["dlp"]["custom_patterns"].as_array().unwrap().len(), 1);

        restore_home_env(original_home, original_userprofile);
        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn filt4_strict_profile_shrinks_more_than_audit() {
        let strict = ProfileSettings {
            max_line_length: Some(20),
            remove_comments: Some(true),
            minify_json: None,
            json_only: None,
        };
        let audit = ProfileSettings {
            max_line_length: None,
            remove_comments: Some(false),
            minify_json: None,
            json_only: None,
        };
        let input = format!("{} // noise", "x".repeat(80));
        let strict_out = apply_profile_settings(&input, &strict);
        let audit_out = apply_profile_settings(&input, &audit);
        assert!(
            strict_out.len() < audit_out.len(),
            "strict profile should reduce output measurably"
        );
    }

    #[test]
    fn apply_profile_settings_truncates_lines() {
        let profile = ProfileSettings {
            max_line_length: Some(10),
            remove_comments: None,
            minify_json: None,
            json_only: None,
        };
        let out = apply_profile_settings("12345678901\nshort", &profile);
        assert!(out.starts_with("123456789…"));
        assert!(out.contains("short"));
    }

    #[test]
    fn apply_profile_settings_removes_line_comments() {
        let profile = ProfileSettings {
            max_line_length: None,
            remove_comments: Some(true),
            minify_json: None,
            json_only: None,
        };
        let out = apply_profile_settings("keep // drop tail\n# skip\nok", &profile);
        assert_eq!(out, "keep\nok");
    }
}
