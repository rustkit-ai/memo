use anyhow::Result;
use chrono::{DateTime, Utc};
use memo_core::{Entry, Store};
use std::path::Path;

pub struct InjectBlock {
    pub last_entry: Option<Entry>,
    pub todos: Vec<Entry>,
    pub recent_tags: Vec<String>,
    pub entry_count: usize,
}

impl InjectBlock {
    pub fn build(store: &Store) -> Result<Self> {
        let entries = store.list(Some(20))?;
        Self::from_entries(store, entries)
    }

    pub fn build_since(store: &Store, since: DateTime<Utc>) -> Result<Self> {
        let entries = store.list_since(since, Some(20))?;
        Self::from_entries(store, entries)
    }

    fn from_entries(store: &Store, entries: Vec<Entry>) -> Result<Self> {
        let last_entry = entries.first().cloned();
        let todos = entries
            .iter()
            .filter(|e| e.content.starts_with("todo:") || e.content.starts_with("TODO:"))
            .cloned()
            .collect();
        let recent_tags = store.recent_tags(10)?;
        let entry_count = store.count()?;
        Ok(Self {
            last_entry,
            todos,
            recent_tags,
            entry_count,
        })
    }

    pub fn render_text(&self) -> String {
        let mut out = String::from("## memo context\n");

        if let Some(e) = &self.last_entry {
            let date = e.timestamp.format("%Y-%m-%d");
            out.push_str(&format!("last: {} — \"{}\"\n", date, e.content));
        } else {
            out.push_str("last: (no entries yet)\n");
        }

        for todo in &self.todos {
            out.push_str(&format!("todo: {}\n", todo.content.trim_start_matches("todo:").trim_start_matches("TODO:").trim()));
        }

        if !self.recent_tags.is_empty() {
            out.push_str(&format!("recent tags: {}\n", self.recent_tags.join(" · ")));
        }

        out
    }

    pub fn render_json(&self) -> Result<String> {
        let value = serde_json::json!({
            "last_entry": self.last_entry.as_ref().map(|e| serde_json::json!({
                "timestamp": e.timestamp.to_rfc3339(),
                "content": e.content,
                "tags": e.tags,
            })),
            "todos": self.todos.iter().map(|e| serde_json::json!({
                "timestamp": e.timestamp.to_rfc3339(),
                "content": e.content,
            })).collect::<Vec<_>>(),
            "recent_tags": self.recent_tags,
            "entry_count": self.entry_count,
        });
        Ok(serde_json::to_string_pretty(&value)?)
    }
}

const INSTRUCTIONS_START: &str = "<!-- memo:instructions:start -->";
const INSTRUCTIONS_END: &str = "<!-- memo:instructions:end -->";

const INSTRUCTIONS_BODY: &str = "\
## memo — persistent agent memory
- Run `memo log \"<what you did>\"` after each significant task
- Run `memo log \"todo: <next step>\"` before ending the session
- Run `memo inject` at the start of a session to recall context
";

pub fn setup(project_dir: &Path) -> Result<SetupResult> {
    // 1. Write instructions + context block into CLAUDE.md
    write_instructions_to_claude_md(project_dir)?;
    let block = InjectBlock {
        last_entry: None,
        todos: vec![],
        recent_tags: vec![],
        entry_count: 0,
    };
    write_to_claude_md(&block, project_dir)?;

    // 2. Install Stop hook into .claude/settings.json
    let hook_installed = install_stop_hook(project_dir)?;

    Ok(SetupResult { hook_installed })
}

pub struct SetupResult {
    pub hook_installed: bool,
}

fn write_instructions_to_claude_md(project_dir: &Path) -> Result<()> {
    let claude_md = project_dir.join("CLAUDE.md");

    let section = format!(
        "{}\n{}{}\n",
        INSTRUCTIONS_START, INSTRUCTIONS_BODY, INSTRUCTIONS_END
    );

    let existing = if claude_md.exists() {
        std::fs::read_to_string(&claude_md)?
    } else {
        String::new()
    };

    let new_content = if existing.contains(INSTRUCTIONS_START) {
        let start = existing.find(INSTRUCTIONS_START).unwrap();
        let end = existing
            .find(INSTRUCTIONS_END)
            .map(|i| i + INSTRUCTIONS_END.len())
            .unwrap_or(existing.len());
        format!("{}{}{}", &existing[..start], section, &existing[end..])
    } else {
        format!("{}\n{}", section, existing)
    };

    std::fs::write(&claude_md, new_content)?;
    Ok(())
}

fn install_stop_hook(project_dir: &Path) -> Result<bool> {
    let claude_dir = project_dir.join(".claude");
    std::fs::create_dir_all(&claude_dir)?;
    let settings_path = claude_dir.join("settings.json");

    let mut root: serde_json::Value = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let memo_hook = serde_json::json!({
        "hooks": [{ "type": "command", "command": "memo inject --claude" }]
    });

    // Check if our hook is already present
    let hooks = root
        .get("hooks")
        .and_then(|h| h.get("Stop"))
        .and_then(|s| s.as_array());

    if let Some(stop_hooks) = hooks {
        let already = stop_hooks.iter().any(|h| {
            h.get("hooks")
                .and_then(|hs| hs.as_array())
                .map(|hs| {
                    hs.iter().any(|cmd| {
                        cmd.get("command")
                            .and_then(|c| c.as_str())
                            .map(|s| s.contains("memo inject"))
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false)
        });
        if already {
            return Ok(false);
        }
    }

    root["hooks"]["Stop"]
        .as_array_mut()
        .map(|arr| arr.push(memo_hook.clone()))
        .unwrap_or_else(|| {
            root["hooks"]["Stop"] = serde_json::json!([memo_hook]);
        });

    std::fs::write(&settings_path, serde_json::to_string_pretty(&root)?)?;
    Ok(true)
}

pub fn write_to_claude_md(block: &InjectBlock, project_dir: &Path) -> Result<()> {
    let claude_md = project_dir.join("CLAUDE.md");
    let section_start = "<!-- memo:start -->";
    let section_end = "<!-- memo:end -->";

    let memo_section = format!(
        "{}\n{}{}\n",
        section_start,
        block.render_text(),
        section_end
    );

    let existing = if claude_md.exists() {
        std::fs::read_to_string(&claude_md)?
    } else {
        String::new()
    };

    let new_content = if existing.contains(section_start) {
        // Replace existing section
        let start = existing.find(section_start).unwrap();
        let end = existing
            .find(section_end)
            .map(|i| i + section_end.len())
            .unwrap_or(existing.len());
        format!("{}{}{}", &existing[..start], memo_section, &existing[end..])
    } else {
        // Prepend
        format!("{}\n{}", memo_section, existing)
    };

    std::fs::write(&claude_md, new_content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_render_text_empty() {
        let block = InjectBlock {
            last_entry: None,
            todos: vec![],
            recent_tags: vec![],
            entry_count: 0,
        };
        let text = block.render_text();
        assert!(text.contains("## memo context"));
        assert!(text.contains("no entries yet"));
    }

    #[test]
    fn test_render_json() {
        let block = InjectBlock {
            last_entry: None,
            todos: vec![],
            recent_tags: vec!["bug".to_string()],
            entry_count: 5,
        };
        let json = block.render_json().unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(val["entry_count"], 5);
        assert_eq!(val["recent_tags"][0], "bug");
    }

    #[test]
    fn test_write_to_claude_md() {
        let dir = env::temp_dir().join(format!("memo_hooks_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let block = InjectBlock {
            last_entry: None,
            todos: vec![],
            recent_tags: vec!["refactor".to_string()],
            entry_count: 1,
        };

        write_to_claude_md(&block, &dir).unwrap();

        let content = std::fs::read_to_string(dir.join("CLAUDE.md")).unwrap();
        assert!(content.contains("<!-- memo:start -->"));
        assert!(content.contains("recent tags: refactor"));

        // Idempotent: write again, should replace not append
        write_to_claude_md(&block, &dir).unwrap();
        let content2 = std::fs::read_to_string(dir.join("CLAUDE.md")).unwrap();
        assert_eq!(content2.matches("<!-- memo:start -->").count(), 1);
    }
}
