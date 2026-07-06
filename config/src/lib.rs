#![deny(missing_docs)]
//! Library to manage the configuration of the afrim input method.
//!
//! It's based on the top of the [`toml`] crate.
//!
//! # Example
//!
//! ```no_run
//! use afrim_config::Config;
//! use std::path::Path;
//!
//! let filepath = Path::new("./data/config_sample.toml");
//! let conf = Config::from_file(&filepath).unwrap();
//!
//! # assert_eq!(conf.extract_data().keys().len(), 23);
//! # #[cfg(feature = "rhai")]
//! # assert_eq!(conf.extract_translators().unwrap().keys().len(), 2);
//! # assert_eq!(conf.extract_translation().keys().len(), 4);
//! ```
//!
//! In case that you want control the filesystem (reading of file), you can use the
//! [`Config::from_filesystem`] method.
//!
//! # Example
//!
//! ```
//! use afrim_config::{Config, FileSystem};
//! use std::{error, path::Path, string::String};
//!
//! // Implements a custom filesystem.
//! struct File {
//!     source: String,
//! }
//!
//! impl File {
//!     pub fn new(source: String) -> Self {
//!         Self { source }
//!     }
//! }
//!
//! impl FileSystem for File {
//!     fn read_to_string(&self, filepath: &Path) -> Result<String, std::io::Error> {
//!         Ok(self.source.to_string())
//!     }
//! }
//!
//! // Sets the config file.
//! let config_file = File::new(r#"
//! [core]
//! auto_commit = false
//!
//! [data]
//! "n*" = "ŋ"
//! "#.to_owned());
//!
//! // Loads the config file.
//! let config = Config::from_filesystem(&Path::new("."), &config_file).unwrap();
//!
//! assert_eq!(config.core.clone().unwrap().auto_commit, Some(false));
//! // Note that the auto_capitalize is enabled by default.
//! assert_eq!(
//!     Vec::from_iter(config.extract_data().into_iter()),
//!     vec![("N*".to_owned(), "Ŋ".to_owned()), ("n*".to_owned(), "ŋ".to_owned())]
//! );
//! ```

use anyhow::{anyhow, Context, Result};
pub use indexmap::IndexMap;
#[cfg(feature = "rhai")]
use rhai::Engine;
#[cfg(feature = "rhai")]
pub use rhai::AST;
use serde::Deserialize;
use std::{fs, path::Path};
use toml::{self};

/// Trait to customize the filesystem.
pub trait FileSystem {
    /// Alternative to the fs::read_to_string.
    fn read_to_string(&self, filepath: &Path) -> Result<String, std::io::Error>;
}

// Representation of the std::fs.
struct StdFileSystem;

impl FileSystem for StdFileSystem {
    fn read_to_string(&self, filepath: &Path) -> Result<String, std::io::Error> {
        fs::read_to_string(filepath)
    }
}

/// Holds information about a configuration.
///
/// # Example
///
/// ```no_run
/// # use afrim_config::{Config, FileSystem};
/// # use std::{error, path::Path, string::String};
/// #
/// # // Implements a custom filesystem.
/// # struct File {
/// #     source: String,
/// # }
/// #
/// # impl File {
/// #     pub fn new(source: String) -> Self {
/// #         Self { source }
/// #     }
/// # }
/// #
/// # impl FileSystem for File {
/// #     fn read_to_string(&self, filepath: &Path) -> Result<String, std::io::Error> {
/// #         Ok(self.source.to_string())
/// #     }
/// # }
/// #
/// # // Sets the config file.
/// # let config_file = File::new(r#"
/// [info]
/// description = "Sample Config File"
/// version = "2023-10-02"
///
/// [data]
/// 2a_ = "á̠"
/// ".?" = { value = "ʔ", alias = ["?."] }
/// emoji = { path = "./emoji.toml" }
///
/// [translation]
/// hey = "hi"
/// hi = { value = "hello", alias = ["hey"] }
/// hola = { values = ["hello"], alias = [] }
/// dictionary = { path = "./dictionary.toml" }
///
/// [translators]
/// date = "./scripts/datetime/date.rhai"
/// # "#.to_owned());
///
/// # // Loads the config file.
/// # Config::from_filesystem(&Path::new("."), &config_file).unwrap();
/// ```
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    /// The core config.
    pub core: Option<CoreConfig>,
    data: Option<IndexMap<String, Data>>,
    #[cfg(feature = "rhai")]
    translators: Option<IndexMap<String, Data>>,
    translation: Option<IndexMap<String, Data>>,
}

/// Core information about a configuration.
///
/// # Example
///
/// ```
/// # use afrim_config::{Config, FileSystem};
/// # use std::{error, path::Path, string::String};
/// #
/// # // Implements a custom filesystem.
/// # struct File {
/// #     source: String,
/// # }
/// #
/// # impl File {
/// #     pub fn new(source: String) -> Self {
/// #         Self { source }
/// #     }
/// # }
/// #
/// # impl FileSystem for File {
/// #     fn read_to_string(&self, filepath: &Path) -> Result<String, std::io::Error> {
/// #         Ok(self.source.to_string())
/// #     }
/// # }
/// #
/// # // Sets the config file.
/// # let config_file = File::new(r#"
/// [core]
/// buffer_size = 32
/// auto_capitalize = false
/// page_size = 10
/// auto_commit = true
/// min_confidence = 0.7
/// # "#.to_owned());
/// #
/// # // Loads the config file.
/// # Config::from_filesystem(&Path::new("."), &config_file).unwrap();
/// ```
#[derive(Deserialize, Debug, Clone)]
pub struct CoreConfig {
    /// The size of the memory (history).
    /// The number of elements that should be tracked.
    pub buffer_size: Option<usize>,
    auto_capitalize: Option<bool>,
    /// The max numbers of predicates to display.
    pub page_size: Option<usize>,
    /// Whether the predicate should be automatically committed.
    pub auto_commit: Option<bool>,
    /// The minimun confidence of the selected predicates.
    pub min_confidence: Option<f64>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
enum Data {
    Simple(String),
    Multi(Vec<String>),
    File(DataFile),
    Detailed(DetailedData),
    MoreDetailed(MoreDetailedData),
}

#[derive(Deserialize, Debug, Clone)]
struct DataFile {
    path: String,
}

#[derive(Deserialize, Debug, Clone)]
struct DetailedData {
    value: String,
    alias: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct MoreDetailedData {
    values: Vec<String>,
    alias: Vec<String>,
}

// Helper function to capitalize a string.
fn capitalize(value: &str) -> Option<String> {
    let mut chars = value.chars();
    let first_char = chars.next()?;
    if !first_char.is_lowercase() {
        return None;
    }
    let mut cap_key = String::with_capacity(value.len());
    for c in first_char.to_uppercase() {
        cap_key.push(c);
    }
    cap_key.push_str(chars.as_str());
    Some(cap_key)
}

impl Config {
    /// Load the configuration from a file.
    pub fn from_file(filepath: &Path) -> Result<Self> {
        Self::from_filesystem(filepath, &StdFileSystem {})
    }

    /// Loads the configuration from a file in using a specified filesystem.
    pub fn from_filesystem(filepath: &Path, fs: &impl FileSystem) -> Result<Self> {
        let mut data = IndexMap::new();
        #[cfg(feature = "rhai")]
        let mut translators = IndexMap::new();
        let mut translation = IndexMap::new();

        let content = fs
            .read_to_string(filepath)
            .with_context(|| format!("Couldn't open file {filepath:?}."))?;
        let root_config: Self = toml::from_str(&content).with_context(|| {
            format!("Failed to parse the root configuration file {filepath:?}.")
        })?;

        let root_core = root_config.core.clone();

        // Pass the already-parsed config directly instead of calling
        // read_config, which would re-read and re-parse the root same file.
        Self::process_config(
            root_config,
            filepath,
            fs,
            &mut data,
            #[cfg(feature = "rhai")]
            &mut translators,
            &mut translation,
        )?;

        Ok(Config {
            core: root_core,
            data: Some(data),
            #[cfg(feature = "rhai")]
            translators: Some(translators),
            translation: Some(translation),
        })
    }

    /// Reads and parses the file at `filepath`, then delegates to
    /// [`Self::process_config`].  Only called for *nested* includes the root
    /// file is handled by `from_filesystem` without a second I/O round-trip.
    fn read_config(
        filepath: &Path,
        fs: &impl FileSystem,
        data: &mut IndexMap<String, Data>,
        #[cfg(feature = "rhai")] translators: &mut IndexMap<String, Data>,
        translation: &mut IndexMap<String, Data>,
    ) -> Result<()> {
        let content = fs
            .read_to_string(filepath)
            .with_context(|| format!("Couldn't open file {filepath:?}."))?;
        let config: Self = toml::from_str(&content)
            .with_context(|| format!("Failed to parse configuration file {filepath:?}."))?;

        Self::process_config(
            config,
            filepath,
            fs,
            data,
            #[cfg(feature = "rhai")]
            translators,
            translation,
        )
    }

    /// Populates `data`, `translators`, and `translation` from an
    /// already-parsed [`Config`], recursively following path-valued entries
    /// via [`Self::read_config`].
    fn process_config(
        config: Self,
        filepath: &Path,
        fs: &impl FileSystem,
        data: &mut IndexMap<String, Data>,
        #[cfg(feature = "rhai")] translators: &mut IndexMap<String, Data>,
        translation: &mut IndexMap<String, Data>,
    ) -> Result<()> {
        let config_path = filepath.parent().unwrap();
        let auto_capitalize = config
            .core
            .as_ref()
            .and_then(|c| c.auto_capitalize)
            .unwrap_or(true);

        for (key, value) in config.data.unwrap_or_default() {
            match value {
                Data::File(DataFile { path }) => {
                    let nested = config_path.join(&path);
                    Self::read_config(
                        &nested,
                        fs,
                        data,
                        #[cfg(feature = "rhai")]
                        translators,
                        translation,
                    )?;
                }
                Data::Simple(value) => {
                    // Borrow key/value for the capitalized entry before moving
                    // them into the main insert below.
                    if auto_capitalize && !key.is_empty() {
                        if let Some(cap_key) = capitalize(&key) {
                            data.entry(cap_key)
                                .or_insert_with(|| Data::Simple(value.to_uppercase()));
                        }
                    }
                    data.insert(key, Data::Simple(value));
                }
                Data::Detailed(DetailedData { value, alias }) => {
                    for k in alias.iter().chain(std::iter::once(&key)) {
                        data.insert(k.clone(), Data::Simple(value.clone()));
                        if auto_capitalize {
                            if let Some(cap_key) = capitalize(k) {
                                data.entry(cap_key)
                                    .or_insert_with(|| Data::Simple(value.to_uppercase()));
                            }
                        }
                    }
                }
                _ => Err(anyhow!("{value:?} not allowed in the data table."))
                    .with_context(|| format!("Invalid configuration file {filepath:?}."))?,
            }
        }

        // Translation
        #[cfg(feature = "rhai")]
        for (key, value) in config.translators.unwrap_or_default() {
            match value {
                Data::File(DataFile { path }) => {
                    let nested = config_path.join(&path);
                    Self::read_config(&nested, fs, data, translators, translation)?;
                }
                Data::Simple(path) => {
                    let abs_path = config_path
                        .join(&path)
                        .into_os_string()
                        .into_string()
                        .unwrap();
                    translators.insert(key, Data::Simple(abs_path));
                }
                _ => Err(anyhow!("{value:?} not allowed in the translator table"))
                    .with_context(|| format!("Invalid configuration file {filepath:?}."))?,
            }
        }

        for (key, value) in config.translation.unwrap_or_default() {
            match value {
                Data::File(DataFile { path }) => {
                    let nested = config_path.join(&path);
                    Self::read_config(
                        &nested,
                        fs,
                        data,
                        #[cfg(feature = "rhai")]
                        translators,
                        translation,
                    )?;
                }
                Data::Simple(_) | Data::Multi(_) => {
                    translation.insert(key, value);
                }
                Data::Detailed(DetailedData { value, alias }) => {
                    for e in alias.iter().chain(std::iter::once(&key)) {
                        translation.insert(e.clone(), Data::Simple(value.clone()));
                    }
                }
                Data::MoreDetailed(MoreDetailedData { values, alias }) => {
                    for k in alias.iter().chain(std::iter::once(&key)) {
                        translation.insert(k.clone(), Data::Multi(values.clone()));
                    }
                }
            }
        }

        Ok(())
    }

    /// Extracts the data from the configuration.
    pub fn extract_data(&self) -> IndexMap<String, String> {
        // with_capacity avoids incremental reallocations during iteration.
        let Some(data) = &self.data else {
            return IndexMap::new();
        };
        let mut result = IndexMap::with_capacity(data.len());
        for (key, value) in data {
            if let Data::Simple(v) = value {
                result.insert(key.clone(), v.clone());
            }
        }
        result
    }

    /// Extracts the translators from the configuration.
    #[cfg(feature = "rhai")]
    pub fn extract_translators(&self) -> Result<IndexMap<String, AST>> {
        self.extract_translators_using_filesystem(&StdFileSystem {})
    }

    /// Extracts the translators from the configuration using the specified
    /// filesystem.
    #[cfg(feature = "rhai")]
    pub fn extract_translators_using_filesystem(
        &self,
        fs: &impl FileSystem,
    ) -> Result<IndexMap<String, AST>> {
        // with_capacity avoids incremental reallocations during iteration.
        let Some(translators) = &self.translators else {
            return Ok(IndexMap::new());
        };
        let engine = Engine::new();
        let mut result = IndexMap::with_capacity(translators.len());

        for (name, file_path) in translators {
            let Data::Simple(file_path) = file_path else {
                continue;
            };
            let file_path = Path::new(file_path.as_str());
            let parent = file_path.parent().unwrap().to_str().unwrap();
            let header = format!(r#"const DIR = {parent:?};"#);
            let body = fs
                .read_to_string(file_path)
                .with_context(|| format!("Couldn't open script file {file_path:?}."))?;
            let ast = engine
                .compile(body)
                .with_context(|| format!("Failed to parse script file {file_path:?}."))?;
            let ast = engine.compile(header).unwrap().merge(&ast);
            result.insert(name.clone(), ast);
        }

        Ok(result)
    }

    /// Extracts the translation from the configuration.
    pub fn extract_translation(&self) -> IndexMap<String, Vec<String>> {
        // with_capacity avoids incremental reallocations during iteration.
        let Some(translation) = &self.translation else {
            return IndexMap::new();
        };
        let mut result = IndexMap::with_capacity(translation.len());
        for (key, value) in translation {
            let value = match value {
                Data::Simple(v) => vec![v.clone()],
                Data::Multi(v) => v.clone(),
                _ => continue,
            };
            result.insert(key.clone(), value);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::Config;
    use std::path::Path;

    #[test]
    fn from_file() {
        let conf = Config::from_file(Path::new("./data/config_sample.toml")).unwrap();

        assert_eq!(
            conf.core.as_ref().map(|core| {
                assert_eq!(core.buffer_size.unwrap(), 64);
                assert!(!core.auto_capitalize.unwrap());
                assert!(!core.auto_commit.unwrap());
                assert_eq!(core.page_size.unwrap(), 10);
                assert_eq!(core.min_confidence.unwrap(), 0.7);
                true
            }),
            Some(true)
        );

        let data = conf.extract_data();
        assert_eq!(data.keys().len(), 23);

        // data and core not provided
        let conf = Config::from_file(Path::new("./data/blank_sample.toml")).unwrap();
        let data = conf.extract_data();
        assert_eq!(data.keys().len(), 0);

        // parsing error
        let conf = Config::from_file(Path::new("./data/invalid_file.toml"));
        assert!(conf.is_err());

        // config file not found
        let conf = Config::from_file(Path::new("./data/not_found"));
        assert!(conf.is_err());
    }

    #[test]
    fn from_invalid_file() {
        // invalid data
        let conf = Config::from_file(Path::new("./data/invalid_data.toml"));
        assert!(conf.is_err());
    }

    #[cfg(feature = "rhai")]
    #[test]
    fn from_file_with_translators() {
        // invalid translator
        let conf = Config::from_file(Path::new("./data/invalid_translator.toml"));
        assert!(conf.is_err());

        let conf = Config::from_file(Path::new("./data/config_sample.toml")).unwrap();
        let translators = conf.extract_translators().unwrap();
        assert_eq!(translators.keys().len(), 2);

        // translators not provided
        let conf = Config::from_file(Path::new("./data/blank_sample.toml")).unwrap();
        let translators = conf.extract_translators().unwrap();
        assert_eq!(translators.keys().len(), 0);

        // scripts parsing error
        let conf = Config::from_file(Path::new("./data/bad_script2.toml")).unwrap();
        assert!(conf.extract_translators().is_err());

        // script file not found
        let conf = Config::from_file(Path::new("./data/bad_script.toml")).unwrap();
        assert!(conf.extract_translators().is_err());
    }

    #[test]
    fn from_file_with_translation() {
        let conf = Config::from_file(Path::new("./data/config_sample.toml")).unwrap();
        let translation = conf.extract_translation();
        assert_eq!(translation.keys().len(), 4);

        let conf = Config::from_file(Path::new("./data/blank_sample.toml")).unwrap();
        let translation = conf.extract_translation();
        assert_eq!(translation.keys().len(), 0);
    }

    #[test]
    fn from_filesystem() {
        use crate::FileSystem;
        use std::fs;

        #[derive(Default)]
        struct FilterFileSystem;

        impl FileSystem for FilterFileSystem {
            fn read_to_string(&self, filepath: &Path) -> Result<String, std::io::Error> {
                let file_stem = filepath.file_stem().unwrap();

                Ok(if file_stem == "data_sample" {
                    fs::read_to_string(filepath)?
                } else {
                    String::new()
                })
            }
        }

        let fs = FilterFileSystem {};
        let filepath = Path::new("./data/data_sample.toml");
        let conf = Config::from_filesystem(filepath, &fs).unwrap();

        assert_eq!(conf.extract_data().keys().len(), 13);
        #[cfg(feature = "rhai")]
        assert_eq!(conf.extract_translators().unwrap().keys().len(), 0);
        assert_eq!(conf.extract_translation().keys().len(), 0);
    }
}
