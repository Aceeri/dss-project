use uuid::Uuid;

use std::collections::HashMap;

// Leaving a good portion of the unused fields commented out for visibility
// but don't deserialize into them since I don't know the real schema and don't
// want this all to fail because something is sometimes not sent, or is an enum, etc.

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Home {
    pub data: Data,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Data {
    // Maybe this should be an enum?
    pub standard_collection: StandardCollection,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardCollection {
    //pub call_to_action: ?,
    pub collection_group: CollectionGroup,
    pub collection_id: Uuid,
    pub containers: Vec<Container>,
    //pub image: Image,
    //pub text: Text,
    //pub video_art: Vec<Image>,
    //pub type: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Container {
    pub set: Set,
    //pub type: String,
    pub style: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Set {
    //pub content_class: String,
    pub items: Option<Vec<Item>>,
    //pub meta: Option<Meta>,
    //pub type: String,
    //pub style: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    //call_to_action: ?,
    //content_id: Uuid,
    //current_availability: Availability,
    //encoded_series_id: String,
    pub image: ImageRefs,
    //series_id: Uuid,
    //text: Text,
    //text_experience_id: Uuid,
    //tags: Vec<Tag>,
    //media_rights: MediaRights,
    //ratings: Vec<Rating>,
    //releases: Vec<Release>,
    //type: String,
    //video_art: Vec<VideoArt>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageRefs {
    pub tile: HashMap<String, Image>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    // Maybe an enum?
    // Map of aspect ratios to details of image specifics.
    pub series: Option<HashMap<String, ImageDetails>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageDetails {
    //master_id: Uuid, // not actually a uuid, probably just a string?
    pub master_width: u32,
    pub master_height: u32,
    pub url: String, // Seems to be resizable based on url encoded parameters.
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoArt {
    pub media_metadata: MediaMetadata,
    pub purpose: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaMetadata {
    pub urls: Vec<Url>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Url {
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub release_date: String,
    pub release_type: String,
    pub release_year: u16,
    //territory: Option<?>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rating {
    //advisories: Vec<?>,
    pub description: Option<String>,
    pub system: String,
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaRights {
    //pub download_blocked: bool,
//pub pcon_blocked: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub display_name: Option<String>,
    //pub type: String,
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Availability {
    pub region: String,
    //pub kids_mode: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Meta {
    pub hits: u32,
    pub offset: u32,
    pub page_size: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionGroup {
    pub collection_group_id: Uuid,
    //pub content_class: String,
    //pub key: String,
    //pub slugs: Vec<Slug>,
}

// Non human friendly data?
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Slug {
    pub language: String,
    pub value: String,
}

#[cfg(test)]
mod test {
    use crate::home::Home;

    // Can we fetch and deserialize the home screen.
    #[test]
    fn deserialize() {
        let url = "https://cd-static.bamgrid.com/dp-117731241344/home.json";
        reqwest::blocking::get(url)
            .expect("response from url")
            .json::<Home>()
            .expect("working deserialization");
    }

    // Can fetch the home screen and load an image correctly. Somewhat redundant with deserialization.
    #[test]
    fn fetch_png() {
        use crate::image::EncodableLayout;

        let url = "https://cd-static.bamgrid.com/dp-117731241344/home.json";
        let home = reqwest::blocking::get(url)
            .expect("response from url")
            .json::<Home>()
            .expect("working deserialization");

        let items = home.data.standard_collection.containers[0]
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
        let series = image.series.as_ref().expect("expected series");
        let image_details = series.get("default").expect("expected default tile image");

        println!("{:?}", image_details);
        let bytes = reqwest::blocking::get(&image_details.url)
            .expect("response from url")
            .bytes()
            .expect("expected jpeg bytes");

        let _img = image::load_from_memory(bytes.as_bytes()).expect("load image from response");
    }
}
