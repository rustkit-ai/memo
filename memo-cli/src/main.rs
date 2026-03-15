use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use memo_core::{Entry, Store};
use memo_hooks::{setup, write_to_claude_md, InjectBlock};
use std::io::Read;
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
        query: String,
    },

    /// Print context block for injection at session start
    Inject {
        /// Write block into CLAUDE.md instead of stdout
        #[arg(long)]
        claude: bool,

        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,

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
    Delete { id: i64 },

    /// List all tags with usage counts
    Tags,

    /// Clear all memory for current project
    Clear {
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Show memory statistics
    Stats,

    /// Set up memo for Claude Code (writes CLAUDE.md + installs Stop hook)
    Setup,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

fn project_dir() -> Result<PathBuf> {
    std::env::current_dir().context("cannot determine current directory")
}

fn parse_duration(s: &str) -> Result<chrono::Duration> {
    let (num_str, unit) = s.split_at(s.len().saturating_sub(1));
    let n: i64 = num_str
        .parse()
        .with_context(|| format!("invalid duration: {s}"))?;
    match unit {
        "d" => Ok(chrono::Duration::days(n)),
        "h" => Ok(chrono::Duration::hours(n)),
        "w" => Ok(chrono::Duration::weeks(n)),
        _ => anyhow::bail!("invalid duration format '{s}': use e.g. 1d, 7d, 24h, 1w"),
    }
}

fn print_entry(e: &Entry) {
    let tags = if e.tags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", e.tags.join(", "))
    };
    println!("#{} {} — {}{}", e.id, e.timestamp.format("%Y-%m-%d %H:%M"), e.content, tags);
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
            println!("Add the following to your project's CLAUDE.md:");
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
            println!("logged: {message}");
        }

        Command::Inject { claude, format, since } => {
            let store = Store::open(&dir)?;
            let block = match since {
                Some(s) => InjectBlock::build_since(&store, chrono::Utc::now() - parse_duration(&s)?)?,
                None => InjectBlock::build(&store)?,
            };

            if claude {
                write_to_claude_md(&block, &dir)?;
                println!("memo context written to CLAUDE.md");
            } else {
                match format {
                    OutputFormat::Json => println!("{}", block.render_json()?),
                    OutputFormat::Text => print!("{}", block.render_text()),
                }
            }
        }

        Command::List { all, tag } => {
            let store = Store::open(&dir)?;
            let limit = if all { None } else { Some(10) };
            let entries = match tag {
                Some(t) => store.list_by_tag(&t, limit)?,
                None => store.list(limit)?,
            };

            if entries.is_empty() {
                println!("no entries yet. run `memo log \"<message>\"` to save one.");
                return Ok(());
            }
            entries.iter().for_each(print_entry);
        }

        Command::Delete { id } => {
            let store = Store::open(&dir)?;
            if store.delete(id)? {
                println!("deleted entry #{id}");
            } else {
                println!("entry #{id} not found");
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
                println!("{tag:<20} {count}");
            }
        }

        Command::Search { query } => {
            let store = Store::open(&dir)?;
            let entries = store.search(&query)?;
            if entries.is_empty() {
                println!("no entries found for query: {query}");
                return Ok(());
            }
            entries.iter().for_each(print_entry);
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
            println!("cleared {} entries", store.clear()?);
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
            let block = InjectBlock::build(&store)?;
            println!("project:      {}", &store.project_id[..8]);
            println!("entries:      {}", block.entry_count);
            println!("tokens saved: ~{}", block.render_text().len() / 4);
            if !block.recent_tags.is_empty() {
                println!("top tags:     {}", block.recent_tags.join(", "));
            }
        }
    }

    Ok(())
}
