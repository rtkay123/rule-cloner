use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to `toml` configuration file
    #[arg(short, long, value_name = "FILE")]
    pub config: PathBuf,

    /// Where to output artifacts
    #[arg(short, long, value_name = "output")]
    pub output: PathBuf,
}

impl Cli {
    pub fn dest_folder(&self, val: &str) -> PathBuf {
        let mut path = self.output.to_path_buf();
        path.push(val);
        path
    }
}
