pub(crate) fn parse_quoted_field(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

pub(crate) fn wrap_with_quotes(s: &str) -> String {
    format!("\"{}\"", s)
}
