//!
#![warn(missing_docs)]
mod code_writer;

use code_writer::CodeWriter;
use frpc_codegen::CodeGen;
use frpc_message::TypeDef;
use std::{env, path::PathBuf};
use std::{fmt::Write, fs};

#[cfg(feature = "serde")]
use serde::Deserialize;

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

impl Config {
    /// Gererate bindings for verious languages/platforms.
    pub fn generate_binding(&self, defs: &[&TypeDef]) -> Result {
        for type_def in defs {
            let writer = CodeWriter::from(*type_def);
            if let Some(config) = &self.typescript {
                writer.generate_typescript_binding(config)?;
            }
        }
        Ok(())
    }
}

/// JS/TS codegen configuration
pub mod typescript {
    use super::*;

    /// JS/TS codegen configuration
    #[derive(Debug, Clone)]
    #[cfg_attr(feature = "serde", derive(Deserialize))]
    pub struct Config {
        /// Specify an output folder for all emitted files.
        #[cfg_attr(feature = "serde", serde(default = "out_dir"))]
        #[cfg_attr(feature = "serde", serde(rename = "out-dir"))]
        pub out_dir: PathBuf,
        /// preserve file extension when importing modules
        #[cfg_attr(feature = "serde", serde(default))]
        #[cfg_attr(feature = "serde", serde(rename = "preserve-import-extension"))]
        pub preserve_import_extension: bool,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                out_dir: out_dir(),
                preserve_import_extension: false,
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
/// codegen configuration
pub struct Config {
    /// It generate js/ts bindings when present
    pub typescript: Option<typescript::Config>,
}

#[doc(hidden)]
pub fn out_dir() -> PathBuf {
    if let Ok(dir) = env::var("CARGO_TARGET_DIR") {
        return dir.into();
    }
    if let Ok(cwd) = env::current_dir() {
        return cwd;
    }
    manifest_dir()
}

#[doc(hidden)]
pub fn manifest_dir() -> PathBuf {
    if let Ok(dir) = env::var("CARGO_MANIFEST_DIR") {
        return dir.into();
    }
    "./".into()
}
