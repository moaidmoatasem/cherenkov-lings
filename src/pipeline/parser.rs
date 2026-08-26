use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct WorkflowDefinition {
    pub name: Option<String>,
    #[serde(default)]
    pub on: TriggerConfig,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub concurrency: Option<ConcurrencyConfig>,
    #[serde(default)]
    pub jobs: HashMap<String, JobDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
#[derive(Default)]
pub enum TriggerConfig {
    Single(String),
    Multiple(Vec<String>),
    Detailed(HashMap<String, serde_yaml::Value>),
    #[default]
    None,
}

impl TriggerConfig {
    pub fn triggers(&self) -> Vec<String> {
        match self {
            TriggerConfig::Single(s) => vec![s.clone()],
            TriggerConfig::Multiple(v) => v.clone(),
            TriggerConfig::Detailed(m) => m.keys().cloned().collect(),
            TriggerConfig::None => Vec::new(),
        }
    }

    pub fn has_trigger(&self, trigger_name: &str) -> bool {
        self.triggers()
            .iter()
            .any(|t| t.eq_ignore_ascii_case(trigger_name))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ConcurrencyConfig {
    Simple(String),
    Detailed {
        group: String,
        #[serde(rename = "cancel-in-progress", default)]
        cancel_in_progress: Option<bool>,
    },
}

impl ConcurrencyConfig {
    pub fn group_name(&self) -> &str {
        match self {
            ConcurrencyConfig::Simple(s) => s.as_str(),
            ConcurrencyConfig::Detailed { group, .. } => group.as_str(),
        }
    }

    pub fn cancels_in_progress(&self) -> bool {
        match self {
            ConcurrencyConfig::Simple(_) => false,
            ConcurrencyConfig::Detailed {
                cancel_in_progress, ..
            } => cancel_in_progress.unwrap_or(false),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct JobDefinition {
    pub name: Option<String>,
    #[serde(rename = "runs-on")]
    pub runs_on: Option<RunsOnConfig>,
    pub strategy: Option<StrategyDefinition>,
    #[serde(default)]
    pub steps: Vec<StepDefinition>,
    #[serde(rename = "timeout-minutes")]
    pub timeout_minutes: Option<u64>,
    pub needs: Option<NeedsConfig>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(rename = "if")]
    pub condition: Option<String>,
    pub concurrency: Option<ConcurrencyConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum RunsOnConfig {
    Single(String),
    Multiple(Vec<String>),
    Expression(String),
}

impl RunsOnConfig {
    pub fn as_str_repr(&self) -> String {
        match self {
            RunsOnConfig::Single(s) => s.clone(),
            RunsOnConfig::Multiple(v) => v.join(", "),
            RunsOnConfig::Expression(e) => e.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum NeedsConfig {
    Single(String),
    Multiple(Vec<String>),
}

impl NeedsConfig {
    pub fn as_list(&self) -> Vec<String> {
        match self {
            NeedsConfig::Single(s) => vec![s.clone()],
            NeedsConfig::Multiple(v) => v.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct StrategyDefinition {
    pub matrix: Option<MatrixDefinition>,
    #[serde(rename = "fail-fast")]
    pub fail_fast: Option<bool>,
    #[serde(rename = "max-parallel")]
    pub max_parallel: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MatrixDefinition {
    #[serde(flatten)]
    pub dimensions: HashMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub include: Option<Vec<HashMap<String, serde_yaml::Value>>>,
    #[serde(default)]
    pub exclude: Option<Vec<HashMap<String, serde_yaml::Value>>>,
}

impl MatrixDefinition {
    /// Extracts explicit matrix variable lists (filtering out `include` and `exclude` keys).
    pub fn variable_dimensions(&self) -> HashMap<String, Vec<String>> {
        let mut vars = HashMap::new();
        for (k, v) in &self.dimensions {
            if k == "include" || k == "exclude" {
                continue;
            }
            if let Some(list) = v.as_sequence() {
                let items: Vec<String> = list
                    .iter()
                    .map(|item| match item {
                        serde_yaml::Value::String(s) => s.clone(),
                        serde_yaml::Value::Number(n) => n.to_string(),
                        serde_yaml::Value::Bool(b) => b.to_string(),
                        _ => format!("{:?}", item),
                    })
                    .collect();
                vars.insert(k.clone(), items);
            } else if let Some(s) = v.as_str() {
                vars.insert(k.clone(), vec![s.to_string()]);
            }
        }
        vars
    }

    /// Calculate all matrix combinations (Cartesian product) including `include` and excluding `exclude`.
    pub fn expand_combinations(&self) -> Vec<HashMap<String, String>> {
        let dimensions = self.variable_dimensions();
        if dimensions.is_empty() {
            if let Some(ref inc) = self.include {
                return inc
                    .iter()
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| {
                                let val_str = match v {
                                    serde_yaml::Value::String(s) => s.clone(),
                                    serde_yaml::Value::Number(n) => n.to_string(),
                                    serde_yaml::Value::Bool(b) => b.to_string(),
                                    _ => format!("{:?}", v),
                                };
                                (k.clone(), val_str)
                            })
                            .collect()
                    })
                    .collect();
            }
            return vec![HashMap::new()];
        }

        let keys: Vec<String> = dimensions.keys().cloned().collect();
        let mut results: Vec<HashMap<String, String>> = vec![HashMap::new()];

        for key in &keys {
            let values = dimensions.get(key).unwrap();
            let mut next_results = Vec::new();
            for current in &results {
                for val in values {
                    let mut combo = current.clone();
                    combo.insert(key.clone(), val.clone());
                    next_results.push(combo);
                }
            }
            results = next_results;
        }

        // Apply exclude filter
        if let Some(ref excludes) = self.exclude {
            results.retain(|combo| {
                !excludes.iter().any(|ex| {
                    ex.iter().all(|(k, v)| {
                        let val_str = match v {
                            serde_yaml::Value::String(s) => s.clone(),
                            serde_yaml::Value::Number(n) => n.to_string(),
                            serde_yaml::Value::Bool(b) => b.to_string(),
                            _ => format!("{:?}", v),
                        };
                        combo.get(k) == Some(&val_str)
                    })
                })
            });
        }

        // Apply include additions/overrides
        if let Some(ref includes) = self.include {
            for inc in includes {
                let inc_map: HashMap<String, String> = inc
                    .iter()
                    .map(|(k, v)| {
                        let val_str = match v {
                            serde_yaml::Value::String(s) => s.clone(),
                            serde_yaml::Value::Number(n) => n.to_string(),
                            serde_yaml::Value::Bool(b) => b.to_string(),
                            _ => format!("{:?}", v),
                        };
                        (k.clone(), val_str)
                    })
                    .collect();

                // Check if this include matches an existing combination to augment
                let mut matched = false;
                for combo in &mut results {
                    let keys_match = inc_map.iter().all(|(k, v)| {
                        if let Some(cur_val) = combo.get(k) {
                            cur_val == v
                        } else {
                            true
                        }
                    });
                    if keys_match {
                        for (k, v) in &inc_map {
                            combo.insert(k.clone(), v.clone());
                        }
                        matched = true;
                    }
                }
                // If it doesn't match any, append it as a new combination
                if !matched {
                    results.push(inc_map);
                }
            }
        }

        results
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct StepDefinition {
    pub name: Option<String>,
    pub id: Option<String>,
    pub uses: Option<String>,
    pub run: Option<String>,
    #[serde(default)]
    pub with: HashMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(rename = "if")]
    pub condition: Option<String>,
    #[serde(rename = "timeout-minutes")]
    pub timeout_minutes: Option<u64>,
    #[serde(rename = "continue-on-error")]
    pub continue_on_error: Option<bool>,
}

impl StepDefinition {
    pub fn display_name(&self) -> String {
        if let Some(ref n) = self.name {
            n.clone()
        } else if let Some(ref u) = self.uses {
            u.clone()
        } else if let Some(ref r) = self.run {
            let first_line = r.lines().next().unwrap_or("run step").trim();
            if first_line.len() > 40 {
                format!("{}...", &first_line[..37])
            } else {
                first_line.to_string()
            }
        } else {
            "unnamed step".to_string()
        }
    }

    pub fn is_artifact_upload(&self) -> bool {
        if let Some(ref uses) = self.uses
            && (uses.starts_with("actions/upload-artifact")
                || uses.starts_with("codecov/codecov-action"))
        {
            return true;
        }
        if let Some(ref run) = self.run {
            let lower = run.to_lowercase();
            if (lower.contains("allure")
                || lower.contains("test-results")
                || lower.contains("junit"))
                && (lower.contains("upload")
                    || lower.contains("publish")
                    || lower.contains("archive"))
            {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone)]
pub enum ParseError {
    IoError(String),
    YamlError(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IoError(msg) => write!(f, "Failed to read workflow file: {}", msg),
            ParseError::YamlError(msg) => write!(f, "Invalid YAML syntax in workflow: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

pub fn parse_workflow_str(yaml_content: &str) -> Result<WorkflowDefinition, ParseError> {
    serde_yaml::from_str::<WorkflowDefinition>(yaml_content)
        .map_err(|e| ParseError::YamlError(e.to_string()))
}

pub fn parse_workflow_file(path: &Path) -> Result<WorkflowDefinition, ParseError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| ParseError::IoError(format!("{}: {}", path.display(), e)))?;
    parse_workflow_str(&content)
}
