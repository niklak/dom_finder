use dom_finder::{Config, Finder, Value};
use dom_query::Document;

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
        pipeline: [ [ policy_highlight ] ]
      - name: label
        base_path: .label
        extract: text
        pipeline: [ [ trim_space ] ]
      - name: nofollow
        base_path: h2.result__title > a[href][rel=nofollow]
        extract: rel
        cast: bool
";

const HTML_DOC: &str = include_str!("../test_data/page_0.html");
const HTML_DOC_NUT: &str = include_str!("../test_data/page_nutrition.html");

#[test]
fn get_first_string_value() {
    let cfg = Config::from_yaml(CFG_YAML).unwrap();
    let finder = Finder::new(&cfg).unwrap();

    let results = finder.parse(HTML_DOC);

    let url: String = results
        .from_path("root.results.0.url")
        .and_then(|v| v.into())
        .unwrap();
    assert_eq!(url, "https://ethereum.org/en/");
}
#[test]
fn get_count_results() {
    let cfg = Config::from_yaml(CFG_YAML).unwrap();
    let finder = Finder::new(&cfg).unwrap();

    let results = finder.parse(HTML_DOC);

    let count_opt: Option<i64> = results.from_path("root.results.#").and_then(|v| v.into());
    assert_eq!(count_opt.unwrap(), 21);
}

#[test]
fn get_flat_array_from_array_objects() {
    let cfg = Config::from_yaml(CFG_YAML).unwrap();
    let finder = Finder::new(&cfg).unwrap();

    let results = finder.parse(HTML_DOC);

    let urls_opt: Option<Vec<String>> = results
        .from_path("root.results.#.url")
        .and_then(|v| v.into());
    let urls = urls_opt.unwrap();

    let expected_urls = vec![
        "https://ethereum.org/en/",
        "https://en.wikipedia.org/wiki/Ethereum",
        "https://coinmarketcap.com/currencies/ethereum/",
        "https://www.coindesk.com/price/ethereum/",
        "https://ethereum.org/en/what-is-ethereum/",
        "https://www.investopedia.com/terms/e/ethereum.asp",
        "https://www.google.com/finance/quote/ETH-USD",
        "https://www.coinbase.com/price/ethereum",
        "https://ethereum.org/en/eth/",
        "https://www.kraken.com/prices/ethereum",
        "https://www.forbes.com/digital-assets/assets/ethereum-eth/",
        "https://www.coindesk.com/learn/what-is-ethereum/",
        "https://www.forbes.com/advisor/investing/cryptocurrency/what-is-ethereum-ether/",
        "https://finance.yahoo.com/quote/ETH-USD/",
        "https://www.tradingview.com/symbols/ETHUSD/",
        "https://ethereum.org/en/learn/",
        "https://uk.investing.com/crypto/ethereum",
        "https://ethereum.org/en/about/",
        "https://etherscan.io/",
        "https://twitter.com/ethereum",
        "https://www.coingecko.com/en/coins/ethereum",
    ];
    assert_eq!(urls, expected_urls);
}

#[test]
fn remove_selection() {
    let cfg_yaml = r"
  name: root
  base_path: html
  children:
    - name: feedback
      base_path: div#links.results div.feedback-btn
      extract: text
      remove_selection: true
      pipeline: [ [ trim_space ] ]
  ";
    let cfg = Config::from_yaml(cfg_yaml).unwrap();
    let finder = Finder::new(&cfg).unwrap();
    let doc = Document::from(HTML_DOC);

    let res = finder.parse_document(&doc);
    let feedback_caption: Option<String> = res.from_path("root.feedback").and_then(|v| v.into());
    assert_eq!(feedback_caption.unwrap(), "Feedback");
    let html = doc.html();
    assert!(!html.contains("feedback-btn"));
}

#[test]
fn result_is_empty() {
    let cfg_yaml = r"
    name: root
    base_path: html
    children:
      - name: results
        base_path: div.serp__results div.result
        many: true
        children:
          # some non-existing element
          - name: label
            base_path: .label
            extract: text
  ";
    let cfg = Config::from_yaml(cfg_yaml).unwrap();
    let finder = Finder::new(&cfg).unwrap();
    let doc = Document::from(HTML_DOC);

    let res = finder.parse_document(&doc);
    let val = res.from_path("root.results");
    assert!(val.is_none());
}

#[test]
fn inner_text() {
    let cfg_yaml = r"
    name: root
    base_path: html
    children:
      - name: title
        base_path: h1
        extract: inner_text
  ";
    let cfg = Config::from_yaml(cfg_yaml).unwrap();
    let finder = Finder::new(&cfg).unwrap();
    let doc = Document::from(HTML_DOC_NUT);

    let res = finder.parse_document(&doc);
    let title: Option<String> = res.from_path("root.title").and_then(|v| v.into());
    assert_eq!(title.unwrap(), "Fruit Nutrition Facts");
    // while `extract: text` will capture `A Brief List of Fruit Nutrition Facts`
}

#[test]
fn inner_html() {
    let cfg_yaml = r"
    name: root
    base_path: html
    children:
      - name: title
        base_path: h1
        extract: inner_html
  ";
    let cfg = Config::from_yaml(cfg_yaml).unwrap();
    let finder = Finder::new(&cfg).unwrap();
    let doc = Document::from(HTML_DOC_NUT);

    let res = finder.parse_document(&doc);
    let title: Option<String> = res.from_path("root.title").and_then(|v| v.into());
    assert_eq!(
        title.unwrap(),
        "<span>A Brief List of </span>Fruit Nutrition Facts"
    );
}

#[test]
fn extract_vec_string() {
    let cfg_yaml: &str = r"
  name: root
  base_path: html
  children:
    - name: urls
      base_path: h2.result__title > a[href]
      many: true
      extract: href
  ";

    let finder: Finder = Config::from_yaml(cfg_yaml).unwrap().try_into().unwrap();
    let doc = Document::from(HTML_DOC);
    let res = finder.parse_document(&doc);
    let urls: Option<Vec<String>> = res.from_path("root.urls").and_then(|v| v.into());
    assert_eq!(urls.unwrap().len(), 21);
}

#[test]
fn value_wrong_extraction_entity() {
    let finder: Finder = Config::from_yaml(CFG_YAML).unwrap().try_into().unwrap();
    let doc = Document::from(HTML_DOC);
    let res = finder.parse_document(&doc);
    let url_val: Option<Value> = res.from_path("root.results.0.url");
    let none_val: Option<Value> = url_val.and_then(|v| v.from_path("nonsense"));
    assert!(none_val.is_none());
}
