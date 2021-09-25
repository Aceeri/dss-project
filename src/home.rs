
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct Home {
    pub data: StandardCollection,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StandardCollection {
    //pub call_to_action: ?,
    pub collection_group: CollectionGroup,
    pub collection_id: Uuid,
    pub containers: Vec<Container>,
    pub image: Image,
    pub text: Text,
    pub video_art: Vec<Image>,
    //type: String,
}

pub struct Container {
    set: Set,
    //type: String,
    style: String,
}

pub struct Set {
    content_class: String,
    items: Vec<Item>,
    meta: Meta,
    //type: String,
    style: String,
}

pub struct Meta {
    hits: u32,
    offset: u32,
    page_size: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CollectionGroup {
    pub collection_group_id: Uuid,
    pub content_class: String,
    pub key: String,
    pub slugs: Vec<Slug>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Slug {
    pub language: String,
    pub value: String,
}