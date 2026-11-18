//! Tools the agent can call. Their implementations run on a tool host (a
//! user's machine via `co tool-host`, or the desktop app); the server only
//! knows their definitions and routes calls.

use serde::{Deserialize, Serialize};

/// A tool as described to the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
    /// JSON Schema of the arguments.
    pub input_schema: serde_json::Value,
}

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("unknown tool: {0}")]
    Unknown(String),
    #[error("invalid arguments: {0}")]
    InvalidArgs(String),
    #[error("{0}")]
    Failed(String),
}

/// The output of one tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    pub text: String,
    pub is_error: bool,
}

/// Every tool that runs on a host.
pub fn definitions() -> Vec<ToolDef> {
    vec![ToolDef {
        name: "bash",
        description: "Run a shell command in the session's working directory.",
        input_schema: serde_json::json!({
            "type": "object",
            "properties": { "command": { "type": "string" } },
            "required": ["command"],
        }),
    }]
}

/// Runs a tool call on this machine.
pub async fn execute(
    name: &str,
    args: &serde_json::Value,
    cwd: &std::path::Path,
) -> Result<ToolOutput, ToolError> {
    match name {
        "bash" => {
            let command = args["command"]
                .as_str()
                .ok_or_else(|| ToolError::InvalidArgs("command must be a string".into()))?;
            let out = tokio::process::Command::new("bash")
                .arg("-c")
                .arg(command)
                .current_dir(cwd)
                .output()
                .await
                .map_err(|e| ToolError::Failed(e.to_string()))?;
            let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
            text.push_str(&String::from_utf8_lossy(&out.stderr));
            Ok(ToolOutput {
                text,
                is_error: !out.status.success(),
            })
        }
        other => Err(ToolError::Unknown(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn bash_runs() {
        let out = super::execute(
            "bash",
            &serde_json::json!({"command": "echo hi"}),
            ".".as_ref(),
        )
        .await
        .unwrap();
        assert_eq!(out.text, "hi\n");
        assert!(!out.is_error);
    }
}
