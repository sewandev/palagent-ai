use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CharacterEntry {
    pub player_uid: String,
    pub instance_id: String,
    pub raw_data: Vec<u8>,
}

#[derive(Serialize, Debug, Clone)]
pub struct InventoryItem {
    pub slot_index: u32,
    pub item_id: String,
    pub count: u32,
}

#[derive(Serialize, Debug, Clone)]
pub struct PalSummary {
    pub character_id: String,
    pub gender: String,
    pub level: u32,
    pub exp: u64,
    pub hp: f64,
    pub max_hp: f64,
    pub satiety: f64,
    pub physical_health: String,
    pub friendship: u32,
    pub talents: HashMap<String, u32>,
    pub passive_skills: Vec<String>,
    pub slot_index: u32,
}

#[derive(Serialize, Debug, Clone)]
pub struct PlayerSummary {
    pub player_uid: String,
    pub instance_id: String,
    pub nickname: String,
    pub level: u32,
    pub exp: u64,
    pub hp: f64,
    pub max_hp: f64,
    pub full_stomach: f64,
    pub physical_health: String,
    pub technology_points: u32,
    pub customization: HashMap<String, Value>,
    pub unlocked_technologies: Vec<String>,
    pub active_quest: String,
    pub completed_quests: Vec<String>,
    pub fast_travel_points: Vec<String>,
    pub relics_found: u32,
    pub notes_found: Vec<String>,
    pub npc_talk_counts: HashMap<String, u32>,
    pub common_inventory: Vec<InventoryItem>,
    pub weapons: Vec<InventoryItem>,
    pub armor: Vec<InventoryItem>,
    pub active_pals: Vec<PalSummary>,
    pub palbox_pals: Vec<PalSummary>,
}

#[derive(Serialize, Debug, Clone)]
pub struct BaseCampSummary {
    pub base_camp_id: String,
    pub group_id: String,
    pub level: u32,
    pub coordinates: (f64, f64, f64),
}

#[derive(Serialize, Debug, Clone)]
pub struct GuildSummary {
    pub guild_id: String,
    pub guild_name: String,
    pub admin_player_uid: String,
    pub members: Vec<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct OutputJson {
    pub status: String,
    pub world_path: String,
    pub game_mode: String,
    pub players: Vec<PlayerSummary>,
    pub base_camps: Vec<BaseCampSummary>,
    pub guilds: Vec<GuildSummary>,
}
