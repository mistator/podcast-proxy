use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(rename = "rss")]
pub struct Feed {
    #[serde(rename = "@version")]
    pub version: String,
    pub channel: Channel,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    pub title: String,
    #[serde(rename = "itunes:title")]
    pub itunes_title: Option<String>,
    pub link: Url,
    #[serde(rename = "atom:link")]
    pub atom_link: Option<AtomLink>,
    pub description: String,
    #[serde(rename = "itunes:type")]
    pub itunes_type: Option<String>, // TODO make enum (episodic/serial)
    pub language: Option<String>,
    pub copyright: Option<String>,
    pub managing_editor: Option<String>,
    pub web_master: Option<String>,
    #[serde(rename = "itunes:author")]
    pub author: Option<String>,
    #[serde(rename = "itunes:owner")]
    pub owner: Option<ITunesOwner>,
    pub pub_date: Option<String>,
    pub last_build_date: Option<String>,
    pub category: Option<String>,
    #[serde(rename = "itunes:category", default)]
    pub itunes_categories: Vec<ITunesCategory>,
    pub generator: Option<String>,
    pub docs: Option<Url>,
    pub cloud: Option<Cloud>,
    pub ttl: Option<u32>,
    pub image: Option<Image>,
    #[serde(rename = "itunes:image")]
    pub itunes_image: Option<ITunesImage>,
    pub rating: Option<String>,
    pub text_input: Option<TextInput>,
    #[serde(default)]
    pub skip_hours: Vec<Hour>,
    #[serde(rename = "item", default)]
    pub items: Vec<Item>,
    #[serde(rename = "itunes:explicit")]
    pub explicit: Option<String>, // TODO: make bool (yes/no)
    #[serde(rename = "itunes:block")]
    pub itunes_block: Option<String>, // TODO: make bool (yes/no)
    #[serde(rename = "itunes:complete")]
    pub itunes_complete: Option<String>, // TODO: make bool (yes/no)

}

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    pub url: Url,
    pub title: String,
    pub link: Url,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub description: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, Deref, DerefMut)]
pub struct ITunesImage {
    #[serde(rename = "@href")]
    pub value: Url
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ITunesCategory {
    #[serde(rename = "@text")]
    pub text: String,
    #[serde(rename = "itunes:category")]
    pub subcategory: Option<ITunesSubCategory>,
}

#[derive(Debug, Serialize, Deserialize, Deref, DerefMut)]
pub struct ITunesSubCategory {
    #[serde(rename = "@text")]
    pub text: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ITunesOwner {
    #[serde(rename = "itunes:name")]
    pub name: String,
    #[serde(rename = "itunes:email")]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cloud {
    #[serde(rename = "@domain")]
    pub domain: String,
    #[serde(rename = "@port")]
    pub port: u16,
    #[serde(rename = "@path")]
    pub path: String,
    #[serde(rename = "@registerProcedure")]
    pub register_procedure: String,
    #[serde(rename = "@protocol")]
    pub protocol: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextInput {
    pub title: String,
    pub description: String,
    pub name: String,
    pub link: Url,
}

#[derive(Debug, Serialize, Deserialize, Deref, DerefMut)]
pub struct Hour {
    pub value: u8
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AtomLink {
    #[serde(rename = "@href")]
    pub href: Url,
    #[serde(rename = "@rel")]
    pub rel: Option<String>,
    #[serde(rename = "@type")]
    pub mime_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    pub title: Option<String>,
    #[serde(rename = "itunes:title")]
    pub itunes_title: Option<String>,
    #[serde(rename = "itunes:season")]
    pub season: Option<u32>,
    #[serde(rename = "itunes:episode")]
    pub episode: Option<u32>,
    #[serde(rename = "itunes:episodeType")]
    pub episode_type: Option<String>, // TODO: make enum (full/trailer/bonus)
    pub link: Option<Url>,
    pub description: Option<String>,
    pub author: Option<String>,
    #[serde(rename = "category", default)]
    pub categories: Vec<Category>,
    pub comments: Option<Url>,
    pub enclosure: Option<Enclosure>,
    pub guid: Option<Guid>,
    #[serde(rename = "pubDate")]
    pub pub_date: Option<String>,
    pub source: Option<Source>,
    #[serde(rename = "itunes:duration")]
    pub duration: Option<String>,
    #[serde(rename = "itunes:image")]
    pub image: Option<ITunesImage>,
    #[serde(rename = "itunes:explicit")]
    pub explicit: Option<String>, // TODO: make bool (yes/no)
    #[serde(rename = "itunes:block")]
    pub block: Option<String> // TODO: make bool (yes/no)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    #[serde(rename = "@domain")]
    pub domain: Option<String>,
    #[serde(rename = "#text")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Enclosure {
    #[serde(rename = "@url")]
    pub url: Url,
    #[serde(rename = "@length")]
    pub length: u32,
    #[serde(rename = "@type")]
    pub mime_type: String,
}

fn default_true() -> bool { true }

#[derive(Debug, Serialize, Deserialize)]
pub struct Guid {
    #[serde(rename = "@isPermaLink", default = "default_true")]
    pub is_perma_link: bool,
    #[serde(rename = "#text")]
    pub value: String
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Source {
    #[serde(rename = "@url")]
    pub url: Url,
    #[serde(rename = "#text")]
    pub title: String,
}

impl Feed {
    pub(crate) fn to_xml(&self) -> Result<String, serde_xml_rs::Error> {
        let config = serde_xml_rs::SerdeXml::new()
            .namespace("itunes", "http://www.itunes.com/dtds/podcast-1.0.dtd")
            .namespace("atom", "http://www.w3.org/2005/Atom");

        config.to_string(self)
    }
}