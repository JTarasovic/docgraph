use clap::Parser;

#[derive(Parser)]
#[command(
    name = "docgraph",
    version,
    about = "Repository-native document graphs"
)]
struct Cli;

fn main() {
    Cli::parse();
}
