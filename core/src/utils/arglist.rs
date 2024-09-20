use ahash::AHashSet;

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
        let mut args = AHashSet::new();
        for arg in arglist.args.into_iter() {
            args.insert(arg);
        }
        let mut list = args.into_iter().collect::<Vec<_>>();
        list.sort();
        list
    }
}
