use clap::{Parser, Subcommand};
use std::collections::HashSet;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use my_crawler::extract_links;
use tracing::{error, info};


#[derive(Serialize, Deserialize)]
struct CrawlResult {
    url: Url,
    links: Vec<Url>
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Crawl {
        #[arg(short, long)]
        url: String,
        #[arg(short, long, default_value_t=10)]
        tasks: usize
    },

    List,
    Get {key: String}
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

    let db = sled::open("data")?;
    match args.command {
        Commands::Crawl {url, tasks} => {
            info!("Starting crawling for {} with {} tasks...", url, tasks);
            let start_url = Url::parse(&url)?;
            let mut known_urls = HashSet::<Url>::new();
            known_urls.insert(start_url.clone());

            let (tx, mut rx) = mpsc::channel(tasks);
            let mut active_tasks = 1_usize;

            let first_tx = tx.clone();

            tokio::spawn(async move {
                crawl_worker(start_url, first_tx).await;
            });


            while let Some(result) = rx.recv().await {
                active_tasks -= 1;
                info!("Finished: {} (Found {} links)", result.url, result.links.len());

                let json_bytes = serde_json::to_vec(&result)?;
                let _ = db.insert(result.url.as_str(), json_bytes);


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
        }
        Commands::List => {
            println!("Listing all data...");
            let urls = db
                .iter()
                .filter_map(|x| {
                if let Ok((key, v)) = x {
                    if let Ok(byte_url) = std::str::from_utf8(&key){
                        return Some(byte_url.to_string());
                    }
                }
                None
            }).collect::<Vec<String>>();
            for url in urls {
                println!("{}", url);
            }
        }
        Commands::Get {key} => {
            match db.get(key.as_str()) ? {
                Some(bytes) => {
                    let data: CrawlResult = serde_json::from_slice(&bytes)?;

                    info!("Data found for: {}", key);
                    println!("URL:{}", data.url);
                    println!("Links found: {}", data.links.len());

                    for link in data.links.iter().take(5){
                        println!(" - {}", link);
                    }
                }
                None => {
                    println!("{}のデータは見つかりませんでした。", key);
                }
            }
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