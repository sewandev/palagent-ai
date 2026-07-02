use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::OnceLock;

static DB_PATH: OnceLock<PathBuf> = OnceLock::new();

#[derive(serde::Deserialize)]
struct PalworldDataJson {
    pals: Vec<serde_json::Value>,
    passives: Vec<serde_json::Value>,
    items: Vec<serde_json::Value>,
    breeding_exceptions: Vec<serde_json::Value>,
    active_skills: Vec<serde_json::Value>,
    recipes: Vec<serde_json::Value>,
    pal_drops: Vec<serde_json::Value>,
}

pub fn init_database() {
    let home_dir = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| "C:\\".to_string());
    let db_dir = std::path::Path::new(&home_dir).join(".palagent-ai");
    let _ = std::fs::create_dir_all(&db_dir);
    let target_path = db_dir.join("palworld_data.db");

    let _ = DB_PATH.set(target_path);

    let db_exists_and_populated = if let Some(conn) = get_conn() {
        if let Ok(count) = conn.query_row(
            "SELECT COUNT(*) FROM pals WHERE internal_id = 'BlackPuppy'",
            [],
            |row| row.get::<_, i64>(0),
        ) {
            count > 0
        } else {
            false
        }
    } else {
        false
    };

    if db_exists_and_populated {
        return;
    }

    if let Some(mut conn) = get_conn() {
        if let Ok(tx) = conn.transaction() {
            // Create tables
            tx.execute(
                "CREATE TABLE IF NOT EXISTS pals (
                    internal_id TEXT PRIMARY KEY,
                    name_en TEXT NOT NULL,
                    name_es TEXT NOT NULL,
                    breed_power INTEGER DEFAULT 1500,
                    kindling INTEGER DEFAULT 0,
                    watering INTEGER DEFAULT 0,
                    planting INTEGER DEFAULT 0,
                    generating INTEGER DEFAULT 0,
                    handwork INTEGER DEFAULT 0,
                    gathering INTEGER DEFAULT 0,
                    lumbering INTEGER DEFAULT 0,
                    mining INTEGER DEFAULT 0,
                    medicine INTEGER DEFAULT 0,
                    cooling INTEGER DEFAULT 0,
                    transporting INTEGER DEFAULT 0,
                    farming INTEGER DEFAULT 0
                )",
                [],
            )
            .ok();

            tx.execute(
                "CREATE TABLE IF NOT EXISTS passives (
                    internal_id TEXT PRIMARY KEY,
                    name_en TEXT NOT NULL,
                    name_es TEXT NOT NULL,
                    description_en TEXT,
                    description_es TEXT
                )",
                [],
            )
            .ok();

            tx.execute(
                "CREATE TABLE IF NOT EXISTS items (
                    internal_id TEXT PRIMARY KEY,
                    name_en TEXT NOT NULL,
                    name_es TEXT NOT NULL
                )",
                [],
            )
            .ok();

            tx.execute(
                "CREATE TABLE IF NOT EXISTS breeding_exceptions (
                    parent_a TEXT,
                    parent_b TEXT,
                    child TEXT,
                    PRIMARY KEY (parent_a, parent_b)
                )",
                [],
            )
            .ok();

            tx.execute(
                "CREATE TABLE IF NOT EXISTS active_skills (
                    internal_id TEXT PRIMARY KEY,
                    name_en TEXT NOT NULL,
                    name_es TEXT NOT NULL,
                    power INTEGER DEFAULT 0,
                    cooldown INTEGER DEFAULT 0,
                    element TEXT
                )",
                [],
            )
            .ok();

            tx.execute(
                "CREATE TABLE IF NOT EXISTS recipes (
                    item_id TEXT,
                    ingredient_id TEXT,
                    count INTEGER,
                    PRIMARY KEY (item_id, ingredient_id)
                )",
                [],
            )
            .ok();

            tx.execute(
                "CREATE TABLE IF NOT EXISTS pal_drops (
                    pal_id TEXT,
                    item_id TEXT,
                    chance INTEGER,
                    min_qty INTEGER,
                    max_qty INTEGER,
                    PRIMARY KEY (pal_id, item_id)
                )",
                [],
            )
            .ok();

            // Populate Pals
            #[allow(clippy::type_complexity)]
            let pals_data: &[(
                &str,
                &str,
                &str,
                i32,
                i32,
                i32,
                i32,
                i32,
                i32,
                i32,
                i32,
                i32,
                i32,
                i32,
                i32,
                i32,
            )] = &[
                (
                    "SheepBall",
                    "Lamball",
                    "Lamball",
                    1470,
                    0,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                    0,
                    0,
                    0,
                    1,
                    1,
                ),
                (
                    "PinkCat", "Cattiva", "Cattiva", 1460, 0, 0, 0, 0, 1, 1, 0, 1, 0, 0, 1, 0,
                ),
                (
                    "ChickenPal",
                    "Chikipi",
                    "Chikipi",
                    1500,
                    0,
                    0,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                    0,
                    0,
                    0,
                    1,
                ),
                (
                    "Carbunclo",
                    "Lifmunk",
                    "Lifmunk",
                    1430,
                    0,
                    0,
                    1,
                    0,
                    1,
                    1,
                    1,
                    0,
                    1,
                    0,
                    0,
                    0,
                ),
                (
                    "Kitsunebi",
                    "Foxsparks",
                    "Foxsparks",
                    1400,
                    1,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "BluePlatypus",
                    "Fuack",
                    "Fuack",
                    1350,
                    0,
                    1,
                    0,
                    0,
                    1,
                    0,
                    0,
                    0,
                    0,
                    0,
                    1,
                    0,
                ),
                (
                    "ElecCat", "Sparkit", "Sparkit", 1410, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 0,
                ),
                (
                    "Alpaca", "Melpaca", "Melpaca", 1100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                ),
                (
                    "Bastet", "Mau", "Mau", 1480, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                ),
                (
                    "Bastet_Ice",
                    "Mau Cryst",
                    "Mau Cryst",
                    1450,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    1,
                    0,
                    1,
                ),
                (
                    "CaptainPenguin",
                    "Penking",
                    "Penking",
                    950,
                    0,
                    2,
                    0,
                    0,
                    2,
                    0,
                    0,
                    2,
                    0,
                    0,
                    2,
                    0,
                ),
                (
                    "CatBat", "Tombat", "Tombat", 1020, 0, 0, 0, 0, 0, 2, 0, 2, 0, 0, 2, 0,
                ),
                (
                    "CatMage", "Katress", "Katress", 1050, 0, 0, 0, 0, 2, 0, 0, 0, 2, 0, 2, 0,
                ),
                (
                    "CowPal",
                    "Mozzarina",
                    "Mozzarina",
                    1220,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    1,
                ),
                (
                    "CuteButterfly",
                    "Cinnamoth",
                    "Cinnamoth",
                    1260,
                    0,
                    0,
                    2,
                    0,
                    0,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                    0,
                ),
                (
                    "DreamDemon",
                    "Daedream",
                    "Daedream",
                    1220,
                    0,
                    0,
                    0,
                    0,
                    1,
                    1,
                    0,
                    0,
                    1,
                    0,
                    1,
                    0,
                ),
                (
                    "DrillGame",
                    "Digtoise",
                    "Digtoise",
                    1070,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    3,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "ElecPanda",
                    "Grizzbolt",
                    "Grizzbolt",
                    510,
                    0,
                    0,
                    0,
                    3,
                    2,
                    0,
                    2,
                    0,
                    0,
                    0,
                    3,
                    0,
                ),
                (
                    "FairyDragon",
                    "Elphidran",
                    "Elphidran",
                    560,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    2,
                    0,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "FairyDragon_Water",
                    "Elphidran Aqua",
                    "Elphidran Aqua",
                    530,
                    0,
                    3,
                    0,
                    0,
                    0,
                    0,
                    2,
                    0,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "LazyDragon",
                    "Relaxaurus",
                    "Relaxaurus",
                    280,
                    0,
                    2,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    1,
                    0,
                ),
                (
                    "LazyDragon_Electric",
                    "Relaxaurus Lux",
                    "Relaxaurus Lux",
                    250,
                    0,
                    0,
                    0,
                    3,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    1,
                    0,
                ),
                (
                    "Eagle", "Galeclaw", "Galeclaw", 1050, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 2, 0,
                ),
                (
                    "PurpleSpider",
                    "Tarantriss",
                    "Tarantriss",
                    950,
                    0,
                    0,
                    0,
                    0,
                    1,
                    2,
                    0,
                    0,
                    1,
                    0,
                    0,
                    0,
                ),
                (
                    "JetDragon",
                    "Jetragon",
                    "Jetragon",
                    100,
                    0,
                    0,
                    0,
                    0,
                    0,
                    3,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "Anubis", "Anubis", "Anubis", 570, 0, 0, 0, 0, 4, 0, 0, 3, 0, 0, 2, 0,
                ),
                (
                    "GrassPanda",
                    "Mossanda",
                    "Mossanda",
                    620,
                    0,
                    0,
                    2,
                    0,
                    2,
                    0,
                    2,
                    0,
                    0,
                    0,
                    3,
                    0,
                ),
                (
                    "GrassPanda_Electric",
                    "Mossanda Lux",
                    "Mossanda Lux",
                    590,
                    0,
                    0,
                    0,
                    2,
                    2,
                    0,
                    2,
                    0,
                    0,
                    0,
                    3,
                    0,
                ),
                (
                    "Deer",
                    "Eikthyrdeer",
                    "Eikthyrdeer",
                    1090,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    2,
                    0,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "Deer_Ground",
                    "Eikthyrdeer Terra",
                    "Eikthyrdeer Terra",
                    1060,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    2,
                    0,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "GigaHorn", "Fenglope", "Fenglope", 1040, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0,
                ),
                (
                    "Sweeney", "Sweepa", "Sweepa", 920, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0,
                ),
                (
                    "Swee", "Swee", "Swee", 1420, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0,
                ),
                (
                    "Raiju", "Rayhound", "Rayhound", 740, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0,
                ),
                (
                    "FireKirin",
                    "Pyrin",
                    "Pyrin",
                    580,
                    2,
                    0,
                    0,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "FireKirin_Dark",
                    "Pyrin Noct",
                    "Pyrin Noct",
                    550,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    2,
                    0,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "IceDeer", "Reptyro", "Reptyro", 700, 3, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0,
                ),
                (
                    "SakuraSaurus",
                    "Broncherry",
                    "Broncherry",
                    930,
                    0,
                    0,
                    3,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "SakuraSaurus_Water",
                    "Broncherry Aqua",
                    "Broncherry Aqua",
                    900,
                    0,
                    3,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "GreyFox", "Vaelet", "Vaelet", 1010, 0, 0, 1, 0, 2, 1, 0, 0, 3, 0, 1, 0,
                ),
                (
                    "Baphomet",
                    "Incineram",
                    "Incineram",
                    690,
                    1,
                    0,
                    0,
                    0,
                    2,
                    0,
                    0,
                    1,
                    0,
                    0,
                    2,
                    0,
                ),
                (
                    "Baphomet_Dark",
                    "Incineram Noct",
                    "Incineram Noct",
                    660,
                    0,
                    0,
                    0,
                    0,
                    2,
                    0,
                    0,
                    1,
                    0,
                    0,
                    2,
                    0,
                ),
                (
                    "Caprity", "Caprity", "Caprity", 1190, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                ),
                (
                    "BerryGoat",
                    "Caprity",
                    "Caprity",
                    1190,
                    0,
                    0,
                    2,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    1,
                ),
                (
                    "GrassMammoth",
                    "Mammorest",
                    "Mammorest",
                    260,
                    0,
                    0,
                    2,
                    0,
                    0,
                    0,
                    2,
                    2,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "Hedgehog", "Jolthog", "Jolthog", 1450, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
                ),
                (
                    "Hedgehog_Ice",
                    "Jolthog Cryst",
                    "Jolthog Cryst",
                    1440,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                ),
                (
                    "Penguin",
                    "Pengullet",
                    "Pengullet",
                    1390,
                    0,
                    1,
                    0,
                    0,
                    1,
                    0,
                    0,
                    0,
                    0,
                    1,
                    1,
                    0,
                ),
                (
                    "FeatherDactyl",
                    "Celaray",
                    "Celaray",
                    1080,
                    0,
                    1,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    2,
                    0,
                ),
                (
                    "WindChimes",
                    "Hangyu",
                    "Hangyu",
                    1430,
                    0,
                    0,
                    0,
                    0,
                    1,
                    1,
                    0,
                    0,
                    0,
                    0,
                    2,
                    0,
                ),
                (
                    "WindChimes_Ice",
                    "Hangyu Cryst",
                    "Hangyu Cryst",
                    1410,
                    0,
                    0,
                    0,
                    0,
                    1,
                    1,
                    0,
                    0,
                    0,
                    1,
                    2,
                    0,
                ),
                (
                    "Yeti", "Wumpo", "Wumpo", 290, 0, 0, 0, 0, 1, 0, 3, 0, 0, 1, 4, 0,
                ),
                (
                    "Yeti_Grass",
                    "Wumpo Botan",
                    "Wumpo Botan",
                    270,
                    0,
                    0,
                    2,
                    0,
                    1,
                    0,
                    3,
                    0,
                    0,
                    0,
                    4,
                    0,
                ),
                (
                    "Vixy", "Vixy", "Vixy", 1450, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1,
                ),
                (
                    "LizardMan",
                    "Leezpunk",
                    "Leezpunk",
                    1030,
                    0,
                    0,
                    0,
                    0,
                    1,
                    1,
                    0,
                    0,
                    0,
                    0,
                    2,
                    0,
                ),
                (
                    "LizardMan_Fire",
                    "Leezpunk Ignis",
                    "Leezpunk Ignis",
                    1000,
                    1,
                    0,
                    0,
                    0,
                    1,
                    1,
                    0,
                    0,
                    0,
                    0,
                    2,
                    0,
                ),
                (
                    "FlameBuffalo",
                    "Arsox",
                    "Arsox",
                    850,
                    2,
                    0,
                    0,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "Serpent", "Surfent", "Surfent", 840, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ),
                (
                    "Serpent_Coal",
                    "Surfent Terra",
                    "Surfent Terra",
                    820,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    1,
                    0,
                ),
                (
                    "SharkKid", "Gobfin", "Gobfin", 1020, 0, 2, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0,
                ),
                (
                    "SharkKid_Fire",
                    "Gobfin Ignis",
                    "Gobfin Ignis",
                    990,
                    2,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                    0,
                    0,
                    0,
                    1,
                    0,
                ),
                (
                    "Boar", "Rushoar", "Rushoar", 1120, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0,
                ),
                (
                    "HawkBird", "Nitewing", "Nitewing", 820, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 2, 0,
                ),
                (
                    "Garm", "Direhowl", "Direhowl", 1010, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0,
                ),
                (
                    "LazyCatfish",
                    "Dumud",
                    "Dumud",
                    1010,
                    0,
                    1,
                    0,
                    0,
                    0,
                    0,
                    0,
                    2,
                    0,
                    0,
                    1,
                    0,
                ),
                (
                    "FlowerDinosaur",
                    "Dinossom",
                    "Dinossom",
                    930,
                    0,
                    0,
                    2,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "RobinHood",
                    "Robinquill",
                    "Robinquill",
                    830,
                    0,
                    0,
                    1,
                    0,
                    2,
                    1,
                    1,
                    0,
                    1,
                    0,
                    2,
                    0,
                ),
                (
                    "CuteFox", "Vixy", "Vixy", 1450, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1,
                ),
                (
                    "BOSS_FairyDragon",
                    "Elphidran (Alpha)",
                    "Elphidran (Alfa)",
                    560,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    2,
                    0,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "BOSS_JetDragon",
                    "Jetragon (Alpha)",
                    "Jetragon (Alfa)",
                    100,
                    0,
                    0,
                    0,
                    0,
                    0,
                    3,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "BOSS_LazyDragon",
                    "Relaxaurus (Alpha)",
                    "Relaxaurus (Alfa)",
                    280,
                    0,
                    2,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    1,
                    0,
                ),
                (
                    "BOSS_Anubis",
                    "Anubis (Alpha)",
                    "Anubis (Alfa)",
                    570,
                    0,
                    0,
                    0,
                    0,
                    4,
                    0,
                    0,
                    3,
                    0,
                    0,
                    2,
                    0,
                ),
                (
                    "BOSS_Grizzbolt",
                    "Grizzbolt (Alpha)",
                    "Grizzbolt (Alfa)",
                    510,
                    0,
                    0,
                    0,
                    3,
                    2,
                    0,
                    2,
                    0,
                    0,
                    0,
                    3,
                    0,
                ),
                (
                    "BOSS_MastaBeast",
                    "Grizzbolt (Alpha)",
                    "Grizzbolt (Alfa)",
                    510,
                    0,
                    0,
                    0,
                    3,
                    2,
                    0,
                    2,
                    0,
                    0,
                    0,
                    3,
                    0,
                ),
                (
                    "BOSS_WeaselDragon",
                    "Chillet (Alpha)",
                    "Chillet (Alfa)",
                    1010,
                    0,
                    0,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                    0,
                    0,
                    1,
                    0,
                ),
                (
                    "BOSS_Boar",
                    "Rushoar (Alpha)",
                    "Rushoar (Alfa)",
                    1120,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "WhiteAlienDragon",
                    "Xenogard",
                    "Xenogard",
                    110,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    4,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "DarkAlien",
                    "Xenovader",
                    "Xenovader",
                    420,
                    0,
                    0,
                    0,
                    0,
                    2,
                    2,
                    2,
                    2,
                    0,
                    0,
                    2,
                    0,
                ),
                (
                    "NightLady",
                    "Selyne",
                    "Selyne",
                    120,
                    0,
                    0,
                    0,
                    3,
                    3,
                    0,
                    0,
                    0,
                    3,
                    0,
                    0,
                    0,
                ),
                (
                    "FrogMan", "Croajiro", "Croajiro", 1200, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0,
                ),
                (
                    "FlowerGentle",
                    "Lullu",
                    "Lullu",
                    1150,
                    0,
                    0,
                    2,
                    0,
                    0,
                    1,
                    0,
                    0,
                    2,
                    0,
                    0,
                    0,
                ),
                (
                    "MushroomDino",
                    "Shroomer",
                    "Shroomer",
                    900,
                    0,
                    0,
                    2,
                    0,
                    0,
                    1,
                    2,
                    0,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "Golem", "Knocklem", "Knocklem", 150, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 4, 0,
                ),
                (
                    "PinkRabbit",
                    "Ribbonun",
                    "Ribbonun",
                    1230,
                    0,
                    0,
                    0,
                    0,
                    1,
                    1,
                    0,
                    0,
                    0,
                    0,
                    1,
                    0,
                ),
                (
                    "FlameBuffalo",
                    "Arsox",
                    "Arsox",
                    800,
                    2,
                    0,
                    0,
                    0,
                    0,
                    0,
                    2,
                    0,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "CatBat", "Tombat", "Tombat", 950, 0, 0, 0, 0, 0, 2, 0, 2, 0, 0, 2, 0,
                ),
                (
                    "Serpent", "Surfent", "Surfent", 660, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ),
                (
                    "Ronin", "Bushi", "Bushi", 680, 2, 0, 0, 0, 1, 1, 3, 0, 0, 0, 2, 0,
                ),
                (
                    "LazyCatfish",
                    "Dumud",
                    "Dumud",
                    1010,
                    0,
                    2,
                    0,
                    0,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                    1,
                    0,
                ),
                (
                    "HawkBird", "Nitewing", "Nitewing", 650, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0,
                ),
                (
                    "RobinHood",
                    "Robinquill",
                    "Robinquill",
                    810,
                    0,
                    0,
                    2,
                    0,
                    2,
                    1,
                    1,
                    0,
                    1,
                    0,
                    2,
                    0,
                ),
                (
                    "CatMage", "Katress", "Katress", 770, 0, 0, 0, 0, 2, 0, 0, 0, 2, 0, 2, 0,
                ),
                (
                    "ThunderDog",
                    "Rayhound",
                    "Rayhound",
                    740,
                    0,
                    0,
                    0,
                    2,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "NegativeKoala",
                    "Depresso",
                    "Depresso",
                    1240,
                    0,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                    1,
                    0,
                    0,
                    1,
                    0,
                ),
                (
                    "WoolFox", "Cremis", "Cremis", 1450, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1,
                ),
                (
                    "HadesBird",
                    "Helzephyr",
                    "Helzephyr",
                    600,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    3,
                    0,
                ),
                (
                    "WeaselDragon_Fire",
                    "Chillet Ignis",
                    "Chillet Ignis",
                    1010,
                    1,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "CatVampire",
                    "Felbat",
                    "Felbat",
                    720,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    3,
                    0,
                    0,
                    0,
                ),
                (
                    "KendoFrog",
                    "Ribbunny",
                    "Ribbunny",
                    1200,
                    0,
                    0,
                    0,
                    0,
                    1,
                    1,
                    0,
                    0,
                    0,
                    0,
                    1,
                    0,
                ),
                (
                    "MoonQueen",
                    "Lunaris",
                    "Lunaris",
                    830,
                    0,
                    0,
                    0,
                    0,
                    3,
                    0,
                    0,
                    0,
                    0,
                    0,
                    1,
                    0,
                ),
                (
                    "SkyDragon_Grass",
                    "Quivern Botan",
                    "Quivern Botan",
                    500,
                    0,
                    0,
                    2,
                    0,
                    0,
                    2,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "CaptainPenguin",
                    "Penking",
                    "Penking",
                    520,
                    0,
                    2,
                    0,
                    0,
                    2,
                    0,
                    0,
                    2,
                    0,
                    2,
                    2,
                    0,
                ),
                (
                    "Suzaku_Water",
                    "Suzaku Aqua",
                    "Suzaku Aqua",
                    400,
                    0,
                    3,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "NightFox", "Nox", "Nox", 1180, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
                ),
                (
                    "Ganesha", "Grintale", "Grintale", 670, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ),
                (
                    "IceFox", "Foxcicle", "Foxcicle", 910, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0,
                ),
                (
                    "DarkCrow", "Cawgnito", "Cawgnito", 970, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ),
                (
                    "MopBaby", "Swee", "Swee", 1250, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
                ),
                (
                    "DarkScorpion",
                    "Menasting",
                    "Menasting",
                    400,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    2,
                    3,
                    0,
                    0,
                    0,
                    0,
                ),
                (
                    "FlyingManta",
                    "Celeray",
                    "Celeray",
                    1020,
                    0,
                    1,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    3,
                    0,
                ),
                (
                    "WeaselDragon",
                    "Chillet",
                    "Chillet",
                    1010,
                    0,
                    0,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                ),
            ];

            for p in pals_data {
                tx.execute(
                    "INSERT OR REPLACE INTO pals VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                    rusqlite::params![p.0, p.1, p.2, p.3, p.4, p.5, p.6, p.7, p.8, p.9, p.10, p.11, p.12, p.13, p.14, p.15],
                ).ok();
            }

            // Populate Passives
            let passives_data: &[(&str, &str, &str, &str, &str)] = &[
                (
                    "Legend",
                    "Legend",
                    "Leyenda",
                    "Attack +20%, Defense +20%, Movement Speed +15%",
                    "Ataque +20%, Defensa +20%, Velocidad de movimiento +15%",
                ),
                (
                    "Rare",
                    "Lucky",
                    "Raro",
                    "Work Speed +15%, Attack +15%",
                    "Velocidad de trabajo +15%, Ataque +15%",
                ),
                (
                    "PAL_ALLAttack_up1",
                    "Vanguard",
                    "Fiero",
                    "Player Attack +10%",
                    "Ataque del jugador +10%",
                ),
                (
                    "PAL_ALLAttack_up2",
                    "Stronghold Strategist",
                    "Feroz",
                    "Player Defense +10%",
                    "Defensa del jugador +10%",
                ),
                (
                    "PAL_ALLAttack_down1",
                    "Coward",
                    "Cobarde",
                    "Attack -10%",
                    "Ataque -10%",
                ),
                (
                    "PAL_ALLAttack_down2",
                    "Pacifist",
                    "Pacifista",
                    "Attack -20%",
                    "Ataque -20%",
                ),
                (
                    "PAL_ALLDefense_up1",
                    "Hard Skin",
                    "De piel dura",
                    "Defense +10%",
                    "Defensa +10%",
                ),
                (
                    "PAL_ALLDefense_up2",
                    "Burly Body",
                    "Cuerpo fuerte",
                    "Defense +20%",
                    "Defensa +20%",
                ),
                (
                    "PAL_ALLDefense_down1",
                    "Brittle",
                    "Frágil",
                    "Defense -10%",
                    "Defensa -10%",
                ),
                (
                    "MoveSpeed_up1",
                    "Runner",
                    "Corredor",
                    "Movement Speed +10%",
                    "Velocidad de movimiento +10%",
                ),
                (
                    "MoveSpeed_up2",
                    "Swift",
                    "Veloz",
                    "Movement Speed +20%",
                    "Velocidad de movimiento +20%",
                ),
                (
                    "MoveSpeed_up3",
                    "Nimble",
                    "Ágil",
                    "Movement Speed +10%",
                    "Velocidad de movimiento +10%",
                ),
                (
                    "WorkSpeed_up1",
                    "Serious",
                    "Serio",
                    "Work Speed +20%",
                    "Velocidad de trabajo +20%",
                ),
                (
                    "WorkSpeed_up2",
                    "Artisan",
                    "Artesano",
                    "Work Speed +50%",
                    "Velocidad de trabajo +50%",
                ),
                (
                    "WorkSpeed_down1",
                    "Slacker",
                    "Vago",
                    "Work Speed -30%",
                    "Velocidad de trabajo -30%",
                ),
                (
                    "MotivationalLeader",
                    "Motivational Leader",
                    "Líder Motivacional",
                    "Player Work Speed +25%",
                    "Velocidad de trabajo del jugador +25% cuando está en el equipo",
                ),
                (
                    "ElementBoost_Fire_1_PAL",
                    "Fire Boost (10%)",
                    "Pirófilo",
                    "Fire damage +10%",
                    "Daño de tipo Fuego +10%",
                ),
                (
                    "ElementBoost_Fire_2_PAL",
                    "Flame Emperor",
                    "Señor de las Llamas",
                    "Fire damage +20%",
                    "Daño de tipo Fuego +20%",
                ),
                (
                    "ElementBoost_Ice_1_PAL",
                    "Cold Blood",
                    "Sangre fría",
                    "Ice damage +10%",
                    "Daño de tipo Hielo +10%",
                ),
                (
                    "ElementBoost_Ice_2_PAL",
                    "Ice Emperor",
                    "Señor del Hielo",
                    "Ice damage +20%",
                    "Daño de tipo Hielo +20%",
                ),
                (
                    "ElementBoost_Dragon_1_PAL",
                    "Dragon Boost (10%)",
                    "Amante de dragones",
                    "Dragon damage +10%",
                    "Daño de tipo Dragón +10%",
                ),
                (
                    "ElementBoost_Dragon_2_PAL",
                    "Divine Dragon",
                    "Dragón Divino",
                    "Dragon damage +20%",
                    "Daño de tipo Dragón +20%",
                ),
                (
                    "ElementBoost_Dark_1_PAL",
                    "Dark Boost (10%)",
                    "Amante de sombras",
                    "Dark damage +10%",
                    "Daño de tipo Oscuro +10%",
                ),
                (
                    "ElementBoost_Dark_2_PAL",
                    "Lord of the Underworld",
                    "Señor de las Sombras",
                    "Dark damage +20%",
                    "Daño de tipo Oscuro +20%",
                ),
                (
                    "ElementBoost_Electric_1_PAL",
                    "Electric Boost (10%)",
                    "Electrófilo",
                    "Electric damage +10%",
                    "Daño de tipo Eléctrico +10%",
                ),
                (
                    "ElementBoost_Electric_2_PAL",
                    "Lord of Lightning",
                    "Señor del Rayo",
                    "Electric damage +20%",
                    "Daño de tipo Eléctrico +20%",
                ),
                (
                    "ElementBoost_Earth_1_PAL",
                    "Earth Boost (10%)",
                    "Amante de tierra",
                    "Earth damage +10%",
                    "Daño de tipo Tierra +10%",
                ),
                (
                    "ElementBoost_Earth_2_PAL",
                    "Lord of Earth",
                    "Señor de la Tierra",
                    "Earth damage +20%",
                    "Daño de tipo Tierra +20%",
                ),
                (
                    "ElementBoost_Leaf_1_PAL",
                    "Grass Boost (10%)",
                    "Amante de plantas",
                    "Grass damage +10%",
                    "Daño de tipo Planta +10%",
                ),
                (
                    "ElementBoost_Leaf_2_PAL",
                    "Spirit Emperor",
                    "Señor de la Selva",
                    "Grass damage +20%",
                    "Daño de tipo Planta +20%",
                ),
                (
                    "ElementBoost_Water_1_PAL",
                    "Water Boost (10%)",
                    "Amante de agua",
                    "Water damage +10%",
                    "Daño de tipo Agua +10%",
                ),
                (
                    "ElementBoost_Water_2_PAL",
                    "Lord of the Sea",
                    "Señor del Agua",
                    "Water damage +20%",
                    "Daño de tipo Agua +20%",
                ),
                (
                    "Sanity_down1",
                    "Positive Thinking",
                    "Pensamiento Positivo",
                    "Sanity drops 10% slower",
                    "Disminución de SAN un 10% más lenta",
                ),
                (
                    "Sanity_down2",
                    "Zen Mind",
                    "Mente Zen",
                    "Sanity drops 15% slower",
                    "Disminución de SAN un 15% más lenta",
                ),
                (
                    "Sanity_up1",
                    "Unstable",
                    "Inestable",
                    "Sanity drops 10% faster",
                    "Disminución de SAN un 10% más rápida",
                ),
                (
                    "Sanity_up2",
                    "Neurotic",
                    "Neurótico",
                    "Sanity drops 15% faster",
                    "Disminución de SAN un 15% más rápida",
                ),
                (
                    "FullStomach_Up_1",
                    "Glutton",
                    "Glotón",
                    "Satiety drops 10% faster",
                    "El hambre aumenta un 10% más rápido",
                ),
                (
                    "FullStomach_Down_1",
                    "Diet Lover",
                    "Poco comedor",
                    "Satiety drops 10% slower",
                    "El hambre aumenta un 10% más lento",
                ),
                (
                    "FullStomach_Down_2",
                    "Dainty Eater",
                    "Cuerpo esbelto",
                    "Satiety drops 20% slower",
                    "El hambre aumenta un 20% más lento",
                ),
                (
                    "Stamina_Up_2",
                    "Stamina",
                    "Resistencia",
                    "Player Stamina +20%",
                    "Resistencia al Hielo o resistencia general del jugador",
                ),
                (
                    "ElementBoost_Dragon_2_PAL",
                    "Divine Dragon",
                    "Dragón Divino",
                    "Dragon damage +20%",
                    "Daño de tipo Dragón +20%",
                ),
                (
                    "ElementBoost_Aqua_2_PAL",
                    "Lord of the Sea",
                    "Señor del Agua",
                    "Water damage +20%",
                    "Daño de tipo Agua +20%",
                ),
                (
                    "Test_PalEgg_HatchingSpeed_Up",
                    "Egg Hatcher",
                    "Acelerador de huevos",
                    "Egg hatching speed +100%",
                    "Velocidad de eclosión de huevos +100%",
                ),
                (
                    "TrainerWorkSpeed_UP_1",
                    "Motivator",
                    "Líder motivador",
                    "Player work speed +25%",
                    "Velocidad de trabajo del jugador +25% cuando está en el equipo",
                ),
                (
                    "PAL_oraora",
                    "Vanguard",
                    "Vanguardia",
                    "Player attack +10%",
                    "Ataque del jugador +10% cuando está en el equipo",
                ),
                (
                    "PAL_sadist",
                    "Sadist",
                    "Sádico",
                    "Attack +15%, Defense -15%",
                    "Ataque +15%, Defensa -15%",
                ),
                (
                    "ElementResist_Ice_1_PAL",
                    "Cold Resistant",
                    "Cuerpo gélido",
                    "Ice damage resistance +10%",
                    "Resistencia al daño de tipo Hielo +10%",
                ),
                (
                    "ElementBoost_Earth_2_PAL",
                    "Lord of Earth",
                    "Señor de la Tierra",
                    "Earth damage +20%",
                    "Daño de tipo Tierra +20%",
                ),
                (
                    "PAL_FullStomach_Up_2",
                    "Glutton",
                    "Glotón",
                    "Satiety drops 15% faster",
                    "El hambre aumenta un 15% más rápido",
                ),
                (
                    "Noukin",
                    "Musclehead",
                    "Musculoso",
                    "Attack +30%, Work Speed -50%",
                    "Ataque +30%, Velocidad de trabajo -50%",
                ),
                (
                    "ElementBoost_Leaf_1_PAL",
                    "Grass Boost (10%)",
                    "Amante de plantas",
                    "Grass damage +10%",
                    "Daño de tipo Planta +10%",
                ),
                (
                    "ElementResist_Aqua_1_PAL",
                    "Water Resistant",
                    "Cuerpo acuático",
                    "Water damage resistance +10%",
                    "Resistencia al daño de tipo Agua +10%",
                ),
                (
                    "TrainerATK_UP_1",
                    "Vanguard",
                    "Vanguardia",
                    "Player attack +10%",
                    "Ataque del jugador +10% cuando está en el equipo",
                ),
                (
                    "PAL_Sanity_Down_2",
                    "Neurotic",
                    "Neurótico",
                    "Sanity drops 15% faster",
                    "Disminución de SAN un 15% más rápida",
                ),
                (
                    "CoolTimeReduction_Down_1",
                    "Clumsy",
                    "Torpe",
                    "Active skill cooldown +10%",
                    "Tiempo de enfriamiento de habilidades +10%",
                ),
                (
                    "PAL_FullStomach_Down_1",
                    "Diet Lover",
                    "Poco comedor",
                    "Satiety drops 10% slower",
                    "El hambre aumenta un 10% más lento",
                ),
            ];

            for p in passives_data {
                tx.execute(
                    "INSERT OR REPLACE INTO passives VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![p.0, p.1, p.2, p.3, p.4],
                )
                .ok();
            }

            // Populate Items
            let items_data: &[(&str, &str, &str)] = &[
                ("wood", "Wood", "Madera"),
                ("stone", "Stone", "Piedra"),
                ("berries", "Berries", "Bayas"),
                ("palsphere", "Pal Sphere", "Esfera Pal"),
                ("pal_crystal_s", "Paldium Fragment", "Fragmento de Paldio"),
                ("wool", "Wool", "Lana"),
                ("fiber", "Fiber", "Fibra"),
                ("egg", "Egg", "Huevo"),
                ("meat_sheepball", "Lamball Mutton", "Carne de Lamball"),
                ("meat_chickenpal", "Chikipi Poultry", "Carne de Chikipi"),
                ("berryseeds", "Berry Seeds", "Semillas de bayas"),
                ("copperore", "Copper Ore", "Mineral de cobre"),
                ("coal", "Coal", "Carbón"),
                ("sulfur", "Sulfur", "Azufre"),
                ("flame_organ", "Flame Organ", "Órgano de ignición"),
                ("electric_organ", "Electric Organ", "Órgano de generación"),
            ];

            for i in items_data {
                tx.execute(
                    "INSERT OR REPLACE INTO items VALUES (?1, ?2, ?3)",
                    rusqlite::params![i.0, i.1, i.2],
                )
                .ok();
            }

            // Populate Breeding Exceptions
            let exceptions_data: &[(&str, &str, &str)] = &[
                ("Mossanda", "Grizzbolt", "Trizer"),
                ("Relaxaurus", "Sparkit", "Orserk"),
                ("Helzephyr", "Frostallion", "Frostallion Noct"),
                ("Grizzbolt", "Wildiris", "Lyleen"),
                ("Lyleen", "Menasting", "Lyleen Noct"),
                ("Incineram", "Maraith", "Incineram Noct"),
                ("Pyrin", "Katress", "Pyrin Noct"),
            ];

            for e in exceptions_data {
                tx.execute(
                    "INSERT OR REPLACE INTO breeding_exceptions VALUES (?1, ?2, ?3)",
                    rusqlite::params![e.0, e.1, e.2],
                )
                .ok();
            }

            // Populate Active Skills
            let skills_data: &[(&str, &str, &str, i32, i32, &str)] = &[
                ("AirCanon", "Air Cannon", "Cañón de Aire", 25, 2, "Neutral"),
                ("HydroLaser", "Hydro Laser", "Hidroláser", 150, 55, "Water"),
                (
                    "DragonBreath",
                    "Dragon Breath",
                    "Aliento de Dragón",
                    70,
                    15,
                    "Dragon",
                ),
                (
                    "DarkLaser",
                    "Dark Laser",
                    "Láser de Sombra",
                    150,
                    55,
                    "Dark",
                ),
                (
                    "FireBlast",
                    "Fire Blast",
                    "Explosión Ígnea",
                    150,
                    55,
                    "Fire",
                ),
                ("WindCutter", "Wind Cutter", "Cortavientos", 30, 2, "Grass"),
                ("AquaGun", "Aqua Gun", "Pistola de Agua", 40, 4, "Water"),
                (
                    "ElectroBall",
                    "Electro Ball",
                    "Bola de Trueno",
                    40,
                    4,
                    "Electric",
                ),
                (
                    "SandBlast",
                    "Sand Blast",
                    "Explosión de Arena",
                    40,
                    4,
                    "Earth",
                ),
                ("IceMissile", "Ice Missile", "Misil de Hielo", 30, 3, "Ice"),
            ];

            for s in skills_data {
                tx.execute(
                    "INSERT OR REPLACE INTO active_skills VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![s.0, s.1, s.2, s.3, s.4, s.5],
                )
                .ok();
            }

            // Populate Recipes
            let recipes_data: &[(&str, &str, i32)] = &[
                ("palsphere", "wood", 3),
                ("palsphere", "stone", 3),
                ("palsphere", "pal_crystal_s", 1),
                ("palsphere_mega", "wood", 5),
                ("palsphere_mega", "stone", 5),
                ("palsphere_mega", "pal_crystal_s", 2),
                ("palsphere_giga", "wood", 10),
                ("palsphere_giga", "stone", 10),
                ("palsphere_giga", "pal_crystal_s", 3),
            ];

            for r in recipes_data {
                tx.execute(
                    "INSERT OR REPLACE INTO recipes VALUES (?1, ?2, ?3)",
                    rusqlite::params![r.0, r.1, r.2],
                )
                .ok();
            }

            // Populate Pal Drops (pal_id, item_id, chance, min_qty, max_qty)
            let drops_data: &[(&str, &str, i32, i32, i32)] = &[
                ("SheepBall", "wool", 100, 1, 2),
                ("SheepBall", "meat_sheepball", 100, 1, 1),
                ("PinkCat", "fiber", 100, 1, 2),
                ("ChickenPal", "egg", 100, 1, 1),
                ("ChickenPal", "meat_chickenpal", 100, 1, 1),
                ("Kitsunebi", "flame_organ", 100, 1, 1),
                ("ElecCat", "electric_organ", 100, 1, 1),
                ("Hedgehog", "electric_organ", 100, 1, 1),
                ("Hedgehog_Ice", "flame_organ", 0, 0, 0), // Cryst doesn't drop flame
                ("Boar", "leather", 100, 1, 1),
                ("Alpaca", "wool", 100, 2, 4),
            ];

            for d in drops_data {
                tx.execute(
                    "INSERT OR REPLACE INTO pal_drops VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![d.0, d.1, d.2, d.3, d.4],
                )
                .ok();
            }

            let json_loaded = || -> Result<(), String> {
                let json_path = db_dir.join("palworld_data.json");
                if json_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&json_path) {
                        return populate_from_json_str(&tx, &content);
                    }
                }
                Err("No local JSON cache".to_string())
            }();

            let _ = json_loaded;
            tx.commit().ok();
        }
    }
}

fn get_conn() -> Option<Connection> {
    let path = DB_PATH.get()?;
    Connection::open(path).ok()
}

fn populate_from_json_str(tx: &rusqlite::Transaction, content: &str) -> Result<(), String> {
    let data: PalworldDataJson =
        serde_json::from_str(content).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let _ = tx.execute("DELETE FROM pals", []);
    let _ = tx.execute("DELETE FROM passives", []);
    let _ = tx.execute("DELETE FROM items", []);
    let _ = tx.execute("DELETE FROM breeding_exceptions", []);
    let _ = tx.execute("DELETE FROM active_skills", []);
    let _ = tx.execute("DELETE FROM recipes", []);
    let _ = tx.execute("DELETE FROM pal_drops", []);

    for p in &data.pals {
        if let Some(arr) = p.as_array() {
            if arr.len() >= 16 {
                let _ = tx.execute(
                    "INSERT OR REPLACE INTO pals VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                    rusqlite::params![
                        arr[0].as_str().unwrap_or_default(),
                        arr[1].as_str().unwrap_or_default(),
                        arr[2].as_str().unwrap_or_default(),
                        arr[3].as_i64().unwrap_or(1500),
                        arr[4].as_i64().unwrap_or(0),
                        arr[5].as_i64().unwrap_or(0),
                        arr[6].as_i64().unwrap_or(0),
                        arr[7].as_i64().unwrap_or(0),
                        arr[8].as_i64().unwrap_or(0),
                        arr[9].as_i64().unwrap_or(0),
                        arr[10].as_i64().unwrap_or(0),
                        arr[11].as_i64().unwrap_or(0),
                        arr[12].as_i64().unwrap_or(0),
                        arr[13].as_i64().unwrap_or(0),
                        arr[14].as_i64().unwrap_or(0),
                        arr[15].as_i64().unwrap_or(0),
                    ]
                );
            }
        }
    }

    for p in &data.passives {
        if let Some(arr) = p.as_array() {
            if arr.len() >= 5 {
                let _ = tx.execute(
                    "INSERT OR REPLACE INTO passives VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![
                        arr[0].as_str().unwrap_or_default(),
                        arr[1].as_str().unwrap_or_default(),
                        arr[2].as_str().unwrap_or_default(),
                        arr[3].as_str().unwrap_or_default(),
                        arr[4].as_str().unwrap_or_default(),
                    ],
                );
            }
        }
    }

    for i in &data.items {
        if let Some(arr) = i.as_array() {
            if arr.len() >= 3 {
                let _ = tx.execute(
                    "INSERT OR REPLACE INTO items VALUES (?1, ?2, ?3)",
                    rusqlite::params![
                        arr[0].as_str().unwrap_or_default(),
                        arr[1].as_str().unwrap_or_default(),
                        arr[2].as_str().unwrap_or_default(),
                    ],
                );
            }
        }
    }

    for e in &data.breeding_exceptions {
        if let Some(arr) = e.as_array() {
            if arr.len() >= 3 {
                let _ = tx.execute(
                    "INSERT OR REPLACE INTO breeding_exceptions VALUES (?1, ?2, ?3)",
                    rusqlite::params![
                        arr[0].as_str().unwrap_or_default(),
                        arr[1].as_str().unwrap_or_default(),
                        arr[2].as_str().unwrap_or_default(),
                    ],
                );
            }
        }
    }

    for s in &data.active_skills {
        if let Some(arr) = s.as_array() {
            if arr.len() >= 6 {
                let _ = tx.execute(
                    "INSERT OR REPLACE INTO active_skills VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![
                        arr[0].as_str().unwrap_or_default(),
                        arr[1].as_str().unwrap_or_default(),
                        arr[2].as_str().unwrap_or_default(),
                        arr[3].as_i64().unwrap_or(0),
                        arr[4].as_i64().unwrap_or(0),
                        arr[5].as_str().unwrap_or_default(),
                    ],
                );
            }
        }
    }

    for r in &data.recipes {
        if let Some(arr) = r.as_array() {
            if arr.len() >= 3 {
                let _ = tx.execute(
                    "INSERT OR REPLACE INTO recipes VALUES (?1, ?2, ?3)",
                    rusqlite::params![
                        arr[0].as_str().unwrap_or_default(),
                        arr[1].as_str().unwrap_or_default(),
                        arr[2].as_i64().unwrap_or(0),
                    ],
                );
            }
        }
    }

    for d in &data.pal_drops {
        if let Some(arr) = d.as_array() {
            if arr.len() >= 5 {
                let _ = tx.execute(
                    "INSERT OR REPLACE INTO pal_drops VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![
                        arr[0].as_str().unwrap_or_default(),
                        arr[1].as_str().unwrap_or_default(),
                        arr[2].as_i64().unwrap_or(0),
                        arr[3].as_i64().unwrap_or(0),
                        arr[4].as_i64().unwrap_or(0),
                    ],
                );
            }
        }
    }

    Ok(())
}

pub fn run_update_db_command(is_json: bool) {
    let args: Vec<String> = std::env::args().collect();
    let mut local_path = None;
    for (idx, arg) in args.iter().enumerate() {
        if (arg == "--update-db" || arg == "--datamining") && idx + 1 < args.len() {
            let next_arg = &args[idx + 1];
            if !next_arg.starts_with('-') {
                local_path = Some(next_arg.clone());
            }
        }
    }

    let home_dir = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| "C:\\".to_string());
    let config_dir = std::path::Path::new(&home_dir).join(".palagent-ai");
    let json_path = config_dir.join("palworld_data.json");

    // Case A: Explicit local file path provided
    if let Some(path_str) = local_path {
        if !is_json {
            println!("Updating database from local file: {}...", path_str);
        }
        if let Ok(content) = std::fs::read_to_string(&path_str) {
            if let Some(mut conn) = get_conn() {
                if let Ok(tx) = conn.transaction() {
                    if populate_from_json_str(&tx, &content).is_ok() && tx.commit().is_ok() {
                        let _ = std::fs::write(&json_path, &content);
                        if is_json {
                            println!(
                                    "{}",
                                    serde_json::json!({ "status": "success", "message": format!("Database updated successfully from local file: {}", path_str) })
                                );
                        } else {
                            println!(
                                    "Database updated successfully from local file: {}!",
                                    path_str
                                );
                        }
                        return;
                    }
                }
            }
        }
        if is_json {
            println!(
                "{}",
                serde_json::json!({ "status": "error", "message": format!("Failed to read local file: {}", path_str) })
            );
        } else {
            println!("Error: Failed to read local file: {}", path_str);
        }
        return;
    }

    // Case B: Search for local fallback files in current directory or data/
    let mut local_fallback = None;
    if std::path::Path::new("data/palworld_data.json").exists() {
        local_fallback = Some("data/palworld_data.json".to_string());
    } else if std::path::Path::new("palworld_data.json").exists() {
        local_fallback = Some("palworld_data.json".to_string());
    }

    if let Some(path_str) = local_fallback {
        if !is_json {
            println!("Updating database from local fallback: {}...", path_str);
        }
        if let Ok(content) = std::fs::read_to_string(&path_str) {
            if let Some(mut conn) = get_conn() {
                if let Ok(tx) = conn.transaction() {
                    if populate_from_json_str(&tx, &content).is_ok() && tx.commit().is_ok() {
                        let _ = std::fs::write(&json_path, &content);
                        if is_json {
                            println!(
                                    "{}",
                                    serde_json::json!({ "status": "success", "message": format!("Database updated successfully from local fallback: {}", path_str) })
                                );
                        } else {
                            println!(
                                    "Database updated successfully from local fallback: {}!",
                                    path_str
                                );
                        }
                        return;
                    }
                }
            }
        }
    }

    // Case C: Remote fallback download
    if !is_json {
        println!("Updating PalAgent AI metadata database from remote datamining repository...");
    }

    let mut content = String::new();
    let mut success = false;

    #[cfg(target_os = "windows")]
    let curl_cmd = "curl.exe";
    #[cfg(not(target_os = "windows"))]
    let curl_cmd = "curl";

    if let Ok(output) = std::process::Command::new(curl_cmd)
        .arg("-s")
        .arg("-L")
        .arg("https://raw.githubusercontent.com/sewandev/palagent-ai/main/data/palworld_data.json")
        .output()
    {
        if output.status.success() {
            if let Ok(s) = String::from_utf8(output.stdout) {
                if s.contains("\"pals\"") {
                    content = s;
                    success = true;
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    if !success {
        if let Ok(output) = std::process::Command::new("powershell")
            .arg("-Command")
            .arg("[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.SecurityProtocolType]::Tls12; Invoke-RestMethod -Uri 'https://raw.githubusercontent.com/sewandev/palagent-ai/main/data/palworld_data.json'")
            .output()
        {
            if output.status.success() {
                if let Ok(s) = String::from_utf8(output.stdout) {
                    if s.contains("\"pals\"") {
                        content = s;
                        success = true;
                    }
                }
            }
        }
    }

    if success {
        let _ = std::fs::write(&json_path, &content);
        if let Some(mut conn) = get_conn() {
            if let Ok(tx) = conn.transaction() {
                if populate_from_json_str(&tx, &content).is_ok() && tx.commit().is_ok() {
                    if is_json {
                        println!("{}", serde_json::json!({ "status": "success", "message": "Database updated successfully from datamine sources." }));
                    } else {
                        println!("Database updated successfully from datamine sources!");
                    }
                    return;
                }
            }
        }
    }

    if is_json {
        println!(
            "{}",
            serde_json::json!({ "status": "error", "message": "Failed to update database. Verify your internet connection or supply a local file path." })
        );
    } else {
        println!("Error: Failed to update database. Please check your internet connection or supply a local JSON file path: palagent-ai.exe --update-db [path_to_json]");
    }
}

pub fn translate_pal(internal_id: &str, use_es: bool) -> String {
    let conn = match get_conn() {
        Some(c) => c,
        None => return internal_id.to_string(),
    };

    let query = if use_es {
        "SELECT name_es FROM pals WHERE internal_id = ?1"
    } else {
        "SELECT name_en FROM pals WHERE internal_id = ?1"
    };

    if let Ok(name) = conn.query_row(query, [internal_id], |row| row.get::<_, String>(0)) {
        name
    } else {
        if let Some(base_id) = internal_id.strip_prefix("BOSS_") {
            let base_name = translate_pal(base_id, use_es);
            if base_name != base_id {
                return if use_es {
                    format!("{} (Alfa)", base_name)
                } else {
                    format!("{} (Alpha)", base_name)
                };
            }
        }
        internal_id.to_string()
    }
}

pub fn translate_passive(internal_id: &str, use_es: bool) -> (String, String) {
    let conn = match get_conn() {
        Some(c) => c,
        None => return (internal_id.to_string(), String::new()),
    };

    let query = if use_es {
        "SELECT name_es, description_es FROM passives WHERE internal_id = ?1"
    } else {
        "SELECT name_en, description_en FROM passives WHERE internal_id = ?1"
    };

    if let Ok(res) = conn.query_row(query, [internal_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }) {
        res
    } else {
        (internal_id.to_string(), String::new())
    }
}

pub fn translate_item(internal_id: &str, use_es: bool) -> String {
    let conn = match get_conn() {
        Some(c) => c,
        None => return internal_id.to_string(),
    };

    let query = if use_es {
        "SELECT name_es FROM items WHERE internal_id = ?1"
    } else {
        "SELECT name_en FROM items WHERE internal_id = ?1"
    };

    if let Ok(name) = conn.query_row(query, [internal_id], |row| row.get::<_, String>(0)) {
        name
    } else {
        internal_id.to_string()
    }
}

pub fn get_pal_suitabilities(internal_id: &str) -> std::collections::HashMap<String, i32> {
    let mut suitabilities = std::collections::HashMap::new();
    let conn = match get_conn() {
        Some(c) => c,
        None => return suitabilities,
    };

    let query = "SELECT kindling, watering, planting, generating, handwork, gathering, lumbering, mining, medicine, cooling, transporting, farming FROM pals WHERE internal_id = ?1";
    if let Ok((
        kindling,
        watering,
        planting,
        generating,
        handwork,
        gathering,
        lumbering,
        mining,
        medicine,
        cooling,
        transporting,
        farming,
    )) = conn.query_row(query, [internal_id], |row| {
        Ok((
            row.get::<_, i32>(0)?,
            row.get::<_, i32>(1)?,
            row.get::<_, i32>(2)?,
            row.get::<_, i32>(3)?,
            row.get::<_, i32>(4)?,
            row.get::<_, i32>(5)?,
            row.get::<_, i32>(6)?,
            row.get::<_, i32>(7)?,
            row.get::<_, i32>(8)?,
            row.get::<_, i32>(9)?,
            row.get::<_, i32>(10)?,
            row.get::<_, i32>(11)?,
        ))
    }) {
        if kindling > 0 {
            suitabilities.insert("Kindling".to_string(), kindling);
        }
        if watering > 0 {
            suitabilities.insert("Watering".to_string(), watering);
        }
        if planting > 0 {
            suitabilities.insert("Planting".to_string(), planting);
        }
        if generating > 0 {
            suitabilities.insert("Electricity".to_string(), generating);
        }
        if handwork > 0 {
            suitabilities.insert("Handwork".to_string(), handwork);
        }
        if gathering > 0 {
            suitabilities.insert("Gathering".to_string(), gathering);
        }
        if lumbering > 0 {
            suitabilities.insert("Lumbering".to_string(), lumbering);
        }
        if mining > 0 {
            suitabilities.insert("Mining".to_string(), mining);
        }
        if medicine > 0 {
            suitabilities.insert("Medicine".to_string(), medicine);
        }
        if cooling > 0 {
            suitabilities.insert("Cooling".to_string(), cooling);
        }
        if transporting > 0 {
            suitabilities.insert("Transporting".to_string(), transporting);
        }
        if farming > 0 {
            suitabilities.insert("Farming".to_string(), farming);
        }
    }
    suitabilities
}

pub fn check_breeding_exception(parent_a: &str, parent_b: &str) -> Option<String> {
    let conn = get_conn()?;
    let query = "SELECT child FROM breeding_exceptions WHERE (parent_a = ?1 AND parent_b = ?2) OR (parent_a = ?2 AND parent_b = ?1)";
    conn.query_row(query, [parent_a, parent_b], |row| row.get::<_, String>(0))
        .ok()
}

#[allow(dead_code)]
pub fn get_pal_breed_power(internal_id: &str) -> i32 {
    let conn = match get_conn() {
        Some(c) => c,
        None => return 1500,
    };

    let query = "SELECT breed_power FROM pals WHERE internal_id = ?1";
    conn.query_row(query, [internal_id], |row| row.get::<_, i32>(0))
        .unwrap_or(1500)
}

pub fn translate_active_skill(
    internal_id: &str,
    use_es: bool,
) -> Option<(String, i32, i32, String)> {
    let conn = get_conn()?;
    let query = if use_es {
        "SELECT name_es, power, cooldown, element FROM active_skills WHERE internal_id = ?1"
    } else {
        "SELECT name_en, power, cooldown, element FROM active_skills WHERE internal_id = ?1"
    };

    conn.query_row(query, [internal_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i32>(1)?,
            row.get::<_, i32>(2)?,
            row.get::<_, String>(3)?,
        ))
    })
    .ok()
}

pub fn get_recipe(item_id: &str) -> Vec<(String, i32)> {
    let mut ingredients = Vec::new();
    let conn = match get_conn() {
        Some(c) => c,
        None => return ingredients,
    };

    let query = "SELECT ingredient_id, count FROM recipes WHERE item_id = ?1";
    if let Ok(mut stmt) = conn.prepare(query) {
        if let Ok(mut rows) = stmt.query([item_id]) {
            while let Ok(Some(row)) = rows.next() {
                if let (Ok(ing), Ok(cnt)) = (row.get::<_, String>(0), row.get::<_, i32>(1)) {
                    ingredients.push((ing, cnt));
                }
            }
        }
    }
    ingredients
}

pub fn get_breed_power_by_name(name: &str) -> i32 {
    let conn = match get_conn() {
        Some(c) => c,
        None => return 1500,
    };

    let query =
        "SELECT breed_power FROM pals WHERE internal_id = ?1 OR name_en = ?1 OR name_es = ?1";
    conn.query_row(query, [name], |row| row.get::<_, i32>(0))
        .unwrap_or(1500)
}

pub fn find_closest_pal_by_breed_power(target_power: i32) -> String {
    let conn = match get_conn() {
        Some(c) => c,
        None => return "SheepBall".to_string(),
    };

    let query = "SELECT internal_id FROM pals WHERE internal_id NOT LIKE 'BOSS_%' ORDER BY abs(breed_power - ?1) ASC, internal_id ASC LIMIT 1";
    conn.query_row(query, [target_power], |row| row.get::<_, String>(0))
        .unwrap_or_else(|_| "SheepBall".to_string())
}

pub fn is_valid_pal(internal_id: &str) -> bool {
    let conn = match get_conn() {
        Some(c) => c,
        None => return false,
    };

    let base_id = internal_id.strip_prefix("BOSS_").unwrap_or(internal_id);

    let query = "SELECT 1 FROM pals WHERE internal_id = ?1 OR name_en = ?1 OR name_es = ?1";
    conn.query_row(query, [base_id], |_| Ok(())).is_ok()
}

// 1. Finding breeding combinations for a target child Pal
pub fn find_breeding_parents_for_target(child_name: &str) -> Vec<(String, String)> {
    let mut parents = Vec::new();
    let conn = match get_conn() {
        Some(c) => c,
        None => return parents,
    };

    // Resolve child internal ID or name
    let child_id_query =
        "SELECT internal_id FROM pals WHERE internal_id = ?1 OR name_en = ?1 OR name_es = ?1";
    let target_child_id: String =
        match conn.query_row(child_id_query, [child_name], |row| row.get(0)) {
            Ok(id) => id,
            Err(_) => return parents,
        };

    // Load all non-boss Pals for combinations
    let mut stmt = match conn
        .prepare("SELECT internal_id, breed_power FROM pals WHERE internal_id NOT LIKE 'BOSS_%'")
    {
        Ok(s) => s,
        Err(_) => return parents,
    };

    let pals_list: Vec<(String, i32)> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    // Check exceptions first
    let mut exc_stmt = conn.prepare("SELECT parent_a, parent_b FROM breeding_exceptions WHERE child = ?1 OR child = (SELECT name_en FROM pals WHERE internal_id = ?1)").unwrap();
    if let Ok(mut rows) = exc_stmt.query([&target_child_id]) {
        while let Ok(Some(row)) = rows.next() {
            if let (Ok(pa), Ok(pb)) = (row.get::<_, String>(0), row.get::<_, String>(1)) {
                parents.push((pa, pb));
            }
        }
    }

    // Normal combinations via breed power
    for i in 0..pals_list.len() {
        for j in i..pals_list.len() {
            let pa = &pals_list[i];
            let pb = &pals_list[j];
            let avg_power = (pa.1 + pb.1 + 1) / 2;
            let resulting_child = find_closest_pal_by_breed_power(avg_power);
            if resulting_child == target_child_id {
                parents.push((pa.0.clone(), pb.0.clone()));
            }
        }
    }

    parents
}

// 2. Query drops of a Pal
pub fn get_pal_drops(pal_name: &str) -> Vec<(String, i32, i32, i32)> {
    let mut drops = Vec::new();
    let conn = match get_conn() {
        Some(c) => c,
        None => return drops,
    };

    let pal_id_query =
        "SELECT internal_id FROM pals WHERE internal_id = ?1 OR name_en = ?1 OR name_es = ?1";
    let target_pal_id: String = match conn.query_row(pal_id_query, [pal_name], |row| row.get(0)) {
        Ok(id) => id,
        Err(_) => return drops,
    };

    let mut stmt = conn
        .prepare("SELECT item_id, chance, min_qty, max_qty FROM pal_drops WHERE pal_id = ?1")
        .unwrap();
    if let Ok(mut rows) = stmt.query([&target_pal_id]) {
        while let Ok(Some(row)) = rows.next() {
            if let (Ok(item), Ok(chance), Ok(min_q), Ok(max_q)) = (
                row.get::<_, String>(0),
                row.get::<_, i32>(1),
                row.get::<_, i32>(2),
                row.get::<_, i32>(3),
            ) {
                drops.push((item, chance, min_q, max_q));
            }
        }
    }
    drops
}

// 3. Query Pals dropping a specific item
pub fn get_pals_dropping_item(item_name: &str) -> Vec<String> {
    let mut pals = Vec::new();
    let conn = match get_conn() {
        Some(c) => c,
        None => return pals,
    };

    let item_id_query =
        "SELECT internal_id FROM items WHERE internal_id = ?1 OR name_en = ?1 OR name_es = ?1";
    let target_item_id: String = match conn.query_row(item_id_query, [item_name], |row| row.get(0))
    {
        Ok(id) => id,
        Err(_) => return pals,
    };

    let mut stmt = conn
        .prepare("SELECT pal_id FROM pal_drops WHERE item_id = ?1")
        .unwrap();
    if let Ok(mut rows) = stmt.query([&target_item_id]) {
        while let Ok(Some(row)) = rows.next() {
            if let Ok(pal) = row.get::<_, String>(0) {
                pals.push(pal);
            }
        }
    }
    pals
}

pub fn calculate_capture_rate(
    pal_level: i32,
    current_hp: i32,
    max_hp: i32,
    sphere_type: &str,
    lifmunk_level: i32,
) -> f64 {
    let sphere_multiplier = match sphere_type.to_ascii_lowercase().as_str() {
        "palsphere" | "common" | "comun" => 1.0,
        "palsphere_mega" | "mega" => 2.0,
        "palsphere_giga" | "giga" => 3.0,
        "palsphere_tera" | "tera" => 5.0,
        "palsphere_ultra" | "ultra" => 7.0,
        "palsphere_legendary" | "legendary" | "legendaria" => 12.0,
        _ => 1.0,
    };

    let base_rate = (100.0 - (pal_level as f64 * 2.2)).max(4.0);

    let hp_ratio = if max_hp > 0 {
        current_hp as f64 / max_hp as f64
    } else {
        1.0
    };
    let hp_multiplier = 1.0 + (1.0 - hp_ratio) * 2.5;

    let lifmunk_multiplier = 1.0 + (lifmunk_level as f64 * 0.10);

    let final_rate = base_rate * hp_multiplier * sphere_multiplier * lifmunk_multiplier;
    final_rate.min(100.0)
}

pub fn get_db_schema_summary(use_es: bool) -> String {
    if use_es {
        "Base de datos SQLite local (palworld_data.db):\n\
         Tablas estáticas:\n\
         - pals (pals registrados y su poder de crianza/aptitudes):\n\
           internal_id (TEXT PRIMARY KEY), name_en (TEXT), name_es (TEXT), breed_power (INTEGER), kindling..farming (INTEGER)\n\
         - passives (habilidades pasivas):\n\
           internal_id (TEXT PRIMARY KEY), name_en (TEXT), name_es (TEXT), description_en (TEXT), description_es (TEXT)\n\
         - items (objetos y materiales):\n\
           internal_id (TEXT PRIMARY KEY), name_en (TEXT), name_es (TEXT)\n\
         - breeding_exceptions (excepciones de crianza especiales):\n\
           parent_a (TEXT), parent_b (TEXT), child (TEXT), PRIMARY KEY (parent_a, parent_b)\n\
         - active_skills (habilidades de combate):\n\
           internal_id (TEXT PRIMARY KEY), name_en (TEXT), name_es (TEXT), power (INTEGER), cooldown (INTEGER), element (TEXT)\n\
         - recipes (recetas de crafteo):\n\
           item_id (TEXT), ingredient_id (TEXT), count (INTEGER), PRIMARY KEY (item_id, ingredient_id)\n\
         - pal_drops (objetos que sueltan los Pals):\n\
           pal_id (TEXT), item_id (TEXT), chance (INTEGER), min_qty (INTEGER), max_qty (INTEGER), PRIMARY KEY (pal_id, item_id)\n\n\
         Nota: Toda la información de la partida del usuario (Pals en posesión, coordenadas de cofres, salud de campamentos) se analiza y procesa en caliente en memoria desde el archivo Level.sav directamente, por lo que no reside de forma persistente en SQLite.".to_string()
    } else {
        "Local SQLite telemetry DB (palworld_data.db):\n\
         Static Tables:\n\
         - pals (registered Pals, breeding power, and work suitabilities):\n\
           internal_id (TEXT PRIMARY KEY), name_en (TEXT), name_es (TEXT), breed_power (INTEGER), kindling..farming (INTEGER)\n\
         - passives (passive skills):\n\
           internal_id (TEXT PRIMARY KEY), name_en (TEXT), name_es (TEXT), description_en (TEXT), description_es (TEXT)\n\
         - items (in-game items and materials):\n\
           internal_id (TEXT PRIMARY KEY), name_en (TEXT), name_es (TEXT)\n\
         - breeding_exceptions (special unique breeding combos):\n\
           parent_a (TEXT), parent_b (TEXT), child (TEXT), PRIMARY KEY (parent_a, parent_b)\n\
         - active_skills (combat active skills):\n\
           internal_id (TEXT PRIMARY KEY), name_en (TEXT), name_es (TEXT), power (INTEGER), cooldown (INTEGER), element (TEXT)\n\
         - recipes (crafting recipes ingredients):\n\
           item_id (TEXT), ingredient_id (TEXT), count (INTEGER), PRIMARY KEY (item_id, ingredient_id)\n\
         - pal_drops (items dropped by wild Pals):\n\
           pal_id (TEXT), item_id (TEXT), chance (INTEGER), min_qty (INTEGER), max_qty (INTEGER), PRIMARY KEY (pal_id, item_id)\n\n\
         Note: All user live save data (Pals in possession, base coordinates, chest contents) is processed dynamically in memory directly from Level.sav, and is not stored persistently in the SQLite DB.".to_string()
    }
}
