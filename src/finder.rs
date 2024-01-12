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

/// Finder is the main struct that is used to parse the html
#[derive(Debug)]
pub struct Finder<'a> {
    name: &'a str,
    extract: &'a str,
    cast: CastType,
    join_sep: &'a str,
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
    pub fn new(config: &'a Config) -> Result<Finder<'a>, ParseError> {
        Self::from_config(config, true)
    }

    fn from_config(config: &'a Config, is_root: bool) -> Result<Finder<'a>, ParseError> {
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
            name: config.name.as_str(),
            extract: config.extract.as_str(),
            cast: config.cast,
            join_sep: config.join_sep.as_str(),
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
        let sel = Selection::from(doc.root());
        let val = self.parse_value(&sel);
        let mut m: InnerMap = InnerMap::default();
        m.insert(self.name.to_string(), val);
        Value::Object(m)
    }

    pub fn parse_value(&self, root: &Selection) -> Value {
        let sel: Selection = if self.inherit {
            root.clone()
        } else if self.parent {
            root.select_matcher(self.get_matcher()).parent()
        } else {
            root.select_matcher(self.get_matcher())
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
                    .filter_map(|item| self.adjust_result_value(item))
                    .collect();

                if !self.join_sep.is_empty() {
                    Value::from(tmp_res.join::<&str>(self.join_sep))
                } else {
                    Value::from_iter(tmp_res.into_iter().map(|it| cast_value(it, self.cast)))
                }
            }
            _ => {
                let item = sel.first();
                if let Some(tmp_val) = self.adjust_result_value(item) {
                    cast_value(tmp_val, self.cast)
                } else {
                    Value::Null
                }
            }
        };

        if self.remove_selection {
            let mut rem_sel = sel;
            rem_sel.remove();
        }
        v
    }

    /// Adjusts the result value according to the extract type and the pipeline
    fn adjust_result_value(&self, sel: Selection) -> Option<String> {
        if let Some(extracted) = extract_data(sel, self.extract) {
            if let Some(ref pipeline) = self.pipeline {
                Some(pipeline.handle(extracted))
            } else {
                Some(extracted)
            }
        } else {
            None
        }
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
                            //values.push([(key, val)].into_iter().collect::<InnerMap>()); -- wrong
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
/// * `s` - the value to cast
/// * `cast` - the type to cast to
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

/// Extracts the data from the given selection according to the extract type
/// The extract type can be one of the following:
/// - text - extracts the text of the selection
/// - inner_text - extracts the text of the selection without the text of the children
/// - html - extracts the html of the selection
#[inline(always)]
fn extract_data(sel: Selection, extract_type: &str) -> Option<String> {
    match extract_type {
        EXTRACT_TEXT => Some(sel.text().to_string()),
        EXTRACT_INNER_TEXT => Some(get_inner_text(sel).to_string()),
        EXTRACT_HTML => Some(sel.html().to_string()),
        _ => sel.attr(extract_type).map(|attr| attr.to_string()),
    }
}

/// Returns the inner text of the selection without the text of the children
#[inline(always)]
fn get_inner_text(sel: Selection) -> StrTendril {
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
        let cfg = Config::from_yaml(cfg_yml).unwrap();

        let finder = Finder::new(&cfg);
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
}
