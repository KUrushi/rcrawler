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
    let mut fetch_target_urls:VecDeque<Url> = VecDeque::new();
    fetch_target_urls.push_back(url.clone());

    let mut already_fetched = HashSet::<Url>::new();

    while let Some(url) = fetch_target_urls.pop_front() {
        if already_fetched.insert(url.clone()){
            let links = fetch_links(&url).await;
            match links {
                Ok(links) => {
                    println!("Found {} links on page: [{}]", links.len(), url.as_str());
                    for link in links {
                        fetch_target_urls.push_back(link);
                    }
                } Err(e) => {
                    eprintln!("{}",e);
                }
            }
        }

    }
    Ok(())
}