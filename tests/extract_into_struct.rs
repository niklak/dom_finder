use std::option::Option;
use dom_finder::{Config, Finder, Value};

///A test example of how to extract a struct from a Value

trait FromValue: Sized {
    fn from_value(value: Value) -> Option<Self>;
    
}

#[derive(Debug, Default)]
struct SerpLink {
    url: String,
    title: String,
}

impl FromValue for SerpLink{
    fn from_value(value: Value) -> Option<Self> {
        match value {
            Value::Object(o) => {
                let url: Option<String> = o.get("url").and_then(|v| v.to_owned().into());
                let title: Option<String> = o.get("title").and_then(|v| v.to_owned().into());
                Some(Self { url: url.unwrap_or_default(), title: title.unwrap_or_default() })
            },
            _ => None,
            
        }
    }
}

#[derive(Debug)]
struct SerpItem {
    link: SerpLink,
    snippet: String,
    index: i64,
}

impl FromValue for SerpItem {
    fn from_value(value: Value) -> Option<Self> {
        match value {
            Value::Object(o) => {
                let link: Option<SerpLink> = o.get("link").and_then(|v| SerpLink::from_value(v.to_owned()));
                let snippet: Option<String> = o.get("snippet").and_then(|v| v.to_owned().into());
                let index: Option<i64> = o.get("index").and_then(|v| v.to_owned().into());
                Some(Self { link: link.unwrap_or_default(), snippet: snippet.unwrap_or_default(), index: index.unwrap_or_default() })
            },
            _ => None,
        }
    }
}

#[derive(Debug)]
struct Serp {
    items: Vec<SerpItem>,
}

impl FromValue for Serp {
    fn from_value(value: Value) -> Option<Self> {
        if let Some(val) = value.from_path("root.results") {
            match val {
                Value::Array(items) => {
                    let items: Vec<SerpItem> = items.into_iter().filter_map(|v| SerpItem::from_value(v)).collect();
                    Some(Self { items })
                },
                _ => None,
            }
        }else {
            None
        }
    }
    
}

const CFG_YAML: &str = r"
name: root
base_path: html
children:
  - name: results
    base_path: div.serp__results div.result
    many: true
    enumerate: true
    children:
      - name: link
        base_path: h2.result__title > a
        pipeline: [ [ policy_highlight ] ]
        children:
          - name: url
            inherit: true
            extract: href
            pipeline: [ [ policy_highlight ] ]
          - name: title
            inherit: true
            extract: text
            pipeline: [ [ policy_highlight ] ]

      - name: snippet
        base_path: a.result__snippet
        extract: html
        pipeline: [ [ policy_highlight ] ]
";

const HTML_DOC: &str = include_str!("../test_data/ethereum.html");

#[test]
fn get_last_url() {
    let cfg = Config::from_yaml(CFG_YAML).unwrap();
    let finder = Finder::new(&cfg).unwrap();

    let results = finder.parse(HTML_DOC);

    let serp = Serp::from_value(results).unwrap();
    assert_eq!(serp.items[serp.items.len() -1].link.url, "https://www.coingecko.com/en/coins/ethereum");
}


#[test]
fn get_count_results() {
    let cfg = Config::from_yaml(CFG_YAML).unwrap();
    let finder = Finder::new(&cfg).unwrap();

    let results = finder.parse(HTML_DOC);

    let serp = Serp::from_value(results).unwrap();
    assert_eq!(serp.items.len(), 21);
}


