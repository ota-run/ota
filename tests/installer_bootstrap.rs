// Copyright (C) 2026, Ota. All Rights Reserved.
// Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.

//! Installer fixtures require separately approved, network-denied OS containment.
//! They are ignored by ordinary test runs. Fake downloads, hash commands, extraction,
//! and Cargo record their boundary crossings; none contacts a release server.

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};
use tempfile::TempDir;

const ARCHIVE: &str = "installer archive fixture\n";
type ModeCase = (Vec<(&'static str, &'static str)>, Vec<&'static str>);

#[derive(Clone, Copy, Debug)]
enum Fault {
    ArchiveDownload,
    PartialArchiveDownload,
    ManifestDownload,
    PartialManifestDownload,
    UnreadableManifest,
    EmptyManifest,
    MissingEntry,
    DuplicateEntry,
    DuplicateAlias,
    ConflictingAlias,
    ShortDigest,
    NonHexDigest,
    ExtraField,
    MalformedOtherEntry,
    MissingHashTool,
    FailingHashTool,
    InvalidHashOutput,
    MismatchedHash,
}

const FAULTS: &[Fault] = &[
    Fault::ArchiveDownload,
    Fault::PartialArchiveDownload,
    Fault::ManifestDownload,
    Fault::PartialManifestDownload,
    Fault::UnreadableManifest,
    Fault::EmptyManifest,
    Fault::MissingEntry,
    Fault::DuplicateEntry,
    Fault::DuplicateAlias,
    Fault::ConflictingAlias,
    Fault::ShortDigest,
    Fault::NonHexDigest,
    Fault::ExtraField,
    Fault::MalformedOtherEntry,
    Fault::MissingHashTool,
    Fault::FailingHashTool,
    Fault::InvalidHashOutput,
    Fault::MismatchedHash,
];

fn digest() -> String {
    format!("{:x}", Sha256::digest(ARCHIVE.as_bytes()))
}

fn manifest(asset: &str, fault: Option<Fault>) -> String {
    let hash = digest();
    match fault {
        Some(Fault::EmptyManifest) => String::new(),
        Some(Fault::MissingEntry) => format!("{hash}  another-asset.tar.gz\n"),
        Some(Fault::DuplicateEntry) => format!("{hash}  {asset}\n{hash}  {asset}\n"),
        Some(Fault::DuplicateAlias) => format!("{hash}  {asset}\n{hash}  dist/{asset}\n"),
        Some(Fault::ConflictingAlias) => {
            format!("{hash}  {asset}\n{}  dist/{asset}\n", "0".repeat(64))
        }
        Some(Fault::ShortDigest) => format!("1234  {asset}\n"),
        Some(Fault::NonHexDigest) => format!("{}  {asset}\n", "g".repeat(64)),
        Some(Fault::ExtraField) => format!("{hash}  {asset}  unexpected\n"),
        Some(Fault::MalformedOtherEntry) => {
            format!("{hash}  {asset}\nnot-a-digest  another-asset.tar.gz\n")
        }
        _ => format!("{hash}  {asset}\n"),
    }
}

struct Fixture {
    root: TempDir,
    events: PathBuf,
}

struct Run {
    success: bool,
    events: String,
    output: String,
}

impl Run {
    fn count(&self, event: &str) -> usize {
        self.events.lines().filter(|line| *line == event).count()
    }

    fn assert_refused_before_payload(&self, label: &str) {
        assert!(!self.success, "{label}: {}", self.output);
        for event in ["extract", "install", "payload", "cargo"] {
            assert_eq!(self.count(event), 0, "{label}: {event}: {}", self.events);
        }
    }
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("fixture directory");
        for path in ["home", "tmp", "bin", "work", "destination"] {
            fs::create_dir(root.path().join(path)).expect("fixture subdirectory");
        }
        let events = root.path().join("events.txt");
        fs::write(&events, "").expect("event log");
        Self { root, events }
    }

    fn configure(&self, command: &mut Command, asset: &str, fault: Option<Fault>) {
        command
            .env_clear()
            .current_dir(self.root.path().join("work"))
            .env("HOME", self.root.path().join("home"))
            .env("TMPDIR", self.root.path().join("tmp"))
            .env("TEMP", self.root.path().join("tmp"))
            .env("TMP", self.root.path().join("tmp"))
            .env("PATH", self.root.path().join("bin"))
            .env("OTA_BIN_DIR", self.root.path().join("destination"))
            .env("OTA_ASCII", "1")
            .env("NO_COLOR", "1")
            .env("LC_ALL", "C")
            .env("OTAFIX_EVENTS", &self.events)
            .env("OTAFIX_ARCHIVE", ARCHIVE)
            .env("OTAFIX_MANIFEST", manifest(asset, fault))
            .env("OTAFIX_HASH", digest())
            .env("OTAFIX_DOWNLOAD", "ok")
            .env("OTAFIX_HASH_MODE", "ok");
        match fault {
            Some(Fault::ArchiveDownload) => {
                command.env("OTAFIX_DOWNLOAD", "archive-fail");
            }
            Some(Fault::PartialArchiveDownload) => {
                command.env("OTAFIX_DOWNLOAD", "archive-partial");
            }
            Some(Fault::ManifestDownload) => {
                command.env("OTAFIX_DOWNLOAD", "manifest-fail");
            }
            Some(Fault::PartialManifestDownload) => {
                command.env("OTAFIX_DOWNLOAD", "manifest-partial");
            }
            Some(Fault::UnreadableManifest) => {
                command.env("OTAFIX_DOWNLOAD", "manifest-directory");
            }
            Some(Fault::MissingHashTool) => {
                command.env("OTAFIX_HASH_MODE", "missing");
            }
            Some(Fault::FailingHashTool) => {
                command.env("OTAFIX_HASH_MODE", "fail");
            }
            Some(Fault::InvalidHashOutput) => {
                command.env("OTAFIX_HASH", "invalid");
            }
            Some(Fault::MismatchedHash) => {
                command.env("OTAFIX_HASH", "0".repeat(64));
            }
            _ => {}
        }
    }

    fn checkout_marker(&self) {
        fs::write(
            self.root.path().join("work/Cargo.toml"),
            "[package]\nname = \"ota\"\nversion = \"0.0.0\"\n",
        )
        .expect("checkout marker");
    }

    fn finish(&self, command: &mut Command) -> Run {
        let output = self.root.path().join("output.txt");
        let stdout = File::create(&output).expect("fixture output");
        command
            .stdin(Stdio::null())
            .stdout(stdout.try_clone().expect("clone output"))
            .stderr(stdout);
        let mut child = command.spawn().expect("start contained fixture");
        let deadline = Instant::now() + Duration::from_secs(10);
        let status = loop {
            if let Some(status) = child.try_wait().expect("fixture status") {
                break status;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("installer fixture exceeded its deadline");
            }
            thread::sleep(Duration::from_millis(10));
        };
        let events = fs::read_to_string(&self.events).expect("read events");
        let output = fs::read_to_string(output).expect("read output");
        assert!(events.len() <= 16_384 && output.len() <= 65_536);
        Run {
            success: status.success(),
            events,
            output,
        }
    }
}

#[cfg(unix)]
mod posix {
    use super::*;
    use std::os::unix::fs::{PermissionsExt, symlink};

    const ASSET: &str = "ota-x86_64-unknown-linux-gnu.tar.gz";

    pub(super) fn script(path: &Path, body: &str) {
        fs::write(path, body).expect("fixture script");
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("executable fixture");
    }

    fn utility(tool: &str) -> PathBuf {
        [Path::new("/usr/bin"), Path::new("/bin")]
            .iter()
            .map(|directory| directory.join(tool))
            .find(|path| path.is_file())
            .unwrap_or_else(|| panic!("required read-only fixture utility: {tool}"))
    }

    fn prepare(f: &Fixture, fault: Option<Fault>, alternate_hash: bool) {
        let bin = f.root.path().join("bin");
        for tool in [
            "tr", "awk", "mktemp", "rm", "grep", "sed", "basename", "dirname", "chmod",
        ] {
            symlink(utility(tool), bin.join(tool)).expect("fixture utility link");
        }
        script(
            &bin.join("install"),
            &format!(
                "#!/bin/sh\nprintf 'install\\n' >> \"$OTAFIX_EVENTS\"\n[ \"${{OTAFIX_INSTALL_FAULT:-}}\" != stage ] || exit 12\nexec {} \"$@\"\n",
                utility("install").display()
            ),
        );
        script(
            &bin.join("mkdir"),
            &format!(
                "#!/bin/sh\nfor arg do last=\"$arg\"; done\nif [ \"$last\" = \"$OTA_BIN_DIR\" ]; then\n  printf 'directory\\n' >> \"$OTAFIX_EVENTS\"\n  [ \"${{OTAFIX_INSTALL_FAULT:-}}\" != directory ] || exit 11\nfi\nexec {} \"$@\"\n",
                utility("mkdir").display()
            ),
        );
        script(
            &bin.join("mv"),
            &format!(
                "#!/bin/sh\nprintf 'replace\\n' >> \"$OTAFIX_EVENTS\"\n[ \"${{OTAFIX_INSTALL_FAULT:-}}\" != replace ] || exit 13\nexec {} \"$@\"\n",
                utility("mv").display()
            ),
        );
        script(
            &bin.join("cat"),
            &format!(
                "#!/bin/sh\nprintf 'schedule-request\\n' >> \"$OTAFIX_EVENTS\"\nexec {} \"$@\"\n",
                utility("cat").display()
            ),
        );
        script(
            &bin.join("pwsh"),
            &format!(
                "#!/bin/sh\ncase \"$*\" in\n  *Move-Item*)\n    printf 'replace\\n' >> \"$OTAFIX_EVENTS\"\n    case \"${{OTAFIX_INSTALL_FAULT:-}}\" in replace) exit 1;; locked) exit 32;; esac\n    exec {} \"$OTA_BIN_DIR/ota.exe.new\" \"$OTA_BIN_DIR/ota.exe\" ;;\n  *) exit 0 ;;\nesac\n",
                utility("mv").display()
            ),
        );
        script(
            &bin.join("uname"),
            "#!/bin/sh\ncase \"$1\" in -s) echo \"${OTAFIX_OS:-Linux}\";; -m) echo x86_64;; *) exit 1;; esac\n",
        );
        script(
            &bin.join("curl"),
            r#"#!/bin/sh
while [ "$#" -gt 0 ]; do
  case "$1" in
    -o) out="$2"; shift 2 ;;
    https://*) url="$1"; shift ;;
    *) shift ;;
  esac
done
printf 'download:%s\n' "$url" >> "$OTAFIX_EVENTS"
case "$url" in
  */ota-checksums.txt)
    case "$OTAFIX_DOWNLOAD" in
      manifest-fail) exit 22 ;;
      manifest-directory) mkdir "$out"; exit $? ;;
    esac
    printf '%s' "$OTAFIX_MANIFEST" > "$out"
    [ "$OTAFIX_DOWNLOAD" != manifest-partial ] || exit 22 ;;
  *)
    [ "$OTAFIX_DOWNLOAD" != archive-fail ] || exit 22
    printf '%s' "$OTAFIX_ARCHIVE" > "$out"
    [ "$OTAFIX_DOWNLOAD" != archive-partial ] || exit 22 ;;
esac
"#,
        );
        if !matches!(fault, Some(Fault::MissingHashTool)) {
            let hash = if alternate_hash {
                "sha256sum"
            } else {
                "shasum"
            };
            script(
                &bin.join(hash),
                r#"#!/bin/sh
printf 'hash\n' >> "$OTAFIX_EVENTS"
[ "$OTAFIX_HASH_MODE" != fail ] || exit 1
printf '%s  fixture-archive\n' "$OTAFIX_HASH"
"#,
            );
        }
        let extraction = r#"#!/bin/sh
printf 'extract\n' >> "$OTAFIX_EVENTS"
while [ "$#" -gt 0 ]; do
  case "$1" in -C|-d) destination="$2"; shift 2;; *) shift;; esac
done
printf '#!/bin/sh\nprintf "payload\\n" >> "$OTAFIX_EVENTS"\nprintf "ota v0.0.0\\n"\n' > "$destination/${OTAFIX_BINARY:-ota}"
"#;
        script(&bin.join("tar"), extraction);
        script(&bin.join("unzip"), extraction);
        script(
            &bin.join("cargo"),
            r#"#!/bin/sh
printf 'cargo\ncargo-args:%s\n' "$*" >> "$OTAFIX_EVENTS"
mkdir -p "$HOME/.cargo/bin"
printf '#!/bin/sh\nprintf "payload\\n" >> "$OTAFIX_EVENTS"\nprintf "ota v0.0.0\\n"\n' > "$HOME/.cargo/bin/ota"
chmod +x "$HOME/.cargo/bin/ota"
"#,
        );
    }

    fn run(
        f: &Fixture,
        fault: Option<Fault>,
        alternate_hash: bool,
        env: &[(&str, &str)],
        flags: &[&str],
    ) -> Run {
        prepare(f, fault, alternate_hash);
        let mut command = Command::new("/bin/sh");
        let windows = env
            .iter()
            .any(|(key, value)| *key == "OTAFIX_OS" && value.starts_with("MINGW"));
        let asset = if windows {
            "ota-x86_64-pc-windows-msvc.zip"
        } else {
            ASSET
        };
        f.configure(&mut command, asset, fault);
        command.env("OTAFIX_BINARY", if windows { "ota.exe" } else { "ota" });
        command
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/install.sh"))
            .args(flags);
        for (name, value) in env {
            command.env(name, value);
        }
        f.finish(&mut command)
    }

    fn seed_existing(f: &Fixture, binary: &str) -> String {
        let existing =
            "#!/bin/sh\nprintf 'existing\\n' >> \"$OTAFIX_EVENTS\"\nprintf 'ota v0.0.0\\n'\n";
        script(&f.root.path().join("destination").join(binary), existing);
        script(
            &f.root
                .path()
                .join("destination")
                .join(format!("{binary}.new")),
            "#!/bin/sh\nprintf 'payload\\n' >> \"$OTAFIX_EVENTS\"\n",
        );
        existing.to_owned()
    }

    #[test]
    #[ignore = "requires approved network-denied installer fixture containment"]
    fn publication_failures_do_not_execute_existing_or_staged_binaries() {
        for windows in [false, true] {
            for (fault, installs, replacements) in
                [("directory", 0, 0), ("stage", 1, 0), ("replace", 1, 1)]
            {
                let f = Fixture::new();
                let binary = if windows { "ota.exe" } else { "ota" };
                let existing = seed_existing(&f, binary);
                let mut env = vec![("OTAFIX_INSTALL_FAULT", fault)];
                if windows {
                    env.push(("OTAFIX_OS", "MINGW64_NT"));
                }
                let result = run(&f, None, false, &env, &[]);
                assert!(
                    !result.success,
                    "{fault}, windows={windows}: {}",
                    result.output
                );
                assert_eq!(result.count("hash"), 1);
                assert_eq!(result.count("extract"), 1);
                assert_eq!(result.count("directory"), 1);
                assert_eq!(result.count("install"), installs);
                assert_eq!(result.count("replace"), replacements);
                for event in ["existing", "payload", "cargo", "schedule-request"] {
                    assert_eq!(result.count(event), 0, "{fault}: {}", result.events);
                }
                assert!(!result.output.contains("READY"), "{}", result.output);
                assert_eq!(
                    fs::read_to_string(f.root.path().join("destination").join(binary)).unwrap(),
                    existing
                );
            }
        }
    }

    #[test]
    #[ignore = "requires approved network-denied installer fixture containment"]
    fn simulated_windows_sharing_violation_retains_the_pending_path() {
        let f = Fixture::new();
        let existing = seed_existing(&f, "ota.exe");
        let result = run(
            &f,
            None,
            false,
            &[
                ("OTAFIX_OS", "MINGW64_NT"),
                ("OTAFIX_INSTALL_FAULT", "locked"),
            ],
            &[],
        );
        assert!(result.success, "{}", result.output);
        for event in [
            "hash",
            "extract",
            "directory",
            "install",
            "replace",
            "schedule-request",
        ] {
            assert_eq!(result.count(event), 1, "{}", result.events);
        }
        for event in ["existing", "payload", "cargo"] {
            assert_eq!(result.count(event), 0);
        }
        assert!(!result.output.contains("READY"));
        assert!(result.output.contains("pending:"));
        assert_eq!(
            fs::read_to_string(f.root.path().join("destination/ota.exe")).unwrap(),
            existing
        );
    }

    #[test]
    #[ignore = "requires approved network-denied installer fixture containment"]
    fn release_faults_refuse_without_extraction_payload_or_cargo() {
        for fault in FAULTS {
            for custom_origin in [false, true] {
                let f = Fixture::new();
                let env = if custom_origin {
                    vec![("OTA_RELEASE_BASE", "https://fixture.invalid/releases")]
                } else {
                    vec![]
                };
                run(&f, Some(*fault), false, &env, &[])
                    .assert_refused_before_payload(&format!("{fault:?}, custom={custom_origin}"));
            }
        }
    }

    #[test]
    #[ignore = "requires approved network-denied installer fixture containment"]
    fn verified_latest_release_does_not_build_the_current_checkout() {
        for alternate_hash in [false, true] {
            let f = Fixture::new();
            f.checkout_marker();
            let result = run(&f, None, alternate_hash, &[], &[]);
            assert!(result.success, "{}", result.output);
            assert_eq!(result.count("extract"), 1);
            assert_eq!(result.count("install"), 1);
            assert_eq!(result.count("payload"), 1);
            assert_eq!(result.count("cargo"), 0);
            assert!(result.events.contains("/latest/download/"));
        }
    }

    #[test]
    #[ignore = "requires approved network-denied installer fixture containment"]
    fn singular_dist_alias_and_uppercase_digest_remain_valid() {
        let f = Fixture::new();
        let manifest = format!("{}  dist/{ASSET}\n", digest().to_uppercase());
        let result = run(
            &f,
            None,
            false,
            &[("OTAFIX_MANIFEST", &manifest), ("OTA_VERSION", "v0.0.0")],
            &["--from-release"],
        );
        assert!(result.success, "{}", result.output);
        assert_eq!(result.count("extract"), 1);
        assert_eq!(result.count("install"), 1);
        assert_eq!(result.count("payload"), 1);
        assert_eq!(result.count("cargo"), 0);
        assert!(result.events.contains("/download/v0.0.0/"));
    }

    #[test]
    #[ignore = "requires approved network-denied installer fixture containment"]
    fn failing_preferred_hash_tool_does_not_try_another_verifier() {
        let f = Fixture::new();
        script(
            &f.root.path().join("bin/sha256sum"),
            "#!/bin/sh\nprintf 'alternate-hash\\n' >> \"$OTAFIX_EVENTS\"\nprintf '%s  archive\\n' \"$OTAFIX_HASH\"\n",
        );
        let result = run(&f, Some(Fault::FailingHashTool), false, &[], &[]);
        result.assert_refused_before_payload("failing preferred hash tool");
        assert_eq!(result.count("hash"), 1);
        assert_eq!(result.count("alternate-hash"), 0);
    }

    #[test]
    #[ignore = "requires approved network-denied installer fixture containment"]
    fn invalid_and_conflicting_modes_refuse_before_source_or_release_work() {
        let cases: Vec<ModeCase> = vec![
            (vec![("OTA_INSTALL_MODE", "typo")], vec![]),
            (vec![], vec!["--from-source", "--from-release"]),
            (vec![], vec!["--from-git", "--from-source"]),
            (vec![("OTA_INSTALL_MODE", "release")], vec!["--from-source"]),
            (vec![("OTA_GIT_REV", "fixture-rev")], vec![]),
            (vec![("OTA_GIT_TAG", "fixture-tag")], vec![]),
            (vec![("OTA_GIT_BRANCH", "fixture-branch")], vec![]),
            (
                vec![("OTA_GIT_URL", "https://fixture.invalid/repo")],
                vec![],
            ),
            (vec![("OTA_VERSION", "v0.0.0")], vec!["--from-source"]),
            (vec![("OTA_VERSION", "v0.0.0")], vec!["--from-git"]),
            (vec![("OTA_ALLOW_MISSING_CHECKSUMS", "1")], vec![]),
            (
                vec![
                    ("OTA_RELEASE_BASE", "https://fixture.invalid/releases"),
                    ("OTA_ALLOW_MISSING_CHECKSUMS", "1"),
                ],
                vec![],
            ),
            (
                vec![("OTA_GIT_REV", "rev"), ("OTA_GIT_BRANCH", "branch")],
                vec!["--from-git"],
            ),
        ];
        for (env, flags) in cases {
            let f = Fixture::new();
            let result = run(&f, None, false, &env, &flags);
            result.assert_refused_before_payload(&format!("{env:?} {flags:?}"));
            assert!(!result.events.contains("download:"));
        }
    }

    #[test]
    #[ignore = "requires approved network-denied installer fixture containment"]
    fn explicitly_requested_local_and_git_source_paths_remain_available() {
        for (flags, env, expected) in [
            (
                vec!["--from-source"],
                vec![],
                "install --path . --locked --force",
            ),
            (
                vec!["--from-git"],
                vec![("OTA_GIT_REV", "fixture-rev")],
                "--rev fixture-rev",
            ),
            (vec![], vec![("OTA_INSTALL_MODE", "source")], "--path ."),
        ] {
            let f = Fixture::new();
            let result = run(&f, None, false, &env, &flags);
            assert!(result.success, "{}", result.output);
            assert_eq!(result.count("cargo"), 1);
            assert_eq!(result.count("extract"), 0);
            assert!(result.events.contains(expected));
            assert!(!result.events.contains("download:"));
        }
    }
}

#[cfg(unix)]
mod powershell_native_tar {
    use super::*;

    fn run(f: &Fixture, tar_exit: i32) -> Run {
        let shell = PathBuf::from(
            std::env::var_os("OTA_TEST_PWSH")
                .expect("contained validation must provide an approved native OTA_TEST_PWSH"),
        );
        assert!(shell.is_absolute() && shell.is_file());
        let (platform, target) = match std::env::consts::OS {
            "linux" => ("Linux", "unknown-linux-gnu"),
            "macos" => ("OSX", "apple-darwin"),
            other => panic!("unsupported native PowerShell fixture OS: {other}"),
        };
        let (os_arch, arch) = match std::env::consts::ARCH {
            "x86_64" => ("x64", "x86_64"),
            "aarch64" => ("arm64", "aarch64"),
            other => panic!("unsupported native PowerShell fixture architecture: {other}"),
        };
        let tar = f.root.path().join("bin/tar");
        super::posix::script(
            &tar,
            r#"#!/bin/sh
printf 'native-tar\n' >> "$OTAFIX_EVENTS"
while [ "$#" -gt 0 ]; do
  case "$1" in -C) destination="$2"; shift 2;; *) shift;; esac
done
printf '#!/bin/sh\nprintf "payload\\n" >> "$OTAFIX_EVENTS"\n' > "$destination/ota"
printf 'partial-binary\n' >> "$OTAFIX_EVENTS"
exit "$OTAFIX_TAR_EXIT"
"#,
        );
        let wrapper = f.root.path().join("native-tar-fixture.ps1");
        fs::write(
            &wrapper,
            r#"$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$platform = switch ($env:OTAFIX_PLATFORM) {
    'Linux' { [Runtime.InteropServices.OSPlatform]::Linux }
    'OSX' { [Runtime.InteropServices.OSPlatform]::OSX }
    default { throw 'unsupported native PowerShell fixture platform' }
}
if (-not [Runtime.InteropServices.RuntimeInformation]::IsOSPlatform($platform) -or
    [Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLowerInvariant() -ne $env:OTAFIX_OS_ARCH) {
    throw 'approved native PowerShell OS/architecture prerequisite is not met'
}
$tarCommand = Get-Command tar -ErrorAction Stop
if ($tarCommand.CommandType -ne 'Application' -or $tarCommand.Source -cne $env:OTAFIX_TAR) {
    throw 'fixture requires the native TAR executable, not a PowerShell function'
}
function global:Add-FixtureEvent([string]$Value) {
    [IO.File]::AppendAllText($env:OTAFIX_EVENTS, "$Value`n")
}
function global:Invoke-WebRequest {
    [CmdletBinding()]
    param([string]$Uri, [string]$OutFile)
    $content = if ($Uri.EndsWith('/ota-checksums.txt')) { $env:OTAFIX_MANIFEST } else { $env:OTAFIX_ARCHIVE }
    [IO.File]::WriteAllText($OutFile, $content)
}
function global:Get-FileHash {
    [CmdletBinding()]
    param([string]$Algorithm, [string]$LiteralPath)
    Add-FixtureEvent 'hash'
    [pscustomobject]@{ Hash = $env:OTAFIX_HASH }
}
function global:Expand-Archive {
    [CmdletBinding()]
    param([string]$Path, [string]$DestinationPath, [switch]$Force)
    Add-FixtureEvent 'zip-branch'
    throw 'the native TAR control must not use the Windows ZIP branch'
}
function global:Get-ChildItem {
    [CmdletBinding()]
    param([string]$Path, [string]$Filter, [switch]$Recurse)
    Add-FixtureEvent 'discovery'
    Microsoft.PowerShell.Management\Get-ChildItem @PSBoundParameters
}
function global:Copy-Item {
    [CmdletBinding()]
    param([string]$LiteralPath, [string]$Destination, [switch]$Force)
    Add-FixtureEvent 'install'
    throw 'fixture stops before installing a payload'
}
function global:cargo {
    Add-FixtureEvent 'cargo'
    $global:LASTEXITCODE = 37
}
& $env:OTAFIX_INSTALLER
exit $LASTEXITCODE
"#,
        )
        .expect("native TAR PowerShell fixture");
        let mut command = Command::new(shell);
        f.configure(&mut command, &format!("ota-{arch}-{target}.tar.gz"), None);
        command
            .env("OTAFIX_PLATFORM", platform)
            .env("OTAFIX_OS_ARCH", os_arch)
            .env("OTAFIX_TAR", tar)
            .env("OTAFIX_TAR_EXIT", tar_exit.to_string())
            .env("POWERSHELL_TELEMETRY_OPTOUT", "1")
            .env("POWERSHELL_UPDATECHECK", "Off")
            .env(
                "PSModuleAnalysisCachePath",
                f.root.path().join("tmp/ModuleAnalysisCache"),
            )
            .env(
                "OTAFIX_INSTALLER",
                Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/bootstrap.ps1"),
            )
            .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-File"])
            .arg(wrapper);
        f.finish(&mut command)
    }

    #[test]
    #[ignore = "requires approved containment and a matching native OTA_TEST_PWSH toolchain"]
    fn native_tar_partial_failure_refuses_before_discovery_or_copy() {
        for (tar_exit, discovery) in [(17, 0), (0, 1)] {
            let f = Fixture::new();
            let existing = "#!/bin/sh\nprintf 'existing\\n' >> \"$OTAFIX_EVENTS\"\n";
            let destination = f.root.path().join("destination/ota");
            super::posix::script(&destination, existing);
            let result = run(&f, tar_exit);
            assert!(!result.success, "fixture must refuse or stop before copy");
            for event in ["hash", "native-tar", "partial-binary"] {
                assert_eq!(result.count(event), 1, "tar={tar_exit}: {}", result.output);
            }
            assert_eq!(result.count("discovery"), discovery, "{}", result.output);
            assert_eq!(result.count("install"), discovery, "{}", result.output);
            for event in ["payload", "existing", "cargo", "zip-branch"] {
                assert_eq!(result.count(event), 0, "{}", result.events);
            }
            assert!(!result.output.contains("READY"));
            assert_eq!(fs::read_to_string(destination).unwrap(), existing);
        }
    }
}

#[cfg(windows)]
mod powershell {
    use super::*;

    fn shell() -> PathBuf {
        let root = std::env::var_os("SystemRoot").expect("Windows system directory");
        PathBuf::from(root).join("System32/WindowsPowerShell/v1.0/powershell.exe")
    }

    fn asset() -> String {
        let arch = match std::env::consts::ARCH {
            "x86_64" => "x86_64",
            "aarch64" => "aarch64",
            other => panic!("unsupported Windows fixture architecture: {other}"),
        };
        format!("ota-{arch}-pc-windows-msvc.zip")
    }

    fn command(f: &Fixture, wrapper: &Path, installer: &str, fault: Option<Fault>) -> Command {
        let mut command = Command::new(shell());
        f.configure(&mut command, &asset(), fault);
        command
            .env("SystemRoot", std::env::var_os("SystemRoot").unwrap())
            .env("USERPROFILE", f.root.path().join("home"))
            .env("LOCALAPPDATA", f.root.path().join("home"))
            .env("POWERSHELL_TELEMETRY_OPTOUT", "1")
            .env("POWERSHELL_UPDATECHECK", "Off")
            .env(
                "PSModuleAnalysisCachePath",
                f.root.path().join("tmp/ModuleAnalysisCache"),
            )
            .env(
                "OTAFIX_INSTALLER",
                Path::new(env!("CARGO_MANIFEST_DIR")).join(installer),
            )
            .args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
            ])
            .arg(wrapper);
        command
    }

    fn run(f: &Fixture, fault: Option<Fault>, env: &[(&str, &str)], flags: &[&str]) -> Run {
        let wrapper = f.root.path().join("fixture.ps1");
        fs::write(
            &wrapper,
            r#"$ErrorActionPreference = 'Stop'
function global:Add-FixtureEvent([string]$Value) {
    [IO.File]::AppendAllText($env:OTAFIX_EVENTS, "$Value`n")
}
function global:Invoke-WebRequest {
    [CmdletBinding()]
    param([string]$Uri, [string]$OutFile)
    Add-FixtureEvent "download:$Uri"
    if ($Uri.EndsWith('/ota-checksums.txt')) {
        if ($env:OTAFIX_DOWNLOAD -eq 'manifest-fail') { throw 'fixture manifest download failure' }
        if ($env:OTAFIX_DOWNLOAD -eq 'manifest-directory') {
            [IO.Directory]::CreateDirectory($OutFile) | Out-Null
            return
        }
        [IO.File]::WriteAllText($OutFile, $env:OTAFIX_MANIFEST)
        if ($env:OTAFIX_DOWNLOAD -eq 'manifest-partial') { throw 'fixture partial manifest download' }
    } else {
        if ($env:OTAFIX_DOWNLOAD -eq 'archive-fail') { throw 'fixture archive download failure' }
        [IO.File]::WriteAllText($OutFile, $env:OTAFIX_ARCHIVE)
        if ($env:OTAFIX_DOWNLOAD -eq 'archive-partial') { throw 'fixture partial archive download' }
    }
}

function global:Get-Command {
    [CmdletBinding()]
    param([string]$Name)
    if ($Name -eq 'Get-FileHash' -and $env:OTAFIX_HASH_MODE -eq 'missing') { return }
    Microsoft.PowerShell.Core\Get-Command @PSBoundParameters
}
function global:Get-FileHash {
    [CmdletBinding()]
    param([string]$Algorithm, [string]$LiteralPath)
    Add-FixtureEvent 'hash'
    if ($env:OTAFIX_HASH_MODE -eq 'fail') { throw 'fixture hash failure' }
    [pscustomobject]@{ Hash = $env:OTAFIX_HASH }
}
function global:Expand-Archive {
    [CmdletBinding()]
    param([string]$Path, [string]$DestinationPath, [switch]$Force)
    Add-FixtureEvent 'extract'
    [IO.File]::WriteAllText((Join-Path $DestinationPath 'ota.exe'), 'fixture payload, never executable')
}
function global:Copy-Item {
    [CmdletBinding()]
    param([string]$LiteralPath, [string]$Destination, [switch]$Force)
    Add-FixtureEvent 'install'
    throw 'fixture stops before installing a payload'
}
function global:cargo {
    Add-FixtureEvent 'cargo'
    Add-FixtureEvent "cargo-args:$args"
    $global:LASTEXITCODE = 37
}
$installerArguments = @{}
foreach ($argument in $args) {
    switch -CaseSensitive ($argument) {
        '-FromSource' { $installerArguments.FromSource = $true }
        '-FromGit' { $installerArguments.FromGit = $true }
        '-FromRelease' { $installerArguments.FromRelease = $true }
        default { throw "unexpected fixture argument: $argument" }
    }
}
& $env:OTAFIX_INSTALLER @installerArguments
exit $LASTEXITCODE
"#,
        )
        .expect("PowerShell fixture");
        let mut command = command(f, &wrapper, "scripts/bootstrap.ps1", fault);
        command.args(flags);
        for (name, value) in env {
            command.env(name, value);
        }
        f.finish(&mut command)
    }

    fn run_wrapper(f: &Fixture, flags: &[&str]) -> Run {
        let wrapper = f.root.path().join("wrapper-fixture.ps1");
        fs::write(
            &wrapper,
            r#"$ErrorActionPreference = 'Stop'
function global:Add-FixtureEvent([string]$Value) {
    [IO.File]::AppendAllText($env:OTAFIX_EVENTS, "$Value`n")
}
function global:New-Item {
    [CmdletBinding()]
    param([string]$ItemType, [string]$Path, [switch]$Force)
    Add-FixtureEvent 'wrapper-temp'
    Microsoft.PowerShell.Management\New-Item @PSBoundParameters
}
function global:Invoke-WebRequest {
    [CmdletBinding()]
    param([string]$Uri, [string]$OutFile)
    Add-FixtureEvent 'bootstrap-download'
    [IO.File]::WriteAllText($OutFile, '# fixture bootstrap: must never execute')
}
function global:pwsh {
    Add-FixtureEvent 'bootstrap-delegation'
    Add-FixtureEvent "delegation-args:$args"
    $global:LASTEXITCODE = 0
}
$installerArguments = @{}
foreach ($argument in $args) {
    switch -CaseSensitive ($argument) {
        '-FromSource' { $installerArguments.FromSource = $true }
        '-FromGit' { $installerArguments.FromGit = $true }
        '-FromRelease' { $installerArguments.FromRelease = $true }
        '-SetupPath' { $installerArguments.SetupPath = $true }
        default { throw "unexpected fixture argument: $argument" }
    }
}
& $env:OTAFIX_INSTALLER @installerArguments
exit $LASTEXITCODE
"#,
        )
        .expect("wrapper fixture");
        let mut command = command(f, &wrapper, "scripts/install.ps1", None);
        command.args(flags);
        f.finish(&mut command)
    }

    #[test]
    #[ignore = "requires approved network-denied installer fixture containment"]
    fn wrapper_rejects_all_conflicting_switches_before_bootstrap_work() {
        for mut flags in [
            vec!["-FromSource", "-FromGit"],
            vec!["-FromSource", "-FromRelease"],
            vec!["-FromGit", "-FromRelease"],
            vec!["-FromSource", "-FromGit", "-FromRelease"],
        ] {
            for setup_path in [false, true] {
                let f = Fixture::new();
                if setup_path {
                    flags.push("-SetupPath");
                }
                let result = run_wrapper(&f, &flags);
                assert!(!result.success, "{flags:?}: {}", result.output);
                assert!(result.output.contains("conflicting explicit install modes"));
                assert_eq!(result.count("wrapper-temp"), 0);
                assert_eq!(result.count("bootstrap-download"), 0);
                assert_eq!(result.count("bootstrap-delegation"), 0);
            }
        }
    }

    #[test]
    #[ignore = "requires approved network-denied installer fixture containment"]
    fn wrapper_preserves_default_single_switch_and_setup_path_routing() {
        for selector in [
            None,
            Some("-FromSource"),
            Some("-FromGit"),
            Some("-FromRelease"),
        ] {
            for setup_path in [false, true] {
                let f = Fixture::new();
                let mut flags = selector.into_iter().collect::<Vec<_>>();
                if setup_path {
                    flags.push("-SetupPath");
                }
                let result = run_wrapper(&f, &flags);
                assert!(result.success, "{flags:?}: {}", result.output);
                assert_eq!(result.count("bootstrap-delegation"), 1);
                let downloads = usize::from(selector != Some("-FromSource"));
                assert_eq!(result.count("bootstrap-download"), downloads);
                assert_eq!(result.count("wrapper-temp"), downloads);
                for flag in ["-FromSource", "-FromGit", "-FromRelease", "-SetupPath"] {
                    assert_eq!(result.events.contains(flag), flags.contains(&flag));
                }
            }
        }
    }

    #[test]
    #[ignore = "requires approved network-denied installer fixture containment"]
    fn release_faults_refuse_without_extraction_installation_or_cargo() {
        for fault in FAULTS {
            for custom_origin in [false, true] {
                let f = Fixture::new();
                let env = if custom_origin {
                    vec![("OTA_RELEASE_BASE", "https://fixture.invalid/releases")]
                } else {
                    vec![]
                };
                run(&f, Some(*fault), &env, &[])
                    .assert_refused_before_payload(&format!("{fault:?}, custom={custom_origin}"));
            }
        }
    }

    #[test]
    #[ignore = "requires approved network-denied installer fixture containment"]
    fn verified_latest_release_reaches_only_the_fixture_install_boundary() {
        let f = Fixture::new();
        f.checkout_marker();
        let result = run(&f, None, &[], &[]);
        assert!(!result.success, "fixture must stop before copying payload");
        assert_eq!(result.count("extract"), 1, "{}", result.output);
        assert_eq!(result.count("install"), 1, "{}", result.output);
        assert_eq!(result.count("payload"), 0);
        assert_eq!(result.count("cargo"), 0);
        assert!(result.events.contains("/latest/download/"));
    }

    #[test]
    #[ignore = "requires approved network-denied installer fixture containment"]
    fn singular_dist_alias_and_uppercase_digest_pass_the_integrity_boundary() {
        let f = Fixture::new();
        let manifest = format!("{}  dist/{}\n", digest().to_uppercase(), asset());
        let result = run(
            &f,
            None,
            &[("OTAFIX_MANIFEST", &manifest), ("OTA_VERSION", "v0.0.0")],
            &["-FromRelease"],
        );
        assert!(!result.success, "fixture must stop before copying payload");
        assert_eq!(result.count("extract"), 1, "{}", result.output);
        assert_eq!(result.count("install"), 1, "{}", result.output);
        assert_eq!(result.count("payload"), 0);
        assert_eq!(result.count("cargo"), 0);
        assert!(result.events.contains("/download/v0.0.0/"));
    }

    #[test]
    #[ignore = "requires approved network-denied installer fixture containment"]
    fn mode_conflicts_and_checksum_bypass_refuse_before_work() {
        let cases: Vec<ModeCase> = vec![
            (vec![("OTA_INSTALL_MODE", "typo")], vec![]),
            (vec![], vec!["-FromSource", "-FromRelease"]),
            (vec![], vec!["-FromGit", "-FromSource"]),
            (vec![("OTA_INSTALL_MODE", "release")], vec!["-FromGit"]),
            (vec![("OTA_GIT_REV", "fixture-rev")], vec![]),
            (vec![("OTA_GIT_TAG", "fixture-tag")], vec![]),
            (vec![("OTA_GIT_BRANCH", "fixture-branch")], vec![]),
            (
                vec![("OTA_GIT_URL", "https://fixture.invalid/repo")],
                vec![],
            ),
            (vec![("OTA_VERSION", "v0.0.0")], vec!["-FromSource"]),
            (vec![("OTA_VERSION", "v0.0.0")], vec!["-FromGit"]),
            (vec![("OTA_ALLOW_MISSING_CHECKSUMS", "1")], vec![]),
            (
                vec![
                    ("OTA_RELEASE_BASE", "https://fixture.invalid/releases"),
                    ("OTA_ALLOW_MISSING_CHECKSUMS", "1"),
                ],
                vec![],
            ),
            (
                vec![("OTA_GIT_REV", "rev"), ("OTA_GIT_BRANCH", "branch")],
                vec!["-FromGit"],
            ),
        ];
        for (env, flags) in cases {
            let f = Fixture::new();
            let result = run(&f, None, &env, &flags);
            result.assert_refused_before_payload(&format!("{env:?} {flags:?}"));
            assert!(!result.events.contains("download:"));
        }
    }

    #[test]
    #[ignore = "requires approved network-denied installer fixture containment"]
    fn explicit_source_modes_reach_only_the_fake_cargo() {
        for (flags, env, expected) in [
            (
                vec!["-FromSource"],
                vec![],
                "install --path . --locked --force",
            ),
            (
                vec!["-FromGit"],
                vec![("OTA_GIT_REV", "fixture-rev")],
                "--rev fixture-rev",
            ),
            (vec![], vec![("OTA_INSTALL_MODE", "source")], "--path ."),
        ] {
            let f = Fixture::new();
            let result = run(&f, None, &env, &flags);
            assert!(!result.success, "fake Cargo exits with a refusal");
            assert_eq!(result.count("cargo"), 1, "{}", result.output);
            assert!(result.events.contains(expected), "{}", result.events);
            assert_eq!(result.count("extract"), 0);
            assert_eq!(result.count("payload"), 0);
            assert!(!result.events.contains("download:"));
        }
    }
}
