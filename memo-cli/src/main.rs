use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use memo_core::Store;
use memo_hooks::{setup, write_to_claude_md, InjectBlock};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "memo", version, about = "Persistent memory for AI agents")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize project memory
    Init,

    /// Save a memory entry
    Log {
        /// Message to log. Use "-" to read from stdin.
        message: String,
        #[arg(long, action = clap::ArgAction::Append)]
        tag: Vec<String>,
    },

    /// Search memory entries
    Search {
        /// Query string to search for in entry content
        query: String,
    },

    /// Print context block for injection at session start
    Inject {
        /// Write block into CLAUDE.md instead of stdout
        #[arg(long)]
        claude: bool,

        /// Output format
        #[arg(long, value_name = "FORMAT", default_value = "text")]
        format: String,

        /// Limit to entries newer than this duration (e.g. 1d, 7d, 24h, 1w)
        #[arg(long, value_name = "DURATION")]
        since: Option<String>,
    },

    /// List recent memory entries
    List {
        /// Show all entries (default: last 10)
        #[arg(long)]
        all: bool,

        /// Filter entries by tag
        #[arg(long)]
        tag: Option<String>,
    },

    /// Delete a memory entry by id
    Delete {
        /// Entry ID to delete
        id: i64,
    },

    /// List all tags with usage counts
    Tags,

    /// Clear all memory for current project
    Clear {
        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Show memory statistics
    Stats,

    /// Set up memo for Claude Code (writes CLAUDE.md + installs Stop hook)
    Setup,
}

fn project_dir() -> Result<PathBuf> {
    std::env::current_dir().context("cannot determine current directory")
}

/// Parse duration strings like "1d", "7d", "24h", "1w" into chrono::Duration.
fn parse_duration(s: &str) -> Result<chrono::Duration> {
    if let Some(num_str) = s.strip_suffix('d') {
        let days: i64 = num_str
            .parse()
            .with_context(|| format!("invalid duration: {}", s))?;
        return Ok(chrono::Duration::days(days));
    }
    if let Some(num_str) = s.strip_suffix('h') {
        let hours: i64 = num_str
            .parse()
            .with_context(|| format!("invalid duration: {}", s))?;
        return Ok(chrono::Duration::hours(hours));
    }
    if let Some(num_str) = s.strip_suffix('w') {
        let weeks: i64 = num_str
            .parse()
            .with_context(|| format!("invalid duration: {}", s))?;
        return Ok(chrono::Duration::weeks(weeks));
    }
    anyhow::bail!(
        "invalid duration format '{}': use e.g. 1d, 7d, 24h, 1w",
        s
    )
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let dir = project_dir()?;

    match cli.command {
        Command::Init => {
            let store = Store::open(&dir)?;
            println!("memo initialized for project {}", &store.project_id[..8]);
            println!("db: ~/.local/share/memo/{}.db", store.project_id);
            println!();
            println!("Add the following to your project's CLAUDE.md to auto-inject context:");
            println!();
            println!("```");
            println!("<!-- memo:start -->");
            println!("<!-- memo:end -->");
            println!("```");
            println!();
            println!("Or run `memo inject --claude` to write it automatically.");
        }

        Command::Log { message, tag } => {
            let message = if message == "-" {
                use std::io::Read;
                let mut buf = String::new();
                std::io::stdin()
                    .read_to_string(&mut buf)
                    .context("failed to read from stdin")?;
                let trimmed = buf.trim().to_string();
                anyhow::ensure!(!trimmed.is_empty(), "stdin message was empty");
                trimmed
            } else {
                message
            };
            let store = Store::open(&dir)?;
            store.save(&message, &tag)?;
            println!("logged: {}", message);
        }

        Command::Inject { claude, format, since } => {
            let store = Store::open(&dir)?;
            let block = if let Some(since_str) = since {
                let duration = parse_duration(&since_str)?;
                let since_dt = chrono::Utc::now() - duration;
                InjectBlock::build_since(&store, since_dt)?
            } else {
                InjectBlock::build(&store)?
            };

            if claude {
                write_to_claude_md(&block, &dir)?;
                println!("memo context written to CLAUDE.md");
            } else {
                match format.as_str() {
                    "json" => println!("{}", block.render_json()?),
                    _ => print!("{}", block.render_text()),
                }
            }
        }

        Command::List { all, tag } => {
            let store = Store::open(&dir)?;
            let limit = if all { None } else { Some(10) };
            let entries = if let Some(t) = tag {
                store.list_by_tag(&t, limit)?
            } else {
                store.list(limit)?
            };

            if entries.is_empty() {
                println!("no entries yet. run `memo log \"<message>\"` to save one.");
                return Ok(());
            }

            for entry in &entries {
                let date = entry.timestamp.format("%Y-%m-%d %H:%M");
                let tags = if entry.tags.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", entry.tags.join(", "))
                };
                println!("#{} {} — {}{}", entry.id, date, entry.content, tags);
            }
        }

        Command::Delete { id } => {
            let store = Store::open(&dir)?;
            if store.delete(id)? {
                println!("deleted entry #{}", id);
            } else {
                println!("entry #{} not found", id);
            }
        }

        Command::Tags => {
            let store = Store::open(&dir)?;
            let tags = store.all_tags()?;
            if tags.is_empty() {
                println!("no tags yet");
                return Ok(());
            }
            for (tag, count) in &tags {
                println!("{:<20} {}", tag, count);
            }
        }

        Command::Search { query } => {
            let store = Store::open(&dir)?;
            let entries = store.search(&query)?;

            if entries.is_empty() {
                println!("no entries found for query: {}", query);
                return Ok(());
            }

            for entry in &entries {
                let date = entry.timestamp.format("%Y-%m-%d %H:%M");
                let tags = if entry.tags.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", entry.tags.join(", "))
                };
                println!("#{} {} — {}{}", entry.id, date, entry.content, tags);
            }
        }

        Command::Clear { yes } => {
            if !yes {
                eprint!("clear all memory for this project? [y/N] ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("aborted");
                    return Ok(());
                }
            }
            let store = Store::open(&dir)?;
            let n = store.clear()?;
            println!("cleared {} entries", n);
        }

        Command::Setup => {
            let result = setup(&dir)?;
            println!("✓ CLAUDE.md updated with memo instructions and context block");
            if result.hook_installed {
                println!("✓ Stop hook installed in .claude/settings.json");
                println!("  → memo inject --claude will run automatically at end of each session");
            } else {
                println!("  Stop hook already present, skipped");
            }
            println!();
            println!("Run `memo log \"<message>\"` to start logging.");
        }

        Command::Stats => {
            let store = Store::open(&dir)?;
            let count = store.count()?;
            let tags = store.recent_tags(20)?;
            let block = InjectBlock::build(&store)?;
            // Rough token estimate: chars in inject block / 4
            let tokens_saved = block.render_text().len() / 4;
            println!("project:      {}", &store.project_id[..8]);
            println!("entries:      {}", count);
            println!("tokens saved: ~{}", tokens_saved);
            if !tags.is_empty() {
                println!("top tags:     {}", tags.join(", "));
            }
        }
    }

    Ok(())
}
