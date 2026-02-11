use clap::Parser;
use perflab::CmdLine;

fn main() {
    let cl = CmdLine::parse();

    perflab::execute(cl.command);
}
