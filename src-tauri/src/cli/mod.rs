pub mod files;
pub mod launcher;
pub mod runner;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "flomotion", version, about = "FloMotion desktop shell and CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Command {
    /// Show whether the app is running and the page is connected
    Status,
    /// Print the agent state: role, system prompt, tools and live context
    Agent {
        /// Start over from the bootstrap role, dropping the bound project
        #[arg(long)]
        reset: bool,
    },
    /// Run one tool by name with a JSON input
    Act {
        /// Tool name as listed by the agent command
        name: String,
        /// JSON object with the tool input, defaults to {}
        input: Option<String>,
        /// Read the JSON input from a file instead
        #[arg(short = 'f', long)]
        input_file: Option<PathBuf>,
        /// Seconds to wait for a batch job before returning its pending state
        #[arg(long, default_value_t = 90)]
        wait: u64,
    },
    /// Keep waiting for a batch job started by a previous act
    Job {
        /// Job id printed by act
        id: String,
        /// Seconds to wait before returning the pending state
        #[arg(long, default_value_t = 90)]
        wait: u64,
    },
    /// Import a STEP model into the open workspace
    Import {
        /// Path of the .step or .stp file
        path: PathBuf,
    },
    /// Write an export of the focused item to disk
    Export {
        /// What to export: step, stl, assembly_step, gcode or kicad
        kind: String,
        /// Component id for part exports
        #[arg(long)]
        id: Option<String>,
        /// Directory to write into, defaults to a temp folder
        #[arg(short = 'o', long)]
        out: Option<PathBuf>,
    },
    /// Print the instructions an AI agent needs to drive FloMotion
    Skill,
}
