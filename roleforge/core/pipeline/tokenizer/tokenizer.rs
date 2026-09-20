// Input: A LoadedFile, already read into memory.
// Output: Ordered clean Roles, or a local tokenizer error.

use super::{TokenizeError, role_content, role_scanner};
use crate::core::pipeline::models::{CleanRole, LoadedFile, SourceInfo};

pub(crate) fn tokenize(file: &LoadedFile) -> Result<Vec<CleanRole>, TokenizeError> {
    let raw_roles = role_scanner::scan(&file.content)?;
    Ok(raw_roles
        .into_iter()
        .enumerate()
        .map(|(position, raw)| CleanRole {
            index: position + 1,
            name: raw.name.to_owned(),
            body: role_content::clean_body(raw.body),
            source: SourceInfo {
                declaration_line: raw.declaration_line,
            },
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn run(source: &str) -> Vec<CleanRole> {
        tokenize(&LoadedFile {
            content: source.to_owned(),
            path: PathBuf::from("not-read-by-tokenizer.rfg"),
        })
        .unwrap()
    }

    #[test]
    fn single_role_preserves_name_body_and_utf8_at_eof() {
        assert_eq!(
            run("@role Directory\n  שלום 🌍\n\tdata  "),
            vec![CleanRole {
                index: 1,
                name: "Directory".to_owned(),
                source: SourceInfo {
                    declaration_line: 1
                },
                body: "  שלום 🌍\n\tdata  ".to_owned(),
            }]
        );
    }

    #[test]
    fn declaration_lines_include_comments_and_blank_lines() {
        let roles = run("# heading\n\n@role First\nbody\n\n\n# comment\n@role Second\nbody");
        assert_eq!(roles[0].source.declaration_line, 3);
        assert_eq!(roles[1].source.declaration_line, 8);
    }

    #[test]
    fn multiple_roles_have_positional_indexes_and_allow_repeated_names() {
        let roles = run("@role lower_case\na\n@role PlayerInventory\nb\n@role lower_case\nc");
        assert_eq!(
            roles
                .iter()
                .map(|r| (r.index, r.name.as_str(), r.body.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (1, "lower_case", "a\n"),
                (2, "PlayerInventory", "b\n"),
                (3, "lower_case", "c")
            ]
        );
    }

    #[test]
    fn outside_and_indented_comments_do_not_create_roles() {
        let roles =
            run("# @role Fake\n  # outside\n\n@role Real\nx\n  # @role Fake\n@role Next\ny");
        assert_eq!(roles.len(), 2);
        assert_eq!(roles[0].name, "Real");
        assert_eq!(roles[0].body, "x\n\n");
        assert_eq!(roles[1].name, "Next");
    }

    #[test]
    fn comments_preserve_lf_crlf_and_existing_blank_lines() {
        let roles = run(
            "@role Sample\r\nabc\r\n# comment\r\n\t# comment\n\r\n\u{2003}# unicode indent\r\ndef\n",
        );
        assert_eq!(roles[0].body, "abc\r\n\r\n\n\r\n\r\ndef\n");
    }

    #[test]
    fn inline_hashes_and_role_specific_syntax_are_opaque() {
        let body =
            "value = 10 # inline comment\ncolor = #FF0000\n@end\n@roles Other\ntext @role Other\n";
        assert_eq!(run(&format!("@role Sample\n{body}"))[0].body, body);
    }

    #[test]
    fn empty_and_comment_only_sources_have_no_roles() {
        for source in ["", "\n \t\r\n", "# comment", " # one\r\n# two\n"] {
            assert!(run(source).is_empty());
        }
    }

    #[test]
    fn adjacent_declarations_and_declaration_at_eof_have_empty_bodies() {
        let roles = run("@role First\n@role Second");
        assert_eq!(roles.len(), 2);
        assert!(roles.iter().all(|role| role.body.is_empty()));
    }

    #[test]
    fn final_comment_without_newline_does_not_add_one() {
        assert_eq!(run("@role Sample\nabc\n # final")[0].body, "abc\n");
        assert_eq!(run("@role Sample\n# final")[0].body, "");
    }

    #[test]
    fn malformed_outer_source_returns_a_local_line_error() {
        for (source, expected) in [
            (
                "# heading\nstray content\n@role A",
                TokenizeError::ContentBeforeRole { line: 2 },
            ),
            (
                "@role A\nbody\n@role \n",
                TokenizeError::MissingRoleName { line: 3 },
            ),
        ] {
            let file = LoadedFile {
                content: source.to_owned(),
                path: PathBuf::new(),
            };
            assert_eq!(tokenize(&file), Err(expected));
        }
    }

    #[test]
    fn indented_declarations_before_first_role_are_content_errors() {
        for indent in ["    ", "\t"] {
            let file = LoadedFile {
                content: format!("# heading\n{indent}@role Directory\n@role Valid\n"),
                path: PathBuf::new(),
            };
            assert_eq!(
                tokenize(&file),
                Err(TokenizeError::ContentBeforeRole { line: 2 })
            );
        }
    }

    #[test]
    fn indented_declarations_inside_role_remain_opaque_content() {
        let body = "    @role Directory # body content\n\t@role Other\n";
        let roles = run(&format!("@role First\n{body}@role Second\n"));
        assert_eq!(roles.len(), 2);
        assert_eq!(roles[0].body, body);
        assert_eq!(roles[1].name, "Second");
    }

    #[test]
    fn declaration_comments_are_removed_without_changing_body_hashes() {
        for declaration in [
            "@role Directory # comment\n",
            "@role Directory      # comment\r\n",
        ] {
            let body = "value = 10 # inline\ncolor = #FF0000\n";
            let roles = run(&format!("    # indented comment\n{declaration}{body}"));
            assert_eq!(roles.len(), 1);
            assert_eq!(roles[0].name, "Directory");
            assert_eq!(roles[0].body, body);
        }
        assert_eq!(run("@role Directory # comment at EOF")[0].name, "Directory");
    }

    #[test]
    fn missing_names_including_comment_only_declarations_are_errors() {
        for declaration in ["@role", "@role\n", "@role \t\r\n", "@role # comment\n"] {
            let file = LoadedFile {
                content: format!("# heading\n{declaration}"),
                path: PathBuf::new(),
            };
            assert_eq!(
                tokenize(&file),
                Err(TokenizeError::MissingRoleName { line: 2 })
            );
        }
    }
}
