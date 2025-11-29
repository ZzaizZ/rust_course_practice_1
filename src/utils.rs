pub(crate) fn strip_quotes(mut s: String) -> String {
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s.remove(0);
        s.pop();
    }
    s
}

pub(crate) fn wrap_with_quotes(s: &str) -> String {
    format!("\"{}\"", s)
}
