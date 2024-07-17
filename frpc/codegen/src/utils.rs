use std::fmt;

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

pub fn uppercase_first(data: &str) -> String {
    let mut out = String::new();
    let mut first = true;
    for value in data.chars() {
        if first {
            out.push_str(&value.to_uppercase().to_string());
            first = false;
        } else {
            out.push(value);
        }
    }
    out
}

// fn capitalize_by(path: &str, sep: char) -> String {
//     let mut out = String::new();
//     let mut capitalize_next = true;
//     for ch in path.chars() {
//         if ch == sep {
//             capitalize_next = true;
//         } else if capitalize_next {
//             capitalize_next = false;
//             out.push(ch.to_ascii_uppercase());
//         } else {
//             out.push(ch);
//         }
//     }
//     out
// }

