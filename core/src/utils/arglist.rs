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
        let mut unique_elements = AHashSet::new();
        let mut list = vec![];
        for arg in arglist.args.into_iter() {
            if unique_elements.insert(arg.clone()) {
                list.push(arg);
            }
        }
        list
    }
}
