use regex::Regex;

use crate::errors::PipelineError;

use super::errors::ParseError;
use super::sanitize_policy;

// Constants representing the names of different pipeline processing procedures
const REGEX_PROC: &str = "regex";
const REPLACE_PROC: &str = "replace";
const EXTRACT_JSON: &str = "extract_json";
const TRIM_SPACE: &str = "trim_space";
const TRIM: &str = "trim";
const HTML_UNESCAPE: &str = "html_unescape";
const POLICY_HIGHLIGHT: &str = "policy_highlight";
const POLICY_TABLE: &str = "policy_table";
const POLICY_LIST: &str = "policy_list";
const POLICY_COMMON: &str = "policy_common";

/// Represents a pipeline of processing procedures.
#[derive(Debug)]
pub struct Pipeline<'a> {
    procs: Vec<Proc<'a>>,
}

impl<'a> Pipeline<'a> {
    /// Creates a new `Pipeline` instance based on the provided raw pipelines.
    ///
    /// # Arguments
    ///
    /// * `raw_pipelines` - A reference to a vector of vectors of strings representing the raw pipeline elements.
    ///
    /// # Returns
    ///
    /// Returns a new `Result<Pipeline, ParseError>` instance. Because regex can fail to compile and user can provide an invalid procedure.
    pub fn new(raw_pipelines: &'a Vec<Vec<String>>) -> Result<Self, ParseError> {
        let mut procs = vec![];
        for proc_args in raw_pipelines {
            if let Some((proc_name, args)) = proc_args.split_first() {
                let proc = Proc::new(proc_name, args)?;
                procs.push(proc);
            }
        }
        Ok(Self { procs })
    }

    /// Handles the given value by applying all the processing procedures in the pipeline.
    ///
    /// # Arguments
    ///
    /// * `value` - The input value to be processed.
    ///
    /// # Returns
    ///
    /// Returns the processed value as a string.
    pub fn handle(&self, value: String) -> String {
        let mut res: String = value;
        for command in self.procs.iter() {
            res = command.handle(res)
        }
        res
    }
}

/// Represents a processing procedure in the pipeline.
#[derive(Debug)]
pub enum Proc<'a> {
    /// requires one argument - the regex pattern. Currently it captures only a value from the first matching group.
    Regex(Regex),
    /// requires two arguments - the old and the new string.
    Replace(&'a str, &'a str),
    /// requires one argument - the path to the json value, if the string represents a json.
    ExtractJson(&'a str),
    /// requires no arguments. It trims spaces at the start and the end of the string.
    TrimSpace,
    /// requires one argument - it trims characters from the (start and end of) string with the cut set.
    Trim(Vec<char>),
    /// unescape html entities, requires no arguments.
    HtmlUnescape,
    /// removes all html tags from the result except `<b>`, `<em>`, and `<i>`,  requires no arguments.
    PolicyHighlight,
    /// removes all html tags from the result except  tags from  `PolicyHighlight` and
    /// `<table>`, `<tr>`, `<td>`, `<th>`, `<tbody>`, `<thead>`, `<caption>`, requires no arguments.
    PolicyTable,
    /// removes all html tags from the result except  tags from  `PolicyHighlight` and  
    /// `<ul>`, `<ol>`, `<li>`, `<dl>`, `<dt>`, `<dd>`, requires no arguments.
    PolicyList,
    /// removes all html tags from the result except  tags from  `PolicyHighlight`,
    /// `PolicyTable` and `PolicyList`, requires no arguments.
    PolicyCommon
}

impl<'a> Proc<'a> {
    /// Creates a new `Proc` instance based on the provided `proc_args`.
    ///
    /// # Arguments
    ///
    /// * `proc_args` - A slice of strings representing the arguments for the `Proc`.
    ///
    /// # Returns
    ///
    /// Returns a new `Result<Option<Proc>, ParseError>` instance. Because:
    /// * regex can fail to compile
    /// * user can provide an invalid procedure
    /// * user can provide an invalid number of arguments for a procedure

    fn new(proc_name: &str, args: &'a [String]) -> Result<Self, PipelineError> {
        let proc_opt = match proc_name {
            REGEX_PROC => {
                validate_args_len(proc_name, args.len(), 1)?;
                let re = Regex::new(&args[0])?;
                Proc::Regex(re)
            }
            EXTRACT_JSON => {
                validate_args_len(proc_name, args.len(), 1)?;
                Proc::ExtractJson(&args[0])
            }
            REPLACE_PROC => {
                validate_args_len(proc_name, args.len(), 2)?;
                Proc::Replace(&args[0], &args[1])
            }
            TRIM_SPACE => Proc::TrimSpace,
            TRIM => {
                validate_args_len(proc_name, args.len(), 1)?;
                let cut_set: Vec<char> = args[0].chars().collect();
                Proc::Trim(cut_set)
            }
            HTML_UNESCAPE => Proc::HtmlUnescape,
            POLICY_HIGHLIGHT => Proc::PolicyHighlight,
            POLICY_TABLE => Proc::PolicyTable,
            POLICY_LIST => Proc::PolicyList,
            POLICY_COMMON => Proc::PolicyCommon,
            _ => return Err(PipelineError::ProcDoesNotExist(proc_name.to_string())),
        };
        Ok(proc_opt)
    }

    /// Handles the given value by applying the processing procedure.
    ///
    /// # Arguments
    ///
    /// * `value` - The input value to be processed.
    ///
    /// # Returns
    ///
    /// Returns the processed value as a string.
    fn handle(&self, value: String) -> String {
        match self {
            Proc::Regex(re) => match re.captures(&value) {
                Some(m) => m.get(1).map_or("", |s| s.as_str()).to_string(),
                None => "".to_string(),
            },
            Proc::Replace(old, new) => value.replace(old, new),
            Proc::ExtractJson(path) => gjson::get(&value, path).to_string(),
            Proc::TrimSpace => value.trim().to_string(),
            Proc::Trim(pat) => value.trim_matches(pat.as_slice()).to_string(),
            Proc::HtmlUnescape => html_escape::decode_html_entities(&value).to_string(),
            Proc::PolicyHighlight => sanitize_policy::HIGHLIGHT_POLICY.clean(&value),
            Proc::PolicyTable => sanitize_policy::TABLE_POLICY.clean(&value),
            Proc::PolicyList => sanitize_policy::LIST_POLICY.clean(&value),
            Proc::PolicyCommon => sanitize_policy::COMMON_POLICY.clean(&value),
        }
    }
}

fn validate_args_len(proc_name: &str, args_len: usize, len: usize) -> Result<(), PipelineError> {
    if args_len < len {
        return Err(PipelineError::ProcNotEnoughArguments(
            proc_name.to_string(),
            args_len,
            len,
        ));
    }
    Ok(())
}
