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
    #[arg(help = "Name of the window to open or address, default \"flomotion\". Also FLOMOTION_WINDOW")]
    #[arg(long, global = true)]
    pub window: Option<String>,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Command {
    #[command(about = "Show whether the app is running and the page is connected")]
    Status,
    #[command(about = "Print the agent state: role, system prompt, tools and live context")]
    Agent {
        #[arg(help = "Start over from the bootstrap role, dropping the bound project")]
        #[arg(long)]
        reset: bool,
    },
    #[command(about = "Run one tool by name with a JSON input")]
    Act {
        #[arg(help = "Tool name as listed by the agent command")]
        name: String,
        #[arg(help = "JSON object with the tool input, defaults to {}")]
        input: Option<String>,
        #[arg(help = "Read the JSON input from a file instead")]
        #[arg(short = 'f', long)]
        input_file: Option<PathBuf>,
        #[arg(help = "Seconds to wait for a batch job before returning it pending, the page decides when omitted")]
        #[arg(long)]
        wait: Option<u64>,
        #[arg(help = "A local file the tool needs: its type is filled into the input, and the upload the tool hands back is performed and confirmed before returning")]
        #[arg(long)]
        file: Option<PathBuf>,
    },
    #[command(about = "Sign this window in: without a code prints a link for the user to approve, with the code the user got signs in")]
    Login {
        #[arg(help = "One-time code shown in the browser after approving the link")]
        #[arg(long)]
        code: Option<String>,
    },
    #[command(about = "Keep waiting for a batch job started by a previous act")]
    Job {
        #[arg(help = "Job id printed by act")]
        id: String,
        #[arg(help = "Seconds to wait before returning the pending state, the page decides when omitted")]
        #[arg(long)]
        wait: Option<u64>,
    },
    #[command(about = "Import a file into the open project: a STEP model into the focused workspace, or a MuJoCo MJCF zip as a new workspace")]
    Import {
        #[arg(help = "Path of the file")]
        path: PathBuf,
        #[arg(help = "How to hand the file to the app: auto (default), path or inline")]
        #[arg(long)]
        transport: Option<String>,
    },
    #[command(about = "Write an export of the focused item to disk")]
    Export {
        #[arg(help = "What to export: step, stl, assembly_step, gcode or kicad")]
        kind: String,
        #[arg(help = "Component id for part exports")]
        #[arg(long)]
        id: Option<String>,
        #[arg(help = "Directory to write into, defaults to a temp folder")]
        #[arg(short = 'o', long)]
        out: Option<PathBuf>,
    },
    #[command(about = "Print the window's live state and its recent command log, to see where a command stopped")]
    Diagnose,
    #[command(about = "Print the instructions an AI agent needs to drive FloMotion")]
    Skill,
}
