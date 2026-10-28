use foible::Trace;

#[derive(Trace)]
union NotAnEnum {
    a: u8,
}

fn main() {}
