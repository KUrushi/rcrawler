use clap::Parser;
use std::collections::HashSet;
use reqwest::Url;
use tokio::sync::mpsc;
use my_crawler::extract_links;
use tracing::{error, info};

struct CrawlResult {
    url: Url,
    links: Vec<Url>
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    url: String,
    #[arg(short, long, default_value_t=10)]
    tasks: usize
}




async fn fetch_links(url: &Url) -> Result<Vec<Url>, Box<dyn std::error::Error + Send + Sync>> {
    let body = reqwest::get(url.as_str()).await?.text().await?;
    let links = extract_links(url, body.as_str());
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
            error!("Failed to fetch {}: {}", url, e);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // 1. 対象のURL
    let start_url = Url::parse(&args.url)?;
    let mut known_urls = HashSet::<Url>::new();
    known_urls.insert(start_url.clone());

    // 1. チャネルを作成 (tx: 送信機、 rx: 受信機)
    // バッファサイズは32
    let (tx, mut rx) = mpsc::channel(args.tasks);
    let mut active_tasks = 1_usize;

    // 2. 最初のタスクを起動
    // txを複製して、workerに持たせる
    let first_tx = tx.clone();

    tokio::spawn(async move {
        crawl_worker(start_url, first_tx).await;
    });

    info!("Starting crawling concurrently");

    while let Some(result) = rx.recv().await {
        active_tasks -= 1;
        info!("Finished: {} (Found {} links)", result.url, result.links.len());
        
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

#[cfg(test)]
mod tests {
    use my_crawler::extract_links;
    use super::*;



    #[tokio::test]
    async fn test_fetch_links_integration() {
        let mut server = mockito::Server::new_async().await;
        let html = r#"
        <html>
            <body>
                <a href="/next">相対パス</a>
                <a href="https://google.com">絶対パス</a>
                <a>リンクなし</a>
            </body>
        </html>
    "#;
        let _ = server.mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(html.to_string())
            .create_async().await;

        let base_url = Url::parse(&server.url()).unwrap();
        let result = fetch_links(&base_url).await.unwrap();

        let expected_links = vec![
            base_url.join("/next").unwrap(),
            Url::parse("https://google.com/").unwrap()
        ];

        assert_eq!(result, expected_links);

    }
}