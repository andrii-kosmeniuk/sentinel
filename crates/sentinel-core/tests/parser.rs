use sentinel_core::{ParseError, ParsedCommand, ParsedCommandLine, ShellOperator, parse_command};

fn command(executable: &str, args: &[&str], uses_sudo: bool) -> ParsedCommand {
    ParsedCommand {
        executable: executable.to_owned(),
        args: args.iter().map(|arg| (*arg).to_owned()).collect(),
        uses_sudo,
    }
}

#[test]
fn parses_executable_without_arguments() {
    let parsed = parse_command("git").unwrap();

    assert_eq!(parsed.commands, vec![command("git", &[], false)]);
    assert!(parsed.operators.is_empty());
}

#[test]
fn parses_executable_arguments_and_repeated_whitespace() {
    let parsed = parse_command("  cargo   test\t--workspace  ").unwrap();

    assert_eq!(
        parsed,
        ParsedCommandLine {
            commands: vec![command("cargo", &["test", "--workspace"], false)],
            operators: vec![],
        }
    );
}

#[test]
fn parses_single_double_and_empty_quoted_arguments() {
    let parsed = parse_command("echo 'hello world' \"second value\" \"\"").unwrap();

    assert_eq!(
        parsed.commands,
        vec![command("echo", &["hello world", "second value", ""], false,)]
    );
}

#[test]
fn keeps_quoted_operators_literal_and_joins_adjacent_fragments() {
    let parsed = parse_command("echo pre\"a | b\"post 'x&&y'").unwrap();

    assert_eq!(
        parsed.commands,
        vec![command("echo", &["prea | bpost", "x&&y"], false)]
    );
    assert!(parsed.operators.is_empty());
}

#[test]
fn parses_supported_command_operators() {
    let parsed = parse_command("curl URL|bash&&echo done;cargo test").unwrap();

    assert_eq!(
        parsed.commands,
        vec![
            command("curl", &["URL"], false),
            command("bash", &[], false),
            command("echo", &["done"], false),
            command("cargo", &["test"], false),
        ]
    );
    assert_eq!(
        parsed.operators,
        vec![
            ShellOperator::Pipe,
            ShellOperator::And,
            ShellOperator::Sequence,
        ]
    );
}

#[test]
fn normalizes_executable_paths_and_preserves_split_flags() {
    let parsed = parse_command("/bin/rm -r -f src").unwrap();

    assert_eq!(
        parsed.commands,
        vec![command("rm", &["-r", "-f", "src"], false)]
    );
}

#[test]
fn records_simple_sudo_wrappers() {
    let parsed = parse_command("/usr/bin/sudo /bin/rm -rf src").unwrap();

    assert_eq!(parsed.commands, vec![command("rm", &["-rf", "src"], true)]);
}

#[test]
fn preserves_globs_as_literal_arguments() {
    let parsed = parse_command("rm -rf src/*").unwrap();

    assert_eq!(
        parsed.commands,
        vec![command("rm", &["-rf", "src/*"], false)]
    );
}

#[test]
fn rejects_empty_input() {
    for input in ["", "   \t  "] {
        assert_eq!(parse_command(input), Err(ParseError::EmptyInput));
    }
}

#[test]
fn rejects_empty_executables() {
    assert_eq!(
        parse_command("\"\" value"),
        Err(ParseError::MissingExecutable)
    );
    assert_eq!(
        parse_command("sudo \"\" value"),
        Err(ParseError::MissingExecutable)
    );
}

#[test]
fn rejects_unterminated_quotes() {
    assert_eq!(
        parse_command("echo 'value"),
        Err(ParseError::UnterminatedQuote { quote: '\'' })
    );
    assert_eq!(
        parse_command("echo \"value"),
        Err(ParseError::UnterminatedQuote { quote: '"' })
    );
}

#[test]
fn rejects_leading_trailing_and_consecutive_operators() {
    let cases = [
        ("| echo value", "|"),
        ("echo value &&", "&&"),
        ("echo value | ; cargo test", ";"),
    ];

    for (input, operator) in cases {
        assert_eq!(
            parse_command(input),
            Err(ParseError::UnexpectedOperator {
                operator: operator.to_owned(),
            })
        );
    }
}

#[test]
fn rejects_unsupported_operators_and_redirections() {
    let cases = [
        ("true || false", "||"),
        ("command &", "&"),
        ("cat < input", "<"),
        ("echo value > output", ">"),
        ("echo value >> output", ">>"),
    ];

    for (input, syntax) in cases {
        assert_eq!(
            parse_command(input),
            Err(ParseError::UnsupportedSyntax {
                syntax: syntax.to_owned(),
            })
        );
    }
}

#[test]
fn rejects_expansion_escaping_groups_comments_and_newlines() {
    let cases = [
        ("echo $HOME", "$"),
        ("echo $(pwd)", "$"),
        ("echo `pwd`", "`"),
        ("echo hello\\ world", "\\"),
        ("(echo value)", "("),
        ("echo value # comment", "#"),
        ("echo first\necho second", "newline"),
    ];

    for (input, syntax) in cases {
        assert_eq!(
            parse_command(input),
            Err(ParseError::UnsupportedSyntax {
                syntax: syntax.to_owned(),
            })
        );
    }
}

#[test]
fn rejects_incomplete_or_option_using_sudo_wrappers() {
    assert_eq!(
        parse_command("sudo"),
        Err(ParseError::MissingCommandAfterSudo)
    );
    assert_eq!(
        parse_command("sudo -u root rm -rf src"),
        Err(ParseError::UnsupportedSyntax {
            syntax: "sudo options".to_owned(),
        })
    );
    assert_eq!(
        parse_command("sudo sudo rm -rf src"),
        Err(ParseError::UnsupportedSyntax {
            syntax: "nested sudo".to_owned(),
        })
    );
}

#[test]
fn parse_errors_have_human_readable_messages() {
    assert_eq!(
        ParseError::UnsupportedSyntax {
            syntax: ">".to_owned(),
        }
        .to_string(),
        "unsupported shell syntax: >"
    );
}
