// Input: An opaque raw Role body.
// Output: Body text with full-line comment content removed, preserving newlines.

pub(super) fn clean_body(raw: &str) -> String {
    let mut body = String::with_capacity(raw.len());
    for line in raw.split_inclusive('\n') {
        if line.trim_start().starts_with('#') {
            if line.ends_with("\r\n") {
                body.push_str("\r\n");
            } else if line.ends_with('\n') {
                body.push('\n');
            }
        } else {
            body.push_str(line);
        }
    }
    body
}
