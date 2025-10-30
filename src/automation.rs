use std::time::Duration;

use chrono::Local;
use tokio::time::sleep;

use crate::{
    config::{CharacterConfig, MissionStrategy},
};

/// Adapter to pass per-character config into automation without borrowing issues.
#[derive(Debug, Clone)]
pub struct AutomationCfg {
    pub auto_tavern: bool,
    pub auto_expeditions: bool,
    pub auto_dungeons: bool,
    pub auto_pets: bool,
    pub mission_strategy: MissionStrategy,
}

impl From<&CharacterConfig> for AutomationCfg {
    fn from(c: &CharacterConfig) -> Self {
        Self {
            auto_tavern: c.auto_tavern,
            auto_expeditions: c.auto_expeditions,
            auto_dungeons: c.auto_dungeons,
            auto_pets: c.auto_pets,
            mission_strategy: c.mission_strategy,
        }
    }
}

pub struct TickOutcome {
    pub did_something: bool,
    pub summary: String,
}

/// These mirror the data you should get back from your session listing endpoints.
#[derive(Debug, Clone)]
pub struct Quest {
    pub id: u32,
    pub minutes: u32,
    pub gold: u64,
    pub xp: u64,
    pub mushrooms: u8,
}
#[derive(Debug, Clone)]
pub struct Expedition {
    pub id: u32,
    pub minutes: u32,
    pub gold: u64,
    pub xp: u64,
    pub mushrooms: u8,
}

/// Runs a single automation tick for a character.
/// Expected available calls on your session:
/// - collect_tavern_reward, list_tavern, start_tavern
/// - collect_expedition_reward, list_expeditions, start_expedition
/// - next_dungeon_candidate, fight_dungeon
/// - next_pet_candidate, fight_pet
/// - refresh_gamestate
pub async fn tick<S, GS>(
    cfg: &AutomationCfg,
    session: &mut S,
    gs: &mut GS,
) -> TickOutcome
where
    S: SessionLike,
    GS: GameStateLike,
{
    let mut did = false;
    let mut parts: Vec<String> = Vec::new();

    if cfg.auto_tavern {
        if let Some(s) = tavern_tick(cfg, session, gs).await { did = true; parts.push(s); }
    }
    if cfg.auto_expeditions {
        if let Some(s) = expedition_tick(cfg, session, gs).await { did = true; parts.push(s); }
    }
    if cfg.auto_dungeons {
        if let Some(s) = dungeon_tick(session, gs).await { did = true; parts.push(s); }
    }
    if cfg.auto_pets {
        if let Some(s) = pets_tick(session, gs).await { did = true; parts.push(s); }
    }

    TickOutcome { did_something: did, summary: parts.join(" | ") }
}

async fn tavern_tick<S, GS>(
    cfg: &AutomationCfg,
    session: &mut S,
    gs: &mut GS,
) -> Option<String>
where
    S: SessionLike,
    GS: GameStateLike,
{
    if gs.tavern_end_time().map(|t| t <= Local::now()).unwrap_or(false) {
        let _ = session.collect_tavern_reward().await.ok()?;
        let _ = session.refresh_gamestate(gs).await.ok()?;
    }
    if gs.tavern_end_time().is_none() || gs.tavern_end_time().unwrap() <= Local::now() {
        let quests = session.list_tavern().await.ok()?;
        let pick = pick_mission(
            quests.into_iter().map(|q| Quest {
                id: q.id(),
                minutes: q.minutes(),
                gold: q.gold(),
                xp: q.xp(),
                mushrooms: q.mushrooms(),
            }).collect(),
            cfg.mission_strategy
        );
        if let Some(q) = pick {
            let _ = session.start_tavern(q.id).await.ok()?;
            let _ = session.refresh_gamestate(gs).await.ok()?;
            return Some(format!("tavern:{}m", q.minutes));
        }
    }
    None
}

async fn expedition_tick<S, GS>(
    cfg: &AutomationCfg,
    session: &mut S,
    gs: &mut GS,
) -> Option<String>
where
    S: SessionLike,
    GS: GameStateLike,
{
    if gs.expedition_end_time().map(|t| t <= Local::now()).unwrap_or(false) {
        let _ = session.collect_expedition_reward().await.ok()?;
        let _ = session.refresh_gamestate(gs).await.ok()?;
    }
    if gs.expedition_end_time().is_none() || gs.expedition_end_time().unwrap() <= Local::now() {
        let exps = session.list_expeditions().await.ok()?;
        let pick = pick_mission(
            exps.into_iter().map(|e| Expedition {
                id: e.id(),
                minutes: e.minutes(),
                gold: e.gold(),
                xp: e.xp(),
                mushrooms: e.mushrooms(),
            }).collect(),
            cfg.mission_strategy
        );
        if let Some(e) = pick {
            let _ = session.start_expedition(e.id).await.ok()?;
            let _ = session.refresh_gamestate(gs).await.ok()?;
            return Some(format!("expedition:{}m", e.minutes));
        }
    }
    None
}

async fn dungeon_tick<S, GS>(session: &mut S, gs: &mut GS) -> Option<String>
where
    S: SessionLike,
    GS: GameStateLike,
{
    if !gs.dungeon_ready() { return None; }
    let next = session.next_dungeon_candidate(gs).await.ok()?;
    if let Some(d) = next {
        let r = session.fight_dungeon(d.ident()).await.ok()?;
        let _ = session.refresh_gamestate(gs).await.ok()?;
        return Some(format!("dungeon:{}:{}", d.ident(), if r.win() {"win"} else {"lose"}));
    }
    None
}

async fn pets_tick<S, GS>(session: &mut S, gs: &mut GS) -> Option<String>
where
    S: SessionLike,
    GS: GameStateLike,
{
    if !gs.pet_ready() { return None; }
    let cand = session.next_pet_candidate(gs).await.ok()?;
    if let Some(p) = cand {
        let r = session.fight_pet(p.element(), p.slot()).await.ok()?;
        let _ = session.refresh_gamestate(gs).await.ok()?;
        return Some(format!("pet:{}:{}", p.element_str(), if r.win() {"win"} else {"lose"}));
    }
    None
}

/// Simple perpetual loop you can start per-character if you want a dedicated task.
/// Most users will integrate the smaller `tick` into their `AutoPoll` subscription instead.
pub async fn auto_loop<S, GS>(cfg: AutomationCfg, mut session: S, mut gs: GS)
where
    S: SessionLike,
    GS: GameStateLike,
{
    loop {
        let out = tick(&cfg, &mut session, &mut gs).await;
        let delay = if out.did_something { Duration::from_secs(5) } else { Duration::from_secs(30) };
        sleep(delay).await;
        let _ = session.refresh_gamestate(&mut gs).await;
    }
}

/// Trait façade so this module doesn’t depend on your exact `sf_api` structs.
/// Implement these for your session and gamestate wrappers.
#[async_trait::async_trait]
pub trait SessionLike: Send {
    type QuestLike: Send + Sync;
    type ExpeditionLike: Send + Sync;
    type DungeonCand: Send + Sync;
    type PetCand: Send + Sync;
    type FightRes: Send + Sync;

    async fn collect_tavern_reward(&mut self) -> Result<(), String>;
    async fn list_tavern(&mut self) -> Result<Vec<Self::QuestLike>, String>;
    async fn start_tavern(&mut self, id: u32) -> Result<(), String>;

    async fn collect_expedition_reward(&mut self) -> Result<(), String>;
    async fn list_expeditions(&mut self) -> Result<Vec<Self::ExpeditionLike>, String>;
    async fn start_expedition(&mut self, id: u32) -> Result<(), String>;

    async fn next_dungeon_candidate<GS: GameStateLike + Send>(
        &mut self,
        gs: &GS,
    ) -> Result<Option<Self::DungeonCand>, String>;
    async fn fight_dungeon(&mut self, ident: u32) -> Result<Self::FightRes, String>;

    async fn next_pet_candidate<GS: GameStateLike + Send>(
        &mut self,
        gs: &GS,
    ) -> Result<Option<Self::PetCand>, String>;
    async fn fight_pet(&mut self, element: u8, slot: u8) -> Result<Self::FightRes, String>;

    async fn refresh_gamestate<GS: GameStateLike + Send>(&mut self, gs: &mut GS) -> Result<(), String>;
}

pub trait GameStateLike {
    fn tavern_end_time(&self) -> Option<chrono::DateTime<chrono::Local>>;
    fn expedition_end_time(&self) -> Option<chrono::DateTime<chrono::Local>>;
    fn dungeon_ready(&self) -> bool;
    fn pet_ready(&self) -> bool;
}

/// Shape adapters for whatever your `sf_api` returns for quests/expeditions/dungeons/pets.
pub trait QuestLike {
    fn id(&self) -> u32;
    fn minutes(&self) -> u32;
    fn gold(&self) -> u64;
    fn xp(&self) -> u64;
    fn mushrooms(&self) -> u8;
}
pub trait ExpeditionLike {
    fn id(&self) -> u32;
    fn minutes(&self) -> u32;
    fn gold(&self) -> u64;
    fn xp(&self) -> u64;
    fn mushrooms(&self) -> u8;
}
pub trait DungeonLike {
    fn ident(&self) -> u32;
}
pub trait PetLike {
    fn element(&self) -> u8;
    fn slot(&self) -> u8;
    fn element_str(&self) -> &'static str { "elem" }
}
pub trait FightResultLike {
    fn win(&self) -> bool;
}

/// Shared mission picker for quests and expeditions.
fn pick_mission<T>(mut items: Vec<T>, strat: MissionStrategy) -> Option<T>
where
    T: MissionLike + Clone,
{
    // Save all mushrooms by default: do not consider missions that cost mushrooms.
    items.retain(|q| q.mushrooms() as u32 == 0);
    match strat {
        MissionStrategy::Shortest => {
            items.sort_by_key(|q| (q.mushrooms(), q.minutes(), -(q.gold() as i64)));
            items.into_iter().next()
        }
        MissionStrategy::MostGold => {
            items.sort_by(|a,b| b.gold().cmp(&a.gold()));
            items.into_iter().next()
        }
        MissionStrategy::BestGoldPerMinute => {
            items.sort_by(|a,b| rate(b.gold(), b.minutes()).cmp(&rate(a.gold(), a.minutes())));
            items.into_iter().next()
        }
        MissionStrategy::BestXpPerMinute => {
            items.sort_by(|a,b| rate(b.xp(), b.minutes()).cmp(&rate(a.xp(), a.minutes())));
            items.into_iter().next()
        }
        MissionStrategy::Smartest => {
            let mut best: Option<(f64, T)> = None;
            let mut max_gpm = 0f64;
            let mut max_xpm = 0f64;
            for q in &items { max_gpm = max_gpm.max(gpm(q)); max_xpm = max_xpm.max(xpm(q)); }
            for q in items.into_iter() {
                let t = q.minutes() as f64;
                let g = if max_gpm > 0.0 { gpm(&q) / max_gpm } else { 0.0 };
                let x = if max_xpm > 0.0 { xpm(&q) / max_xpm } else { 0.0 };
                let speed = 1.0 / t.max(1.0);
                let mush_penalty = if q.mushrooms() > 0 { 0.15 } else { 0.0 };
                let score = 0.4*g + 0.4*x + 0.2*speed - mush_penalty;
                match &mut best {
                    None => best = Some((score, q)),
                    Some((s, _)) if score > *s => best = Some((score, q)),
                    _ => {}
                }
            }
            best.map(|(_, q)| q)
        }
    }
}

trait MissionLike {
    fn id(&self) -> u32;
    fn minutes(&self) -> u32;
    fn gold(&self) -> u64;
    fn xp(&self) -> u64;
    fn mushrooms(&self) -> u8;
}
impl MissionLike for Quest {
    fn id(&self) -> u32 { self.id }
    fn minutes(&self) -> u32 { self.minutes }
    fn gold(&self) -> u64 { self.gold }
    fn xp(&self) -> u64 { self.xp }
    fn mushrooms(&self) -> u8 { self.mushrooms }
}
impl MissionLike for Expedition {
    fn id(&self) -> u32 { self.id }
    fn minutes(&self) -> u32 { self.minutes }
    fn gold(&self) -> u64 { self.gold }
    fn xp(&self) -> u64 { self.xp }
    fn mushrooms(&self) -> u8 { self.mushrooms }
}

fn rate(value: u64, minutes: u32) -> i64 {
    if minutes == 0 { return i64::MAX; }
    (value as i128 * 1_000_000i128 / minutes as i128) as i64
}
fn gpm<Q: MissionLike>(q: &Q) -> f64 {
    if q.minutes() == 0 { return f64::INFINITY; }
    q.gold() as f64 / q.minutes() as f64
}
fn xpm<Q: MissionLike>(q: &Q) -> f64 {
    if q.minutes() == 0 { return f64::INFINITY; }
    q.xp() as f64 / q.minutes() as f64
}