use foible::Trace;

#[derive(Debug, Trace)]
enum E {
    #[from]
    Unit,
}

fn main() {}
