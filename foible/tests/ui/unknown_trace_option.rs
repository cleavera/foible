use foible::Trace;

#[derive(Debug, Trace)]
#[trace(erorr)]
enum E {
    A,
}

fn main() {}
