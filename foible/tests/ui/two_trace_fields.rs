use foible::Trace;

#[derive(Debug, Trace)]
enum Leaf {
    Gone,
}

#[derive(Debug, Trace)]
enum E {
    Two(Leaf, Leaf),
}

fn main() {}
