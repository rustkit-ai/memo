use anyhow::Result;
use chrono::{DateTime, Utc};
use memo_core::{Entry, Store};
use std::fs;
use std::path::Path;

pub struct InjectBlock {
    pub last_entry: Option<Entry>,
    pub todos: Vec<Entry>,
    pub recent_tags: Vec<String>,
    pub entry_count: usize,
}

impl InjectBlock {
    pub fn build(store: &Store) -> Result<Self> {
        Self::from_store(store, store.list(Some(20))?)
    }

    pub fn build_since(store: &Store, since: DateTime<Utc>) -> Result<Self> {
        Self::from_store(store, store.list_since(since, Some(20))?)
    }

    fn from_store(store: &Store, entries: Vec<Entry>) -> Result<Self> {
        Ok(Self {
            last_entry: entries.first().cloned(),
            todos: entries
                .iter()
                .filter(|e| e.content.to_ascii_lowercase().starts_with("todo:"))
                .cloned()
                .collect(),
            recent_tags: store.recent_tags(10)?,
            entry_count: store.count()?,
        })
    }

    pub fn render_text(&self) -> String {
        use std::fmt::Write;
        let mut out = String::from("## memo context\n");

        match &self.last_entry {
            Some(e) => writeln!(out, "last: {} — \"{}\"", e.timestamp.format("%Y-%m-%d"), e.content).unwrap(),
            None => out.push_str("last: (no entries yet)\n"),
        }

        for todo in &self.todos {
            let text = todo.content
                .trim_start_matches(|c: char| c.is_ascii_alphabetic() || c == ':')
                .trim();
            writeln!(out, "todo: {text}").unwrap();
        }

        if !self.recent_tags.is_empty() {
            writeln!(out, "recent tags: {}", self.recent_tags.join(" · ")).unwrap();
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

pub struct SetupResult {
    pub claude_hook_installed: bool,
    pub cursor_rules_written: bool,
}

pub fn setup(project_dir: &Path) -> Result<SetupResult> {
    // Claude Code
    write_instructions_to_claude_md(project_dir)?;
    write_to_claude_md(&InjectBlock::empty(), project_dir)?;
    let claude_hook_installed = install_stop_hook(project_dir)?;

    // Cursor
    let cursor_rules_written = write_cursor_rules(project_dir)?;

    Ok(SetupResult { claude_hook_installed, cursor_rules_written })
}

pub fn write_to_claude_md(block: &InjectBlock, project_dir: &Path) -> Result<()> {
    patch_markdown_section(
        &project_dir.join("CLAUDE.md"),
        "<!-- memo:start -->",
        "<!-- memo:end -->",
        &block.render_text(),
    )
}

const MEMO_INSTRUCTIONS: &str = "\
- Run `memo inject` at the start of every session to recall context\n\
- Run `memo log \"<what you did>\"` after each significant task\n\
- Run `memo log \"todo: <next step>\"` before ending the session\n";

fn write_cursor_rules(project_dir: &Path) -> Result<bool> {
    let rules_dir = project_dir.join(".cursor").join("rules");
    let rules_path = rules_dir.join("memo.mdc");

    if rules_path.exists() {
        return Ok(false);
    }

    fs::create_dir_all(&rules_dir)?;
    fs::write(
        &rules_path,
        format!(
            "---\ndescription: memo persistent memory instructions\nalwaysApply: true\n---\n\n\
             ## memo — persistent agent memory\n{MEMO_INSTRUCTIONS}"
        ),
    )?;
    Ok(true)
}

fn write_instructions_to_claude_md(project_dir: &Path) -> Result<()> {
    patch_markdown_section(
        &project_dir.join("CLAUDE.md"),
        "<!-- memo:instructions:start -->",
        "<!-- memo:instructions:end -->",
        &format!("## memo — persistent agent memory\n{MEMO_INSTRUCTIONS}"),
    )
}

/// Replace or prepend a delimited section in a Markdown file.
/// The section is identified by `start` and `end` HTML comment markers.
/// If the file doesn't exist it is created. If the section doesn't exist it is prepended.
fn patch_markdown_section(path: &Path, start: &str, end: &str, content: &str) -> Result<()> {
    let existing = if path.exists() { fs::read_to_string(path)? } else { String::new() };
    let section = format!("{start}\n{content}{end}\n");

    let new_content = if let Some(s) = existing.find(start) {
        let e = existing.find(end).map(|i| i + end.len()).unwrap_or(existing.len());
        format!("{}{}{}", &existing[..s], section, &existing[e..])
    } else {
        format!("{section}\n{existing}")
    };

    fs::write(path, new_content)?;
    Ok(())
}

fn install_stop_hook(project_dir: &Path) -> Result<bool> {
    let claude_dir = project_dir.join(".claude");
    fs::create_dir_all(&claude_dir)?;
    let settings_path = claude_dir.join("settings.json");

    let mut root: serde_json::Value = if settings_path.exists() {
        serde_json::from_str(&fs::read_to_string(&settings_path)?).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // Check if memo inject hook is already installed
    let already_installed = root
        .get("hooks")
        .and_then(|h| h.get("Stop"))
        .and_then(|s| s.as_array())
        .is_some_and(|stop_hooks| {
            stop_hooks.iter().any(|h| {
                h.get("hooks")
                    .and_then(|hs| hs.as_array())
                    .is_some_and(|hs| {
                        hs.iter().any(|cmd| {
                            cmd.get("command")
                                .and_then(|c| c.as_str())
                                .is_some_and(|s| s.contains("memo inject"))
                        })
                    })
            })
        });

    if already_installed {
        return Ok(false);
    }

    let memo_hook = serde_json::json!({
        "hooks": [{ "type": "command", "command": "memo inject --claude" }]
    });

    match root["hooks"]["Stop"].as_array_mut() {
        Some(arr) => arr.push(memo_hook),
        None => root["hooks"]["Stop"] = serde_json::json!([memo_hook]),
    }

    fs::write(&settings_path, serde_json::to_string_pretty(&root)?)?;
    Ok(true)
}

impl InjectBlock {
    fn empty() -> Self {
        Self { last_entry: None, todos: vec![], recent_tags: vec![], entry_count: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_render_text_empty() {
        let block = InjectBlock::empty();
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
        let val: serde_json::Value = serde_json::from_str(&block.render_json().unwrap()).unwrap();
        assert_eq!(val["entry_count"], 5);
        assert_eq!(val["recent_tags"][0], "bug");
    }

    #[test]
    fn test_patch_markdown_section_create_and_replace() {
        let dir = env::temp_dir().join(format!("memo_hooks_test_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("CLAUDE.md");

        patch_markdown_section(&path, "<!-- s -->", "<!-- e -->", "content\n").unwrap();
        let c = fs::read_to_string(&path).unwrap();
        assert!(c.contains("<!-- s -->") && c.contains("content"));

        // Idempotent
        patch_markdown_section(&path, "<!-- s -->", "<!-- e -->", "content\n").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap().matches("<!-- s -->").count(), 1);
    }

    #[test]
    fn test_write_to_claude_md() {
        let dir = env::temp_dir().join(format!("memo_hooks_write_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();

        let block = InjectBlock {
            last_entry: None,
            todos: vec![],
            recent_tags: vec!["refactor".to_string()],
            entry_count: 1,
        };

        write_to_claude_md(&block, &dir).unwrap();
        let content = fs::read_to_string(dir.join("CLAUDE.md")).unwrap();
        assert!(content.contains("<!-- memo:start -->"));
        assert!(content.contains("recent tags: refactor"));

        write_to_claude_md(&block, &dir).unwrap();
        assert_eq!(
            fs::read_to_string(dir.join("CLAUDE.md")).unwrap().matches("<!-- memo:start -->").count(),
            1
        );
    }
}
