use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use chrono::{DateTime, Local};
use log::trace;
use nohash_hasher::IntMap;
use sf_api::{
    gamestate::{GameState, underworld::Underworld, unlockables::ScrapBook},
    session::Session,
};
use sf_api::command::Command as SFCommand;
use tokio::time::sleep;

use crate::{
    AccountIdent, AttackTarget, CharacterInfo, config::CharacterConfig,
    login::PlayerAuth, message::Message,
};

pub struct AccountInfo {
    pub name: String,
    pub ident: AccountIdent,
    pub auth: PlayerAuth,
    pub last_updated: DateTime<Local>,
    pub status: Arc<Mutex<AccountStatus>>,
    pub scrapbook_info: Option<ScrapbookInfo>,
    pub underworld_info: Option<UnderworldInfo>,
    pub automation_queue: Vec<SFCommand>,
}

pub struct UnderworldInfo {
    pub underworld: Underworld,
    pub best: Vec<CharacterInfo>,
    pub max_level: u16,
    pub attack_log: Vec<(DateTime<Local>, String, bool)>,
    pub auto_lure: bool,
}

impl UnderworldInfo {
    pub fn new(
        gs: &GameState,
        config: Option<&CharacterConfig>,
    ) -> Option<Self> {
        let underworld = gs.underworld.as_ref()?.clone();
        let avg_lvl = underworld
            .units
            .as_array()
            .iter()
            .map(|a| a.level as u64)
            .sum::<u64>() as f32
            / 3.0;
        Some(Self {
            underworld,
            best: Default::default(),
            max_level: avg_lvl as u16 + 20,
            attack_log: Vec::new(),
            auto_lure: config.map(|a| a.auto_lure).unwrap_or(false),
        })
    }
}

pub struct ScrapbookInfo {
    pub scrapbook: ScrapBook,
    pub best: Vec<AttackTarget>,
    pub max_level: u16,
    pub max_attributes: u32,
    pub blacklist: IntMap<u32, (String, usize)>,
    pub attack_log: Vec<(DateTime<Local>, AttackTarget, bool)>,
    pub auto_battle: bool,
}

impl ScrapbookInfo {
    pub fn new(
        gs: &GameState,
        config: Option<&CharacterConfig>,
    ) -> Option<Self> {
        let max_attributes = {
            let base = gs.character.attribute_basis.as_array();
            let bonus = gs.character.attribute_additions.as_array();
            let total = base.iter().chain(bonus).sum::<u32>();
            let expected_battle_luck = 1.2f32;
            (total as f32 * expected_battle_luck) as u32
        };

        Some(Self {
            scrapbook: gs.character.scrapbook.as_ref()?.clone(),
            best: Default::default(),
            max_level: gs.character.level,
            max_attributes,
            blacklist: Default::default(),
            attack_log: Default::default(),
            auto_battle: config.map(|a| a.auto_battle).unwrap_or(false),
        })
    }
}

impl AccountInfo {
    pub fn new(
        name: &str,
        auth: PlayerAuth,
        ident: AccountIdent,
    ) -> AccountInfo {
        AccountInfo {
            name: name.to_string(),
            auth,
            scrapbook_info: None,
            underworld_info: None,
            last_updated: Local::now(),
            status: Arc::new(Mutex::new(AccountStatus::LoggingIn)),
            ident,
            automation_queue: Vec::new(),
        }
    }
}

pub enum AccountStatus {
    LoggingIn,
    Idle(Box<Session>, Box<GameState>),
    Busy(Box<GameState>, Box<str>),
    FatalError(String),
    LoggingInAgain,
}

impl AccountStatus {
    pub fn take_session<T: Into<Box<str>>>(
        &mut self,
        reason: T,
    ) -> Option<Box<Session>> {
        let mut res = None;
        *self = match std::mem::replace(self, AccountStatus::LoggingInAgain) {
            AccountStatus::Idle(a, b) => {
                res = Some(a);
                AccountStatus::Busy(b, reason.into())
            }
            x => x,
        };
        res
    }

    pub fn put_session(&mut self, session: Box<Session>) {
        *self = match std::mem::replace(self, AccountStatus::LoggingInAgain) {
            AccountStatus::Busy(a, _) => AccountStatus::Idle(session, a),
            x => x,
        };
    }
}

pub struct AutoAttackChecker {
    pub player_status: Arc<Mutex<AccountStatus>>,
    pub ident: AccountIdent,
}

impl AutoAttackChecker {
    pub async fn check(&self) -> Message {
        let next_fight: Option<DateTime<Local>> = {
            match &*self.player_status.lock().unwrap() {
                AccountStatus::Idle(_, session) => {
                    session.arena.next_free_fight
                }
                _ => None,
            }
        };
        if let Some(next) = next_fight {
            let remaining = next - Local::now();
            if let Ok(remaining) = remaining.to_std() {
                tokio::time::sleep(remaining).await;
            }
        };
        tokio::time::sleep(Duration::from_millis(fastrand::u64(1000..=3000)))
            .await;

        Message::AutoBattlePossible { ident: self.ident }
    }
}

pub struct AutoLureChecker {
    pub player_status: Arc<Mutex<AccountStatus>>,
    pub ident: AccountIdent,
}

impl AutoLureChecker {
    pub async fn check(&self) -> Message {
        let lured = {
            match &*self.player_status.lock().unwrap() {
                AccountStatus::Idle(_, session) => {
                    session.underworld.as_ref().map(|a| a.lured_today)
                }
                _ => None,
            }
        };
        let Some(0..=4) = lured else {
            // Either no underworld, or already lured the max
            tokio::time::sleep(Duration::from_millis(fastrand::u64(
                5000..=10_000,
            )))
            .await;
            return Message::AutoLureIdle;
        };

        tokio::time::sleep(Duration::from_millis(fastrand::u64(3000..=5000)))
            .await;

        Message::AutoLurePossible { ident: self.ident }
    }
}

pub struct AutoPoll {
    pub player_status: Arc<Mutex<AccountStatus>>,
    pub ident: AccountIdent,
}

impl AutoPoll {
    pub async fn check(&self) -> Message {
        sleep(Duration::from_millis(fastrand::u64(5000..=10000))).await;
        let mut session = {
            let mut lock = self.player_status.lock().unwrap();
            let res = lock.take_session("Auto Poll");
            match res {
                Some(res) => res,
                None => return Message::PlayerNotPolled { ident: self.ident },
            }
        };

        trace!("Sending poll {:?}", self.ident);

        let Ok(resp) = session
            .send_command(&sf_api::command::Command::Update)
            .await
        else {
            return Message::PlayerCommandFailed {
                ident: self.ident,
                session,
                attempt: 0,
            };
        };
        let mut lock = self.player_status.lock().unwrap();
        let gs = match &mut *lock {
            AccountStatus::Busy(gs, _) => gs,
            _ => {
                lock.put_session(session);
                return Message::PlayerNotPolled { ident: self.ident };
            }
        };
        if gs.update(resp).is_err() {
            return Message::PlayerCommandFailed {
                ident: self.ident,
                session,
                attempt: 0,
            };
        }
        lock.put_session(session);
        Message::PlayerPolled { ident: self.ident }
    }
}

pub struct AutoMissionsChecker {
    pub player_status: Arc<Mutex<AccountStatus>>,
    pub ident: AccountIdent,
    pub first_tick: bool,
}

impl AutoMissionsChecker {
    pub async fn check(&mut self) -> Message {
        if self.first_tick {
            self.first_tick = false;
            log::debug!("AutoMissions {:?}: first tick, triggering soon", self.ident);
            sleep(Duration::from_millis(fastrand::u64(200..=600))).await;
            return Message::RunAutomationTick { ident: self.ident };
        }
        let is_idle = {
            let s = self.player_status.lock().unwrap();
            matches!(&*s, AccountStatus::Idle(_, _))
        };
        if !is_idle {
            let backoff = fastrand::u64(1500..=3500);
            log::debug!("AutoMissions {:?}: not idle, retry in {}ms", self.ident, backoff);
            sleep(Duration::from_millis(backoff)).await;
            return Message::RunAutomationTick { ident: self.ident };
        }

        use chrono::Local;
        let now = Local::now();
        let mut next_due: Option<chrono::DateTime<Local>> = None;
        let mut due_now = false;

        if let AccountStatus::Idle(_, gs) = &*self.player_status.lock().unwrap() {
            use sf_api::gamestate::tavern::CurrentAction;
            // Tavern: quest end or expedition waiting stage
            match &gs.tavern.current_action {
                CurrentAction::Quest { busy_until, .. } => {
                    if *busy_until > now { next_due = Some(*busy_until); } else { due_now = true; }
                }
                CurrentAction::Expedition => {
                    if let Some(active) = gs.tavern.expeditions.active() {
                        use sf_api::gamestate::tavern::ExpeditionStage;
                        if let ExpeditionStage::Waiting(until) = active.current_stage() {
                            if until > now { next_due = Some(until); } else { due_now = true; }
                        }
                    }
                }
                _ => {}
            }

            // Pets: PvP and exploration cooldowns
            if let Some(pets) = &gs.pets {
                match pets.opponent.next_free_battle {
                    Some(t) => { if t > now { next_due = next_due.map_or(Some(t), |a| Some(a.min(t))); } else { due_now = true; } },
                    None => { due_now = true; }
                }
                match pets.next_free_exploration {
                    Some(t) => { if t > now { next_due = next_due.map_or(Some(t), |a| Some(a.min(t))); } else { due_now = true; } },
                    None => { due_now = true; }
                }
            }

            // Dungeons: next free fight timer
            match gs.dungeons.next_free_fight {
                Some(t) => { if t > now { next_due = next_due.map_or(Some(t), |a| Some(a.min(t))); } else { due_now = true; } },
                None => { due_now = true; }
            }

            // Guild: hydra next battle
            if let Some(guild) = &gs.guild {
                if let Some(t) = guild.hydra.next_battle { if t > now { next_due = next_due.map_or(Some(t), |a| Some(a.min(t))); } else { due_now = true; } }
            }
        }

        if due_now {
            let jitter = fastrand::u64(400..=1200);
            log::debug!("AutoMissions {:?}: one or more actions due now, jitter {}ms", self.ident, jitter);
            sleep(Duration::from_millis(jitter)).await;
        } else if let Some(t) = next_due {
            if t > now {
                let max_interval = std::time::Duration::from_secs(120);
                let wait_full = (t - now).to_std().unwrap_or_default();
                let wait = if wait_full > max_interval { max_interval } else { wait_full };
                log::debug!(
                    "AutoMissions {:?}: next due at {}, waiting {:?}",
                    self.ident,
                    t.format("%H:%M:%S"),
                    wait
                );
                tokio::time::sleep(wait).await;
            } else {
                let jitter = fastrand::u64(400..=1200);
                log::debug!("AutoMissions {:?}: due now, jitter {}ms", self.ident, jitter);
                sleep(Duration::from_millis(jitter)).await;
            }
        } else {
            let backoff = fastrand::u64(30_000..=60_000);
            log::debug!("AutoMissions {:?}: no timers found, retry in {}ms", self.ident, backoff);
            sleep(Duration::from_millis(backoff)).await;
        }

        let jitter = fastrand::u64(300..=1200);
        log::trace!("AutoMissions {:?}: post-wait jitter {}ms", self.ident, jitter);
        sleep(Duration::from_millis(jitter)).await;
        Message::RunAutomationTick { ident: self.ident }
    }
}
