
# DOM_FINDER

[![Crates.io version](https://img.shields.io/crates/v/dom_finder.svg?style=flat)](https://crates.io/crates/dom_finder)
[![Download](https://img.shields.io/crates/d/dom_finder.svg?style=flat)](https://crates.io/crates/dom_finder)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat)](https://docs.rs/dom_finder)
[![ci](https://github.com/niklak/dom_finder/actions/workflows/rust.yml/badge.svg)](https://github.com/niklak/dom_finder/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/niklak/dom_finder/graph/badge.svg?token=MFMJPTYVWT)](https://codecov.io/gh/niklak/dom_finder)

`dom_finder` is a Rust crate that provides functionality for finding elements in the Document Object Model (DOM) of HTML documents. 
It allows you to easily locate specific elements based on various CSS criteria. 
With `dom_finder`, you can extract data from HTML documents and transform it before getting the result.

Currently, the functionality relies on YAML configuration.


## Examples

#### General

```rust

use dom_finder::{Config, Finder, Value};

const CFG_YAML: &str = r"
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
        sanitize_policy: highlight
        pipeline: [ [ normalize_spaces ] ]
";

const HTML_DOC: &str = include_str!("../test_data/page_0.html");


fn main() {
    // Loading config from yaml str, -- &str can be retrieved from file or buffer,
    let cfg = Config::from_yaml(CFG_YAML).unwrap();
    // Creating a new Finder instance
    let finder = Finder::new(&cfg).unwrap();

    // or in one line:
    // let finder: Finder = Config::from_yaml(CFG_YAML).unwrap().try_into().unwrap();
    
    // parsing html-string (actually &str), and getting the result as `Value`.
    // Returned `Value` from `parse` method is always `Value::Object` and it has only one key (String).
    let results: Value = finder.parse(HTML_DOC);

    // from the `Value` we can navigate to descendant (inline) value, by path,
    // similar like `gjson` has, but in `Value` case -- path is primitive.
    // For more examples, please check out the `tests/` folder.

    // Getting the count of results by using `from_path` method.
    // We know that `results` is `Value::Array`, 
    // because in the config we set `many: true` for `results`.
    // if the Value option is Array (actually Vector), we can query it by: # or a (positive) number.
    let count_opt: Option<i64> = results.from_path("root.results.#").and_then(|v| v.into());
    assert_eq!(count_opt.unwrap(), 21);


    // Getting an exact Value, and casting it to a real value
    // Same way we can retrieve all urls inside `results` array, 
    // by specifying path as `root.results.#.url`.
    // If there will no `url` key, or it will not have a Value::String type, 
    // it will return None, otherwise -- Some
    let url: String = results.from_path("root.results.0.url")
    .and_then(| v| v.into()).unwrap();
    assert_eq!(url, "https://ethereum.org/en/");

    // Also the `Value` instance can be serialized with serde serializer 
    // (like json or any other available)
    // Useful if you just need to send parsed data with http response, 
    // or put parsed data into the database
    let serialized = serde_json::to_string(&res).unwrap();
}
```

#### Remove selection

```rust
use dom_finder::{Config, Finder};
use dom_query::Document;


const HTML_DOC: &str = include_str!("../test_data/page_0.html");


fn main() {

  // Create finder, like in previous example
  let cfg_yaml = r"
  name: root
  base_path: html
  children:
  - name: feedback
    base_path: div#links.results div.feedback-btn
    extract: html
    remove_selection: true
    pipeline: [ [ trim_space ] ]
  ";
  let cfg = Config::from_yaml(cfg_yaml).unwrap();
  let finder = Finder::new(&cfg).unwrap();

  // Create dom_query::Document
  let doc = Document::from(HTML_DOC);

  // Parse the document
  // As we set remove_selection it matched selection will be removed from the document.
  // But the value of matched selection will be available in the result
  let res = finder.parse_document(&doc);
  let feedback_caption: Option<String> = res.from_path("root.feedback").and_then(|v| v.into());
  assert_eq!(feedback_caption.unwrap(), "Feedback");

  let html = doc.html();
  // html document doesn't contain feedback button anymore. 
  assert!(!html.contains("feedback-btn"));
}

```

#### More examples

- [examples/multithread_scope.rs](./examples/multithread_scope.rs)
- [examples/multithread.rs](./examples/multithread.rs)
 

## Features

- `json_cfg` -- optional, allow to load config from JSON string.

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)

at your option.

 Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.