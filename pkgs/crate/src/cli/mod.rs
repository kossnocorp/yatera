use crate::prelude::*;

#[derive(Cli)]
#[usage(
    run,
    bin = "tera",
    about = "Yet Another Tera CLI",
    default_subcommand = "render"
)]
pub struct YtrCli {
    #[usage(subcommand)]
    pub command: YtrCmd,
}

impl YtrCli {
    pub fn main() {
        YtrCli::parse().run().unwrap_or_else(|err| {
            eprintln!("Error: {:?}", err);
            std::process::exit(1);
        });
    }
}
