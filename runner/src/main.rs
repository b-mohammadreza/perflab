use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about=None)]
#[command(propagate_version=true)]
struct CmdLine {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Runs the benchmark. Options: --bench <name>, --compiler <name> -- <flags...>
    Run {
        #[arg(short, long, value_name = "name")]
        bench: String,

        #[arg(short, long, value_name = "path")]
        compiler: String,

        #[arg(trailing_var_arg=true, allow_hyphen_values=true)]
        compiler_args: Vec<String>,
    }
}

fn main() {
    let cl = CmdLine::parse();

    match cl.command {
        Commands::Run {bench, compiler, compiler_args} => { 
            println!("WIP - bench<{bench}>, compiler<{compiler}>, flags<{:?}>", compiler_args);
        }
    }
}
