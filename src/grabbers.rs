use url::Url;
use crate::feed::Feed;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/156.0.0.0 Safari/537.36";

pub trait Grabber {
    async fn make_feed(&self, url: &Url, public_url: &Url) -> anyhow::Result<Feed>;
    async fn get_item_url(&self, id: &String) -> anyhow::Result<Url>;
    async fn download_url(&self, url: &Url) -> anyhow::Result<String> {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()?;

        let response = client.get(url.as_str()).send().await?;
        response.text().await.map_err(anyhow::Error::from)
    }
}