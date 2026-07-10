//                █████
//               ░░███
//       ██████  ███████    ██████
//      ███░░███░░░███░    ░░░░░███
//     ░███ ░███  ░███      ███████
//     ░███ ░███  ░███ ███ ███░░███
//     ░░██████   ░░█████ ░░████████
//      ░░░░░░     ░░░░░   ░░░░░░░░
//
//   Copyright (C) 2026 — 2026, Ota. All Rights Reserved.
//
//   DO NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::schema::TaskDotnetRestoreHydrationSourceSpec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DotnetFeedIdentity {
    pub name: String,
    pub url: String,
}

pub(crate) fn resolve_dotnet_config_sources(
    contract_path: &Path,
    source: &TaskDotnetRestoreHydrationSourceSpec,
) -> Result<Vec<DotnetFeedIdentity>, String> {
    let Some(config_file) = source.config_file.as_deref() else {
        return Ok(Vec::new());
    };
    let root = contract_path.parent().ok_or_else(|| {
        format!(
            "cannot resolve NuGet config `{config_file}` because the contract has no parent directory"
        )
    })?;
    let path = root.join(&source.cwd).join(config_file);
    let contents = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read NuGet config `{}`: {error}", path.display()))?;
    let sources = parse_nuget_config_sources(&contents)
        .map_err(|error| format!("cannot parse NuGet config `{}`: {error}", path.display()))?;
    if let Some(source) = sources
        .iter()
        .find(|source| contains_environment_placeholder(&source.url))
    {
        return Err(format!(
            "NuGet config `{}` source `{}` contains an unresolved environment placeholder",
            path.display(),
            source.name
        ));
    }
    Ok(sources)
}

fn contains_environment_placeholder(value: &str) -> bool {
    let bytes = value.as_bytes();
    for start in 0..bytes.len() {
        if bytes[start] != b'%' {
            continue;
        }
        let Some(end_offset) = bytes[start + 1..].iter().position(|byte| *byte == b'%') else {
            continue;
        };
        let candidate = &value[start + 1..start + 1 + end_offset];
        if candidate
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            && candidate
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        {
            return true;
        }
    }
    false
}

fn parse_nuget_config_sources(contents: &str) -> Result<Vec<DotnetFeedIdentity>, String> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(true);
    let mut section = ConfigSection::None;
    let mut sources = Vec::<DotnetFeedIdentity>::new();
    let mut disabled = BTreeSet::<String>::new();
    let mut open_elements = Vec::<String>::new();

    loop {
        match reader.read_event().map_err(|error| error.to_string())? {
            Event::Start(entry) => {
                open_elements.push(
                    std::str::from_utf8(entry.name().as_ref())
                        .map_err(|error| error.to_string())?
                        .to_string(),
                );
                section = match entry.name().as_ref() {
                    b"packageSources" => ConfigSection::PackageSources,
                    b"disabledPackageSources" => ConfigSection::DisabledPackageSources,
                    _ => section,
                };
                apply_config_entry(&entry, section, &mut sources, &mut disabled)?;
            }
            Event::Empty(entry) => {
                apply_config_entry(&entry, section, &mut sources, &mut disabled)?;
            }
            Event::End(entry) => {
                let name = std::str::from_utf8(entry.name().as_ref())
                    .map_err(|error| error.to_string())?
                    .to_string();
                let Some(open) = open_elements.pop() else {
                    return Err(format!("unexpected closing element `{name}`"));
                };
                if open != name {
                    return Err(format!(
                        "closing element `{name}` does not match open element `{open}`"
                    ));
                }
                if matches!(
                    name.as_bytes(),
                    b"packageSources" | b"disabledPackageSources"
                ) {
                    section = ConfigSection::None;
                }
            }
            Event::Eof => {
                if let Some(open) = open_elements.last() {
                    return Err(format!("unclosed element `{open}`"));
                }
                break;
            }
            _ => {}
        }
    }

    sources.retain(|source| !disabled.contains(&source.name));
    Ok(sources)
}

fn apply_config_entry(
    entry: &BytesStart<'_>,
    section: ConfigSection,
    sources: &mut Vec<DotnetFeedIdentity>,
    disabled: &mut BTreeSet<String>,
) -> Result<(), String> {
    let name = entry.name();
    let name = name.as_ref();
    if !matches!(name, b"add" | b"remove" | b"clear") {
        return Ok(());
    }
    let attributes = config_attributes(entry)?;
    match section {
        ConfigSection::PackageSources => match name {
            b"clear" => sources.clear(),
            b"remove" => {
                if let Some(key) = attributes.get("key") {
                    sources.retain(|source| source.name != *key);
                }
            }
            b"add" => {
                let (Some(key), Some(value)) = (attributes.get("key"), attributes.get("value"))
                else {
                    return Ok(());
                };
                if let Some(existing) = sources.iter_mut().find(|source| source.name == *key) {
                    existing.url = value.clone();
                } else {
                    sources.push(DotnetFeedIdentity {
                        name: key.clone(),
                        url: value.clone(),
                    });
                }
            }
            _ => {}
        },
        ConfigSection::DisabledPackageSources => match name {
            b"clear" => disabled.clear(),
            b"remove" => {
                if let Some(key) = attributes.get("key") {
                    disabled.remove(key);
                }
            }
            b"add" => {
                let (Some(key), Some(value)) = (attributes.get("key"), attributes.get("value"))
                else {
                    return Ok(());
                };
                if value.eq_ignore_ascii_case("true") {
                    disabled.insert(key.clone());
                } else {
                    disabled.remove(key);
                }
            }
            _ => {}
        },
        ConfigSection::None => {}
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ConfigSection {
    None,
    PackageSources,
    DisabledPackageSources,
}

fn config_attributes(
    entry: &BytesStart<'_>,
) -> Result<std::collections::BTreeMap<String, String>, String> {
    entry
        .attributes()
        .map(|attribute| {
            let attribute = attribute.map_err(|error| error.to_string())?;
            let key = std::str::from_utf8(attribute.key.as_ref())
                .map_err(|error| error.to_string())?
                .to_string();
            let value = attribute
                .unescape_value()
                .map_err(|error| error.to_string())?
                .into_owned();
            Ok((key, value))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{contains_environment_placeholder, parse_nuget_config_sources};

    #[test]
    fn resolves_active_nuget_config_sources_in_declaration_order() {
        let sources = parse_nuget_config_sources(
            r#"<configuration>
  <packageSources>
    <clear />
    <add key="public" value="https://packages.example.test/public" />
    <add key="private" value="https://packages.example.test/private" />
  </packageSources>
  <disabledPackageSources>
    <add key="private" value="true" />
  </disabledPackageSources>
</configuration>"#,
        )
        .expect("config should parse");

        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].name, "public");
        assert_eq!(sources[0].url, "https://packages.example.test/public");
    }

    #[test]
    fn rejects_unclosed_nuget_config() {
        let error = parse_nuget_config_sources("<configuration>")
            .expect_err("unclosed XML must not resolve as config truth");
        assert!(error.contains("unclosed element `configuration`"));
    }

    #[test]
    fn recognizes_nuget_environment_placeholders() {
        assert!(contains_environment_placeholder(
            "https://%NUGET_FEED_HOST%/v3/index.json"
        ));
        assert!(!contains_environment_placeholder(
            "https://example.test/feed%20name"
        ));
    }
}
