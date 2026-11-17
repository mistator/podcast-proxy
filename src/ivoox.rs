use std::sync::LazyLock;
use anyhow::anyhow;
use regex::Regex;
use url::Url;
use crate::feed::Feed;
use crate::grabbers::Grabber;


pub struct Ivoox;

impl Grabber for Ivoox {
    async fn make_feed(&self, url: &Url, public_url: &Url) -> anyhow::Result<Feed> {
        static RE: LazyLock<Regex> = LazyLock::new(||
            Regex::new(r"/.+_sq_(.+)_1\.html").unwrap());

        if url.domain() != Some("www.ivoox.com") {
            return Err(anyhow!("invalid url received"));
        }

        let caps = RE.captures(url.path()).ok_or(anyhow!("invalid url received"))?;
        let feed_id = &caps[1];

        let url =
            Url::parse(format!("https://feeds.ivoox.com/feed_fg_{feed_id}_filtro_1.xml").as_str())?;

        let text = self.download_url(&url).await?;
        let mut feed = serde_xml_rs::from_str::<Feed>(text.as_str())?;

        for item in feed.channel.items.iter_mut() {
            let guid = item.guid.as_ref().ok_or(anyhow!("missing guid for item"))?.value.clone();
            let id = guid.split("/").last().ok_or(anyhow!("invalid guid for item"))?;
            let enclosure = item.enclosure.as_mut().ok_or(anyhow!("missing enclosure for item"))?;

            enclosure.url = 
                Url::parse(format!("{}/item/ivoox/{}", public_url.to_string(), id).as_str())
                    .map_err(anyhow::Error::from)?;
        }

        Ok(feed)
    }

    async fn get_item_url(&self, id: &String) -> anyhow::Result<Url> {
        Url::parse(format!("https://www.ivoox.com/listen_mn_{id}_feed_1.mp3").as_str())
            .map_err(anyhow::Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn grab_ivoox_feed() {
        let ivoox = Ivoox;

        let feed = ivoox.make_feed(&Url::parse("https://www.ivoox.com/podcast-cuarto-milenio_sq_f17162_1.html").unwrap()).await;

        assert!(feed.is_ok())
    }
}
