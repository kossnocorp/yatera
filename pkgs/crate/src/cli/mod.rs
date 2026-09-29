use crate::prelude::*;

#[derive(Cli)]
#[usage(bin = "tera", about = "Render a Tera v2 template from stdin to stdout")]
pub struct YtrCli {
    #[usage(flatten)]
    pub render: YtrCmdRender,
}

impl YtrCli {
    pub fn main() {
        YtrCli::parse().render.run().unwrap_or_else(|err| {
            eprintln!("Error: {:?}", err);
            std::process::exit(1);
        });
    }
}
