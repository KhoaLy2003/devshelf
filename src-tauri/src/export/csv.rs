//! RFC 4180 writer for the export. Values are written exactly as stored: quoting is only what the
//! format requires, and nothing is rewritten (no spreadsheet formula escaping).

use zeroize::Zeroizing;

/// A new file starts with the UTF-8 byte order mark; Excel on Windows needs it to read accents.
pub fn new_csv() -> Zeroizing<Vec<u8>> {
    Zeroizing::new(vec![0xEF, 0xBB, 0xBF])
}

/// Appends one row ending in CRLF. A field is quoted when it holds a comma, quote, CR or LF.
pub fn write_row(out: &mut Vec<u8>, fields: &[&str]) {
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            out.push(b',');
        }
        write_field(out, field);
    }
    out.extend_from_slice(b"\r\n");
}

/// Encodes straight into `out`, so no intermediate copy of a plaintext value is made.
fn write_field(out: &mut Vec<u8>, field: &str) {
    if !field.contains([',', '"', '\n', '\r']) {
        out.extend_from_slice(field.as_bytes());
        return;
    }
    out.push(b'"');
    let mut utf8 = [0u8; 4];
    for ch in field.chars() {
        if ch == '"' {
            out.push(b'"');
        }
        out.extend_from_slice(ch.encode_utf8(&mut utf8).as_bytes());
    }
    out.push(b'"');
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(fields: &[&str]) -> String {
        let mut out = Vec::new();
        write_row(&mut out, fields);
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn plain_and_empty_fields_are_not_quoted() {
        assert_eq!(render(&["a", "", "b c"]), "a,,b c\r\n");
    }

    #[test]
    fn comma_quote_and_line_breaks_are_quoted_and_inner_quotes_doubled() {
        assert_eq!(
            render(&["x,y", "say \"hi\"", "one\ntwo", "a\r\nb"]),
            "\"x,y\",\"say \"\"hi\"\"\",\"one\ntwo\",\"a\r\nb\"\r\n"
        );
    }

    #[test]
    fn values_are_written_unchanged_without_formula_escaping() {
        assert_eq!(render(&["=1+1"]), "=1+1\r\n");
    }

    #[test]
    fn non_ascii_text_is_written_as_utf8() {
        assert_eq!(render(&["Café, naïve"]), "\"Café, naïve\"\r\n");
    }

    #[test]
    fn new_file_starts_with_the_utf8_bom() {
        assert_eq!(new_csv().as_slice(), &[0xEF, 0xBB, 0xBF]);
    }
}
