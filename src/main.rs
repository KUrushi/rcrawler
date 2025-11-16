use scraper::{Html, Selector};
use std::collections::{HashSet};
use reqwest::Url;
use tokio::sync::mpsc;

struct CrawlResult {
    url: Url,
    links: Vec<Url>
}

async fn fetch_links(url: &Url) -> Result<Vec<Url>, Box<dyn std::error::Error + Send + Sync>> {
    let body = reqwest::get(url.as_str()).await?.text().await?;
    let document = Html::parse_document(&body);
    let selector = Selector::parse("a").unwrap();

    let mut links = Vec::new();

    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            if let Ok(full_url) = url.join(href) {
                links.push(full_url);
            }
        }
    }
    Ok(links)
}


async fn crawl_worker(url: Url, tx: mpsc::Sender<CrawlResult>) {
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    match fetch_links(&url).await {
        Ok(links) => {
            let result = CrawlResult {url, links};
            if let Err(_) = tx.send(result).await {
                return;
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch {}: {}", url, e);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 対象のURL
    let start_url = Url::parse("https://www.rust-lang.org")?;
    let mut known_urls = HashSet::<Url>::new();
    known_urls.insert(start_url.clone());

    // 1. チャネルを作成 (tx: 送信機、 rx: 受信機)
    // バッファサイズは32
    let (tx, mut rx) = mpsc::channel(32);
    let mut active_tasks = 1_usize;

    // 2. 最初のタスクを起動
    // txを複製して、workerに持たせる
    let first_tx = tx.clone();

    tokio::spawn(async move {
        crawl_worker(start_url, first_tx).await;
    });

    println!("Starting crawling concurrently");

    while let Some(result) = rx.recv().await {
        active_tasks -= 1;
        println!("Finished: {} (Found {} links)", result.url, result.links.len());
        
        for link in result.links {
            if let (Some(src), Some(tgt)) = (result.url.host(), link.host()) {
                if src == tgt && known_urls.insert(link.clone()) {
                    active_tasks += 1;
                    let tx_clone = tx.clone();
                    tokio::spawn(async move {
                        crawl_worker(link, tx_clone).await;
                    });
                }
            }
        }

        if active_tasks == 0 {
            break;
        }
    }

    Ok(())
}