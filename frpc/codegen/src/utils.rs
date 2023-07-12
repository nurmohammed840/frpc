use std::fmt;

// Todo: Use `&Vec<String>` instade of `&str`
pub fn write_doc_comments(f: &mut impl fmt::Write, lines: &str) -> fmt::Result {
    if lines.is_empty() {
        return Ok(());
    }
    writeln!(f, "/**")?;
    for line in lines.trim().lines() {
        writeln!(f, " * {line}")?;
    }
    writeln!(f, " */")
}

pub fn join(strings: impl Iterator<Item = String>, separator: &str) -> String {
    let mut string = String::new();
    let mut first = true;
    for s in strings {
        if first {
            first = false;
        } else {
            string.push_str(separator);
        }
        string.push_str(&s);
    }
    string
}

pub fn object_ident_from(path: &str) -> String {
    let mut out = String::new();

    let mut idents = path.split("::");
    idents.next();

    let mut sep_next = false;
    for mut ident in idents {
        if let Some(rest) = ident.strip_prefix("r#") {
            ident = rest;
        }
        if sep_next {
            out += "_";
        } else {
            sep_next = true;
        }
        out += &capitalize_by(ident, '_');
    }
    out
}

fn capitalize_by(path: &str, sep: char) -> String {
    let mut out = String::new();
    let mut capitalize_next = true;
    for ch in path.chars() {
        if ch == sep {
            capitalize_next = true;
        } else if capitalize_next {
            capitalize_next = false;
            out.push(ch.to_ascii_uppercase());
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_object_name_from_path() {
        assert_eq!(
            object_ident_from("let::r#use::new::class_name"),
            "Use_New_ClassName"
        );
    }
}
