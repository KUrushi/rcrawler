use scraper::{Html, Selector};
use reqwest::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 対象のURL
    let url = Url::parse("https://www.rust-lang.org")?;

    println!("Fetching: {}", url.as_str());

    // 2. HTMLを取得
    // reqwest::get でアクセスし、`.text().await`でHTMLの文字列を取得する
    let body = reqwest::get(url.as_str()).await?.text().await?;

    // 3. HTMLを解析可能な形式に変換 (Parse)
    let document = Html::parse_document(&body);
    // 4. 抽出したい要素を指定 (CSSセレクタ)
    let selector = Selector::parse("a").unwrap();

    for element in document.select(&selector) {
        if let Some(link) = element.value().attr("href") {
            // url.joinは引数が完全なURLの場合には引数のURLが返ってくる
            match url.join(link) {
                Ok(link) => {
                    println!("Getting Link: {}", link.as_str());
                }
                Err(e) => {
                    eprintln!("Failed to get link: {}", e);
                }
            }
        } else {
            println!("Link is not found");
        }
    }

    Ok(())
}

