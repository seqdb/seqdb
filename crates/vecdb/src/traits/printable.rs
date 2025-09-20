pub trait PrintableIndex {
    fn to_string() -> &'static str;
    fn to_possible_strings() -> &'static [&'static str];
}

impl PrintableIndex for usize {
    fn to_string() -> &'static str {
        "usize"
    }

    fn to_possible_strings() -> &'static [&'static str] {
        &["usize"]
    }
}
