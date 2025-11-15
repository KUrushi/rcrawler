use scraper::{Html, Selector};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    // 1. 対象のURL
    let url = "https://www.rust-lang.org";

    println!("Fetching: {url}");

    // 2. HTMLを取得
    // reqwest::get でアクセスし、`.text().await`でHTMLの文字列を取得する
    let body = reqwest::get(url).await?.text().await?;

    // 3. HTMLを解析可能な形式に変換 (Parse)
    let document = Html::parse_document(&body);
    // 4. 抽出したい要素を指定 (CSSセレクタ)
    let selector = Selector::parse("a").unwrap();

    for element in document.select(&selector) {
        if let Some(link) = element.value().attr("href") {
            println!("Link: {link}");
        } else {
            println!("Link is not found");
        }
    }

    Ok(())
}

