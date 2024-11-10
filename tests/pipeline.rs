use dom_finder::{Config, Finder};

const CFG_YAML: &str = r"
name: root
base_path: html
children:
  - name: title
    base_path: h1
    extract: text
    pipeline: [ [normalize_spaces] ]
    
  - name: results
    base_path: table tr.nutrition-item
    many: true
    children:
      - name: name
        base_path: td:nth-child(1)
        extract: text
      - name: calories
        base_path: td:nth-child(2)
        extract: text
        pipeline: [ [regex_find, '^(\d+)'] ]
        cast: int
      - name: vitamin_c
        base_path: td:nth-child(3)
        extract: text
      - name: sugar
        base_path: td:nth-child(4)
        extract: text
        pipeline: [ [regex_find, '^([\d.]+)'] ]
        cast: float
      - name: carbohydrates
        inherit: true
        extract: data-nutrition
        pipeline: [ [extract_json, 'carbs' ], [regex_find, '^([\d.]+)'] ]
        cast: float
      - name: fiber
        inherit: true
        extract: data-nutrition
        pipeline: [ [extract_json, 'fiber' ], [regex_find, '^([\d.]+)'] ]
        cast: float
      - name: protein
        inherit: true
        extract: data-nutrition
        pipeline: [ [extract_json, 'protein' ], [regex_find, '^([\d.]+)'] ]
        cast: float
        
";

const HTML_DOC: &str = include_str!("../test_data/page_nutrition.html");

#[test]
fn pipeline_extract_first_item() {
    let cfg = Config::from_yaml(CFG_YAML).unwrap();
    let finder = Finder::new(&cfg).unwrap();

    let results = finder.parse(HTML_DOC);

    let first_item = results.from_path("root.results.0").unwrap();

    let name: String = first_item.from_path("name").and_then(|v| v.into()).unwrap();

    let calories: i64 = first_item
        .from_path("calories")
        .and_then(|v| v.into())
        .unwrap();

    let vitamin_c: String = first_item
        .from_path("vitamin_c")
        .and_then(|v| v.into())
        .unwrap();

    let sugar: f64 = first_item
        .from_path("sugar")
        .and_then(|v| v.into())
        .unwrap();
    let carbohydrates: f64 = first_item
        .from_path("carbohydrates")
        .and_then(|v| v.into())
        .unwrap();
    let fiber: f64 = first_item
        .from_path("fiber")
        .and_then(|v| v.into())
        .unwrap();
    let protein: f64 = first_item
        .from_path("protein")
        .and_then(|v| v.into())
        .unwrap();
    let got = (
        name,
        calories,
        vitamin_c,
        sugar,
        carbohydrates,
        fiber,
        protein,
    );
    let expected = (
        "Apple".to_string(),
        52_i64,
        "10.3mg".to_string(),
        10.0,
        14.0,
        2.6,
        0.3,
    );
    assert_eq!(got, expected);


    let title: Option<String> = results.from_path("root.title").and_then(|s| s.into());
    assert_eq!(title.unwrap(), "A Brief List of Fruit Nutrition Facts");
}
