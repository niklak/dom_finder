use once_cell::sync::Lazy;
use regex::Regex;

const SANITIZE_PATTERN: &str = r#"<(?<end>/?)(?<tag>\w+)(?<void>/?)([^>]*)>"#;
const ALLOWED_HIGHLIGHT_TAGS: [&str; 3] = ["b", "i", "em"];
const ALLOWED_TABLE_TAGS: [&str; 10] = [
    "b", "i", "em", "table", "tbody", "td", "th", "tr", "thead", "caption",
];
const ALLOWED_LIST_TAGS: [&str; 9] = ["b", "i", "em", "ol", "ul", "li", "dl", "dt", "dd"];

const ALLOWED_COMMON_TAGS: [&str; 16] = [
    "b", "i", "em", "table", "tbody", "td", "th", "tr", "thead", "caption", 
    "ol", "ul", "li", "dl", "dt", "dd"
];

//One policy can be used many times across many documents during runtime, so we use static lazy initialization to avoid same policy being initialized multiple times.

pub(crate) static HIGHLIGHT_POLICY: Lazy<Sanitization<'static>> =
    Lazy::new(|| Sanitization::new(SANITIZE_PATTERN, &ALLOWED_HIGHLIGHT_TAGS));
pub(crate) static TABLE_POLICY: Lazy<Sanitization<'static>> =
    Lazy::new(|| Sanitization::new(SANITIZE_PATTERN, &ALLOWED_TABLE_TAGS));
pub(crate) static LIST_POLICY: Lazy<Sanitization<'static>> =
    Lazy::new(|| Sanitization::new(SANITIZE_PATTERN, &ALLOWED_LIST_TAGS));
pub (crate) static COMMON_POLICY: Lazy<Sanitization<'static>> =
    Lazy::new(|| Sanitization::new(SANITIZE_PATTERN, &ALLOWED_COMMON_TAGS));

///`Sanitization` represents a lightweight sanitization policy based on regex.
pub struct Sanitization<'a> {
    pub re: Regex,
    pub allowed_tags: &'a [&'a str],
}

impl<'a> Sanitization<'a> {
    pub fn new(pattern: &'a str, allowed_tags: &'a [&'a str]) -> Sanitization<'a> {
        // as long as the pattern is static and checked, this should not panic
        let re = Regex::new(pattern).expect("unable to compile regex");
        Sanitization { re, allowed_tags }
    }

    pub fn clean(&self, html: &str) -> String {
        self.re
            .replace_all(html, |caps: &regex::Captures| {
                let tag_name = &caps["tag"];
                let end = &caps["end"];
                if self
                    .allowed_tags
                    .iter()
                    .any(|allowed_tag| allowed_tag == &tag_name)
                {
                    format!("<{}{}>", end, tag_name)
                } else {
                    String::new()
                }
            })
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn sanitize_with_highlight_policy() {
        let html = r#"
        <div><div><div>Integer <span style="color: red">efficitur</span> orci <b>quam</b></div></div>, in porttitor ipsum tempor et.</div>
        <div>Etiam id sapien quis ex laoreet efficitur.</div>
        <div>Nam <i>dictum</i> ut massa at malesuada.</div>
        <div>Ut nec purus feugiat, <span><em>fringilla nunc ornare, luctus ex</em></span>.</div>"#;
        let sanitized = super::HIGHLIGHT_POLICY.clean(html);
        let expected = "
        Integer efficitur orci <b>quam</b>, in porttitor ipsum tempor et.
        Etiam id sapien quis ex laoreet efficitur.
        Nam <i>dictum</i> ut massa at malesuada.
        Ut nec purus feugiat, <em>fringilla nunc ornare, luctus ex</em>.";
        assert_eq!(sanitized, expected);
    }

    #[test]
    fn sanitize_with_table_policy() {
        let html = r#"
        <table>
            <thead>
                <tr>
                    <th><h4><span style="color: green">Header 1</span></h4></th>
                    <th>Header 2</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td><b><i>Cell 1</i></b></td>
                    <td><em>Cell 2</em></td>
                </tr>
            </tbody>
        </table>
        "#;
        let sanitized = super::TABLE_POLICY.clean(html);
        let expected = "
        <table>
            <thead>
                <tr>
                    <th>Header 1</th>
                    <th>Header 2</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td><b><i>Cell 1</i></b></td>
                    <td><em>Cell 2</em></td>
                </tr>
            </tbody>
        </table>
        ";
        assert_eq!(sanitized, expected);
    }
    #[test]
    fn sanitize_with_list_policy() {
        let html = r#"
        <ul id="unordered-list-1">
            <li><span><b>Item 1</b></span></li>
            <li><b><em>Item 2</em></b></li>
        </ul>
        <ol id="unordered-list-1">
            <li><span><b>Item 1</b><span><img src="item-1.png"/></span></span></li>
            <li><b><em>Item 2</em></b></li>
        </ol>
        <dl id="description-list-1">
            <dt>Term 1</dt>
            <dd>Details 1</dd>
        </dl>
        "#;
        let sanitized = super::LIST_POLICY.clean(html);
        let expected = "
        <ul>
            <li><b>Item 1</b></li>
            <li><b><em>Item 2</em></b></li>
        </ul>
        <ol>
            <li><b>Item 1</b></li>
            <li><b><em>Item 2</em></b></li>
        </ol>
        <dl>
            <dt>Term 1</dt>
            <dd>Details 1</dd>
        </dl>
        ";
        assert_eq!(sanitized, expected);
    }
}
