use anyhow::{Result, bail};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::borrow::Cow;
use tokio::process::Command;

use crate::tool::{Tool, ToolContext, ToolMetadata, ToolResult};

#[derive(Clone, Copy)]
pub enum GitCommand {
    Diff,
    Log,
    Add,
    Commit,
    Branch,
    Switch,
    Restore,
    Show,
    Stash,
}

pub struct GitCommandTool {
    command: GitCommand,
}

impl GitCommandTool {
    pub fn new(command: GitCommand) -> Self {
        Self { command }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DiffArgs {
    #[serde(default)]
    staged: bool,
    path: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LogArgs {
    #[serde(default = "default_log_count")]
    max_count: usize,
    path: Option<String>,
}

fn default_log_count() -> usize {
    10
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PathsArgs {
    paths: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CommitArgs {
    message: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BranchArgs {
    name: Option<String>,
    #[serde(default)]
    delete: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BranchNameArgs {
    branch: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RestoreArgs {
    paths: Vec<String>,
    #[serde(default)]
    staged: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ShowArgs {
    revision: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StashArgs {
    #[serde(default = "default_stash_action")]
    action: String,
    message: Option<String>,
}

fn default_stash_action() -> String {
    "push".to_string()
}

#[async_trait]
impl Tool for GitCommandTool {
    fn metadata(&self) -> ToolMetadata {
        let (name, description, schema) = match self.command {
            GitCommand::Diff => (
                "git_diff",
                "Show changes in the working tree or index.",
                serde_json::json!({
                    "type": "object", "properties": {
                        "staged": { "type": "boolean", "description": "Show staged changes." },
                        "path": { "type": "string", "description": "Limit output to this path." }
                    }, "additionalProperties": false
                }),
            ),
            GitCommand::Log => (
                "git_log",
                "Show recent commits.",
                serde_json::json!({
                    "type": "object", "properties": {
                        "max_count": { "type": "integer", "minimum": 1, "maximum": 100 },
                        "path": { "type": "string" }
                    }, "additionalProperties": false
                }),
            ),
            GitCommand::Add => (
                "git_add",
                "Stage files for the next commit.",
                serde_json::json!({
                    "type": "object", "properties": { "paths": { "type": "array", "items": { "type": "string" }, "minItems": 1 } },
                    "required": ["paths"], "additionalProperties": false
                }),
            ),
            GitCommand::Commit => (
                "git_commit",
                "Create a commit with the staged changes.",
                serde_json::json!({
                    "type": "object", "properties": { "message": { "type": "string", "minLength": 1 } },
                    "required": ["message"], "additionalProperties": false
                }),
            ),
            GitCommand::Branch => (
                "git_branch",
                "List branches, create a branch, or delete a branch.",
                serde_json::json!({
                    "type": "object", "properties": {
                        "name": { "type": "string" },
                        "delete": { "type": "boolean" }
                    }, "additionalProperties": false
                }),
            ),
            GitCommand::Switch => (
                "git_switch",
                "Switch to an existing branch.",
                serde_json::json!({
                    "type": "object", "properties": { "branch": { "type": "string", "minLength": 1 } },
                    "required": ["branch"], "additionalProperties": false
                }),
            ),
            GitCommand::Restore => (
                "git_restore",
                "Restore working-tree files or unstage files.",
                serde_json::json!({
                    "type": "object", "properties": {
                        "paths": { "type": "array", "items": { "type": "string" }, "minItems": 1 },
                        "staged": { "type": "boolean", "description": "Unstage the paths instead of restoring working-tree files." }
                    }, "required": ["paths"], "additionalProperties": false
                }),
            ),
            GitCommand::Show => (
                "git_show",
                "Show a commit or object.",
                serde_json::json!({
                    "type": "object", "properties": { "revision": { "type": "string" } }, "additionalProperties": false
                }),
            ),
            GitCommand::Stash => (
                "git_stash",
                "Push, list, apply, pop, or drop a stash.",
                serde_json::json!({
                    "type": "object", "properties": {
                        "action": { "type": "string", "enum": ["push", "list", "apply", "pop", "drop"] },
                        "message": { "type": "string", "description": "Optional message for push." }
                    }, "additionalProperties": false
                }),
            ),
        };

        ToolMetadata {
            name: Cow::Borrowed(name),
            description: Cow::Borrowed(description),
            schema,
        }
    }

    async fn execute(&self, ctx: &ToolContext, args: Value) -> Result<ToolResult> {
        let mut git_args: Vec<String> = Vec::new();
        match self.command {
            GitCommand::Diff => {
                let args: DiffArgs = serde_json::from_value(args)?;
                git_args.push("diff".into());
                if args.staged {
                    git_args.push("--cached".into());
                }
                if let Some(path) = args.path {
                    git_args.extend(["--".into(), path]);
                }
            }
            GitCommand::Log => {
                let args: LogArgs = serde_json::from_value(args)?;
                if !(1..=100).contains(&args.max_count) {
                    bail!("max_count must be between 1 and 100");
                }
                git_args.extend([
                    "log".into(),
                    "--oneline".into(),
                    format!("-n{}", args.max_count),
                ]);
                if let Some(path) = args.path {
                    git_args.extend(["--".into(), path]);
                }
            }
            GitCommand::Add => {
                let args: PathsArgs = serde_json::from_value(args)?;
                ensure_paths(&args.paths)?;
                git_args.extend(["add".into(), "--".into()]);
                git_args.extend(args.paths);
            }
            GitCommand::Commit => {
                let args: CommitArgs = serde_json::from_value(args)?;
                if args.message.trim().is_empty() {
                    bail!("message must not be empty");
                }
                git_args.extend(["commit".into(), "-m".into(), args.message]);
            }
            GitCommand::Branch => {
                let args: BranchArgs = serde_json::from_value(args)?;
                match (args.name, args.delete) {
                    (None, false) => git_args.extend(["branch".into(), "--list".into()]),
                    (Some(name), false) => {
                        ensure_nonempty(&name, "name")?;
                        git_args.extend(["branch".into(), "--".into(), name]);
                    }
                    (Some(name), true) => {
                        ensure_nonempty(&name, "name")?;
                        git_args.extend(["branch".into(), "-d".into(), "--".into(), name]);
                    }
                    (None, true) => bail!("name is required when deleting a branch"),
                }
            }
            GitCommand::Switch => {
                let args: BranchNameArgs = serde_json::from_value(args)?;
                ensure_nonempty(&args.branch, "branch")?;
                git_args.extend(["switch".into(), "--".into(), args.branch]);
            }
            GitCommand::Restore => {
                let args: RestoreArgs = serde_json::from_value(args)?;
                ensure_paths(&args.paths)?;
                git_args.push("restore".into());
                if args.staged {
                    git_args.push("--staged".into());
                }
                git_args.push("--".into());
                git_args.extend(args.paths);
            }
            GitCommand::Show => {
                let args: ShowArgs = serde_json::from_value(args)?;
                git_args.push("show".into());
                if let Some(revision) = args.revision {
                    if revision.starts_with('-') {
                        bail!("revision must not start with '-'");
                    }
                    git_args.push(revision);
                }
            }
            GitCommand::Stash => {
                let args: StashArgs = serde_json::from_value(args)?;
                match args.action.as_str() {
                    "push" => {
                        git_args.extend(["stash".into(), "push".into()]);
                        if let Some(message) = args.message {
                            git_args.extend(["-m".into(), message]);
                        }
                    }
                    "list" | "apply" | "pop" | "drop" => {
                        if args.message.is_some() {
                            bail!("message is only supported with the push action");
                        }
                        git_args.extend(["stash".into(), args.action]);
                    }
                    _ => bail!("action must be push, list, apply, pop, or drop"),
                }
            }
        }

        run_git(ctx, &git_args).await
    }
}

pub(crate) async fn run_git(ctx: &ToolContext, args: &[String]) -> Result<ToolResult> {
    let output = Command::new("git")
        .args(args)
        .current_dir(&ctx.working_dir)
        .envs(&ctx.env)
        .output()
        .await?;

    let mut content = String::from_utf8_lossy(&output.stdout).into_owned();
    content.push_str(&String::from_utf8_lossy(&output.stderr));

    Ok(ToolResult {
        content,
        is_error: !output.status.success(),
        metadata: Some(serde_json::json!({ "exit_code": output.status.code() })),
    })
}

fn ensure_paths(paths: &[String]) -> Result<()> {
    if paths.is_empty() || paths.iter().any(String::is_empty) {
        bail!("at least one non-empty path is required");
    }
    Ok(())
}

fn ensure_nonempty(value: &str, field: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("{field} must not be empty");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::git::status::GitStatusTool;
    use std::collections::HashMap;
    use std::process::Command as StdCommand;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn git_add_stages_paths_without_interpreting_them_as_options() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("ferris-agent-git-{suffix}"));
        std::fs::create_dir_all(&directory).unwrap();
        let init = StdCommand::new("git")
            .arg("init")
            .current_dir(&directory)
            .output()
            .unwrap();
        assert!(init.status.success());

        let filename = "-draft.txt";
        std::fs::write(directory.join(filename), "staged content").unwrap();
        let context = ToolContext::new(directory.clone(), HashMap::new());
        let result = GitCommandTool::new(GitCommand::Add)
            .execute(&context, serde_json::json!({ "paths": [filename] }))
            .await
            .unwrap();
        assert!(!result.is_error, "{}", result.content);

        let status = GitStatusTool.execute(&context, Value::Null).await.unwrap();
        assert!(
            status.content.contains("A  -draft.txt"),
            "{}",
            status.content
        );
        std::fs::remove_dir_all(directory).unwrap();
    }
}
