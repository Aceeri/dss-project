
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Home {
    pub data: Data,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Data {
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
    //pub items: Vec<Item>,
    pub meta: Meta,
    //pub type: String,
    pub style: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    //call_to_action: ?,
    content_id: Uuid,
    current_availability: Availability,
    encoded_series_id: String,
    //image: Image,
    series_id: Uuid,
    //text: Text,
    text_experience_id: Uuid,
    tags: Vec<Tag>,
    media_rights: MediaRights,
    ratings: Vec<Rating>,
    releases: Vec<Release>,
    //type: String,
    video_art: Vec<VideoArt>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoArt { 
    // TODO
    media_metadata: MediaMetadata,
    purpose: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaMetadata {
    urls: Vec<Url>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Url {
    url: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    release_date: String,
    release_type: String,
    release_year: u16,
    //territory: Option<?>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rating {
    //advisories: Vec<?>,
    description: Option<String>,
    system: String,
    value: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaRights {
    //download_blocked: bool,
    //pcon_blocked: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    display_name: Option<String>,
    //type: String,
    value: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Availability {
    region: String,
    //kids_mode: bool,
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Slug {
    pub language: String,
    pub value: String,
}

#[cfg(test)]
mod test {
    use crate::home::Home;

    #[test]
    fn deserialize() {
        let url = "https://cd-static.bamgrid.com/dp-117731241344/home.json";
        reqwest::blocking::get(url).expect("response from url")
                .json::<Home>().expect("working deserialization");
    }
}