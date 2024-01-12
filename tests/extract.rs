use dom_finder::{Config, Finder};

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
";

const HTML_DOC: &str = include_str!("../test_data/ethereum.html");

#[test]
fn get_first_string_value() {
    let cfg = Config::from_yaml(CFG_YAML).unwrap();
    let finder = Finder::new(&cfg).unwrap();

    let results = finder.parse(HTML_DOC);

    let raw_url = results.from_path("root.results.0.url").unwrap();
    let url_opt: Option<String> = raw_url.into();
    assert_eq!(url_opt.unwrap().as_str(), "https://ethereum.org/en/");
}
#[test]
fn get_count_results() {
    let cfg = Config::from_yaml(CFG_YAML).unwrap();
    let finder = Finder::new(&cfg).unwrap();

    let results = finder.parse(HTML_DOC);

    let raw_url = results.from_path("root.results.#").unwrap();
    let url_opt: Option<i64> = raw_url.into();
    assert_eq!(url_opt.unwrap(), 21);
}

#[test]
fn get_flat_array_from_array_objects() {
    let cfg = Config::from_yaml(CFG_YAML).unwrap();
    let finder = Finder::new(&cfg).unwrap();

    let results = finder.parse(HTML_DOC);

    let raw_url = results.from_path("root.results.#.url").unwrap();
    let urls_opt: Option<Vec<String>> = raw_url.into();
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
