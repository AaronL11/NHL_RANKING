use std::collections::HashMap;

use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
pub struct PlayerResponse {
    #[serde(rename = "playerId")]
    pub id: i32,

    #[serde(flatten, deserialize_with = "combine_names")]
    pub name: String,

    pub position: String,

    #[serde(rename = "currentTeamId")]
    pub team_id: i32,
}

// Helper struct for Serde to grab both firstName and lastName
#[derive(Deserialize)]
struct RawName {
    #[serde(rename = "firstName")]
    first: NameWrapper,
    #[serde(rename = "lastName")]
    last: NameWrapper,
}

#[derive(Deserialize)]
struct NameWrapper {
    default: String,
}

fn combine_names<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = RawName::deserialize(deserializer)?;
    Ok(format!("{} {}", raw.first.default, raw.last.default))
}

fn flatten_name<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let name = NameWrapper::deserialize(deserializer)?;
    Ok(name.default)
}

#[derive(Debug, Deserialize)]
pub struct PlayerStats {
    #[serde(rename = "playerId")]
    id: i32,

    #[serde(rename = "sweaterNumber")]
    number: i32,

    #[serde(deserialize_with = "flatten_name")]
    name: String,

    position: String,

    goals: i32,

    assists: i32,

    #[serde(rename = "plusMinus")]
    pm: i32,

    pim: i32,

    hits: i32,

    pp: i32,

    #[serde(rename = "faceoffWinningPctg")]
    faceoff: i32,

    sog: i32,

    toi: String,

    #[serde(rename = "blockedShots")]
    blocks: i32,

    shifts: i32,

    giveaways: i32,

    takeaways: i32,
}

pub fn collect_players<'de, D>(deserializer: D) -> Result<Vec<PlayerStats>, D::Error>
where
    D: Deserializer<'de>,
{
    let map: HashMap<String, PlayerStats> = HashMap::deserialize(deserializer)?;
    Ok(map.into_values().collect())
}
