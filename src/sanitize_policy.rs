use dom_query::{Document, Node};
use dom_sanitizer::{preset, RestrictivePolicy};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

static HIGHLIGHT_P: Lazy<RestrictivePolicy> = Lazy::new(|| {
    RestrictivePolicy::builder()
        .merge(preset::highlight_policy())
        .build()
});

static TABLE_P: Lazy<RestrictivePolicy> = Lazy::new(|| {
    RestrictivePolicy::builder()
        .merge(preset::highlight_policy())
        .merge(preset::table_policy())
        .build()
});

static LIST_P: Lazy<RestrictivePolicy> = Lazy::new(|| {
    RestrictivePolicy::builder()
        .merge(preset::highlight_policy())
        .exclude_elements(&["dl", "dt", "dd"])
        .merge(preset::list_policy())
        .build()
});

static COMMON_P: Lazy<RestrictivePolicy> = Lazy::new(|| {
    RestrictivePolicy::builder()
        .merge(preset::highlight_policy())
        .exclude_elements(&["dl", "dt", "dd"])
        .merge(preset::table_policy())
        .merge(preset::list_policy())
        .build()
});

pub(crate) fn clean(policy: &RestrictivePolicy, html: &str) -> String {
    let frag = Document::fragment(html);
    policy.sanitize_document(&frag);
    frag.html_root().inner_html().to_string()
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum SanitizePolicy {
    Highlight,
    List,
    Table,
    Common,
    #[default]
    None,
}

impl SanitizePolicy {
    pub(crate) fn sanitize(&self, node: &Node) {
        match self {
            SanitizePolicy::Highlight => HIGHLIGHT_P.sanitize_node(node),
            SanitizePolicy::List => LIST_P.sanitize_node(node),
            SanitizePolicy::Table => TABLE_P.sanitize_node(node),
            SanitizePolicy::Common => COMMON_P.sanitize_node(node),
            SanitizePolicy::None => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sanitize_policy::{HIGHLIGHT_P, LIST_P, TABLE_P};

    #[test]
    fn sanitize_with_highlight_policy() {
        let html = r#"
        <div><div><div>Integer <span style="color: red">efficitur</span> orci <b>quam</b></div></div>, in porttitor ipsum tempor et.</div>
        <div>Etiam id sapien quis ex laoreet efficitur.</div>
        <div>Nam <i>dictum</i> ut massa at malesuada.</div>
        <div>Ut nec purus feugiat, <span><em>fringilla nunc ornare, luctus ex</em></span>.</div>"#;
        let sanitized = super::clean(&HIGHLIGHT_P, html);
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
        let sanitized = super::clean(&TABLE_P, html);
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
        let sanitized = super::clean(&LIST_P, html);
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
