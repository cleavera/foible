use foible::Trace;

#[derive(Debug, Trace)]
enum E {
    #[from]
    Two(u8, u8),
}

fn main() {}
