use scraper::{Html, Selector};
use std::collections::{HashSet, VecDeque};
use reqwest::Url;

async fn fetch_links(url: &Url) -> Result<Vec<Url>, Box<dyn std::error::Error>> {
    let body = reqwest::get(url.as_str()).await?.text().await?;
    let document = Html::parse_document(&body);

    let selector = Selector::parse("a").unwrap();

    let mut links = Vec::new();

    for element in document.select(&selector) {
        if let Some(link) = element.value().attr("href") {
            match url.join(link) {
                Ok(link) => {
                    links.push(link);
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        }
    }
    Ok(links)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 対象のURL
    let url = Url::parse("https://www.rust-lang.org")?;
    let mut fetch_target_urls: VecDeque<Url> = VecDeque::new();
    let mut known_urls = HashSet::<Url>::new();
    fetch_target_urls.push_back(url.clone());
    known_urls.insert(url);


    while let Some(url) = fetch_target_urls.pop_front() {
        let _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        println!("Fetching: {}", url);
        if let Ok(links) = fetch_links(&url).await {
            for link in links {
                if let (Some(src_host), Some(tgt_host)) = (url.host(), link.host()) {
                    if src_host == tgt_host && known_urls.insert(link.clone()) {
                        fetch_target_urls.push_back(link);
                    }
                }
            }
        }
    }
    Ok(())
}