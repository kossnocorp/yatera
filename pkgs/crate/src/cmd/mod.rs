use crate::prelude::*;

mod render;
use render::*;

#[derive(Subcommands)]
#[usage(run)]
pub enum YtrCmd {
    /// Renders Tera template code
    Render(YtrCmdRender),
}
