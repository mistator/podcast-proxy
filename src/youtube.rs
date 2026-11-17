use std::fs;
use anyhow::anyhow;
use derive_more::{Deref, DerefMut};
use directories::ProjectDirs;
use serde::Deserialize;
use url::Url;
use yt_dlp::{Downloader, VideoSelection};
use crate::feed::{AtomLink, Channel, Enclosure, Feed, Guid, ITunesImage, Item};
use crate::grabbers::Grabber;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(rename = "feed")]
struct YtFeed {
    #[serde(rename = "yt:playlistId")]
    playlist_id: String,
    #[serde(rename = "yt:channelId")]
    channel_id: String,
    #[serde(rename = "link")]
    atom_link: Option<AtomLink>,
    id: String,
    title: String,
    author: AtomAuthor,
    published: String,
    #[serde(rename = "entry", default)]
    pub items: Vec<YtItem>,
}

#[derive(Deserialize)]
struct AtomAuthor {
    name: String,
    uri: Url,
}

#[derive(Deserialize)]
struct YtItem {
    id: String,
    #[serde(rename = "yt:videoId")]
    video_id: String,
    #[serde(rename = "yt:channelId")]
    channel_id: String,
    title: String,
    link: AtomLink,
    author: AtomAuthor,
    #[serde(rename = "media:group")]
    media: YtMedia,
    published: String,
    updated: String,
}

#[derive(Deserialize)]
struct YtMedia {
    #[serde(rename = "media:title")]
    title: String,
    #[serde(rename = "media:content")]
    content: YtContent,
    #[serde(rename = "media:thumbnail")]
    thumbnail: YtThumbnail,
    #[serde(rename = "media:description")]
    description: String,
}

#[derive(Deserialize)]
struct YtContent {
    #[serde(rename = "@url")]
    url: Url,
    #[serde(rename = "@type")]
    content_type: String,
    #[serde(rename = "@width")]
    width: u32,
    #[serde(rename = "@height")]
    height: u32,
}

#[derive(Deserialize)]
struct YtThumbnail {
    #[serde(rename = "@url")]
    url: Url,
    #[serde(rename = "@width")]
    width: u32,
    #[serde(rename = "@height")]
    height: u32,
}

#[derive(Clone, Deref, DerefMut)]
pub struct YouTube(Downloader);

impl YouTube {
    pub async fn new() -> anyhow::Result<Self> {
        log::debug!("[YT-new] creating YT instance");
        if let Some(dir) = ProjectDirs::from("", "", "feed") {
            let dir = dir.cache_dir();
            log::debug!("[YT-new] found cache dir: {:?}", dir);

            let libraries_dir = dir.join("libraries");
            let output_dir = dir.join("output");

            fs::create_dir_all(&libraries_dir)?;
            fs::create_dir_all(&output_dir)?;

            log::debug!("[YT-new] downloading binaries for yt-dlp");
            let downloader = Downloader::with_new_binaries(libraries_dir, output_dir)
                .await?
                .build()
                .await?;

            Ok(Self(downloader))
        } else { Err(anyhow!("could not create cache directroy")) }

    }
}

impl Grabber for YouTube {
    async fn make_feed(&self, url: &Url, public_url: &Url) -> anyhow::Result<Feed> {
        if url.domain() != Some("www.youtube.com") {
            return Err(anyhow!("invalid url received"));
        }

        let (_, list_id) = url.query_pairs().find(|(key, _)| {
            dbg!(key);
            key == "list"
        })
            .ok_or(anyhow!("invalid url"))?;

        let feed_url = Url::parse(
            format!("https://www.youtube.com/feeds/videos.xml?playlist_id={list_id}").as_str())?;
        let feed = self.download_url(&feed_url).await?;
        let feed = serde_xml_rs::de::from_str::<YtFeed>(feed.as_str())?;

        Ok(Feed {
            version: "2.0".to_string(),
            channel: Channel {
                title: feed.title.clone(),
                itunes_title: Some(feed.title),
                link: Url::parse(
                    format!("https://www.youtube.com/playlist?list={}", feed.playlist_id).as_str(),
                )?,
                atom_link: feed.atom_link,
                description: "".to_string(),
                itunes_type: None,
                language: None,
                copyright: None,
                managing_editor: None,
                web_master: None,
                author: Some(feed.author.name),
                owner: None,
                pub_date: Some(feed.published),
                last_build_date: None,
                category: None,
                itunes_categories: vec![],
                generator: None,
                docs: None,
                cloud: None,
                ttl: None,
                image: None,
                itunes_image: None,
                rating: None,
                text_input: None,
                skip_hours: vec![],
                items: feed.items.into_iter().map(|item| {
                    let url = Url::parse(format!("{}item/youtube/{}", public_url.to_string(), item.video_id).as_str())
                        .unwrap();

                    Item {
                        title: Some(item.title.clone()),
                        itunes_title: Some(item.title),
                        season: None,
                        episode: None,
                        episode_type: None,
                        link: Some(url.clone()),
                        description: Some(item.media.description),
                        author: Some(item.author.name),
                        categories: vec![],
                        comments: None,
                        enclosure: Some(Enclosure {
                            url,
                            length: 0,
                            mime_type: "audio/mpeg".to_string(),
                        }),
                        guid: Some(Guid { is_perma_link: false, value: item.id }),
                        pub_date: Some(item.published),
                        source: None,
                        duration: None,
                        image: Some(ITunesImage{
                            value: item.media.thumbnail.url,
                        }),
                        explicit: None,
                        block: None,
                    }
                }).collect(),
                explicit: None,
                itunes_block: None,
                itunes_complete: None,
            },
        })
    }

    async fn get_item_url(&self, id: &String) -> anyhow::Result<Url> {
        let video_url = format!("https://www.youtube.com/watch?v={}", id);

        let info = self.fetch_video_infos(video_url).await?;
        let audio_format = info.best_audio_format().ok_or(anyhow!("no audio url found"))?;
        let url = audio_format.url()?;

        Url::parse(url.as_str()).map_err(anyhow::Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn test() {
        let youtube = YouTube::new().await.unwrap();
        let feed = youtube.make_feed(&Url::parse("https://www.youtube.com/watch?v=mStOFBvToUI&list=PLzuFY9Ixj9Z6nf7z6t5YmPDTLBgksk8Ts").unwrap()).await.unwrap();

        assert_eq!(feed.version, "2.0")
    }
}