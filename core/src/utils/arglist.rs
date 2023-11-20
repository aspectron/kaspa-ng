#[derive(Default)]
pub struct Arglist {
    pub args: Vec<String>,
}

impl Arglist {
    pub fn push(&mut self, arg: impl Into<String>) {
        self.args.push(arg.into());
    }
}

impl From<Arglist> for Vec<String> {
    fn from(arglist: Arglist) -> Self {
        arglist.args
    }
}
