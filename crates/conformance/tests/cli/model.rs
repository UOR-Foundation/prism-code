//! The modeled command line, read from the LexLean sources.
//!
//! The expectations in these tests come from the model and nowhere else, so a
//! change to the model changes what the generated program is held to. The
//! reader understands only the literal shapes the sources use: records written
//! as `Name {field := value, ...}`, string literals with the escapes `\n`,
//! `\"` and `\\`, and hexadecimal byte literals. Anything else is an error that
//! names the place it stopped, because a reader that guessed would be a second
//! source of the model.

use std::collections::HashMap;
use std::path::Path;

/// One entry of the modeled exit-code map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// Lowercase name shown in the exit-code table.
    pub name: String,
    /// Bytes that begin every report of this outcome.
    pub prefix: String,
    /// Exit status for this outcome.
    pub exit_code: u8,
}

/// A titled block of modeled help text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    /// Single-line heading.
    pub title: String,
    /// Block text.
    pub body: String,
}

/// A named example script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Example {
    /// Name used as the `examples` operand.
    pub name: String,
    /// The script, printed unchanged.
    pub text: String,
}

/// One acceptance vector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vector {
    /// Request bytes.
    pub request: Vec<u8>,
    /// Modeled response bytes.
    pub response: Vec<u8>,
}

/// The command line the application model declares.
#[derive(Debug, Clone)]
pub struct Cli {
    /// Name of the generated program.
    pub cargo_name: String,
    /// Version the program reports.
    pub cargo_version: String,
    /// Version of the system model's product.
    pub product_version: String,
    /// Largest request, in bytes.
    pub request_maximum: usize,
    /// Subcommand that checks a request.
    pub command: String,
    /// Purpose paragraph, with its trailing newline.
    pub description: String,
    /// Exit status of every usage failure.
    pub usage_code: u8,
    /// Exit status of every input or output failure.
    pub io_code: u8,
    /// Exit status of a refused oversized request, when the model declares one.
    pub limit_code: Option<u8>,
    /// The exit-code map.
    pub outcomes: Vec<Outcome>,
    /// Help sections in declaration order.
    pub sections: Vec<Section>,
    /// Named examples in declaration order.
    pub examples: Vec<Example>,
    /// Acceptance vectors in declaration order.
    pub vectors: Vec<Vector>,
    /// Request prefixes the workbench frames its guided and raw requests with.
    pub frame_prefixes: Vec<String>,
}

impl Cli {
    /// The outcome whose prefix begins `report`.
    pub fn classify(&self, report: &[u8]) -> Option<&Outcome> {
        self.outcomes
            .iter()
            .find(|outcome| report.starts_with(outcome.prefix.as_bytes()))
    }

    /// Exit status of a request over the request maximum: the declared limit
    /// code, or that of the `error` outcome when the model declares none.
    pub fn limit_exit_code(&self) -> u8 {
        self.limit_code
            .unwrap_or_else(|| self.outcome("error").exit_code)
    }

    /// Whether the workbench frames this request: a modeled request prefix
    /// followed by U+001F. The page answers a framed request in its own JSON
    /// result protocol, so the command line has no report class for it.
    pub fn is_framed(&self, request: &[u8]) -> bool {
        self.frame_prefixes.iter().any(|prefix| {
            request
                .strip_prefix(prefix.as_bytes())
                .is_some_and(|rest| rest.first() == Some(&0x1f))
        })
    }

    /// The outcome with this name.
    pub fn outcome(&self, name: &str) -> &Outcome {
        self.outcomes
            .iter()
            .find(|outcome| outcome.name == name)
            .unwrap_or_else(|| panic!("the model declares no outcome named {name}"))
    }
}

/// The recovery text the workbench states for one code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recovery {
    /// The code token the report carries.
    pub code: String,
    /// What the code means.
    pub explanation: String,
    /// How to fix it.
    pub fix: String,
}

/// A workbench example and the acceptance vector it is drawn from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExampleSpec {
    /// Example name.
    pub name: String,
    /// Outcome the example demonstrates.
    pub outcome: String,
    /// Position of its acceptance vector, from 0.
    pub vector: usize,
}

/// The guidance the workbench shows beside the page.
#[derive(Debug, Clone)]
pub struct Guidance {
    /// Statement forms as `(usage, meaning)`.
    pub forms: Vec<(String, String)>,
    /// Status texts as `(name, text)`.
    pub statuses: Vec<(String, String)>,
    /// Recovery text for every error and finding code.
    pub recoveries: Vec<Recovery>,
    /// Examples and the vectors they come from.
    pub examples: Vec<ExampleSpec>,
}

struct Cursor<'a> {
    text: &'a str,
    at: usize,
}

impl<'a> Cursor<'a> {
    fn preview(&self) -> String {
        self.text[self.at..].chars().take(48).collect()
    }

    fn expect(&mut self, literal: &str) -> Result<(), String> {
        if self.text[self.at..].starts_with(literal) {
            self.at += literal.len();
            Ok(())
        } else {
            Err(format!("expected {literal:?} before {:?}", self.preview()))
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect("\"")?;
        let mut value = String::new();
        let mut chars = self.text[self.at..].char_indices();
        while let Some((offset, character)) = chars.next() {
            match character {
                '"' => {
                    self.at += offset + 1;
                    return Ok(value);
                }
                '\\' => match chars.next() {
                    Some((_, 'n')) => value.push('\n'),
                    Some((_, '"')) => value.push('"'),
                    Some((_, '\\')) => value.push('\\'),
                    other => return Err(format!("unsupported escape {other:?}")),
                },
                other => value.push(other),
            }
        }
        Err("unterminated string literal".to_owned())
    }

    fn identifier(&mut self) -> String {
        let name: String = self.text[self.at..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        self.at += name.len();
        name
    }

    fn number(&mut self) -> Result<u64, String> {
        let digits: String = self.text[self.at..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        let value = digits
            .parse()
            .map_err(|_| format!("expected a number before {:?}", self.preview()))?;
        self.at += digits.len();
        Ok(value)
    }

    fn bytes(&mut self) -> Result<Vec<u8>, String> {
        self.expect("0x")?;
        let digits: String = self.text[self.at..]
            .chars()
            .take_while(char::is_ascii_hexdigit)
            .collect();
        if digits.len() & 1 == 1 {
            return Err("a byte literal has an odd number of digits".to_owned());
        }
        self.at += digits.len();
        (0..digits.len())
            .step_by(2)
            .map(|index| {
                u8::from_str_radix(&digits[index..index + 2], 16).map_err(|e| e.to_string())
            })
            .collect()
    }
}

/// A cursor after the only occurrence of `marker`.
fn field<'a>(text: &'a str, marker: &str) -> Result<Cursor<'a>, String> {
    let count = text.matches(marker).count();
    if count != 1 {
        return Err(format!("{marker:?} occurs {count} times, expected once"));
    }
    let at = text
        .find(marker)
        .ok_or_else(|| format!("{marker:?} is absent"))?;
    Ok(Cursor {
        text,
        at: at + marker.len(),
    })
}

/// A cursor after every occurrence of `marker`, in source order.
fn records<'a>(text: &'a str, marker: &str) -> Vec<Cursor<'a>> {
    text.match_indices(marker)
        .map(|(at, _)| Cursor {
            text,
            at: at + marker.len(),
        })
        .collect()
}

fn small(value: u64, what: &str) -> Result<u8, String> {
    u8::try_from(value).map_err(|_| format!("{what} {value} exceeds UInt8"))
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn cli(root: &Path) -> Result<Cli, String> {
    let spec = read(&root.join("src/Specification.lex.tex"))?;
    let system = read(&root.join("src/System.lex.tex"))?;
    let guide = read(&root.join("src/Guide.lex.tex"))?;
    let usize_of = |value: u64| usize::try_from(value).map_err(|e| e.to_string());
    let mut limit = field(&spec, "limitCode := (ctor Option.")?;
    let limit_code = if limit.text[limit.at..].starts_with("none [UInt8])") {
        None
    } else {
        limit.expect("some [UInt8] (")?;
        Some(small(limit.number()?, "limitCode")?)
    };
    let mut outcomes = Vec::new();
    for mut at in records(&spec, "TextCliOutcome {name := ") {
        let name = at.string()?;
        at.expect(", prefix := ")?;
        let prefix = at.string()?;
        at.expect(", exitCode := (")?;
        let exit_code = small(at.number()?, "exitCode")?;
        outcomes.push(Outcome {
            name,
            prefix,
            exit_code,
        });
    }
    let mut sections = Vec::new();
    for mut at in records(&spec, "TextCliSection {title := ") {
        let title = at.string()?;
        at.expect(", body := ")?;
        sections.push(Section {
            title,
            body: at.string()?,
        });
    }
    let mut examples = Vec::new();
    for mut at in records(&spec, "TextCliExample {name := ") {
        let name = at.string()?;
        at.expect(", text := ")?;
        examples.push(Example {
            name,
            text: at.string()?,
        });
    }
    let mut vectors = Vec::new();
    for mut at in records(&spec, "AcceptanceVector {request := ") {
        let request = at.bytes()?;
        at.expect(", response := ")?;
        vectors.push(Vector {
            request,
            response: at.bytes()?,
        });
    }
    let mut frame_prefixes = Vec::new();
    for mut at in records(&guide, "requestPrefix := ") {
        frame_prefixes.push(at.string()?);
    }
    Ok(Cli {
        cargo_name: field(&spec, "cargoName := ")?.string()?,
        cargo_version: field(&spec, "cargoVersion := ")?.string()?,
        product_version: field(&system, "productVersion := ")?.string()?,
        request_maximum: usize_of(field(&spec, "requestMaximum := (")?.number()?)?,
        command: field(&spec, "command := ")?.string()?,
        description: field(&spec, "description := ")?.string()?,
        usage_code: small(field(&spec, "usageCode := (")?.number()?, "usageCode")?,
        io_code: small(field(&spec, "ioCode := (")?.number()?, "ioCode")?,
        limit_code,
        outcomes,
        sections,
        examples,
        vectors,
        frame_prefixes,
    })
}

/// Read the modeled command line from `root`, panicking with the reason when
/// the sources are not in the shape this reader understands.
pub fn load(root: &Path) -> Cli {
    cli(root).unwrap_or_else(|error| panic!("reading the modeled command line: {error}"))
}

/// The constructor a match arm names: the last path segment of its first token.
fn constructor(arm: &str) -> Option<&str> {
    let token = arm.trim().strip_prefix('|')?.split_whitespace().next()?;
    token.rsplit('.').next()
}

/// The text of the definition called `name`, from its `name:` line to its
/// closing brace.
fn definition<'a>(text: &'a str, name: &str) -> Result<&'a str, String> {
    let marker = format!("  name: {name},\n");
    let start = text
        .find(&marker)
        .ok_or_else(|| format!("no definition named {name}"))?;
    let body = &text[start..];
    let end = body
        .find("\n}\n")
        .ok_or_else(|| format!("definition {name} is not closed"))?;
    Ok(&body[..end])
}

fn guidance(root: &Path) -> Result<Guidance, String> {
    let workbench = read(&root.join("src/Workbench.lex.tex"))?;
    let guide = read(&root.join("src/Guide.lex.tex"))?;
    let engine = read(&root.join("src/Engine.lex.tex"))?;
    let mut forms = Vec::new();
    for mut at in records(&workbench, "StatementForm {usage := ") {
        let usage = at.string()?;
        at.expect(", meaning := ")?;
        forms.push((usage, at.string()?));
    }
    let mut statuses = Vec::new();
    for mut at in records(&workbench, "StatusText {name := ") {
        let name = at.string()?;
        at.expect(", text := (Guide::")?;
        let source = at.identifier();
        at.expect(")")?;
        let marker = format!("name: {source},");
        let start = guide
            .find(&marker)
            .ok_or_else(|| format!("Guide has no definition named {source}"))?;
        let body = guide[start..]
            .find(" body: ")
            .ok_or_else(|| format!("Guide definition {source} has no body"))?;
        let mut text = Cursor {
            text: &guide,
            at: start + body + " body: ".len(),
        };
        statuses.push((name, text.string()?));
    }
    let mut examples = Vec::new();
    for mut at in records(&workbench, "ExampleSpec {name := ") {
        let name = at.string()?;
        at.expect(", outcome := ")?;
        let outcome = at.string()?;
        at.expect(", vector := ")?;
        examples.push(ExampleSpec {
            name,
            outcome,
            vector: usize::try_from(at.number()?).map_err(|e| e.to_string())?,
        });
    }
    // A code is a match arm, inside one of the definitions that name codes,
    // whose whole result is a string literal. A recovery is a match arm whose
    // result is a `recovery` call with two literals.
    let mut codes: HashMap<String, String> = HashMap::new();
    for (source, name) in [
        (&engine, "parseErrorCode"),
        (&engine, "violationCode"),
        (&engine, "stepErrorCode"),
        (&workbench, "requestFaultCode"),
        (&workbench, "findingCode"),
    ] {
        for line in definition(source, name)?.lines() {
            let Some((left, right)) = line.split_once(" => ") else {
                continue;
            };
            let Some(constructor) = constructor(left) else {
                continue;
            };
            let mut at = Cursor { text: right, at: 0 };
            if right.starts_with('"') {
                let token = at.string()?;
                let closing = at.text[at.at..].trim_matches(|c| c == ')' || c == ',' || c == ' ');
                if !closing.is_empty() || token.is_empty() {
                    return Err(format!("{name}: {constructor} does not map to a code"));
                }
                codes.insert(constructor.to_owned(), token);
            }
        }
    }
    let mut arms: Vec<(String, String, String)> = Vec::new();
    for line in workbench.lines() {
        let Some((left, right)) = line.split_once(" => (recovery (") else {
            continue;
        };
        let Some(name) = constructor(left) else {
            continue;
        };
        let (_, rest) = right
            .split_once(')')
            .ok_or_else(|| format!("{name}: recovery call has no code"))?;
        let mut at = Cursor { text: rest, at: 0 };
        at.expect(" ")?;
        let explanation = at.string()?;
        at.expect(" ")?;
        arms.push((name.to_owned(), explanation, at.string()?));
    }
    let mut recoveries = Vec::new();
    for (name, explanation, fix) in arms {
        let code = codes
            .get(&name)
            .ok_or_else(|| format!("{name} has recovery text but no code"))?;
        recoveries.push(Recovery {
            code: code.clone(),
            explanation,
            fix,
        });
    }
    Ok(Guidance {
        forms,
        statuses,
        recoveries,
        examples,
    })
}

/// Read the workbench guidance from `root`, panicking with the reason when the
/// sources are not in the shape this reader understands.
pub fn load_guidance(root: &Path) -> Guidance {
    guidance(root).unwrap_or_else(|error| panic!("reading the workbench guidance: {error}"))
}
