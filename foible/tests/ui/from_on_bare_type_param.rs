use foible::Trace;

#[derive(Debug, Trace)]
enum E<T> {
    #[from]
    Wrapped(T),
}

fn main() {}
