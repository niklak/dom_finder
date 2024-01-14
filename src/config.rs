use serde::{Deserialize, Serialize};

use crate::errors::ValidationError;

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum CastType {
    #[default]
    String,
    Bool,
    Int,
    Float,
}
/// `Config` is a struct that represents the configuration of the `Finder`.
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Config {
    ///`Name` represents a key for result the and every inline element if it is presented.  
    pub name: String,
    ///`BasePath` is a selector's path to the element to handle. May be omitted if the `inherit` option is set to `true`.
    #[serde(default)]
    pub base_path: String,
    ///`Extract` is a selector's path to the element to handle. It may be either extract or children.
    #[serde(default)]
    pub extract: String,
    ///`Cast` is a type of the result value. Accepted values are `text`, `inner_text`, `html` or an html-attribute name.
    #[serde(default)]
    pub cast: CastType,
    ///`JoinSep` is a separator for joining the result values. Works only when `many` is set to `true` and there is no descendant config.
    #[serde(default)]
    pub join_sep: String,
    ///`Many` is a flag that indicates whether the result is an array or not.
    #[serde(default)]
    pub many: bool,
    #[serde(default)]
    ///`Enumerate` adds a index field to the result if it is an array of objects.
    pub enumerate: bool,
    #[serde(default)]
    ///`Inherit` will use parent's base_path (and parent's selector) if it is set to `true`.
    pub inherit: bool,
    #[serde(default)]
    ///`Parent` will use .parent() method of the matcher if it is set to `true`. It means it will use direct parent of the selection. It is distinct from `inherit` option.
    pub parent: bool,
    ///`FirstOccurrence` will stop parsing descendant selections when it will encounter the first non-empty result.
    #[serde(default)]
    pub first_occurrence: bool,
    #[serde(default)]
    ///`RemoveSelection` will remove the selection from the document (html) if it is set to `true`. Currently this implementation is not finished.
    pub remove_selection: bool,
    #[serde(default)]
    ///`Flatten` if it is set to `true` then it will unpack descendant map into parent map.
    pub flatten: bool,
    ///`SplitPath` if it is set to `true` then it will split base_path by `,` for more flexibility. Not implemented yet
    #[serde(default)]
    pub split_path: bool,
    #[serde(default)]
    pub pipeline: Vec<Vec<String>>,
    ///Children is a list of descendant `Config`.
    #[serde(default)]
    pub children: Vec<Config>,
}

impl Config {
    /// Creates a new `Config` instance from the given YAML string.
    pub fn from_yaml(data: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(data)
    }

    /// Validates the `Config` instance.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.name.is_empty() {
            return Err(ValidationError::FieldIsMissing("name".to_string()));
        }
        if self.base_path.is_empty() && !self.inherit {
            // The case when base_path is empty and inherit is true, resolved in Finder::new
            return Err(ValidationError::FieldIsMissing("base_path".to_string()));
        }
        let must_extract = !self.extract.is_empty();
        let must_dive = !self.children.is_empty();
        if must_extract == must_dive {
            return Err(ValidationError::ExtractOrDive);
        }
        Ok(())
    }
}

#[cfg(feature = "json_cfg")]
impl Config {
    /// Creates a new `Config` instance from the given JSON string.
    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_is_valid() {
        let cfg_yml: &str = r"
        name: footer_links
        base_path: footer a[href]
        many: true
        extract: href
        ";
        let cfg = Config::from_yaml(cfg_yml).unwrap();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn config_with_children_is_valid() {
        let cfg_yml: &str = r"
        name: root
        base_path: body
        children:
            - name: footer_links
              base_path: footer a[href]
              many: true
              extract: href
        ";
        let cfg = Config::from_yaml(cfg_yml).unwrap();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    #[should_panic]
    fn config_missing_name_panic() {
        let cfg_yml: &str = r"
            base_path: a[href]
            many: true
            extract: href
        ";
        let cfg = Config::from_yaml(cfg_yml).unwrap();
        assert!(cfg.validate().is_err());
    }
    #[test]
    fn config_missing_name() {
        let cfg_yml: &str = r"
        name:
        base_path: a[href]
        many: true
        extract: href
        ";
        let cfg = Config::from_yaml(cfg_yml).unwrap();
        assert!(cfg.validate().is_err());
    }
    #[test]
    fn config_missing_base_path() {
        let cfg_yml: &str = r"
        name: footer_links
        base_path:
        many: true
        extract: href
        ";
        let cfg = Config::from_yaml(cfg_yml).unwrap();
        assert!(cfg.validate().is_err());
    }
    #[test]
    fn config_no_extract_no_children() {
        let cfg_yml: &str = r"
            name: footer_links
            base_path: a[href]
            many: true
        ";
        let cfg = Config::from_yaml(cfg_yml).unwrap();
        assert!(cfg.validate().is_err());
    }
    #[test]
    fn config_with_extract_with_children() {
        let cfg_yml: &str = r"
            name: footer_links
            base_path: footer p:has(a[href])
            many: true
            extract: text
            children:
                - name: link
                  base_path: a[href]
                  extract: href
        ";
        let cfg = Config::from_yaml(cfg_yml).unwrap();
        assert!(cfg.validate().is_err());
    }
}
