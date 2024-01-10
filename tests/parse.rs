use core::panic;

use dom_finder::{Config, Finder, Value};

const CFG: &str = r"
name: root
base_path: html
children:
  - name: results
    base_path: div.serp__results div.result
    many: true
    children:
      - name: url
        base_path: h2.result__title > a[href]
        extract: href
      - name: title
        base_path: h2.result__title
        extract: text
      - name: snippet
        base_path: a.result__snippet
        extract: html
        pipeline: [ [ policy_highlight ] ]
";

#[test]
fn find_results_extract_value() {
    // TODO: Add test code here

    let cfg = Config::from_yaml(CFG).unwrap();
    let finder = Finder::new(&cfg).unwrap();

    let html = include_str!("../test_data/ethereum.html");

    let results = finder.parse(html);

    let raw_arr = results.get("root.results").unwrap();

    if let Value::Array(ref arr) = raw_arr {
        assert_eq!(arr.len(), 21);
        if let Value::Object(obj) = &arr[0] {
            assert_eq!(obj.len(), 3);
            if let Value::String(url) = obj.get("url").unwrap() {
                assert_eq!(url, "https://ethereum.org/en/");
            } else {
                panic!("not a string");
            }
        } else {
            panic!("not an object");
        }
    } else {
        panic!("not an array");
    }
}
