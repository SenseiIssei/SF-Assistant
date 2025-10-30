use iced::Theme;
use num_format::CustomFormat;
use serde::{Deserialize, Serialize};
use sf_api::session::PWHash;

use crate::{ServerID, server::ServerIdent};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub accounts: Vec<AccountConfig>,
    pub theme: AvailableTheme,
    pub base_name: String,
    pub auto_fetch_newest: bool,
    #[serde(default)]
    pub auto_poll: bool,
    #[serde(default = "default_ui_refresh_ms")]
    pub ui_refresh_ms: u64,
    #[serde(default = "default_threads")]
    pub max_threads: usize,
    #[serde(default = "default_start_threads")]
    pub start_threads: usize,
    #[serde(default)]
    pub show_crawling_restrict: bool,
    #[serde(default = "default_class_icons")]
    pub show_class_icons: bool,
    #[serde(default = "default_blacklist_threshhold")]
    pub blacklist_threshold: usize,

    #[serde(default = "default_locale", skip)]
    pub num_format: CustomFormat,
}

fn default_threads() -> usize {
    10
}

fn default_start_threads() -> usize {
    1
}

fn default_ui_refresh_ms() -> u64 {
    1000
}

fn default_locale() -> CustomFormat {
    let mut cfb = CustomFormat::builder();
    cfb = cfb.separator(",");
    cfb.build().unwrap_or_default()
}

fn default_blacklist_threshhold() -> usize {
    2
}

fn default_class_icons() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        let mut rng = fastrand::Rng::new();
        let mut base_name = rng.alphabetic().to_ascii_uppercase().to_string();
        for _ in 0..rng.u32(6..8) {
            let c = if rng.bool() {
                rng.alphabetic()
            } else {
                rng.digit(10)
            };
            base_name.push(c)
        }

        Self {
            accounts: vec![],
            // Default to a blue/grey palette similar to the old look
            theme: AvailableTheme::Nord,
            base_name,
            auto_fetch_newest: true,
            ui_refresh_ms: default_ui_refresh_ms(),
            max_threads: default_threads(),
            auto_poll: false,
            show_crawling_restrict: false,
            show_class_icons: true,
            blacklist_threshold: default_blacklist_threshhold(),
            num_format: default_locale(),
            start_threads: default_start_threads(),
        }
    }
}

impl Config {
    pub fn get_sso_accounts_mut(
        &mut self,
        name: &str,
    ) -> Option<&mut Vec<SFAccCharacter>> {
        let lower_name = name.to_lowercase();
        self.accounts
            .iter_mut()
            .flat_map(|a| match a {
                AccountConfig::SF {
                    name, characters, ..
                } if name.to_lowercase().trim() == lower_name.trim() => {
                    Some(characters)
                }
                _ => None,
            })
            .next()
    }

    pub fn get_char_conf(
        &self,
        name: &str,
        og_server: ServerID,
    ) -> Option<&CharacterConfig> {
        let mut res = None;

        let lower_name = name.to_lowercase();
        for acc in &self.accounts {
            match acc {
                AccountConfig::Regular {
                    name,
                    server,
                    config,
                    ..
                } => {
                    if ServerIdent::new(server).id != og_server {
                        continue;
                    }
                    if name.to_lowercase().trim() != lower_name.trim() {
                        continue;
                    }
                    res = Some(config);
                    break;
                }
                AccountConfig::SF { characters, .. } => {
                    for c in characters {
                        if ServerIdent::new(&c.ident.server).id != og_server {
                            continue;
                        }
                        if c.ident.name.to_lowercase().trim()
                            != lower_name.trim()
                        {
                            continue;
                        }
                        res = Some(&c.config);
                    }
                }
            }
        }
        res
    }

    pub fn get_char_conf_mut(
        &mut self,
        name: &str,
        og_server: ServerID,
    ) -> Option<&mut CharacterConfig> {
        let mut res = None;

        let lower_name = name.to_lowercase();
        for acc in &mut self.accounts {
            match acc {
                AccountConfig::Regular {
                    name,
                    server,
                    config,
                    ..
                } => {
                    if ServerIdent::new(server).id != og_server {
                        continue;
                    }
                    if name.to_lowercase().trim() != lower_name.trim() {
                        continue;
                    }
                    res = Some(config);
                    break;
                }
                AccountConfig::SF { characters, .. } => {
                    for c in characters {
                        if ServerIdent::new(&c.ident.server).id != og_server {
                            continue;
                        }
                        if c.ident.name.to_lowercase().trim()
                            != lower_name.trim()
                        {
                            continue;
                        }
                        res = Some(&mut c.config);
                    }
                }
            }
        }
        res
    }

    pub fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        let str = toml::to_string_pretty(self)?;
        std::fs::write("helper.toml", str)?;
        Ok(())
    }
    pub fn restore() -> Result<Self, Box<dyn std::error::Error>> {
        let val = std::fs::read_to_string("helper.toml")?;
        Ok(toml::from_str(&val)?)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum AccountCreds {
    Regular {
        name: String,
        pw_hash: PWHash,
        server: String,
    },
    SF {
        name: String,
        pw_hash: PWHash,
    },
}

impl From<AccountConfig> for AccountCreds {
    fn from(value: AccountConfig) -> Self {
        match value {
            AccountConfig::Regular {
                name,
                pw_hash,
                server,
                ..
            } => AccountCreds::Regular {
                name,
                pw_hash,
                server,
            },
            AccountConfig::SF { name, pw_hash, .. } => {
                AccountCreds::SF { name, pw_hash }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum AccountConfig {
    Regular {
        name: String,
        pw_hash: PWHash,
        server: String,
        #[serde(flatten)]
        config: CharacterConfig,
    },
    SF {
        name: String,
        pw_hash: PWHash,
        #[serde(default)]
        characters: Vec<SFAccCharacter>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SFAccCharacter {
    pub ident: SFCharIdent,
    pub config: CharacterConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MissionStrategy {
    Shortest,
    MostGold,
    BestGoldPerMinute,
    BestXpPerMinute,
    Smartest,
}

impl std::fmt::Display for MissionStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MissionStrategy::Shortest => "Shortest",
            MissionStrategy::MostGold => "MostGold",
            MissionStrategy::BestGoldPerMinute => "BestGoldPerMinute",
            MissionStrategy::BestXpPerMinute => "BestXpPerMinute",
            MissionStrategy::Smartest => "Smartest",
        };
        write!(f, "{}", s)
    }
}

fn default_strategy() -> MissionStrategy { MissionStrategy::Smartest }

impl Default for MissionStrategy {
    fn default() -> Self {
        MissionStrategy::Smartest
    }
}

fn default_true() -> bool { true }

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CharacterConfig {
    #[serde(default)]
    pub login: bool,
    #[serde(default)]
    pub auto_battle: bool,
    #[serde(default)]
    pub auto_lure: bool,

    #[serde(default)]
    pub auto_tavern: bool,
    #[serde(default)]
    pub auto_expeditions: bool,
    #[serde(default)]
    pub auto_dungeons: bool,
    #[serde(default)]
    pub auto_pets: bool,
    #[serde(default)]
    pub auto_guild: bool,
    // Guild sub-options
    #[serde(default = "default_true")]
    pub auto_guild_accept_defense: bool,
    #[serde(default = "default_true")]
    pub auto_guild_accept_attack: bool,
    #[serde(default = "default_true")]
    pub auto_guild_hydra: bool,

    #[serde(default = "default_strategy")]
    pub mission_strategy: MissionStrategy,
    #[serde(default)]
    pub reserve_mushrooms: u32,

    // Tavern options
    #[serde(default)]
    pub auto_buy_beer_mushrooms: bool,
    // Use quicksand glasses to finish quests early
    #[serde(default)]
    pub use_glasses_for_tavern: bool,

    // Mushroom budgets (per session/day) for specific actions
    // 0 = don't spend any by default
    #[serde(default)]
    pub max_mushrooms_beer: u32,
    #[serde(default)]
    pub max_mushrooms_dungeon_skip: u32,
    #[serde(default)]
    pub max_mushrooms_pet_skip: u32,

    // Expeditions
    #[serde(default)]
    pub use_glasses_for_expeditions: bool,
    #[serde(default = "default_expedition_reward_priority")]
    pub expedition_reward_priority: ExpeditionRewardPriority,
}

fn default_expedition_reward_priority() -> ExpeditionRewardPriority {
    ExpeditionRewardPriority::MushroomsGoldEggs
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ExpeditionRewardPriority {
    // Mushrooms > Gold/Silver > Pet egg > other
    MushroomsGoldEggs,
    // Gold/Silver > Mushrooms > Pet egg > other
    GoldMushroomsEggs,
    // Pet egg > Mushrooms > Gold/Silver > other
    EggsMushroomsGold,
}

impl Default for ExpeditionRewardPriority {
    fn default() -> Self { ExpeditionRewardPriority::MushroomsGoldEggs }
}

impl std::fmt::Display for ExpeditionRewardPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ExpeditionRewardPriority::MushroomsGoldEggs => "Mushrooms > Gold > Eggs",
            ExpeditionRewardPriority::GoldMushroomsEggs => "Gold > Mushrooms > Eggs",
            ExpeditionRewardPriority::EggsMushroomsGold => "Pet eggs > Mushrooms > Gold",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub struct SFCharIdent {
    pub name: String,
    pub server: String,
}

impl AccountConfig {
    pub fn new(creds: AccountCreds) -> AccountConfig {
        match creds {
            AccountCreds::Regular {
                name,
                pw_hash,
                server,
            } => AccountConfig::Regular {
                name,
                pw_hash,
                server,
                config: Default::default(),
            },
            AccountCreds::SF { name, pw_hash } => AccountConfig::SF {
                name,
                pw_hash,
                characters: Default::default(),
            },
        }
    }
}

#[derive(
    Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq,
)]
pub enum AvailableTheme {
    Light,
    #[default]
    Dark,
    Dracula,
    Nord,
    SolarizedLight,
    SolarizedDark,
    GruvboxLight,
    GruvboxDark,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightLight,
    KanagawaWave,
    KanagawaDragon,
    KanagawaLotus,
    Moonfly,
    Nightfly,
    Oxocarbon,
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for AvailableTheme {
    fn to_string(&self) -> String {
        use AvailableTheme::*;
        match self {
            Light => Theme::Light,
            Dark => Theme::Dark,
            Dracula => Theme::Dracula,
            Nord => Theme::Nord,
            SolarizedLight => Theme::SolarizedLight,
            SolarizedDark => Theme::SolarizedDark,
            GruvboxLight => Theme::GruvboxLight,
            GruvboxDark => Theme::GruvboxDark,
            CatppuccinLatte => Theme::CatppuccinLatte,
            CatppuccinFrappe => Theme::CatppuccinFrappe,
            CatppuccinMacchiato => Theme::CatppuccinMacchiato,
            CatppuccinMocha => Theme::CatppuccinMocha,
            TokyoNight => Theme::TokyoNight,
            TokyoNightStorm => Theme::TokyoNightStorm,
            TokyoNightLight => Theme::TokyoNightLight,
            KanagawaWave => Theme::KanagawaWave,
            KanagawaDragon => Theme::KanagawaDragon,
            KanagawaLotus => Theme::KanagawaLotus,
            Moonfly => Theme::Moonfly,
            Nightfly => Theme::Nightfly,
            Oxocarbon => Theme::Oxocarbon,
        }
        .to_string()
    }
}

impl AvailableTheme {
    pub fn theme(&self) -> Theme {
        use AvailableTheme::*;

        match self {
            Light => Theme::Light,
            Dark => Theme::Dark,
            Dracula => Theme::Dracula,
            Nord => Theme::Nord,
            SolarizedLight => Theme::SolarizedLight,
            SolarizedDark => Theme::SolarizedDark,
            GruvboxLight => Theme::GruvboxLight,
            GruvboxDark => Theme::GruvboxDark,
            CatppuccinLatte => Theme::CatppuccinLatte,
            CatppuccinFrappe => Theme::CatppuccinFrappe,
            CatppuccinMacchiato => Theme::CatppuccinMacchiato,
            CatppuccinMocha => Theme::CatppuccinMocha,
            TokyoNight => Theme::TokyoNight,
            TokyoNightStorm => Theme::TokyoNightStorm,
            TokyoNightLight => Theme::TokyoNightLight,
            KanagawaWave => Theme::KanagawaWave,
            KanagawaDragon => Theme::KanagawaDragon,
            KanagawaLotus => Theme::KanagawaLotus,
            Moonfly => Theme::Moonfly,
            Nightfly => Theme::Nightfly,
            Oxocarbon => Theme::Oxocarbon,
        }
    }
}