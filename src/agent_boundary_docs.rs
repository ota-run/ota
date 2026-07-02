#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct ParsedAgentBoundaryDoc {
    pub(crate) generated_by_ota: bool,
    pub(crate) safe_tasks: Option<Vec<String>>,
    pub(crate) verify_after_changes: Option<Vec<String>>,
    pub(crate) writable_paths: Option<Vec<String>>,
    pub(crate) protected_paths: Option<Vec<String>>,
    pub(crate) task_commands: Vec<ParsedAgentBoundaryTaskCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedAgentBoundaryTaskCommand {
    pub(crate) task_name: String,
    pub(crate) command: String,
    pub(crate) source_key: String,
}

impl ParsedAgentBoundaryDoc {
    pub(crate) fn is_empty(&self) -> bool {
        !self.generated_by_ota
            && self.safe_tasks.is_none()
            && self.verify_after_changes.is_none()
            && self.writable_paths.is_none()
            && self.protected_paths.is_none()
            && self.task_commands.is_empty()
    }
}

pub(crate) fn parse_agent_boundary_doc(contents: &str) -> ParsedAgentBoundaryDoc {
    let generated_by_ota = is_ota_generated_agent_doc(contents);
    let parse_contents = if generated_by_ota {
        contents
            .rfind("# AGENTS.md")
            .and_then(|index| contents.get(index..))
            .unwrap_or(contents)
    } else {
        contents
    };
    let mut parsed = ParsedAgentBoundaryDoc {
        generated_by_ota,
        ..ParsedAgentBoundaryDoc::default()
    };
    let lines = parse_contents.lines().collect::<Vec<_>>();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index].trim_end();
        if let Some(label) = parse_agent_doc_list_label(line) {
            let mut values = Vec::new();
            index += 1;
            while index < lines.len() {
                let item_line = lines[index].trim_end();
                if let Some(value) = parse_agent_doc_task_item(item_line) {
                    values.push(value);
                    index += 1;
                } else {
                    break;
                }
            }
            assign_agent_doc_list_field(&mut parsed, label, values);
            continue;
        }
        if let Some((label, values)) = parse_agent_doc_inline_list(line) {
            assign_agent_doc_list_field(&mut parsed, label, values);
        }
        index += 1;
    }
    parsed.task_commands = parse_agent_doc_task_commands(&lines);
    parsed
}

fn is_ota_generated_agent_doc(contents: &str) -> bool {
    if !contents.contains("Generated from `") {
        return false;
    }

    contents.contains("` by `ota agents`.")
        || contents.contains("## Agent Contract")
            && (contents.contains("- `safe_tasks`:")
                || contents.contains("- `verify_after_changes`:")
                || contents.contains("- `writable_paths`:")
                || contents.contains("- `protected_paths`:"))
}

fn parse_agent_doc_list_label(line: &str) -> Option<&'static str> {
    match line.trim() {
        "- `safe_tasks`:" => Some("safe_tasks"),
        "- `verify_after_changes`:" => Some("verify_after_changes"),
        _ => None,
    }
}

fn parse_agent_doc_task_item(line: &str) -> Option<String> {
    let trimmed = line.trim_end();
    let remainder = trimmed.strip_prefix("  - `")?;
    let (value, _) = remainder.split_once('`')?;
    Some(value.to_string())
}

fn parse_agent_doc_inline_list(line: &str) -> Option<(&'static str, Vec<String>)> {
    let trimmed = line.trim();
    let (label, remainder) = if let Some(remainder) = trimmed.strip_prefix("- `writable_paths`: ") {
        ("writable_paths", remainder)
    } else if let Some(remainder) = trimmed.strip_prefix("- `protected_paths`: ") {
        ("protected_paths", remainder)
    } else {
        return None;
    };

    let values = remainder
        .split(',')
        .filter_map(|segment| {
            let value = segment.trim().strip_prefix('`')?.strip_suffix('`')?;
            (!value.is_empty()).then(|| value.to_string())
        })
        .collect::<Vec<_>>();
    Some((label, values))
}

fn assign_agent_doc_list_field(
    parsed: &mut ParsedAgentBoundaryDoc,
    label: &str,
    values: Vec<String>,
) {
    match label {
        "safe_tasks" => parsed.safe_tasks = Some(values),
        "verify_after_changes" => parsed.verify_after_changes = Some(values),
        "writable_paths" => parsed.writable_paths = Some(values),
        "protected_paths" => parsed.protected_paths = Some(values),
        _ => {}
    }
}

fn parse_agent_doc_task_commands(lines: &[&str]) -> Vec<ParsedAgentBoundaryTaskCommand> {
    let mut commands = Vec::new();
    let mut index = 0usize;
    while index < lines.len() {
        if !is_agent_doc_commands_heading(lines[index]) {
            index += 1;
            continue;
        }
        index += 1;
        while index < lines.len() {
            if is_agent_doc_task_command_table_header(lines, index) {
                index += 2;
                while index < lines.len() {
                    let line = lines[index].trim_end();
                    if let Some(command) = parse_agent_doc_table_task_command(line) {
                        commands.push(command);
                        index += 1;
                    } else {
                        break;
                    }
                }
                continue;
            }
            let line = lines[index].trim_end();
            if line.trim_start().starts_with('#') {
                break;
            }
            if let Some(command) = parse_agent_doc_bullet_task_command(line) {
                commands.push(command);
            } else if let Some(command) = parse_agent_doc_table_task_command(line) {
                commands.push(command);
            }
            index += 1;
        }
    }
    commands
}

fn is_agent_doc_task_command_table_header(lines: &[&str], index: usize) -> bool {
    let Some(header) = lines.get(index).map(|line| line.trim()) else {
        return false;
    };
    let Some(separator) = lines.get(index + 1).map(|line| line.trim()) else {
        return false;
    };
    if !header.starts_with('|') || !separator.starts_with('|') {
        return false;
    }
    let header_cells = header
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_ascii_lowercase())
        .collect::<Vec<_>>();
    let separator_cells = separator
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .collect::<Vec<_>>();
    header_cells.len() >= 2
        && separator_cells.len() >= 2
        && header_cells[0] == "task"
        && header_cells[1] == "command"
        && separator_cells.iter().all(|cell| !cell.is_empty())
        && separator_cells
            .iter()
            .all(|cell| cell.chars().all(|ch| ch == '-' || ch == ':' || ch == ' '))
}

fn is_agent_doc_commands_heading(line: &str) -> bool {
    let trimmed = line.trim();
    let heading = trimmed.trim_start_matches('#').trim();
    trimmed.starts_with('#')
        && matches!(
            heading,
            "Commands" | "Build/Test Commands" | "Build & Development Commands"
        )
}

fn parse_agent_doc_bullet_task_command(line: &str) -> Option<ParsedAgentBoundaryTaskCommand> {
    let trimmed = line.trim();
    let remainder = trimmed.strip_prefix("- ")?;
    let (label, after_label) = remainder.split_once(':')?;
    let command = parse_backticked_command(after_label.trim())?;
    let task_name = canonical_agent_doc_task_name(label)?;
    Some(ParsedAgentBoundaryTaskCommand {
        source_key: task_name.clone(),
        task_name,
        command,
    })
}

fn parse_agent_doc_table_task_command(line: &str) -> Option<ParsedAgentBoundaryTaskCommand> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
        return None;
    }
    if trimmed.contains("---") {
        return None;
    }
    let cells = trimmed
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .collect::<Vec<_>>();
    if cells.len() < 2 {
        return None;
    }
    let label = cells[0];
    let command = parse_backticked_command(cells[1])?;
    let task_name = canonical_agent_doc_task_name(label)?;
    Some(ParsedAgentBoundaryTaskCommand {
        source_key: task_name.clone(),
        task_name,
        command,
    })
}

fn parse_backticked_command(value: &str) -> Option<String> {
    let start = value.find('`')?;
    let tail = value.get(start + 1..)?;
    let end = tail.find('`')?;
    let command = tail.get(..end)?.trim();
    (!command.is_empty()).then(|| command.to_string())
}

fn canonical_agent_doc_task_name(label: &str) -> Option<String> {
    let without_parens = strip_parenthetical_segments(label);
    let normalized = without_parens
        .trim()
        .to_ascii_lowercase()
        .replace('&', "and")
        .replace('/', " ")
        .replace('-', " ");
    let compact = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
    match compact.as_str() {
        "build" => Some(String::from("build")),
        "compile" => Some(String::from("compile")),
        "lint" => Some(String::from("lint")),
        "docs" | "doc" | "generate docs" => Some(String::from("docs")),
        "test" | "tests" | "test all" => Some(String::from("test")),
        "typecheck" | "type check" => Some(String::from("typecheck")),
        "format" | "fmt" => Some(String::from("fmt")),
        _ if compact.starts_with("build ") => Some(String::from("build")),
        _ if compact.starts_with("lint ") => Some(String::from("lint")),
        _ if compact.starts_with("test ") => Some(String::from("test")),
        _ if compact.starts_with("docs ") => Some(String::from("docs")),
        _ if compact.starts_with("compile ") => Some(String::from("compile")),
        _ => None,
    }
}

fn strip_parenthetical_segments(value: &str) -> String {
    let mut stripped = String::with_capacity(value.len());
    let mut depth = 0usize;
    for ch in value.chars() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ if depth == 0 => stripped.push(ch),
            _ => {}
        }
    }
    stripped
}
