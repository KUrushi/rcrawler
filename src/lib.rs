use reqwest::Url;
use tracing::error;
use scraper::{Html, Selector};

pub fn extract_links(base_url: &Url, html: &str) -> Vec<Url> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("a");
    let links= match selector {
        Ok(selector) => {
            let mut links = Vec::new();
            for element in document.select(&selector) {
                if let Some(href) = element.value().attr("href") {
                    if let Ok(full_url) = base_url.join(href){
                        links.push(full_url)
                    }
                }
            }
            links
        }
        Err(e) => {
            error!("{}",e);
            Vec::new()
        }
    };
    links
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_extract_links() {
        let base = Url::parse("https://example.com").unwrap();
        let html = r#"
            <html>
                  <body>
                    <a href="/foo">相対パス</a>
                    <a href="https://rust-lang.org">絶対パス</a>
                    <a>リンクなし</a>
                </body>
            </html>
        "#;
        let expected = vec![Url::parse("https://example.com/foo").unwrap(), Url::parse("https://rust-lang.org").unwrap()];
        let actual = extract_links(&base, html);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_extract_links_with_correct_links() {
        let base = Url::parse("https://example.com").unwrap();
        let html = r#"
            <html>
                <body>
                    <a href="/next">相対パス</a>
                    <a href="https://google.com">絶対パス</a>
                    <a>リンクなし</a>
                </body>
            </html>
        "#;
        let expected = vec![
            Url::parse("https://example.com/next").unwrap(),
            Url::parse("https://google.com").unwrap()];
        let actual = extract_links(&base, html);

        assert_eq!(expected, actual);
    }

}