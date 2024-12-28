use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: i32 = 769;
pub const MINECRAFT_VERSION: &str = "1.21.4";


#[derive(Serialize, Deserialize)]
pub struct Struct {
    pub name: String,
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct Players {
    pub max: i32,
    pub online: i32,
    pub sample: Vec<Struct>,
}

#[derive(Serialize, Deserialize)]
pub struct Version {
    pub name: String,
    pub protocol: i32,
}

#[derive(Serialize, Deserialize)]
pub struct Status {
    pub version: Version,
    pub players: Players,
    pub description: String,
    pub favicon: Option<String>,
    #[serde(rename = "enforcesSecureChat")]
    pub enforces_secure_chat: bool,
    #[serde(rename = "previewsChat")]
    pub previews_chat: bool,
}

impl Status {
    pub fn empty() -> Self {
        Self::build(0, "Minecraft Server".into(), None, false, false)
    }
    
    pub fn build(
        max_players: i32, 
        description: String,
        favicon: Option<String>, 
        enforces_secure_chat: bool,
        previews_chat: bool
    ) -> Self {
        Status {
            version: Version {
                name: MINECRAFT_VERSION.into(),
                protocol: PROTOCOL_VERSION 
            },
            players: Players {
                max: max_players, // configurable
                online: 0,
                sample: vec![]
            },
            description, // configurable
            favicon, // configurable
            enforces_secure_chat, // configurable
            previews_chat, // configurable
        }
    }
    
}

