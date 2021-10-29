use uuid::Uuid;

use std::collections::HashMap;

// Leaving a good portion of the unused fields commented out for visibility
// but don't deserialize into them since I don't know the real schema and don't
// want this all to fail because something is sometimes not sent, or is an enum, etc.

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Home {
    pub data: HomeKind,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefSet {
    pub data: SetKind,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SetKind {
    CuratedSet(Set),
    PersonalizedCuratedSet(Set),
    TrendingSet(Set),
}

impl SetKind {
    pub fn set(&self) -> &Set {
        match self {
            SetKind::CuratedSet(set) => set,
            SetKind::PersonalizedCuratedSet(set) => set,
            SetKind::TrendingSet(set) => set,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum HomeKind {
    StandardCollection(Collection),
}

impl HomeKind {
    pub fn collection(&self) -> &Collection {
        match self {
            HomeKind::StandardCollection(collection) => collection,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    //pub call_to_action: ?,
    pub collection_group: CollectionGroup,
    pub collection_id: Uuid,
    pub containers: Vec<Container>,
    //pub image: Image,
    //pub text: Text,
    //pub video_art: Vec<Image>,
    //pub type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Container {
    pub set: Set,
    //pub type: String,
    pub style: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Set {
    //pub content_class: String,
    pub items: Option<Vec<Item>>,
    //pub meta: Option<Meta>,
    //pub type: String,
    //pub style: Option<String>,
    pub ref_id: Option<Uuid>,
    //pub ref_id_type: String,
    //pub ref_type: String,
    pub text: TextRefs,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    //call_to_action: ?,
    //content_id: Uuid,
    //current_availability: Availability,
    //encoded_series_id: String,
    pub image: ImageRefs,
    //series_id: Uuid,
    pub text: TextRefs,
    //text_experience_id: Uuid,
    //tags: Vec<Tag>,
    //media_rights: MediaRights,
    //ratings: Vec<Rating>,
    //releases: Vec<Release>,
    //type: String,
    //video_art: Vec<VideoArt>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageRefs {
    // Map of aspect ratios to details of image specifics.
    pub tile: HashMap<String, Image>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Image {
    Default { default: ImageDetails },
    Series { default: ImageDetails },
    Program { default: ImageDetails },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRefs {
    pub title: Title,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Title {
    pub slug: Option<Text>,
    pub full: Text,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Text {
    Set { default: TextDetails },
    Collection { default: TextDetails },
    Program { default: TextDetails },
    Series { default: TextDetails },
}

impl Text {
    pub fn details(&self) -> &TextDetails {
        match self {
            Text::Set { default } => default,
            Text::Collection { default } => default,
            Text::Program { default } => default,
            Text::Series { default } => default,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDetails {
    pub content: String,
    pub language: String,
    //sourceEntity: String,
}

impl Image {
    pub fn details(&self) -> &ImageDetails {
        match self {
            Image::Default { default } => default,
            Image::Series { default } => default,
            Image::Program { default } => default,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageDetails {
    //master_id: Uuid, // not actually a uuid, probably just a string?
    pub master_width: u32,
    pub master_height: u32,
    pub url: String, // Seems to be resizable based on url encoded parameters.
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoArt {
    pub media_metadata: MediaMetadata,
    pub purpose: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaMetadata {
    pub urls: Vec<Url>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Url {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub release_date: String,
    pub release_type: String,
    pub release_year: u16,
    //territory: Option<?>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rating {
    //advisories: Vec<?>,
    pub description: Option<String>,
    pub system: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaRights {
    //pub download_blocked: bool,
//pub pcon_blocked: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub display_name: Option<String>,
    //pub type: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Availability {
    pub region: String,
    //pub kids_mode: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Meta {
    pub hits: u32,
    pub offset: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionGroup {
    pub collection_group_id: Uuid,
    //pub content_class: String,
    //pub key: String,
    //pub slugs: Vec<Slug>,
}

// Non human friendly data?
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Slug {
    pub language: String,
    pub value: String,
}

#[cfg(test)]
mod test {
    use super::{Home, RefSet};

    fn fetch_home() -> Home {
        let url = "https://cd-static.bamgrid.com/dp-117731241344/home.json";
        reqwest::blocking::get(url)
            .expect("response from url")
            .json::<Home>()
            .expect("working home deserialization")
    }

    // Can we fetch and deserialize the home screen.
    #[test]
    fn deserialize_home() {
        fetch_home();
    }

    #[test]
    fn deserialize_refset() {
        let url = "https://cd-static.bamgrid.com/dp-117731241344/sets/bd1bfb9a-bbf7-43a0-ac5e-3e3889d7224d.json";
        reqwest::blocking::get(url)
            .expect("response from url")
            .json::<RefSet>()
            .expect("working refset deserialization");
    }

    // Can fetch the home screen and load an image correctly. Somewhat redundant with deserialization.
    #[test]
    fn fetch_png() {
        use crate::image::EncodableLayout;
        let home = fetch_home();

        let items = home.data.collection().containers[0]
            .set
            .items
            .as_ref()
            .expect("expected items");
        let image = items[0]
            .image
            .tile
            .iter()
            .next()
            .expect("expected an image reference")
            .1;

        let image_details = image.details();
        println!("{:?}", image_details);
        let bytes = reqwest::blocking::get(&image_details.url)
            .expect("response from url")
            .bytes()
            .expect("expected jpeg bytes");

        let _img = image::load_from_memory(bytes.as_bytes()).expect("load image from response");
    }

    #[test]
    fn fetch_text() {
        let home = fetch_home();

        let details = home.data.collection().containers[0]
            .set
            .text
            .title
            .full
            .details();
        println!("{:?}", details.content);
    }
}
