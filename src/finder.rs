use std::borrow::Cow;

use dom_query::{Document, Matcher, Selection};
use tendril::StrTendril;

use crate::errors::ParseError;

use super::config::{CastType, Config};
use super::pipeline::Pipeline;
use super::value::{InnerMap, Value};

/// The name of the field that contains the index of the element in the result array.
const INDEX_FIELD: &str = "index";

const EXTRACT_TEXT: &str = "text";
const EXTRACT_INNER_TEXT: &str = "inner_text";
const EXTRACT_HTML: &str = "html";
const EXTRACT_INNER_HTML: &str = "inner_html";

/// Finder is the main struct that is used to parse the html
#[derive(Debug)]
pub struct Finder<'a> {
    name: Cow<'a, str>,
    extract: Cow<'a, str>,
    cast: CastType,
    join_sep: Cow<'a, str>,
    many: bool,
    enumerate: bool,
    inherit: bool,
    parent: bool,
    first_occurrence: bool,
    remove_selection: bool,
    flatten: bool,
    children: Vec<Finder<'a>>,
    matcher: Option<Matcher>,
    pipeline: Option<Pipeline<'a>>,
}

impl<'a> Finder<'a> {
    /// Creates a new Finder instance from the given `Config's` instance
    /// it's lifetime pretty depends on `Config`'s lifetime.
    ///
    /// # Arguments
    /// * `config` - `Config` instance
    ///
    /// # Examples
    /// ```
    /// use dom_finder::{Config, Finder};
    /// let cfg_yml: &str = r"
    /// name: all_links
    /// base_path: html body a[href]
    /// many: true
    /// extract: href
    /// ";
    /// let cfg = Config::from_yaml(cfg_yml).unwrap();
    /// let finder = Finder::new(&cfg);
    /// assert!(finder.is_ok());
    /// ```
    pub fn new<'b>(config: &'b Config) -> Result<Finder<'a>, ParseError> {
        Finder::from_config(config, true)
    }

    fn from_config<'b>(config: &'b Config, is_root: bool) -> Result<Finder<'a>, ParseError> {
        config.validate()?;
        let base_path = config.base_path.as_str();
        let matcher = if !base_path.is_empty() {
            Matcher::new(base_path).ok()
        } else {
            None
        };

        if matcher.is_none() && (is_root || !config.inherit) {
            return Err(ParseError::RequireMatcher);
        }

        let pipeline = if !config.pipeline.is_empty() {
            Some(Pipeline::new(&config.pipeline)?)
        } else {
            None
        };
        let mut p = Finder {
            name: Cow::from(config.name.clone()),
            extract: Cow::from(config.extract.clone()),
            cast: config.cast,
            join_sep: Cow::from(config.join_sep.clone()),
            many: config.many,
            enumerate: config.enumerate,
            inherit: config.inherit,
            parent: config.parent,
            first_occurrence: config.first_occurrence,
            remove_selection: config.remove_selection,
            flatten: config.flatten,
            children: Vec::new(),
            matcher,
            pipeline,
        };

        for inline_config in config.children.iter() {
            p.children.push(Finder::from_config(inline_config, false)?);
        }
        Ok(p)
    }
    /// Either returns the matcher of or panics
    fn get_matcher(&self) -> &Matcher {
        match self.matcher {
            Some(ref m) => m,
            None => {
                panic!("no matcher")
            }
        }
    }
    /// Parses the given html and returns the result as a `Value`
    /// # Arguments
    /// * `html` - the html to parse
    ///
    /// # Returns
    /// `Value::Object`
    ///
    /// # Examples
    /// ```
    /// use dom_finder::{Config, Finder};
    /// let cfg_yml: &str = r"
    /// name: all_links
    /// base_path: html body a[href]
    /// many: true
    /// extract: href
    /// ";
    /// let cfg = Config::from_yaml(cfg_yml).unwrap();
    /// let finder = Finder::new(&cfg).unwrap();
    /// let html = r#"<html><body><a href="https://example.com">example</a></body></html>"#;
    /// let res = finder.parse(html);
    /// let link: Option<String> = res.from_path("all_links.0").and_then(|v| v.into());
    /// assert_eq!(link.unwrap(), "https://example.com");
    /// ```
    pub fn parse(&self, html: &str) -> Value {
        let doc = Document::from(html);
        self.parse_document(&doc)
    }

    /// Parses the given `Document` and returns the result as a `Value`.
    /// Useful when you need access to the `Document` outside of the `Finder`.
    /// # Arguments
    /// * `doc` - the `Document` to parse
    /// # Returns
    /// `Value::Object`
    pub fn parse_document(&self, doc: &Document) -> Value {
        let sel = Selection::from(doc.root());
        let val = self.parse_value(&sel);
        let mut m: InnerMap = InnerMap::default();
        m.insert(self.name.to_string(), val);
        Value::Object(m)
    }

    /// Parses the given Selection and returns the result as a `Value`
    /// # Arguments
    /// * `root` - the root Selection to parse
    /// # Returns
    /// `Value` representing the parsed result
    ///
    /// This method handles different scenarios.
    pub fn parse_value(&self, root: &Selection) -> Value {
        let sel: Selection = if self.inherit {
            root.clone()
        } else if self.parent {
            root.select_matcher(self.get_matcher()).parent()
        } else if self.many {
            root.select_matcher(self.get_matcher())
        } else {
            root.select_single_matcher(self.get_matcher())
        };

        if !sel.exists() {
            return Value::Null;
        }

        let has_children = !self.children.is_empty();

        let v = match (has_children, self.many) {
            (true, true) => self.parse_children_to_slice_maps(&sel),
            (true, false) => self.parse_children_to_map(&sel),
            (false, true) => {
                let tmp_res: Vec<String> = sel
                    .iter()
                    .filter_map(|item| self.handle_selection(&item))
                    .collect();

                if !self.join_sep.is_empty() {
                    Value::from(tmp_res.join(&self.join_sep))
                } else {
                    Value::from_iter(tmp_res.into_iter().map(|it| cast_value(it, self.cast)))
                }
            }
            _ => {
                let item = sel.first();
                if let Some(tmp_val) = self.handle_selection(&item) {
                    cast_value(tmp_val, self.cast)
                } else {
                    Value::Null
                }
            }
        };

        if self.remove_selection {
            sel.remove();
        }
        v
    }

    /// Handles the result selection according to the extract type and the pipeline
    fn handle_selection(&self, sel: &Selection) -> Option<String> {
        extract_data(sel, &self.extract).map(|extracted| {
            let extracted = extracted.to_string();
            if let Some(ref pipeline) = self.pipeline {
                pipeline.handle(extracted)
            } else {
                extracted
            }
        })
    }

    fn parse_children_to_map(&self, element: &Selection) -> Value {
        let mut m = InnerMap::default();
        for inline in self.children.iter() {
            let v = inline.parse_value(element);
            if v.is_empty() {
                continue;
            }

            if inline.flatten {
                if let Value::Object(in_map) = v {
                    for (k, val) in in_map {
                        m.insert(k, val);
                    }
                } else {
                    m.insert(inline.name.to_string(), v);
                }
            } else {
                m.insert(inline.name.to_string(), v);
            }

            if self.first_occurrence {
                break;
            }
        }
        Value::Object(m)
    }

    fn parse_children_to_slice_maps(&self, selection: &Selection) -> Value {
        let mut values: Vec<InnerMap> = Vec::new();
        for item in selection.iter() {
            let mut m: InnerMap = InnerMap::default();
            for inline in self.children.iter() {
                let v = inline.parse_value(&item);
                if v.is_empty() {
                    continue;
                }

                if inline.flatten {
                    if let Value::Object(obj) = v {
                        for (key, val) in obj {
                            // push flat maps right in the result values
                            m.insert(key, val);
                        }
                    } else {
                        m.insert(inline.name.to_string(), v);
                    }
                } else {
                    m.insert(inline.name.to_string(), v);
                }

                if self.first_occurrence {
                    break;
                }
            }
            if !m.is_empty() {
                values.push(m);
            }
        }
        if self.enumerate {
            for (i, item) in values.iter_mut().enumerate() {
                item.insert(INDEX_FIELD.to_string(), Value::Int(i as i64));
            }
        }

        Value::from_iter(values.into_iter().map(Value::Object))
    }
}

/// Casts the value to the specified type
/// The cast type can be one of the following:
/// - bool - casts the value to bool, if the value is empty it is `false`, otherwise it is `true`.
/// - int - casts the value to int
/// - float - casts the value to float
/// - string - casts the value to string
/// # Arguments
/// * `s` - `String`, the value to cast
/// * `cast` - `CastType`, the type to cast to
fn cast_value(s: String, cast: CastType) -> Value {
    match cast {
        CastType::Bool => {
            let mut x: bool = false;
            if !s.is_empty() {
                x = true;
            }
            Value::from(x)
        }
        CastType::Int => Value::from(s.parse::<i64>().unwrap_or(0)),
        CastType::Float => Value::from(s.parse::<f64>().unwrap_or(0.0)),
        _ => Value::from(s),
    }
}

impl<'a> TryFrom<Config> for Finder<'a> {
    type Error = ParseError;
    fn try_from(config: Config) -> Result<Self, Self::Error> {
        Finder::new(&config)
    }
}

/// Extracts the data from the given selection according to the extract type
/// The extract type can be one of the following:
/// - text - extracts the text of the selection
/// - inner_text - extracts the text of the selection without the text of the children
/// - html - extracts the html of the selection
/// - inner_html - extracts the inner html of the selection without it's root node.
#[inline(always)]
fn extract_data(sel: &Selection, extract_type: &str) -> Option<StrTendril> {
    match extract_type {
        EXTRACT_TEXT => Some(sel.text()),
        EXTRACT_INNER_TEXT => Some(get_inner_text(sel)),
        EXTRACT_HTML => sel.try_html(),
        EXTRACT_INNER_HTML => sel.try_inner_html(),
        _ => sel.attr(extract_type),
    }
}

/// Returns the inner text of the selection without the text of the children
#[inline(always)]
fn get_inner_text(sel: &Selection) -> StrTendril {
    let nodes = sel.nodes();

    if nodes.is_empty() {
        return StrTendril::new();
    }

    let base_sel = Selection::from(nodes[0].clone());
    base_sel.children().remove();
    base_sel.text()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_finder_success() {
        let cfg_yml: &str = r"
        name: root
        base_path: html
        children:
          - name: links
            base_path: a[href]
            many: true
            children:
             - name: link
               inherit: true
               extract: href
             - name: title
               inherit: true
               extract: text
             - name: domain
               inherit: true
               extract: href
               pipeline: [[regex, 'https?://([a-zA-Z0-9.-]+)/']]
        ";

        let finder: Result<Finder, _> = Config::from_yaml(cfg_yml).unwrap().try_into();
        assert!(finder.is_ok());
    }

    #[test]
    fn create_finder_inherit_root_fail() {
        let cfg_yml: &str = r"
        name: root
        inherit: true
        extract: text
        ";
        let cfg = Config::from_yaml(cfg_yml).unwrap();

        let finder = Finder::new(&cfg);
        assert!(finder.is_err());
    }

    #[test]
    fn finder_pipeline_missing_arguments() {
        let cfg_yml: &str = r"
        name: root
        base_path: html
        children:
          - name: links
            base_path: a[href]
            many: true
            children:
             - name: domain
               inherit: true
               extract: href
               pipeline: [[regex]]
        ";
        let cfg = Config::from_yaml(cfg_yml).unwrap();

        let finder = Finder::new(&cfg);
        assert!(finder.is_err());
    }

    #[test]
    fn finder_pipeline_non_existing_proc() {
        let cfg_yml: &str = r"
        name: root
        base_path: html
        children:
          - name: all_links
            base_path: a[href]
            many: true
            pipeline: [[non_existing_proc]]
        ";
        let cfg = Config::from_yaml(cfg_yml).unwrap();

        let finder = Finder::new(&cfg);
        assert!(finder.is_err());
    }
}
