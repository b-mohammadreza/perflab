use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about=None)]
#[command(propagate_version = true)]
pub struct CmdLine {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Runs the benchmark. Options: [--cpu <id>] [--perf] --bench <name>, --compiler <name> -- <flags...>
    Run {
        /// logical Linux CPU ID
        #[arg(long, value_name = "id")]
        cpu: Option<u16>,

        /// Collect perf stat counters (best-effort)
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        perf: bool,

        #[arg(short, long, value_name = "name")]
        bench: String,

        #[arg(short, long, value_name = "path")]
        compiler: String,

        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        compiler_args: Vec<String>,
    },
}
