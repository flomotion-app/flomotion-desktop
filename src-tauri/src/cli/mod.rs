pub mod files;
pub mod imports;
pub mod launcher;
pub mod runner;
pub mod uploads;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "flomotion", version, about = "FloMotion desktop shell and CLI")]
pub struct Cli {
    /// Name of the window to open or address, default "flomotion". Also FLOMOTION_WINDOW.
    #[arg(long, global = true)]
    pub window: Option<String>,
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
        /// Seconds to wait for a batch job before returning it pending, the page decides when omitted
        #[arg(long)]
        wait: Option<u64>,
        /// A local file the tool needs: its type is filled into the input, and the
        /// upload the tool hands back is performed and confirmed before returning
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Sign this window in: without a code prints a link for the user to approve, with the code the user got signs in
    Login {
        /// One-time code shown in the browser after approving the link
        #[arg(long)]
        code: Option<String>,
    },
    /// Keep waiting for a batch job started by a previous act
    Job {
        /// Job id printed by act
        id: String,
        /// Seconds to wait before returning the pending state, the page decides when omitted
        #[arg(long)]
        wait: Option<u64>,
    },
    /// Import a file into the open project: a STEP model into the focused workspace, or a MuJoCo MJCF zip as a new workspace
    Import {
        /// Path of the file
        path: PathBuf,
        /// How to hand the file to the app: auto (default), path or inline
        #[arg(long)]
        transport: Option<String>,
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
