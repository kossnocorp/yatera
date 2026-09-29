use crate::prelude::*;

#[derive(Args, Debug)]
pub struct YtrCmdRender {
    #[usage()]
    pub code: String,
    /// Positional values or name=value pairs for variables
    #[usage()]
    pub arguments: Vec<String>,
}

impl Run for YtrCmdRender {
    type Output = Result<()>;

    fn run(self) -> Self::Output {
        Ok(())
    }
}
