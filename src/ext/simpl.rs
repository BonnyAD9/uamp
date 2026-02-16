use unidecode::unidecode_char;

pub fn new_str(s: impl AsRef<str>) -> String {
    let mut res = String::new();
    to_str(s, &mut res);
    res
}

pub fn to_str(s: impl AsRef<str>, out: &mut String) {
    for s in s
        .as_ref()
        .chars()
        .map(|c| unidecode_char(c).to_ascii_lowercase())
    {
        out.extend(s.chars().filter(|a| !a.is_ascii_whitespace()))
    }
}
