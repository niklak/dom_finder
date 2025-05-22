use dom_query::Node;
use dom_sanitizer::{preset, RestrictivePolicy};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tendril::StrTendril;

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


/// Defines a set of predefined sanitization policies for HTML content.
///
/// Each policy allows only a specific subset of safe HTML elements to be retained, removing all others.
#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum SanitizeOption {
    /// Keeps only text and the following inline elements: `b`, `del`, `em`, `i`, `ins`, `mark`, `s`, `small`, `strong`, and `u`.
    Highlight,

    /// Keeps text and all elements from [`SanitizePolicy::Highlight`],  
    /// plus list-related elements: `li`, `ul`, `ol`, `dl`, `dt`, and `dd`.
    List,

    /// Keeps text and all elements from [`SanitizePolicy::Highlight`],  
    /// plus table-related elements: `table`, `caption`, `colgroup`, `col`, `th`, `thead`, `tbody`, `tr`, `td`, and `tfoot`.
    Table,

    /// Keeps text and all elements from [`SanitizePolicy::Highlight`],  
    /// [`SanitizePolicy::List`], and [`SanitizePolicy::Table`].
    Common,

    /// No sanitization is applied; all content is preserved.
    #[default]
    None,
}

impl SanitizeOption {
    pub(crate) fn sanitize(&self, node: &Node) {
        match self {
            SanitizeOption::Highlight => HIGHLIGHT_P.sanitize_node(node),
            SanitizeOption::List => LIST_P.sanitize_node(node),
            SanitizeOption::Table => TABLE_P.sanitize_node(node),
            SanitizeOption::Common => COMMON_P.sanitize_node(node),
            SanitizeOption::None => (),
        }
    }

    pub(crate) fn clean_html(&self, node: &Node) -> Option<StrTendril> {
        if matches!(self, SanitizeOption::None) {
            return node.try_html();
        }
        let fragment = node.to_fragment();
        self.sanitize(&fragment.html_root());
        fragment
            .html_root()
            .try_inner_html()
    }

    pub(crate) fn clean_inner_html(&self, node: &Node) -> Option<StrTendril> {
        if matches!(self, SanitizeOption::None) {
            return node.try_inner_html();
        }
        let fragment = node.to_fragment();
        let Some(frag_node) = fragment.html_root().first_element_child() else {
            return None;
        };
        self.sanitize(&frag_node);
        frag_node.try_inner_html()
    }
}

#[cfg(test)]
mod tests {
    use dom_query::Document;

    #[test]
    fn sanitize_with_highlight_policy() {
        let html = r#"
        <div><div><div>Integer <span style="color: red">efficitur</span> orci <b>quam</b></div></div>, in porttitor ipsum tempor et.</div>
        <div>Etiam id sapien quis ex laoreet efficitur.</div>
        <div>Nam <i>dictum</i> ut massa at malesuada.</div>
        <div>Ut nec purus feugiat, <span><em>fringilla nunc ornare, luctus ex</em></span>.</div>"#;

        let doc = Document::fragment(html);
        let p = super::SanitizeOption::Highlight;
        let sanitized = p.clean_html(&doc.html_root()).unwrap().to_string();
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
        
        let doc = Document::fragment(html);
        let p = super::SanitizeOption::Table;
        let sanitized = p.clean_html(&doc.html_root()).unwrap().to_string();
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
        let doc = Document::fragment(html);
        let p = super::SanitizeOption::List;
        let sanitized = p.clean_html(&doc.html_root()).unwrap().to_string();
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
