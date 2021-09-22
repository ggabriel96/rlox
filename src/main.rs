use clap::{Clap, AppSettings};


#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Path of script to run
    file: Option<String>,
}

fn main() {
    let opts: Opts = Opts::parse();

    println!("Value for path: {}", opts.file.or(Some(String::from("none"))).unwrap());
}
