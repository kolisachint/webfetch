//! Unified CLI: a single `webtools` binary exposing `fetch`, `search`, and an
//! `mcp` stdio server, the way `cargo`/`rg` ship one binary with many commands.

mod mcp;

use std::io::Read;

use clap::{Parser, Subcommand};

use webfetch::types::{ContentType, FetchOptions};
use websearch::types::SearchOptions;

#[derive(Parser)]
#[command(name = "webtools", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch a URL and convert it to token-efficient, reference-style output.
    Fetch {
        /// URL to fetch (and the base for resolving relative links).
        #[arg(long)]
        url: Option<String>,
        /// Read the body from a file (or `-` for stdin) instead of the
        /// network; pair with --url to set the base for relative links.
        #[arg(long)]
        from_file: Option<String>,
        /// Output format: text | markdown | structured.
        #[arg(long, default_value = "text")]
        output: String,
        /// Emit the full FetchResult as JSON.
        #[arg(long)]
        json: bool,
        /// Soft cap on output size, in estimated tokens.
        #[arg(long)]
        max_tokens: Option<usize>,
        /// Request timeout in seconds.
        #[arg(long, default_value_t = 10)]
        timeout: u64,
    },
    /// Search the web (DuckDuckGo Lite) with reference-style result URLs.
    Search {
        #[arg(long)]
        query: String,
        /// Maximum number of results to return.
        #[arg(long, default_value_t = 5)]
        max_results: usize,
        /// Emit the full SearchOutput as JSON.
        #[arg(long)]
        json: bool,
        /// Safe search: "on" or "off" (omit to use DDG's default).
        #[arg(long)]
        safe_search: Option<String>,
        /// Request timeout in seconds.
        #[arg(long, default_value_t = 10)]
        timeout: u64,
    },
    /// Run as an MCP stdio server exposing `fetch` and `search` as tools.
    Mcp,
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        // Concise, single-line error chain for a CLI — no backtrace dump.
        eprintln!("webtools: {err:#}");
        std::process::exit(1);
    }
}

fn read_input(from_file: &str) -> anyhow::Result<String> {
    if from_file == "-" {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        Ok(buf)
    } else {
        Ok(std::fs::read_to_string(from_file)?)
    }
}

fn parse_safe_search(value: Option<&str>) -> Option<bool> {
    match value.map(|s| s.to_ascii_lowercase()) {
        Some(ref s) if s == "on" || s == "strict" => Some(true),
        Some(ref s) if s == "off" || s == "none" => Some(false),
        _ => None,
    }
}

async fn run() -> anyhow::Result<()> {
    match Cli::parse().command {
        Commands::Fetch {
            url,
            from_file,
            output,
            json,
            max_tokens,
            timeout,
        } => {
            let base = url.clone().unwrap_or_default();
            let options = FetchOptions {
                url: base.clone(),
                content_type: ContentType::parse(&output),
                max_tokens,
                timeout_secs: timeout,
            };

            let result = match from_file {
                Some(path) => {
                    // Offline: convert a local/piped body (content-type sniffed).
                    let body = read_input(&path)?;
                    webfetch::convert_body(&body, &base, None, &options)
                }
                None => {
                    if base.is_empty() {
                        anyhow::bail!("provide --url, or --from-file to read a local body");
                    }
                    webfetch::fetch_and_convert(options).await?
                }
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                // A compact citation header in front of human-readable output.
                if !result.title.is_empty() {
                    println!("{}", result.title);
                }
                if !result.final_url.is_empty() {
                    println!("{}", result.final_url);
                }
                if !result.title.is_empty() || !result.final_url.is_empty() {
                    println!();
                }
                println!("{}", result.content);
            }
        }
        Commands::Search {
            query,
            max_results,
            json,
            safe_search,
            timeout,
        } => {
            let options = SearchOptions {
                query,
                max_results: Some(max_results),
                safe_search: parse_safe_search(safe_search.as_deref()),
                timeout_secs: timeout,
            };
            let output = websearch::run_search(options).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("{}", websearch::format_results(&output.results));
                let refs = websearch::render_references(&output.references);
                if !refs.is_empty() {
                    println!("\n{refs}");
                }
            }
        }
        Commands::Mcp => mcp::serve().await?,
    }
    Ok(())
}
