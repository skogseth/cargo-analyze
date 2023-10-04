use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display};
use std::io::BufRead;
use std::str::FromStr;

use cargo_metadata::BuildScript;
use cargo_metadata::Message;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LibraryType {
    Static,
    Dynamic,
    Framework,
}

impl LibraryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Dynamic => "dylib",
            Self::Framework => "framework",
        }
    }
}

impl FromStr for LibraryType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "static" => Ok(LibraryType::Static),
            "dylib" => Ok(LibraryType::Dynamic),
            "framework" => Ok(LibraryType::Framework),
            _ => Err(s.to_owned()),
        }
    }
}

impl Display for LibraryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(self.as_str())
    }
}

/// Parses link-strings of the format '[kind=]name'.
//
/// Based on https://doc.rust-lang.org/rustc/command-line-arguments.html#option-l-link-lib
///
/// Examples:
/// "dylib=z" => Some(LibraryType::Dynamic), "z"
/// "static=z" => Some(LibraryType::Static), "z"
/// "z" => None, "z" (static for "static executables", otherwise dynamic)
fn parse_library_output(s: String) -> (Option<LibraryType>, String) {
    match s.split_once('=') {
        Some((lib_type, lib_name)) => {
            let lib_type = lib_type.parse().unwrap();
            let lib_name = lib_name.to_owned();
            (Some(lib_type), lib_name)
        }
        None => (None, s),
    }
}

#[derive(Debug, Clone)]
pub struct LinkedLibs {
    known: BTreeMap<LibraryType, BTreeSet<String>>,
    unknown: BTreeSet<String>,
}

impl LinkedLibs {
    pub fn new() -> Self {
        Self {
            known: BTreeMap::new(),
            unknown: BTreeSet::new(),
        }
    }

    pub fn add(&mut self, lib_type: Option<LibraryType>, lib_name: String) -> bool {
        match lib_type {
            Some(typ) => self.known.entry(typ).or_default().insert(lib_name),
            None => self.unknown.insert(lib_name),
        }
    }

    pub fn from_metadata(reader: impl BufRead) -> Self {
        let mut libs = Self::new();

        for message in Message::parse_stream(reader) {
            if let Message::BuildScriptExecuted(script) = message.unwrap() {
                let BuildScript { linked_libs, .. } = script;

                if linked_libs.is_empty() {
                    continue;
                }

                for lib in linked_libs {
                    // Convert Utf8PathBuf to String
                    let lib = lib.into_string();

                    // Parse output
                    let (lib_type, lib_name) = parse_library_output(lib);

                    // Add to libs
                    libs.add(lib_type, lib_name);
                }
            }
        }

        libs
    }

    pub fn all_empty(&self) -> bool {
        self.known.is_empty() && self.unknown.is_empty()
    }
}

impl Default for LinkedLibs {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for LinkedLibs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        if self.all_empty() {
            return Ok(());
        }

        writeln!(f, "Linked libraries:")?;

        for (key, val) in self.known.iter() {
            writeln!(f, "{key}: {}", set_to_string(val))?;
        }

        writeln!(f, "unknown: {}", set_to_string(&self.unknown))?;

        Ok(())
    }
}

fn set_to_string<T: Display>(set: &BTreeSet<T>) -> String {
    let mut iter = set.iter();

    let Some(mut prev) = iter.next() else {
        return String::new();
    };

    let mut s = String::from("{ ");

    while let Some(item) = iter.next() {
        s += &format!("{prev}, ");
        prev = item;
    }

    s += &prev.to_string();
    s + " }"
}
