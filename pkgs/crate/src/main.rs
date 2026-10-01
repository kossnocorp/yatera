mod prelude;

mod cli;
pub(crate) use cli::*;

mod cmd;
pub(crate) use cmd::*;

fn main() {
    YtrCli::main();
}
