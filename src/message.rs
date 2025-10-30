use std::{fmt::Write, sync::Arc, time::Duration};

use chrono::Local;
use config::{CharacterConfig, SFAccCharacter, SFCharIdent, MissionStrategy};
use crawler::CrawlerError;
use iced::Command;
use log::{debug, error, info, trace, warn};
use sf_api::{
    gamestate::GameState,
    session::{PWHash, Response, Session},
    sso::SSOProvider,
};
use tokio::time::sleep;
use ui::OverviewAction;

use self::{
    backup::{get_newest_backup, restore_backup, RestoreData},
    login::{SSOIdent, SSOLogin, SSOLoginStatus},
    ui::underworld::LureTarget,
};
use crate::{
    crawler::CrawlerState,
    player::{ScrapbookInfo, UnderworldInfo},
    *,
};

#[derive(Debug, Clone)]
pub enum Message {
    MultiAction {
        action: OverviewAction,
    },
    FontLoaded(Result<(), iced::font::Error>),
    CrawlAllRes {
        servers: Option<Vec<String>>,
        concurrency: usize,
    },
    NextCLICrawling,
    AdvancedLevelRestrict(bool),
    ShowClasses(bool),
    CrawlerSetMinMax {
        server: ServerID,
        min: u32,
        max: u32,
    },
    UpdateResult(bool),
    PlayerSetMaxUndergroundLvl {
        ident: AccountIdent,
        lvl: u16,
    },
    PlayerNotPolled {
        ident: AccountIdent,
    },
    PlayerPolled {
        ident: AccountIdent,
    },
    SetOverviewSelected {
        ident: Vec<AccountIdent>,
        val: bool,
    },
    SSOLoginFailure {
        name: String,
        error: String,
    },
    PlayerRelogSuccess {
        ident: AccountIdent,
        gs: Box<GameState>,
        session: Box<Session>,
    },
    PlayerRelogDelay {
        ident: AccountIdent,
        session: Box<Session>,
    },
    CopyBattleOrder {
        ident: AccountIdent,
    },
    BackupRes {
        server: ServerID,
        error: Option<String>,
    },
    SaveHoF(ServerID),
    PlayerSetMaxLvl {
        ident: AccountIdent,
        max: u16,
    },
    PlayerSetMaxAttributes {
        ident: AccountIdent,
        max: u32,
    },
    PlayerAttack {
        ident: AccountIdent,
        target: AttackTarget,
    },
    PlayerLure {
        ident: AccountIdent,
        target: LureTarget,
    },
    OpenLink(String),
    SSOSuccess {
        auth_name: String,
        chars: Vec<Session>,
        provider: SSOProvider,
    },
    SSORetry,
    SSOAuthError {
        _error: String,
    },
    SetMaxThreads(usize),
    SetStartThreads(usize),
    SetBlacklistThr(usize),
    SetAutoFetch(bool),
    SetAutoPoll(bool),
    ViewSubPage {
        player: AccountIdent,
        page: AccountPage,
    },
    SSOImport {
        pos: usize,
    },
    SSOImportAuto {
        ident: SFCharIdent,
    },
    SSOLoginSuccess {
        name: String,
        pass: PWHash,
        chars: Vec<Session>,
        remember: bool,
        auto_login: bool,
    },
    ViewSettings,
    ChangeTheme(AvailableTheme),
    ViewOverview,
    CrawlerRevived {
        server_id: ServerID,
    },
    CrawlerStartup {
        server: ServerID,
        state: Arc<CrawlerState>,
    },
    AutoBattle {
        ident: AccountIdent,
        state: bool,
    },
    AutoLure {
        ident: AccountIdent,
        state: bool,
    },
    PlayerCommandFailed {
        ident: AccountIdent,
        session: Box<Session>,
        attempt: u64,
    },
    PlayerAttackResult {
        ident: AccountIdent,
        session: Box<Session>,
        against: AttackTarget,
        resp: Box<Response>,
    },
    PlayerLureResult {
        ident: AccountIdent,
        session: Box<Session>,
        against: LureTarget,
        resp: Box<Response>,
    },
    AutoBattlePossible {
        ident: AccountIdent,
    },
    OrderChange {
        server: ServerID,
        new: CrawlingOrder,
    },
    Login {
        account: AccountConfig,
        auto_login: bool,
    },
    RememberMe(bool),
    ClearHof(ServerID),
    CrawlerSetThreads {
        server: ServerID,
        new_count: usize,
    },
    PageCrawled,
    RemoveAccount {
        ident: AccountIdent,
    },
    CharacterCrawled {
        server: ServerID,
        que_id: QueID,
        character: CharacterInfo,
    },
    CrawlerDied {
        server: ServerID,
        error: String,
    },
    ShowPlayer {
        ident: AccountIdent,
    },
    CrawlerIdle(ServerID),
    CrawlerNoPlayerResult,
    CrawlerUnable {
        server: ServerID,
        action: CrawlAction,
        error: CrawlerError,
    },
    ViewLogin,
    LoginNameInputChange(String),
    LoginPWInputChange(String),
    LoginServerChange(String),
    LoginSFSubmit,
    LoginRegularSubmit,
    LoginViewChanged(LoginType),
    LoggininSuccess {
        ident: AccountIdent,
        gs: Box<GameState>,
        session: Box<Session>,
        remember: bool,
    },
    LoggininFailure {
        ident: AccountIdent,
        error: String,
    },
    ResetCrawling {
        server: ServerID,
        status: Box<RestoreData>,
    },
    ConfigSetAutoLogin {
        name: String,
        server: ServerID,
        nv: bool,
    },
    ConfigSetAutoBattle {
        name: String,
        server: ServerID,
        nv: bool,
    },
    ConfigSetAutoLure {
        name: String,
        server: ServerID,
        nv: bool,
    },

    // NEW automation config messages
    ConfigSetAutoTavern {
        name: String,
        server: ServerID,
        nv: bool,
    },
    ConfigSetAutoExpeditions {
        name: String,
        server: ServerID,
        nv: bool,
    },
    ConfigSetAutoDungeons {
        name: String,
        server: ServerID,
        nv: bool,
    },
    ConfigSetAutoPets {
        name: String,
        server: ServerID,
        nv: bool,
    },
    ConfigSetAutoGuild {
        name: String,
        server: ServerID,
        nv: bool,
    },
    ConfigSetAutoGuildAcceptDefense {
        name: String,
        server: ServerID,
        nv: bool,
    },
    ConfigSetAutoGuildAcceptAttack {
        name: String,
        server: ServerID,
        nv: bool,
    },
    ConfigSetAutoGuildHydra {
        name: String,
        server: ServerID,
        nv: bool,
    },
    // Tavern options
    ConfigSetUseTavernGlasses {
        name: String,
        server: ServerID,
        nv: bool,
    },
    // Expeditions options
    ConfigSetUseExpeditionGlasses {
        name: String,
        server: ServerID,
        nv: bool,
    },
    ConfigSetExpeditionRewardPriority {
        name: String,
        server: ServerID,
        nv: crate::config::ExpeditionRewardPriority,
    },
    ConfigSetMissionStrategy {
        name: String,
        server: ServerID,
        nv: MissionStrategy,
    },
    ConfigSetAutoBuyBeerMushrooms {
        name: String,
        server: ServerID,
        nv: bool,
    },

    // Mushroom budget sliders
    ConfigSetMaxMushroomsBeer {
        name: String,
        server: ServerID,
        nv: u32,
    },
    ConfigSetMaxMushroomsDungeonSkip {
        name: String,
        server: ServerID,
        nv: u32,
    },
    ConfigSetMaxMushroomsPetSkip {
        name: String,
        server: ServerID,
        nv: u32,
    },

    AutoLureIdle,
    AutoLurePossible {
        ident: AccountIdent,
    },
    CopyBestLures {
        ident: AccountIdent,
    },
    SetAction(Option<ActionSelection>),
    // Periodic automation tick (Tavern/Expeditions/Dungeons/Pets)
    RunAutomationTick { ident: AccountIdent },
}

impl Helper {
    pub fn handle_msg(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::RunAutomationTick { ident } => {
                let Some(server) = self.servers.0.get_mut(&ident.server_id) else {
                    return Command::none();
                };
                let Some(account) = server.accounts.get_mut(&ident.account) else {
                    return Command::none();
                };
                log::debug!("Automation {:?}: tick start", ident);

                let Some(cfg) = self
                    .config
                    .get_char_conf(&account.name, server.ident.id)
                else {
                    return Command::none();
                };

                if !(cfg.auto_tavern || cfg.auto_expeditions || cfg.auto_dungeons || cfg.auto_pets || cfg.auto_guild) {
                    return Command::none();
                }

                use chrono::Local;
                use sf_api::command::{Command as SFCommand, ExpeditionSetting, TimeSkip};
                use sf_api::gamestate::tavern::{AvailableTasks, CurrentAction, ExpeditionStage};
                use sf_api::gamestate::dungeons::{DungeonProgress, LightDungeon, ShadowDungeon, Dungeon};
                use sf_api::gamestate::unlockables::{HabitatType, HabitatExploration};
                use sf_api::misc::EnumMapGet;
                use strum::IntoEnumIterator;
                use sf_api::gamestate::items::Enchantment;

                let mut status = account.status.lock().unwrap();

                let AccountStatus::Idle(_, gs) = &*status else {
                    log::debug!("Automation {:?}: account not idle, retrying shortly", ident);
                    drop(status);
                    let rerun = Command::perform(
                        async move {
                            tokio::time::sleep(std::time::Duration::from_millis(fastrand::u64(500..=1500))).await;
                        },
                        move |_| Message::RunAutomationTick { ident }
                    );
                    return rerun;
                };

                let now = Local::now();
                log::debug!("Automation {:?}: current_action = {:?}", ident, gs.tavern.current_action);

                // Decide next automation command
                let next_cmd: Option<SFCommand> = {
                    // Handle ongoing quest completion or skipping
                    match &gs.tavern.current_action {
                        CurrentAction::Quest { busy_until, .. } => {
                            if *busy_until <= now {
                                Some(SFCommand::FinishQuest { skip: None })
                            } else {
                                // Consider skipping long waits (glass only; never mushrooms)
                                let remaining = (*busy_until - now)
                                    .to_std()
                                    .unwrap_or_default();
                                if remaining.as_secs() > 60 {
                                    if cfg.use_glasses_for_tavern
                                        && gs.tavern.quicksand_glasses > 0
                                    {
                                        log::debug!(
                                            "Automation {:?}: Quest waiting {}s -> skip with glass (tavern glasses enabled)",
                                            ident,
                                            remaining.as_secs()
                                        );
                                        Some(SFCommand::FinishQuest {
                                            skip: Some(TimeSkip::Glass),
                                        })
                                    } else {
                                        log::debug!(
                                            "Automation {:?}: Quest waiting {}s -> no skip (tavern glasses disabled or none available)",
                                            ident,
                                            remaining.as_secs()
                                        );
                                        None
                                    }
                                } else {
                                    None
                                }
                            }
                        }
                        CurrentAction::Expedition => {
                            // Continue/advance an active expedition if possible
                            if let Some(active) = gs.tavern.expeditions.active() {
                                match active.current_stage() {
                                    ExpeditionStage::Boss(_) => {
                                        log::debug!("Automation {:?}: Expedition boss -> continue", ident);
                                        Some(SFCommand::ExpeditionContinue)
                                    }
                                    ExpeditionStage::Rewards(rewards) => {
                                        if rewards.is_empty() {
                                            log::debug!("Automation {:?}: Expedition rewards empty", ident);
                                            None
                                        } else {
                                            // Choose reward based on configured priority
                                            let mut best_idx = 0usize;
                                            let mut best_rank = i32::MIN;
                                            let prio = cfg.expedition_reward_priority;
                                            for (i, r) in rewards.iter().enumerate() {
                                                let s = format!("{:?}", r).to_lowercase();
                                                let is_mush = s.contains("mushroom");
                                                let is_gold = s.contains("gold") || s.contains("silver");
                                                let is_egg = s.contains("egg");
                                                let rank = match prio {
                                                    crate::config::ExpeditionRewardPriority::MushroomsGoldEggs => {
                                                        if is_mush { 3 } else if is_gold { 2 } else if is_egg { 1 } else { 0 }
                                                    }
                                                    crate::config::ExpeditionRewardPriority::GoldMushroomsEggs => {
                                                        if is_gold { 3 } else if is_mush { 2 } else if is_egg { 1 } else { 0 }
                                                    }
                                                    crate::config::ExpeditionRewardPriority::EggsMushroomsGold => {
                                                        if is_egg { 3 } else if is_mush { 2 } else if is_gold { 1 } else { 0 }
                                                    }
                                                };
                                                if rank > best_rank { best_rank = rank; best_idx = i; }
                                            }
                                            log::debug!("Automation {:?}: Expedition pick reward index {} of {} (priority {:?})", ident, best_idx, rewards.len(), prio);
                                            Some(SFCommand::ExpeditionPickReward { pos: best_idx })
                                        }
                                    }
                                    ExpeditionStage::Encounters(encs) => {
                                        if encs.is_empty() {
                                            log::debug!("Automation {:?}: Expedition encounters empty", ident);
                                            None
                                        } else {
                                            log::debug!("Automation {:?}: Expedition pick first encounter ({} options)", ident, encs.len());
                                            Some(SFCommand::ExpeditionPickEncounter { pos: 0 })
                                        }
                                    }
                                    ExpeditionStage::Waiting(until) => {
                                        let remaining = (until - now)
                                            .to_std()
                                            .unwrap_or_default();
                                        if cfg.use_glasses_for_expeditions
                                            && remaining.as_secs() > 60
                                            && gs.tavern.quicksand_glasses > 0
                                        {
                                            log::debug!("Automation {:?}: Expedition waiting {}s -> skip with glass", ident, remaining.as_secs());
                                            Some(SFCommand::ExpeditionSkipWait {
                                                typ: TimeSkip::Glass,
                                            })
                                        } else {
                                            log::debug!("Automation {:?}: Expedition waiting {}s -> no skip", ident, remaining.as_secs());
                                            None
                                        }
                                    }
                                    ExpeditionStage::Finished
                                    | ExpeditionStage::Unknown => None,
                                }
                            } else {
                                None
                            }
                        }
                        CurrentAction::CityGuard { hours: _hours, busy_until } => {
                            let mut cmd: Option<SFCommand> = None;

                            // If guard duty is finished, collect pay first
                            if *busy_until <= now {
                                log::debug!("Automation {:?}: CityGuard finished -> FinishWork", ident);
                                cmd = Some(SFCommand::FinishWork);
                            }

                            if cfg.auto_dungeons {
                                if let Some(portal) = &gs.dungeons.portal {
                                    if portal.can_fight {
                                        log::debug!("Automation {:?}: Portal fight ready (during CityGuard)", ident);
                                        cmd = Some(SFCommand::FightPortal);
                                    }
                                }
                                if cmd.is_none() {
                                    let next_ready = gs
                                        .dungeons
                                        .next_free_fight
                                        .map(|t| t <= now)
                                        .unwrap_or(true);
                                    let mut use_mush = false;
                                    let can_fight_now = if next_ready {
                                        true
                                    } else if cfg.max_mushrooms_dungeon_skip > 0 && gs.character.mushrooms > 0 {
                                        log::debug!("Automation {:?}: Dungeons not ready, using mushroom to skip (during CityGuard)", ident);
                                        use_mush = true;
                                        true
                                    } else { false };

                                    if can_fight_now {
                                        if let DungeonProgress::Open { finished } = gs.dungeons.progress(LightDungeon::Tower) {
                                            log::debug!("Automation {:?}: Tower ready at level {} (during CityGuard)", ident, finished);
                                            cmd = Some(SFCommand::FightTower { current_level: finished as u8, use_mush });
                                        } else {
                                            let mut best: Option<(Dungeon, u16)> = None;
                                            for d in LightDungeon::iter() {
                                                if d == LightDungeon::Tower { continue; }
                                                if let DungeonProgress::Open { finished } = gs.dungeons.progress(d) {
                                                    let entry = (Dungeon::from(d), finished);
                                                    best = match best { None => Some(entry), Some((_, f)) if finished < f => Some(entry), x => x };
                                                }
                                            }
                                            for d in ShadowDungeon::iter() {
                                                if let DungeonProgress::Open { finished } = gs.dungeons.progress(d) {
                                                    let entry = (Dungeon::from(d), finished);
                                                    best = match best { None => Some(entry), Some((_, f)) if finished < f => Some(entry), x => x };
                                                }
                                            }
                                            if let Some((dng, _)) = best {
                                                log::debug!("Automation {:?}: Dungeon chosen during CityGuard: {:?}", ident, dng);
                                                cmd = Some(SFCommand::FightDungeon { dungeon: dng, use_mushroom: use_mush });
                                            } else {
                                                log::debug!("Automation {:?}: Dungeons ready but no open dungeon/tower found (during CityGuard)", ident);
                                            }
                                        }
                                    } else {
                                        log::debug!("Automation {:?}: Dungeons not ready (during CityGuard) (next_free_fight: {:?}, mushrooms: {})", ident, gs.dungeons.next_free_fight, gs.character.mushrooms);
                                    }
                                }
                            }

                            if cmd.is_none() && cfg.auto_pets {
                                if let Some(pets) = &gs.pets {
                                    let free_now = pets.opponent.next_free_battle.map(|t| t <= now).unwrap_or(true);
                                    if free_now {
                                        log::debug!("Automation {:?}: Pets PvP free (during CityGuard)", ident);
                                        let mut target_hab: Option<HabitatType> = None;
                                        if let Some(h) = pets.opponent.habitat {
                                            if !pets.habitats.get(h).battled_opponent { target_hab = Some(h); }
                                        }
                                        if target_hab.is_none() {
                                            use strum::IntoEnumIterator;
                                            let mut best: Option<(HabitatType, u16)> = None;
                                            for h in HabitatType::iter() {
                                                let hab = pets.habitats.get(h);
                                                if hab.battled_opponent { continue; }
                                                if let Some(p) = hab.pets.iter().max_by_key(|p| p.level) {
                                                    best = match best { None => Some((h, p.level)), Some((_, lvl)) if p.level > lvl => Some((h, p.level)), x => x };
                                                }
                                            }
                                            if let Some((h, _)) = best { target_hab = Some(h); }
                                        }
                                        if let Some(h) = target_hab {
                                            log::debug!("Automation {:?}: Pets PvP habitat {:?} (during CityGuard)", ident, h);
                                            cmd = Some(SFCommand::FightPetOpponent { habitat: h, opponent_id: pets.opponent.id });
                                        } else {
                                            log::debug!("Automation {:?}: Pets PvP ready but no eligible habitat (during CityGuard)", ident);
                                        }
                                    }

                                    if cmd.is_none() {
                                        let next_ready = pets.next_free_exploration.map(|t| t <= now).unwrap_or(true);
                                        let mut use_mush = false;
                                        let can_explore = if next_ready { true } else if cfg.max_mushrooms_pet_skip > 0 && gs.character.mushrooms > 0 { use_mush = true; true } else { false };
                                        if can_explore {
                                            log::debug!("Automation {:?}: Pets exploration free (during CityGuard)", ident);
                                            use strum::IntoEnumIterator;
                                            let mut pick: Option<(HabitatType, u32, u16, u32)> = None;
                                            for hab in HabitatType::iter() {
                                                if let HabitatExploration::Exploring { fights_won, .. } = pets.habitats.get(hab).exploration {
                                                    if let Some(best) = pets.habitats.get(hab).pets.iter().max_by_key(|p| p.level) {
                                                        let entry = (hab, fights_won + 1, best.level, best.id);
                                                        pick = match pick {
                                                            None => Some(entry),
                                                            Some((_, _, lvl, _)) if best.level > lvl => Some(entry),
                                                            x => x,
                                                        };
                                                    }
                                                }
                                            }
                                            if let Some((hab, enemy_pos, _best_lvl, best_id)) = pick {
                                                if use_mush { log::debug!("Automation {:?}: Pets exploration not ready, using mushroom to skip (during CityGuard)", ident); }
                                                log::debug!("Automation {:?}: Pets explore habitat {:?} fight_pos {} pet_id {} (during CityGuard)", ident, hab, enemy_pos, best_id);
                                                cmd = Some(SFCommand::FightPetDungeon { use_mush, habitat: hab, enemy_pos, player_pet_id: best_id });
                                            } else {
                                                log::debug!("Automation {:?}: Pets exploration ready but no habitat currently exploring (during CityGuard)", ident);
                                            }
                                        } else {
                                            log::debug!("Automation {:?}: Pets exploration not ready (during CityGuard) (next_free_exploration: {:?})", ident, pets.next_free_exploration);
                                        }
                                    }
                                }
                            }

                            if cmd.is_none() && cfg.auto_guild {
                                if gs.guild.is_some() && cfg.auto_guild_accept_defense {
                                    log::debug!("Automation {:?}: Guild join defense (during CityGuard)", ident);
                                    cmd = Some(SFCommand::GuildJoinDefense);
                                }
                                if cmd.is_none() && gs.guild.is_some() && cfg.auto_guild_accept_attack {
                                    log::debug!("Automation {:?}: Guild join attack (during CityGuard)", ident);
                                    cmd = Some(SFCommand::GuildJoinAttack);
                                }
                                if cmd.is_none() && cfg.auto_guild_hydra {
                                    if let Some(guild) = &gs.guild {
                                        if guild.hydra.remaining_fights > 0 {
                                            if let Some(next) = guild.hydra.next_battle {
                                                if next <= now {
                                                    log::debug!("Automation {:?}: Guild hydra battle (during CityGuard)", ident);
                                                    cmd = Some(SFCommand::GuildPetBattle { use_mushroom: false });
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            if cmd.is_none() {
                                let portal = gs.dungeons.portal.as_ref().map(|p| p.can_fight).unwrap_or(false);
                                let dng_ready = gs.dungeons.next_free_fight.map(|t| t <= now).unwrap_or(true);
                                let open_dng = {
                                    let mut open = 0u32;
                                    for d in LightDungeon::iter() {
                                        if let DungeonProgress::Open { .. } = gs.dungeons.progress(d) { open += 1; }
                                    }
                                    for d in ShadowDungeon::iter() {
                                        if let DungeonProgress::Open { .. } = gs.dungeons.progress(d) { open += 1; }
                                    }
                                    open
                                };
                                let pets_pvp_ready = gs.pets.as_ref().and_then(|p| p.opponent.next_free_battle).map(|t| t <= now).unwrap_or(false);
                                let pets_explore_ready = gs.pets.as_ref().and_then(|p| p.next_free_exploration).map(|t| t <= now).unwrap_or(false);
                                let hydra_ready = gs.guild.as_ref().and_then(|g| g.hydra.next_battle).map(|t| t <= now).unwrap_or(false);
                                let thirst = gs.tavern.thirst_for_adventure_sec;
                                log::debug!(
                                    "Automation {:?}: CityGuard active. No Tavern tasks allowed. Summary -> portal: {}, dng_ready: {}, open_dng: {}, pets_pvp: {}, pets_explore: {}, hydra: {}, thirst: {}s",
                                    ident, portal, dng_ready, open_dng, pets_pvp_ready, pets_explore_ready, hydra_ready, thirst
                                );
                            }

                            cmd
                        }
                        CurrentAction::Unknown(_) | CurrentAction::Idle => {
                            let mut cmd: Option<SFCommand> = None;

                            if cfg.auto_dungeons {
                                if let Some(portal) = &gs.dungeons.portal {
                                    if portal.can_fight {
                                        log::debug!("Automation {:?}: Portal fight ready", ident);
                                        cmd = Some(SFCommand::FightPortal);
                                    }
                                }
                                if cmd.is_none() {
                                    let next_ready = gs
                                        .dungeons
                                        .next_free_fight
                                        .map(|t| t <= now)
                                        .unwrap_or(true);
                                    let mut use_mush = false;
                                    let can_fight_now = if next_ready { true } else if cfg.max_mushrooms_dungeon_skip > 0 && gs.character.mushrooms > 0 { log::debug!("Automation {:?}: Dungeons not ready, using mushroom to skip", ident); use_mush = true; true } else { false };

                                    if can_fight_now {
                                        if let DungeonProgress::Open { finished } = gs.dungeons.progress(LightDungeon::Tower) {
                                            log::debug!("Automation {:?}: Tower ready at level {}", ident, finished);
                                            cmd = Some(SFCommand::FightTower { current_level: finished as u8, use_mush });
                                        } else {
                                            let mut best: Option<(Dungeon, u16)> = None;
                                            for d in LightDungeon::iter() {
                                                if d == LightDungeon::Tower { continue; }
                                                if let DungeonProgress::Open { finished } = gs.dungeons.progress(d) {
                                                    let entry = (Dungeon::from(d), finished);
                                                    best = match best { None => Some(entry), Some((_, f)) if finished < f => Some(entry), x => x };
                                                }
                                            }
                                            for d in ShadowDungeon::iter() {
                                                if let DungeonProgress::Open { finished } = gs.dungeons.progress(d) {
                                                    let entry = (Dungeon::from(d), finished);
                                                    best = match best { None => Some(entry), Some((_, f)) if finished < f => Some(entry), x => x };
                                                }
                                            }
                                            if let Some((dng, _)) = best {
                                                log::debug!("Automation {:?}: Dungeon chosen: {:?}", ident, dng);
                                                cmd = Some(SFCommand::FightDungeon { dungeon: dng, use_mushroom: use_mush });
                                            } else {
                                                // Ready by timer/mush, but nothing open
                                                log::debug!("Automation {:?}: Dungeons ready but no open dungeon/tower found", ident);
                                            }
                                        }
                                    } else {
                                        log::debug!("Automation {:?}: Dungeons not ready (next_free_fight: {:?}, mushrooms: {})", ident, gs.dungeons.next_free_fight, gs.character.mushrooms);
                                    }
                                }
                            }

                            if cmd.is_none() && cfg.auto_pets {
                                if let Some(pets) = &gs.pets {
                                    let free_now = pets.opponent.next_free_battle.map(|t| t <= now).unwrap_or(true);
                                    if free_now {
                                        log::debug!("Automation {:?}: Pets PvP free", ident);
                                        let mut target_hab: Option<HabitatType> = None;
                                        if let Some(h) = pets.opponent.habitat {
                                            if !pets.habitats.get(h).battled_opponent { target_hab = Some(h); }
                                        }
                                        if target_hab.is_none() {
                                            use strum::IntoEnumIterator;
                                            let mut best: Option<(HabitatType, u16)> = None;
                                            for h in HabitatType::iter() {
                                                let hab = pets.habitats.get(h);
                                                if hab.battled_opponent { continue; }
                                                if let Some(p) = hab.pets.iter().max_by_key(|p| p.level) {
                                                    best = match best { None => Some((h, p.level)), Some((_, lvl)) if p.level > lvl => Some((h, p.level)), x => x };
                                                }
                                            }
                                            if let Some((h, _)) = best { target_hab = Some(h); }
                                        }
                                        if let Some(h) = target_hab {
                                            log::debug!("Automation {:?}: Pets PvP habitat {:?}", ident, h);
                                            cmd = Some(SFCommand::FightPetOpponent { habitat: h, opponent_id: pets.opponent.id });
                                        } else {
                                            log::debug!("Automation {:?}: Pets PvP ready but no eligible habitat (all battled or none with pets)", ident);
                                        }
                                    }

                                    if cmd.is_none() {
                                        let next_ready = pets.next_free_exploration.map(|t| t <= now).unwrap_or(true);
                                        let mut use_mush = false;
                                        let can_explore = if next_ready { true } else if cfg.max_mushrooms_pet_skip > 0 && gs.character.mushrooms > 0 { use_mush = true; true } else { false };
                                        if can_explore {
                                            log::debug!("Automation {:?}: Pets exploration free", ident);
                                            use strum::IntoEnumIterator;
                                            let mut pick: Option<(HabitatType, u32, u16, u32)> = None;
                                            for hab in HabitatType::iter() {
                                                if let HabitatExploration::Exploring { fights_won, .. } = pets.habitats.get(hab).exploration {
                                                    if let Some(best) = pets.habitats.get(hab).pets.iter().max_by_key(|p| p.level) {
                                                        let entry = (hab, fights_won + 1, best.level, best.id);
                                                        pick = match pick {
                                                            None => Some(entry),
                                                            Some((_, _, lvl, _)) if best.level > lvl => Some(entry),
                                                            x => x,
                                                        };
                                                    }
                                                }
                                            }
                                            if let Some((hab, enemy_pos, _best_lvl, best_id)) = pick {
                                                if use_mush { log::debug!("Automation {:?}: Pets exploration not ready, using mushroom to skip", ident); }
                                                log::debug!("Automation {:?}: Pets explore habitat {:?} fight_pos {} pet_id {}", ident, hab, enemy_pos, best_id);
                                                cmd = Some(SFCommand::FightPetDungeon { use_mush, habitat: hab, enemy_pos, player_pet_id: best_id });
                                            } else {
                                                log::debug!("Automation {:?}: Pets exploration ready but no habitat currently exploring", ident);
                                            }
                                        } else {
                                            log::debug!("Automation {:?}: Pets exploration not ready (next_free_exploration: {:?})", ident, pets.next_free_exploration);
                                        }
                                    }
                                }
                            }

                            if cmd.is_none() {
                                cmd = match gs.tavern.available_tasks() {
                                    AvailableTasks::Expeditions(_) if cfg.auto_expeditions => {
                                        if gs.tavern.questing_preference == ExpeditionSetting::PreferQuests
                                            && gs.tavern.can_change_questing_preference() {
                                            log::debug!("Automation {:?}: Switching to Expeditions", ident);
                                            Some(SFCommand::SetQuestsInsteadOfExpeditions { value: ExpeditionSetting::PreferExpeditions })
                                        } else if gs.tavern.thirst_for_adventure_sec > 0 {
                                            log::debug!("Automation {:?}: Starting Expedition 0", ident);
                                            Some(SFCommand::ExpeditionStart { pos: 0 })
                                        } else { None }
                                    }
                                    AvailableTasks::Quests(qs) if cfg.auto_tavern => {
                                        let pick_idx = {
                                            let mut best: Option<(usize, f64)> = None;
                                            for (i, q) in qs.iter().enumerate() {
                                                let minutes = (q.base_length.max(1) as f64) / 60.0;
                                                let gold = q.base_silver as f64;
                                                let xp = q.base_experience as f64;
                                                let score = match cfg.mission_strategy {
                                                    MissionStrategy::Shortest => -minutes,
                                                    MissionStrategy::MostGold => gold,
                                                    MissionStrategy::BestGoldPerMinute => { if minutes > 0.0 { gold / minutes } else { f64::MAX } }
                                                    MissionStrategy::BestXpPerMinute => { if minutes > 0.0 { xp / minutes } else { f64::MAX } }
                                                    MissionStrategy::Smartest => { let speed = 1.0 / minutes.max(1.0); 0.45 * (gold / minutes.max(1.0)) + 0.45 * (xp / minutes.max(1.0)) + 0.10 * speed }
                                                };
                                                log::trace!("Automation {:?}: Quest {} len={}s gold={} xp={} -> score={}", ident, i, q.base_length, q.base_silver, q.base_experience, score);
                                                match best { None => best = Some((i, score)), Some((_, s)) if score > s => best = Some((i, score)), _ => {} }
                                            }
                                            best.map(|a| a.0).unwrap_or(0)
                                        };
                                        let picked = &qs[pick_idx];
                                        if picked.base_length > gs.tavern.thirst_for_adventure_sec {
                                            let extra_beer = gs.character.equipment.has_enchantment(Enchantment::ThirstyWanderer) as u8;
                                            let beer_cap = 10 + extra_beer;
                                            if cfg.auto_buy_beer_mushrooms && cfg.max_mushrooms_beer > 0 && gs.character.mushrooms > 0 && gs.tavern.beer_drunk < beer_cap {
                                                log::debug!("Automation {:?}: Buying beer (drunk {}, cap {})", ident, gs.tavern.beer_drunk, beer_cap);
                                                Some(SFCommand::BuyBeer)
                                            } else {
                                                let mut alt_best: Option<(usize, f64)> = None;
                                                for (i, q) in qs.iter().enumerate() {
                                                    if q.base_length <= gs.tavern.thirst_for_adventure_sec {
                                                        let minutes = (q.base_length.max(1) as f64) / 60.0;
                                                        let gold = q.base_silver as f64;
                                                        let xp = q.base_experience as f64;
                                                        let score = match cfg.mission_strategy {
                                                            MissionStrategy::Shortest => -minutes,
                                                            MissionStrategy::MostGold => gold,
                                                            MissionStrategy::BestGoldPerMinute => { if minutes > 0.0 { gold / minutes } else { f64::MAX } }
                                                            MissionStrategy::BestXpPerMinute => { if minutes > 0.0 { xp / minutes } else { f64::MAX } }
                                                            MissionStrategy::Smartest => { let speed = 1.0 / minutes.max(1.0); 0.45 * (gold / minutes.max(1.0)) + 0.45 * (xp / minutes.max(1.0)) + 0.10 * speed }
                                                        };
                                                        match alt_best { None => alt_best = Some((i, score)), Some((_, s)) if score > s => alt_best = Some((i, score)), _ => {} }
                                                    }
                                                }
                                                if let Some((idx, _)) = alt_best {
                                                    let q = &qs[idx];
                                                    log::debug!("Automation {:?}: Fallback quest {} within thirst (len {}s)", ident, idx, q.base_length);
                                                    Some(SFCommand::StartQuest { quest_pos: idx, overwrite_inv: true })
                                                } else {
                                                    log::debug!("Automation {:?}: No quest fits remaining thirst ({}s) and not buying beer -> waiting", ident, gs.tavern.thirst_for_adventure_sec);
                                                    None
                                                }
                                            }
                                        } else {
                                            log::debug!("Automation {:?}: Starting quest {} (len {}s)", ident, pick_idx, picked.base_length);
                                            Some(SFCommand::StartQuest { quest_pos: pick_idx, overwrite_inv: true })
                                        }
                                    }
                                    _ => None,
                                };
                            }

                            // If thirst is empty and beer is unavailable, start 1h CityGuard before any Guild actions
                            if cmd.is_none() && (cfg.auto_tavern || cfg.auto_expeditions) {
                                let thirst = gs.tavern.thirst_for_adventure_sec;
                                if thirst == 0 {
                                    let extra_beer = gs.character.equipment.has_enchantment(Enchantment::ThirstyWanderer) as u8;
                                    let beer_cap = 10 + extra_beer;
                                    let beer_left = beer_cap.saturating_sub(gs.tavern.beer_drunk);
                                    let can_buy_more_beer = cfg.auto_buy_beer_mushrooms && cfg.max_mushrooms_beer > 0 && gs.character.mushrooms > 0 && gs.tavern.beer_drunk < beer_cap;
                                    if beer_left == 0 || !can_buy_more_beer {
                                        log::debug!("Automation {:?}: Thirst empty and beer exhausted/unavailable -> Start 1h CityGuard", ident);
                                        #[allow(unused_variables)]
                                        {
                                            if cmd.is_none() {
                                                cmd = Some(SFCommand::StartWork { hours: 1 });
                                            }
                                        }
                                    } else {
                                        log::debug!("Automation {:?}: Thirst empty but beer available (drunk {} / cap {}, mushrooms {}, auto_buy {}, beer_budget {}) -> no CityGuard", ident, gs.tavern.beer_drunk, beer_cap, gs.character.mushrooms, cfg.auto_buy_beer_mushrooms, cfg.max_mushrooms_beer);
                                    }
                                }
                            }

                            // Run Guild actions after Tavern/Expeditions and CityGuard decision so primary tasks aren't starved
                            if cmd.is_none() && cfg.auto_guild {
                                if gs.guild.is_some() && cfg.auto_guild_accept_defense {
                                    log::debug!("Automation {:?}: Guild join defense", ident);
                                    cmd = Some(SFCommand::GuildJoinDefense);
                                }
                                if cmd.is_none() && gs.guild.is_some() && cfg.auto_guild_accept_attack {
                                    log::debug!("Automation {:?}: Guild join attack", ident);
                                    cmd = Some(SFCommand::GuildJoinAttack);
                                }
                                if cmd.is_none() && cfg.auto_guild_hydra {
                                    if let Some(guild) = &gs.guild {
                                        if guild.hydra.remaining_fights > 0 {
                                            if let Some(next) = guild.hydra.next_battle {
                                                if next <= now {
                                                    log::debug!("Automation {:?}: Guild hydra battle", ident);
                                                    cmd = Some(SFCommand::GuildPetBattle { use_mushroom: false });
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            if cmd.is_none() {
                                let portal = gs.dungeons.portal.as_ref().map(|p| p.can_fight).unwrap_or(false);
                                let dng_ready = gs.dungeons.next_free_fight.map(|t| t <= now).unwrap_or(true);
                                let open_dng = {
                                    let mut open = 0u32;
                                    for d in LightDungeon::iter() {
                                        if let DungeonProgress::Open { .. } = gs.dungeons.progress(d) { open += 1; }
                                    }
                                    for d in ShadowDungeon::iter() {
                                        if let DungeonProgress::Open { .. } = gs.dungeons.progress(d) { open += 1; }
                                    }
                                    open
                                };
                                let pets_pvp_ready = gs.pets.as_ref().and_then(|p| p.opponent.next_free_battle).map(|t| t <= now).unwrap_or(false);
                                let pets_explore_ready = gs.pets.as_ref().and_then(|p| p.next_free_exploration).map(|t| t <= now).unwrap_or(false);
                                let hydra_ready = gs.guild.as_ref().and_then(|g| g.hydra.next_battle).map(|t| t <= now).unwrap_or(false);
                                let thirst = gs.tavern.thirst_for_adventure_sec;
                                log::debug!(
                                    "Automation {:?}: No action chosen. Summary -> portal: {}, dng_ready: {}, open_dng: {}, pets_pvp: {}, pets_explore: {}, hydra: {}, thirst: {}s",
                                    ident, portal, dng_ready, open_dng, pets_pvp_ready, pets_explore_ready, hydra_ready, thirst
                                );
                            }

                            cmd
                        }
                    }
                };

                // Allow side-actions (dungeons/pets/guild) to run even while Tavern is busy
                let mut cmd = next_cmd;
                if cmd.is_none() {
                    // Try Dungeons first
                    if cfg.auto_dungeons {
                        if let Some(portal) = &gs.dungeons.portal {
                            if portal.can_fight {
                                log::debug!("Automation {:?}: Portal fight ready (side-action)", ident);
                                cmd = Some(SFCommand::FightPortal);
                            }
                        }
                        if cmd.is_none() {
                            let next_ready = gs.dungeons.next_free_fight.map(|t| t <= now).unwrap_or(true);
                            let mut use_mush = false;
                            let can_fight_now = if next_ready { true } else if cfg.max_mushrooms_dungeon_skip > 0 && gs.character.mushrooms > 0 { use_mush = true; true } else { false };
                            if can_fight_now {
                                use sf_api::gamestate::dungeons::{LightDungeon, ShadowDungeon, DungeonProgress};
                                if let DungeonProgress::Open { finished } = gs.dungeons.progress(LightDungeon::Tower) {
                                    log::debug!("Automation {:?}: Tower ready at level {} (side-action)", ident, finished);
                                    cmd = Some(SFCommand::FightTower { current_level: finished as u8, use_mush });
                                } else {
                                    use strum::IntoEnumIterator;
                                    let mut best: Option<(sf_api::gamestate::dungeons::Dungeon, u16)> = None;
                                    for d in LightDungeon::iter() {
                                        if d == LightDungeon::Tower { continue; }
                                        if let DungeonProgress::Open { finished } = gs.dungeons.progress(d) {
                                            let entry = (sf_api::gamestate::dungeons::Dungeon::from(d), finished);
                                            best = match best { None => Some(entry), Some((_, f)) if finished < f => Some(entry), x => x };
                                        }
                                    }
                                    for d in ShadowDungeon::iter() {
                                        if let DungeonProgress::Open { finished } = gs.dungeons.progress(d) {
                                            let entry = (sf_api::gamestate::dungeons::Dungeon::from(d), finished);
                                            best = match best { None => Some(entry), Some((_, f)) if finished < f => Some(entry), x => x };
                                        }
                                    }
                                    if let Some((dng, _)) = best {
                                        log::debug!("Automation {:?}: Dungeon chosen (side-action): {:?}", ident, dng);
                                        cmd = Some(SFCommand::FightDungeon { dungeon: dng, use_mushroom: use_mush });
                                    }
                                }
                            }
                        }
                    }
                    // Try Pets next if still none
                    if cmd.is_none() && cfg.auto_pets {
                        if let Some(pets) = &gs.pets {
                            // Prefer PvP if any habitat has not battled opponent yet, else exploration timer
                            use sf_api::gamestate::unlockables::HabitatType;
                            use strum::IntoEnumIterator;
                            let mut any_pvp_left = false;
                            for h in HabitatType::iter() {
                                let hab = pets.habitats.get(h);
                                if !hab.battled_opponent { any_pvp_left = true; break; }
                            }
                            if any_pvp_left {
                                let free_now = pets.opponent.next_free_battle.map(|t| t <= now).unwrap_or(true);
                                if free_now {
                                    // Choose a habitat for PvP
                                    let mut target_hab: Option<HabitatType> = None;
                                    if let Some(h) = pets.opponent.habitat { if !pets.habitats.get(h).battled_opponent { target_hab = Some(h); } }
                                    if target_hab.is_none() {
                                        let mut best: Option<(HabitatType, u16)> = None;
                                        for h in HabitatType::iter() {
                                            let hab = pets.habitats.get(h);
                                            if hab.battled_opponent { continue; }
                                            if let Some(p) = hab.pets.iter().max_by_key(|p| p.level) {
                                                best = match best { None => Some((h, p.level)), Some((_, lvl)) if p.level > lvl => Some((h, p.level)), x => x };
                                            }
                                        }
                                        if let Some((h, _)) = best { target_hab = Some(h); }
                                    }
                                    if let Some(h) = target_hab {
                                        log::debug!("Automation {:?}: Pets PvP habitat {:?} (side-action)", ident, h);
                                        cmd = Some(SFCommand::FightPetOpponent { habitat: h, opponent_id: pets.opponent.id });
                                    }
                                }
                            } else {
                                // No PvP left; consider exploration if ready
                                let next_ready = pets.next_free_exploration.map(|t| t <= now).unwrap_or(true);
                                let mut use_mush = false;
                                let can_explore = if next_ready { true } else if cfg.max_mushrooms_pet_skip > 0 && gs.character.mushrooms > 0 { use_mush = true; true } else { false };
                                if can_explore {
                                    let mut pick: Option<(HabitatType, u32, u16, u32)> = None;
                                    for hab in HabitatType::iter() {
                                        if let sf_api::gamestate::unlockables::HabitatExploration::Exploring { fights_won, .. } = pets.habitats.get(hab).exploration {
                                            if let Some(best) = pets.habitats.get(hab).pets.iter().max_by_key(|p| p.level) {
                                                let entry = (hab, fights_won + 1, best.level, best.id);
                                                pick = match pick { None => Some(entry), Some((_, _, lvl, _)) if best.level > lvl => Some(entry), x => x };
                                            }
                                        }
                                    }
                                    if let Some((hab, enemy_pos, _best_lvl, best_id)) = pick {
                                        log::debug!("Automation {:?}: Pets explore habitat {:?} fight_pos {} pet_id {} (side-action)", ident, hab, enemy_pos, best_id);
                                        cmd = Some(SFCommand::FightPetDungeon { use_mush, habitat: hab, enemy_pos, player_pet_id: best_id });
                                    }
                                }
                            }
                        }
                    }
                    // Try Guild hydra last
                    if cmd.is_none() && cfg.auto_guild {
                        if let Some(guild) = &gs.guild {
                            if cfg.auto_guild_hydra && guild.hydra.remaining_fights > 0 {
                                if let Some(next) = guild.hydra.next_battle { if next <= now { cmd = Some(SFCommand::GuildPetBattle { use_mushroom: false }); } }
                            }
                        }
                    }
                }

                let cmd = cmd.unwrap_or(SFCommand::Update);
                log::debug!("Automation {:?}: chosen command: {:?}", ident, cmd);

                // Try to acquire a session. If it's temporarily busy (e.g., AutoPoll), don't try to relog; just retry shortly.
                let Some(mut session) = status.take_session("Automation") else {
                    // Queue actionable commands if session is busy; skip queuing plain Update
                    if !matches!(cmd, SFCommand::Update) {
                        // Enforce exclusivity: only one primary Tavern/Expedition/CityGuard command
                        // can be queued at a time. Side-actions (dungeons/pets/guild) are not considered primary.
                        use sf_api::command::Command as SFCommand;
                        let is_primary = |c: &SFCommand| -> bool {
                            matches!(
                                c,
                                // Tavern / Quests
                                SFCommand::StartQuest { .. }
                                    | SFCommand::FinishQuest { .. }
                                    | SFCommand::BuyBeer
                                    | SFCommand::SetQuestsInsteadOfExpeditions { .. }
                                    // Expeditions
                                    | SFCommand::ExpeditionStart { .. }
                                    | SFCommand::ExpeditionContinue
                                    | SFCommand::ExpeditionPickEncounter { .. }
                                    | SFCommand::ExpeditionPickReward { .. }
                                    | SFCommand::ExpeditionSkipWait { .. }
                                    // CityGuard (CityWatch)
                                    | SFCommand::StartWork { .. }
                                    | SFCommand::FinishWork
                            )
                        };

                        if is_primary(&cmd)
                            && account
                                .automation_queue
                                .iter()
                                .any(|q| is_primary(q))
                        {
                            log::debug!(
                                "Automation {:?}: session busy; NOT queueing {:?} because a primary task is already queued (len={})",
                                ident,
                                cmd,
                                account.automation_queue.len()
                            );
                        } else {
                            account.automation_queue.push(cmd.clone());
                            log::debug!(
                                "Automation {:?}: session busy; queueing {:?} (queue_len={})",
                                ident,
                                cmd,
                                account.automation_queue.len()
                            );
                        }
                    } else {
                        log::debug!("Automation {:?}: session busy; skipping Update", ident);
                    }
                    drop(status);
                    let rerun = Command::perform(
                        async move {
                            tokio::time::sleep(std::time::Duration::from_millis(fastrand::u64(400..=1200))).await;
                        },
                        move |_| Message::RunAutomationTick { ident }
                    );
                    return rerun;
                };
                let player_status = account.status.clone();
                let chosen_cmd = cmd.clone();
                drop(status);

                return Command::perform(
                    async move {
                        let resp = session.send_command(&cmd).await;
                        (resp, session)
                    },
                    move |r| match r.0 {
                        Ok(resp) => {
                            log::debug!("Automation {:?}: {:?} response: {:?}", ident, chosen_cmd, resp);
                            let mut lock = player_status.lock().unwrap();
                            let gs = match &mut *lock {
                                AccountStatus::Busy(gs, _) => gs,
                                _ => {
                                    lock.put_session(r.1);
                                    return Message::PlayerNotPolled { ident };
                                }
                            };
                            if gs.update(resp).is_err() {
                                return Message::PlayerCommandFailed {
                                    ident,
                                    session: r.1,
                                    attempt: 0,
                                };
                            }
                            {
                                use strum::IntoEnumIterator;
                                use sf_api::gamestate::dungeons::{LightDungeon, ShadowDungeon, DungeonProgress};
                                let portal = gs.dungeons.portal.as_ref().map(|p| p.can_fight).unwrap_or(false);
                                let dng_next = gs.dungeons.next_free_fight;
                                let open_count = {
                                    let mut c = 0u32;
                                    for d in LightDungeon::iter() {
                                        if let DungeonProgress::Open { .. } = gs.dungeons.progress(d) { c += 1; }
                                    }
                                    for d in ShadowDungeon::iter() {
                                        if let DungeonProgress::Open { .. } = gs.dungeons.progress(d) { c += 1; }
                                    }
                                    c
                                };
                                let pets_pvp = gs.pets.as_ref().and_then(|p| p.opponent.next_free_battle);
                                let pets_exp = gs.pets.as_ref().and_then(|p| p.next_free_exploration);
                                let hydra = gs.guild.as_ref().map(|g| (g.hydra.remaining_fights, g.hydra.next_battle));
                                log::debug!(
                                    "Automation {:?}: post-update snapshot -> portal: {}, dng_next: {:?}, open_dng: {}, pets_pvp: {:?}, pets_exp: {:?}, hydra: {:?}",
                                    ident, portal, dng_next, open_count, pets_pvp, pets_exp, hydra
                                );
                            }
                            lock.put_session(r.1);
                            Message::PlayerPolled { ident }
                        }
                        Err(e) => {
                            log::error!("Automation {:?}: {:?} failed: {:?}", ident, chosen_cmd, e);
                            Message::PlayerCommandFailed {
                                ident,
                                session: r.1,
                                attempt: 0,
                            }
                        },
                    },
                );
            }
            Message::PageCrawled => {}
            Message::CrawlerDied { server, error } => {
                log::error!("Crawler died on {server} - {error}");
                let Some(server) = self.servers.get_mut(&server) else {
                    return Command::none();
                };
                server.crawling = CrawlingStatus::CrawlingFailed(error)
            }
            Message::CharacterCrawled {
                server,
                que_id,
                character,
            } => {
                let Some(server) = self.servers.get_mut(&server) else {
                    return Command::none();
                };

                trace!("{} crawled {}", server.ident.ident, character.name);

                let CrawlingStatus::Crawling {
                    player_info,
                    equipment,
                    que_id: crawl_que_id,
                    last_update,
                    que,
                    recent_failures,
                    naked,
                    ..
                } = &mut server.crawling
                else {
                    return Command::none();
                };

                let crawler_finished = {
                    let mut lock = que.lock().unwrap();
                    if let Some(pb) = &server.headless_progress {
                        let remaining = lock.count_remaining();
                        let crawled = player_info.len();
                        let total = remaining + crawled;
                        pb.set_length(total as u64);
                        pb.set_position(crawled as u64);
                    };
                    lock.in_flight_accounts.remove(&character.name);
                    lock.todo_pages.is_empty() && lock.todo_accounts.is_empty()
                };

                if *crawl_que_id != que_id {
                    return Command::none();
                }

                recent_failures.clear();
                *last_update = Local::now();

                handle_new_char_info(character, equipment, player_info, naked);

                if crawler_finished {
                    let mut commands = vec![];
                    let todo: Vec<_> =
                        server.accounts.values().map(|a| a.ident).collect();
                    for acc in todo {
                        commands.push(self.update_best(acc, false));
                    }
                    return Command::batch(commands);
                }

                if let View::Account { ident, .. } = self.current_view
                    && let Some(current) =
                        server.accounts.get_mut(&ident.account)
                {
                    let ident = current.ident;
                    return self.update_best(ident, true);
                }
            }
            Message::CrawlerIdle(server_id) => {
                let Some(server) = self.servers.get_mut(&server_id) else {
                    return Command::none();
                };
                let CrawlingStatus::Crawling {
                    player_info, que, ..
                } = &mut server.crawling
                else {
                    return Command::none();
                };
                let lock = que.lock().unwrap();
                if server.headless_progress.is_none()
                    || !lock.todo_pages.is_empty()
                    || !lock.todo_accounts.is_empty()
                    || player_info.is_empty()
                {
                    return Command::none();
                }
                let backup = lock.create_backup(player_info);
                let ident = server.ident.ident.to_string();
                let id = server.ident.id;

                return Command::perform(
                    async move { backup.write(&ident).await },
                    move |res| Message::BackupRes {
                        server: id,
                        error: res.err().map(|a| a.to_string()),
                    },
                );
            }
            Message::CrawlerNoPlayerResult => {
                warn!("No player result");
            }
            Message::CrawlerUnable {
                server: server_id,
                action,
                error,
            } => {
                let Some(server) = self.servers.get_mut(&server_id) else {
                    return Command::none();
                };
                let CrawlingStatus::Crawling {
                    que_id,
                    que,
                    recent_failures,
                    crawling_session,
                    ..
                } = &mut server.crawling
                else {
                    return Command::none();
                };

                let mut lock = que.lock().unwrap();
                match &action {
                    CrawlAction::Wait | CrawlAction::InitTodo => {}
                    CrawlAction::Page(a, b) => {
                        if *b != *que_id {
                            return Command::none();
                        }
                        lock.in_flight_pages.retain(|x| x != a);
                        if error == CrawlerError::RateLimit {
                            lock.todo_pages.push(*a);
                            return Command::none();
                        } else {
                            lock.invalid_pages.push(*a);
                        }
                    }
                    CrawlAction::Character(a, b) => {
                        if *b != *que_id {
                            return Command::none();
                        }
                        lock.in_flight_accounts.remove(a);
                        if error == CrawlerError::RateLimit {
                            lock.todo_accounts.push(a.clone());
                            return Command::none();
                        } else {
                            lock.invalid_accounts.push(a.clone());
                        }
                    }
                }

                match error {
                    CrawlerError::NotFound => {
                        return Command::none();
                    }
                    CrawlerError::Generic(err) => warn!(
                        "Crawler was unable to complete: '{action}' on {} -> \
                         {err}",
                        server.ident.id
                    ),
                    CrawlerError::RateLimit => {}
                }

                recent_failures.push(action);

                if recent_failures.len() != 10 {
                    return Command::none();
                }
                debug!("Restarting crawler on {}", server.ident.ident);

                let Some(state) = crawling_session.clone() else {
                    return Command::none();
                };

                let id = server.ident.ident.clone();

                return Command::perform(
                    async move {
                        let mut session_lock = state.session.write().await;
                        loop {
                            debug!("Relog crawler on {}", id);
                            let Ok(resp) = session_lock.login().await else {
                                error!("Could not login crawler on {}", id);
                                sleep(Duration::from_millis(fastrand::u64(
                                    1000..3000,
                                )))
                                .await;
                                continue;
                            };
                            let Ok(new_gs) = GameState::new(resp) else {
                                error!(
                                    "Could not parse GS for crawler on {}",
                                    id
                                );
                                sleep(Duration::from_millis(fastrand::u64(
                                    1000..3000,
                                )))
                                .await;
                                continue;
                            };
                            sleep(Duration::from_secs(5)).await;

                            let mut gs = state.gs.lock().unwrap();
                            *gs = new_gs;
                            return;
                        }
                    },
                    move |()| Message::CrawlerRevived { server_id },
                );
            }
            Message::ViewLogin => self.current_view = View::Login,
            Message::LoginNameInputChange(a) => self.login_state.name = a,
            Message::LoginSFSubmit => {
                return self.login_sf_acc(
                    self.login_state.name.clone(),
                    PWHash::new(&self.login_state.password),
                    self.login_state.remember_me,
                    false,
                );
            }
            Message::LoginPWInputChange(a) => self.login_state.password = a,
            Message::LoginServerChange(a) => self.login_state.server = a,
            Message::LoginRegularSubmit => {
                let pw_hash = PWHash::new(&self.login_state.password.clone());

                return self.login_regular(
                    self.login_state.name.to_string(),
                    self.login_state.server.to_string(),
                    pw_hash,
                    self.login_state.remember_me,
                    Default::default(),
                );
            }
            Message::LoginViewChanged(a) => {
                self.login_state.login_typ = a;
            }
            Message::LoggininSuccess {
                gs,
                session,
                remember,
                ident,
            } => {
                info!("Successfully logged in {ident}");

                let Some(server) = self.servers.0.get_mut(&ident.server_id)
                else {
                    return Command::none();
                };
                let Some(player) = server.accounts.get_mut(&ident.account)
                else {
                    return Command::none();
                };

                if remember {
                    match &player.auth {
                        PlayerAuth::Normal(hash) => {
                            self.config.accounts.retain(|a| match &a {
                                AccountConfig::Regular {
                                    name,
                                    server: server_url,
                                    ..
                                } => {
                                    !(name == &player.name
                                        && server_url == &server.ident.url)
                                }
                                _ => true,
                            });
                            self.config.accounts.push(AccountConfig::new(
                                AccountCreds::Regular {
                                    name: player.name.clone(),
                                    pw_hash: hash.clone(),
                                    server: server.ident.url.clone(),
                                },
                            ));
                            _ = self.config.write();
                        }
                        PlayerAuth::SSO => {}
                    }
                }

                let total_players = gs.hall_of_fames.players_total;
                let total_pages = (total_players as usize).div_ceil(PER_PAGE);

                let char_conf =
                    self.config.get_char_conf(&player.name, ident.server_id);

                player.scrapbook_info = ScrapbookInfo::new(&gs, char_conf);
                player.underworld_info = UnderworldInfo::new(&gs, char_conf);

                *player.status.lock().unwrap() =
                    AccountStatus::Idle(session, gs);

                let server_ident = server.ident.ident.clone();
                let server_id = server.ident.id;
                let afn = self.config.auto_fetch_newest;
                match &server.crawling {
                    CrawlingStatus::Waiting => {
                        server.crawling = CrawlingStatus::Restoring;
                        return Command::perform(
                            async move {
                                let backup =
                                    get_newest_backup(server_ident, afn).await;
                                Box::new(
                                    restore_backup(backup, total_pages).await,
                                )
                            },
                            move |backup| Message::ResetCrawling {
                                server: server_id,
                                status: backup,
                            },
                        );
                    }
                    CrawlingStatus::Crawling { .. } => {
                        let ident = player.ident;
                        return self.update_best(ident, false);
                    }
                    _ => (),
                }
            }
            Message::LoggininFailure { error, ident } => {
                error!("Error loggin in {ident}: {error}");
                let Some((_, player)) = self.servers.get_ident(&ident) else {
                    return Command::none();
                };
                *player.status.lock().unwrap() =
                    AccountStatus::FatalError(error)
            }
            Message::ShowPlayer { ident } => {
                let Some(server) = self.servers.0.get_mut(&ident.server_id)
                else {
                    return Command::none();
                };
                let Some(account) = server.accounts.get_mut(&ident.account)
                else {
                    return Command::none();
                };

                self.current_view = View::Account {
                    ident,
                    page: AccountPage::Scrapbook,
                };

                let CrawlingStatus::Crawling { last_update, .. } =
                    &server.crawling
                else {
                    return Command::none();
                };

                if account.last_updated < *last_update {
                    let ident = account.ident;
                    return self.update_best(ident, false);
                }
            }
            Message::ResetCrawling {
                server: server_id,
                status,
            } => {
                let Some(server) = self.servers.get_mut(&server_id) else {
                    return Command::none();
                };

                let mut commands = vec![];
                match &mut server.crawling {
                    CrawlingStatus::Waiting | CrawlingStatus::Restoring => {
                        server.crawling = status.into_status();
                        commands.push(server.set_threads(
                            self.config.start_threads, &self.config.base_name,
                        ));
                    }
                    CrawlingStatus::Crawling {
                        que_id,
                        que,
                        player_info,
                        equipment,
                        last_update,
                        recent_failures,
                        naked,
                        threads: _,
                        crawling_session: _,
                    } => {
                        let mut que = que.lock().unwrap();
                        que.que_id = status.que_id;
                        que.todo_accounts = status.todo_accounts;
                        que.todo_pages = status.todo_pages;
                        que.invalid_accounts = status.invalid_accounts;
                        que.invalid_pages = status.invalid_pages;
                        que.order = status.order;
                        que.in_flight_pages = vec![];
                        que.in_flight_accounts = Default::default();
                        *que_id = status.que_id;
                        *naked = status.naked;
                        *player_info = status.player_info;
                        *equipment = status.equipment;
                        *last_update = Local::now();
                        recent_failures.clear();
                        drop(que);
                    }
                    CrawlingStatus::CrawlingFailed(_) => {
                        return Command::none();
                    }
                }

                let CrawlingStatus::Crawling { .. } = &server.crawling else {
                    return Command::none();
                };

                let todo: Vec<_> =
                    server.accounts.values().map(|a| a.ident).collect();
                for acc in todo {
                    commands.push(self.update_best(acc, false));
                }
                return Command::batch(commands);
            }
            Message::RemoveAccount { ident } => {
                let Some(server) = self.servers.get_mut(&ident.server_id)
                else {
                    return Command::none();
                };
                if let Some(old) = server.accounts.remove(&ident.account)
                    && matches!(old.auth, PlayerAuth::SSO)
                    && let Ok(mut sl) = old.status.lock()
                    && let Some(session) = sl.take_session("Removing")
                {
                    self.login_state.import_que.push(*session);
                }
                if server.accounts.is_empty()
                    && let CrawlingStatus::Crawling { threads, .. } =
                        &mut server.crawling
                {
                    *threads = 0;
                }

                match &mut self.current_view {
                    View::Account { ident: current, .. }
                        if ident == *current =>
                    {
                        self.current_view = View::Login;
                    }
                    View::Overview { selected, action } => {
                        _ = selected.remove(&ident);
                        *action = None;
                    }
                    _ => {}
                }
            }
            Message::CrawlerSetThreads {
                server: server_id,
                new_count,
            } => {
                let new_count = new_count.clamp(0, self.config.max_threads);
                let Some(server) = self.servers.get_mut(&server_id) else {
                    return Command::none();
                };

                return server.set_threads(new_count, &self.config.base_name);
            }
            Message::ClearHof(server_id) => {
                let Some(server) = self.servers.get_mut(&server_id) else {
                    return Command::none();
                };

                let Some(tp) = server.accounts.iter().find_map(|(_, b)| {
                    match &*b.status.lock().unwrap() {
                        AccountStatus::LoggingInAgain
                        | AccountStatus::LoggingIn
                        | AccountStatus::FatalError(_) => None,
                        AccountStatus::Idle(_, gs)
                        | AccountStatus::Busy(gs, _) => {
                            Some(gs.hall_of_fames.players_total)
                        }
                    }
                }) else {
                    return Command::none();
                };

                let tp = (tp as usize).div_ceil(PER_PAGE);

                let id = server.ident.id;

                return Command::perform(
                    async move { Box::new(restore_backup(None, tp).await) },
                    move |res| Message::ResetCrawling {
                        server: id,
                        status: res,
                    },
                );
            }
            Message::RememberMe(val) => self.login_state.remember_me = val,
            Message::Login {
                account,
                auto_login,
            } => match account {
                AccountConfig::Regular {
                    name,
                    pw_hash,
                    server,
                    ..
                } => {
                    return self.login_regular(
                        name, server, pw_hash, false, auto_login,
                    );
                }
                AccountConfig::SF { name, pw_hash, .. } => {
                    return self.login_sf_acc(name, pw_hash, false, auto_login);
                }
            },
            Message::OrderChange { server, new } => {
                let Some(server) = self.servers.get_mut(&server) else {
                    return Command::none();
                };
                if let CrawlingStatus::Crawling { que, .. } = &server.crawling {
                    let mut que = que.lock().unwrap();
                    que.order = new;
                    new.apply_order(&mut que.todo_pages);
                }
            }
            Message::AutoBattlePossible { ident } => {
                let refetch = self.update_best(ident, true);

                let Some(server) = self.servers.0.get_mut(&ident.server_id)
                else {
                    return Command::none();
                };
                let Some(account) = server.accounts.get_mut(&ident.account)
                else {
                    return Command::none();
                };

                let CrawlingStatus::Crawling { .. } = &server.crawling else {
                    return Command::none();
                };

                let mut status = account.status.lock().unwrap();
                let AccountStatus::Idle(_, gs) = &*status else {
                    return refetch;
                };
                let next = gs.arena.next_free_fight.unwrap_or_default();
                if next > Local::now() + Duration::from_millis(200) {
                    return refetch;
                }

                let Some(mut session) = status.take_session("A Fighting")
                else {
                    return refetch;
                };

                let Some(si) = &account.scrapbook_info else {
                    status.put_session(session);
                    return refetch;
                };

                let total_len = si.best.len();
                let new_len = si.best.iter().filter(|a| !a.is_old()).count();

                if total_len == 0 || (new_len as f32 / total_len as f32) < 0.9 {
                    status.put_session(session);
                    return refetch;
                }

                let Some(target) =
                    si.best.iter().find(|a| !a.is_old()).cloned()
                else {
                    status.put_session(session);
                    return refetch;
                };
                drop(status);

                let tn = target.info.name.clone();
                let fight = Command::perform(
                    async move {
                        let cmd = sf_api::command::Command::Fight {
                            name: tn,
                            use_mushroom: false,
                        };
                        let resp = session.send_command(&cmd).await;
                        (resp, session)
                    },
                    move |r| match r.0 {
                        Ok(resp) => Message::PlayerAttackResult {
                            ident,
                            session: r.1,
                            against: target,
                            resp: Box::new(resp),
                        },
                        Err(_) => Message::PlayerCommandFailed {
                            ident,
                            session: r.1,
                            attempt: 0,
                        },
                    },
                );

                return Command::batch([refetch, fight]);
            }
            Message::PlayerCommandFailed {
                ident,
                mut session,
                attempt,
            } => {
                let Some(server) = self.servers.0.get_mut(&ident.server_id)
                else {
                    return Command::none();
                };
                let Some(player) = server.accounts.get_mut(&ident.account)
                else {
                    return Command::none();
                };

                let mut lock = player.status.lock().unwrap();
                *lock = AccountStatus::LoggingInAgain;
                drop(lock);
                warn!("Logging in {ident} again");
                return Command::perform(
                    async move {
                        let Ok(resp) = session.login().await else {
                            sleep(Duration::from_secs(5)).await;
                            return Err(session);
                        };
                        let Ok(gamestate) = GameState::new(resp) else {
                            sleep(Duration::from_secs(5)).await;
                            return Err(session);
                        };
                        sleep(Duration::from_secs(attempt)).await;
                        Ok((Box::new(gamestate), session))
                    },
                    move |res| match res {
                        Ok((gs, session)) => {
                            Message::PlayerRelogSuccess { ident, gs, session }
                        }
                        Err(session) => Message::PlayerCommandFailed {
                            ident,
                            session,
                            attempt: attempt + 1,
                        },
                    },
                );
            }
            Message::PlayerAttackResult {
                ident,
                session,
                against,
                resp,
            } => {
                let Some(server) = self.servers.0.get_mut(&ident.server_id)
                else {
                    return Command::none();
                };
                let Some(account) = server.accounts.get_mut(&ident.account)
                else {
                    return Command::none();
                };

                let v = account.status.clone();
                let mut lock = v.lock().unwrap();

                let AccountStatus::Busy(s, _) = &mut *lock else {
                    return Command::none();
                };

                if let Err(e) = s.update(*resp) {
                    *lock = AccountStatus::FatalError(e.to_string());
                    return Command::none();
                };

                let Some(last) = &s.last_fight else {
                    return Command::none();
                };

                let nt = against.info.name.clone();
                let ut = against.info.uid;

                let Some(si) = &mut account.scrapbook_info else {
                    return Command::none();
                };

                if last.has_player_won {
                    for new in &against.info.equipment {
                        si.scrapbook.items.insert(*new);
                    }
                }

                si.attack_log.push((
                    Local::now(),
                    against,
                    last.has_player_won,
                ));

                let mut res = Command::none();

                if !last.has_player_won {
                    si.blacklist.entry(ut).or_insert((nt, 0)).1 += 1;
                } else if let CrawlingStatus::Crawling { .. } = &server.crawling
                {
                    let ident = account.ident;
                    res = self.update_best(ident, false);
                }

                lock.put_session(session);
                return res;
            }
            Message::AutoBattle { ident, state } => {
                let Some(server) = self.servers.0.get_mut(&ident.server_id)
                else {
                    return Command::none();
                };
                let Some(player) = server.accounts.get_mut(&ident.account)
                else {
                    return Command::none();
                };

                let Some(si) = &mut player.scrapbook_info else {
                    return Command::none();
                };

                si.auto_battle = state;
            }
            Message::CrawlerStartup { server, state } => {
                let Some(server) = self.servers.get_mut(&server) else {
                    return Command::none();
                };

                let CrawlingStatus::Crawling {
                    crawling_session, ..
                } = &mut server.crawling
                else {
                    return Command::none();
                };
                *crawling_session = Some(state);
            }
            Message::CrawlerRevived { server_id } => {
                info!("Crawler revived");
                let Some(server) = self.servers.get_mut(&server_id) else {
                    return Command::none();
                };
                let CrawlingStatus::Crawling {
                    que,
                    recent_failures,
                    ..
                } = &mut server.crawling
                else {
                    return Command::none();
                };

                let mut que = que.lock().unwrap();

                let mut ok_pages = vec![];
                let mut ok_character = vec![];
                for action in recent_failures.drain(..) {
                    match action {
                        CrawlAction::Wait | CrawlAction::InitTodo => {}
                        CrawlAction::Page(page, que_id) => {
                            if que_id != que.que_id {
                                continue;
                            }
                            ok_pages.push(page);
                        }
                        CrawlAction::Character(name, que_id) => {
                            if que_id != que.que_id {
                                continue;
                            }
                            ok_character.push(name);
                        }
                    }
                }

                que.invalid_pages.retain(|a| !ok_pages.contains(a));
                que.invalid_accounts.retain(|a| !ok_character.contains(a));
                que.todo_accounts.append(&mut ok_character);
                que.todo_pages.append(&mut ok_pages);
            }
            Message::ViewOverview => {
                self.current_view = View::Overview {
                    selected: Default::default(),
                    action: Default::default(),
                };
            }
            Message::ChangeTheme(theme) => {
                self.config.theme = theme;
                _ = self.config.write();
            }
            Message::ConfigSetUseTavernGlasses { name, server, nv } => {
                if let Some(cc) = self.config.get_char_conf_mut(&name, server)
                {
                    cc.use_glasses_for_tavern = nv;
                    let _ = self.config.write();
                }
            }
            Message::ConfigSetUseExpeditionGlasses { name, server, nv } => {
                if let Some(cc) = self.config.get_char_conf_mut(&name, server)
                {
                    cc.use_glasses_for_expeditions = nv;
                    let _ = self.config.write();
                }
            }
            Message::ConfigSetExpeditionRewardPriority { name, server, nv } => {
                if let Some(cc) = self.config.get_char_conf_mut(&name, server)
                {
                    cc.expedition_reward_priority = nv;
                    let _ = self.config.write();
                }
            }
            Message::ViewSettings => {
                self.current_view = View::Settings;
            }
            Message::SSOLoginSuccess {
                name,
                pass,
                mut chars,
                remember,
                auto_login,
            } => {
                let ident = SSOIdent::SF(name.clone());

                let Some(res) = self
                    .login_state
                    .active_sso
                    .iter_mut()
                    .find(|a| a.ident == ident)
                else {
                    return Command::none();
                };
                if remember {
                    self.config.accounts.retain(|a| match &a {
                        AccountConfig::Regular { .. } => true,
                        AccountConfig::SF { name: uuu, .. } => {
                            name.to_lowercase() != uuu.to_lowercase()
                        }
                    });

                    self.config.accounts.push(AccountConfig::SF {
                        name: name.clone(),
                        pw_hash: pass,
                        characters: chars
                            .iter()
                            .map(|a| SFAccCharacter {
                                config: CharacterConfig::default(),
                                ident: SFCharIdent {
                                    name: a.username().to_string(),
                                    server: a.server_url().as_str().to_string(),
                                },
                            })
                            .collect(),
                    });
                    _ = self.config.write();
                }

                if let Some(existing) = self.config.get_sso_accounts_mut(&name)
                {
                    let mut new: HashSet<(ServerIdent, String)> =
                        HashSet::new();
                    for char in &chars {
                        let name = char.username().trim().to_lowercase();
                        new.insert((
                            ServerIdent::new(char.server_url().as_str()),
                            name,
                        ));
                    }

                    let mut modified = false;

                    existing.retain(|a| {
                        let res = new.remove(&(
                            ServerIdent::new(&a.ident.server),
                            a.ident.name.trim().to_lowercase(),
                        ));
                        if !res {
                            modified = true;
                            info!("Removed a SSO char")
                        }
                        res
                    });

                    for (server, name) in new {
                        modified = true;
                        info!("Registered a a new SSO chars");
                        existing.push(SFAccCharacter {
                            config: CharacterConfig::default(),
                            ident: SFCharIdent {
                                name,
                                server: server.url.to_string(),
                            },
                        })
                    }

                    if modified {
                        _ = self.config.write();
                    }
                }

                self.login_state.import_que.append(&mut chars);

                res.status = SSOLoginStatus::Success;
                if auto_login {
                    for acc in &self.config.accounts {
                        let AccountConfig::SF {
                            name: s_name,
                            characters,
                            ..
                        } = acc
                        else {
                            continue;
                        };
                        if s_name != &name {
                            continue;
                        }
                        let mut commands = vec![];
                        for SFAccCharacter { ident, config } in characters {
                            if !config.login {
                                continue;
                            }
                            let ident = ident.clone();
                            commands
                                .push(Command::perform(async {}, move |_| {
                                    Message::SSOImportAuto { ident }
                                }))
                        }
                        return Command::batch(commands);
                    }
                }

                if self.current_view == View::Login
                    && self.login_state.login_typ == LoginType::SSOAccounts
                {
                    self.login_state.login_typ = LoginType::SSOChars;
                };
            }
            Message::SSOImport { pos } => {
                let account = self.login_state.import_que.remove(pos);
                return self.login(account, false, PlayerAuth::SSO, false);
            }
            Message::ViewSubPage { player, page } => {
                self.current_view = View::Account {
                    ident: player,
                    page,
                }
            }
            Message::SetAutoFetch(b) => {
                self.config.auto_fetch_newest = b;
                _ = self.config.write();
            }
            Message::SetMaxThreads(nv) => {
                self.config.max_threads = nv.clamp(0, 50);
                self.config.start_threads = self
                    .config
                    .start_threads
                    .clamp(0, 50.min(self.config.max_threads));
                _ = self.config.write();
            }
            Message::SetStartThreads(nv) => {
                self.config.start_threads =
                    nv.clamp(0, 50.min(self.config.max_threads));
                _ = self.config.write();
            }
            Message::SSOSuccess {
                auth_name,
                mut chars,
                provider,
            } => {
                let ident = match provider {
                    SSOProvider::Google => SSOIdent::Google(auth_name.clone()),
                    SSOProvider::Steam => SSOIdent::Steam(auth_name.clone()),
                };
                if self.login_state.active_sso.iter().any(|a| a.ident == ident)
                {
                    return Command::none();
                };

                let new_sso = SSOLogin {
                    ident,
                    status: SSOLoginStatus::Success,
                };

                self.login_state.active_sso.push(new_sso);
                self.login_state.import_que.append(&mut chars);

                if self.current_view == View::Login
                    && self.login_state.login_typ == LoginType::Google
                    || self.login_state.login_typ == LoginType::Steam
                {
                    self.login_state.login_typ = LoginType::SSOChars;
                };
            }
            Message::SSORetry => {}
            Message::SSOAuthError { .. } => {}
            Message::OpenLink(url) => {
                _ = open::that(url);
            }
            Message::PlayerAttack { ident, target } => {
                let Some(server) = self.servers.get_mut(&ident.server_id)
                else {
                    return Command::none();
                };
                let Some(account) = server.accounts.get_mut(&ident.account)
                else {
                    return Command::none();
                };
                let CrawlingStatus::Crawling { .. } = &server.crawling else {
                    return Command::none();
                };

                let mut status = account.status.lock().unwrap();
                let AccountStatus::Idle(_, gs) = &*status else {
                    return Command::none();
                };
                let next = gs.arena.next_free_fight.unwrap_or_default();
                if next > Local::now() + Duration::from_millis(200)
                    && gs.character.mushrooms == 0
                {
                    return Command::none();
                }

                let Some(mut session) = status.take_session("Fighting") else {
                    return Command::none();
                };
                drop(status);
                let ident = account.ident;
                let tn = target.info.name.clone();
                return Command::perform(
                    async move {
                        let cmd = sf_api::command::Command::Fight {
                            name: tn,
                            use_mushroom: false,
                        };
                        let resp = session.send_command(&cmd).await;
                        (resp, session)
                    },
                    move |r| match r.0 {
                        Ok(resp) => Message::PlayerAttackResult {
                            ident,
                            session: r.1,
                            against: target,
                            resp: Box::new(resp),
                        },
                        Err(_) => Message::PlayerCommandFailed {
                            ident,
                            session: r.1,
                            attempt: 0,
                        },
                    },
                );
            }
            Message::PlayerSetMaxLvl { ident, max } => {
                let Some(server) = self.servers.get_mut(&ident.server_id)
                else {
                    return Command::none();
                };
                let Some(account) = server.accounts.get_mut(&ident.account)
                else {
                    return Command::none();
                };
                let Some(si) = &mut account.scrapbook_info else {
                    return Command::none();
                };
                si.max_level = max;
                return self.update_best(ident, false);
            }
            Message::PlayerSetMaxAttributes { ident, max } => {
                let Some(server) = self.servers.get_mut(&ident.server_id)
                else {
                    return Command::none();
                };
                let Some(account) = server.accounts.get_mut(&ident.account)
                else {
                    return Command::none();
                };
                let Some(si) = &mut account.scrapbook_info else {
                    return Command::none();
                };
                si.max_attributes = max;
                return self.update_best(ident, false);
            }
            Message::SaveHoF(server_id) => {
                let Some(server) = self.servers.get(&server_id) else {
                    return Command::none();
                };

                let CrawlingStatus::Crawling {
                    que, player_info, ..
                } = &server.crawling
                else {
                    return Command::none();
                };

                let lock = que.lock().unwrap();
                let backup = lock.create_backup(player_info);
                drop(lock);
                let id = server.ident.id;
                let ident = server.ident.ident.to_string();

                return Command::perform(
                    async move { backup.write(&ident).await },
                    move |res| Message::BackupRes {
                        server: id,
                        error: res.err().map(|a| a.to_string()),
                    },
                );
            }
            Message::BackupRes {
                server: server_id,
                error,
            } => {
                let Some(server) = self.servers.get_mut(&server_id) else {
                    return Command::none();
                };
                let Some(pb) = server.headless_progress.clone() else {
                    return Command::none();
                };
                if let Some(err) = error {
                    pb.println(err)
                }
                self.servers.0.remove(&server_id);
                pb.finish_and_clear();
                return Command::perform(async {}, |_| {
                    Message::NextCLICrawling
                });
            }
            Message::CopyBattleOrder { ident } => {
                let Some((server, account)) = self.servers.get_ident(&ident)
                else {
                    return Command::none();
                };

                let CrawlingStatus::Crawling {
                    player_info,
                    equipment,
                    que,
                    ..
                } = &server.crawling
                else {
                    return Command::none();
                };

                let Some(si) = &account.scrapbook_info else {
                    return Command::none();
                };

                let mut best = si.best.first().cloned();
                let mut scrapbook = si.scrapbook.items.clone();

                let mut per_player_counts = calc_per_player_count(
                    player_info, equipment, &scrapbook, si,
                    self.config.blacklist_threshold,
                );

                let mut target_list = Vec::new();
                let mut loop_count = 0;
                let lock = que.lock().unwrap();
                let invalid =
                    lock.invalid_accounts.iter().map(|a| a.as_str()).collect();

                while let Some(AttackTarget { missing, info }) = best {
                    if loop_count > 300 || missing == 0 {
                        break;
                    }
                    loop_count += 1;

                    for eq in &info.equipment {
                        if scrapbook.contains(eq) {
                            continue;
                        }
                        let Some(players) = equipment.get(eq) else {
                            continue;
                        };
                        for player in players {
                            let ppc =
                                per_player_counts.entry(*player).or_insert(1);
                            *ppc = ppc.saturating_sub(1);
                        }
                    }

                    scrapbook.extend(info.equipment);
                    target_list.push(info.name);
                    let best_players =
                        find_best(&per_player_counts, player_info, 1, &invalid);
                    best = best_players.into_iter().next();
                }
                drop(lock);
                return iced::clipboard::write(target_list.join("/"));
            }
            Message::PlayerRelogSuccess { ident, gs, session } => {
                info!("Relogin success");
                let Some(server) = self.servers.0.get_mut(&ident.server_id)
                else {
                    return Command::none();
                };
                let Some(player) = server.accounts.get_mut(&ident.account)
                else {
                    return Command::none();
                };

                let mut lock = player.status.lock().unwrap();
                *lock = AccountStatus::Busy(gs, "Waiting".into());
                drop(lock);
                return Command::perform(
                    async {
                        sleep(Duration::from_secs(10)).await;
                    },
                    move |_| Message::PlayerRelogDelay { ident, session },
                );
            }
            Message::SSOLoginFailure { name, error } => {
                self
                    .login_state
                    .active_sso
                    .retain(|a| !matches!(&a.ident, SSOIdent::SF(s) if s.as_str() == name.as_str()));
                self.login_state.login_typ = LoginType::SFAccount;
                self.login_state.error = Some(error)
            }
            Message::PlayerLure { ident, target } => {
                let Some(server) = self.servers.get_mut(&ident.server_id)
                else {
                    return Command::none();
                };
                let Some(account) = server.accounts.get_mut(&ident.account)
                else {
                    return Command::none();
                };
                let Some(ud) = &account.underworld_info else {
                    return Command::none();
                };
                if ud.underworld.lured_today >= 5 {
                    return Command::none();
                }

                let CrawlingStatus::Crawling { .. } = &server.crawling else {
                    return Command::none();
                };

                let mut status = account.status.lock().unwrap();
                let Some(mut session) = status.take_session("Luring") else {
                    return Command::none();
                };
                drop(status);
                let ident = account.ident;
                let tid = target.uid;
                return Command::perform(
                    async move {
                        let cmd = sf_api::command::Command::UnderworldAttack {
                            player_id: tid,
                        };
                        let resp = session.send_command(&cmd).await;
                        (resp, session)
                    },
                    move |r| match r.0 {
                        Ok(resp) => Message::PlayerLureResult {
                            ident,
                            session: r.1,
                            against: target,
                            resp: Box::new(resp),
                        },
                        Err(_) => Message::PlayerCommandFailed {
                            ident,
                            session: r.1,
                            attempt: 0,
                        },
                    },
                );
            }
            Message::PlayerLureResult {
                ident,
                session,
                against,
                resp,
            } => {
                let Some(server) = self.servers.0.get_mut(&ident.server_id)
                else {
                    return Command::none();
                };
                let Some(account) = server.accounts.get_mut(&ident.account)
                else {
                    return Command::none();
                };

                let v = account.status.clone();
                let mut lock = v.lock().unwrap();

                let AccountStatus::Busy(s, _) = &mut *lock else {
                    return Command::none();
                };

                if let Err(e) = s.update(*resp) {
                    *lock = AccountStatus::FatalError(e.to_string());
                    return Command::none();
                };

                let Some(last) = &s.last_fight else {
                    return Command::none();
                };

                let Some(si) = &mut account.underworld_info else {
                    return Command::none();
                };

                si.attack_log.push((
                    Local::now(),
                    against.name,
                    last.has_player_won,
                ));

                if let Some(underworld) = s.underworld.as_ref() {
                    si.underworld = underworld.clone();
                }
                lock.put_session(session);
            }
            Message::PlayerNotPolled { ident } => {
                warn!("Unable to update {ident}")
            }
            Message::PlayerPolled { ident } => {
                let Some(server) = self.servers.0.get_mut(&ident.server_id)
                else {
                    return Command::none();
                };
                let Some(account) = server.accounts.get_mut(&ident.account)
                else {
                    return Command::none();
                };
                let mut lock = account.status.lock().unwrap();
                let gs = match &mut *lock {
                    AccountStatus::Busy(gs, _) | AccountStatus::Idle(_, gs) => {
                        gs
                    }
                    _ => {
                        return Command::none();
                    }
                };

                if let Some(cfg) = self
                    .config
                    .get_char_conf(&account.name, server.ident.id)
                    && cfg.auto_expeditions
                {
                    use sf_api::gamestate::tavern::{CurrentAction, ExpeditionStage};
                    if matches!(gs.tavern.current_action, CurrentAction::Expedition) {
                        if let Some(active) = gs.tavern.expeditions.active() {
                            let needs_more = match active.current_stage() {
                                ExpeditionStage::Waiting(_) | ExpeditionStage::Finished | ExpeditionStage::Unknown => false,
                                _ => true,
                            };
                            if needs_more && account.automation_queue.is_empty() {
                                drop(lock);
                                return Command::perform(
                                    async move {
                                        tokio::time::sleep(std::time::Duration::from_millis(fastrand::u64(30..=90))).await;
                                    },
                                    move |_| Message::RunAutomationTick { ident }
                                );
                            }
                        }
                    }
                }

                if let Some(sbi) = &mut account.underworld_info
                    && let Some(sb) = &gs.underworld
                {
                    sbi.underworld = sb.clone();
                }
                drop(lock);

                if let Some(cmd) = account.automation_queue.first().cloned() {
                    let mut status = account.status.lock().unwrap();
                    if let Some(mut session) = status.take_session("AutomationQueue") {
                        let _ = account.automation_queue.remove(0);
                        log::debug!(
                            "Automation {:?}: sending queued {:?} (remaining={})",
                            ident,
                            cmd,
                            account.automation_queue.len()
                        );
                        let player_status = account.status.clone();
                        let queued_cmd = cmd.clone();
                        let queued_cmd_for_log = queued_cmd.clone();
                        drop(status);

                        return Command::perform(
                            async move {
                                let resp = session.send_command(&queued_cmd).await;
                                (resp, session)
                            },
                            move |r| match r.0 {
                                Ok(resp) => {
                                    log::debug!("Automation {:?}: queued {:?} response: {:?}", ident, queued_cmd_for_log, resp);
                                    let mut lock = player_status.lock().unwrap();
                                    let gs = match &mut *lock {
                                        AccountStatus::Busy(gs, _) => gs,
                                        _ => {
                                            lock.put_session(r.1);
                                            return Message::PlayerNotPolled { ident };
                                        }
                                    };
                                    if gs.update(resp).is_err() {
                                        return Message::PlayerCommandFailed {
                                            ident,
                                            session: r.1,
                                            attempt: 0,
                                        };
                                    }
                                    {
                                        use strum::IntoEnumIterator;
                                        use sf_api::gamestate::dungeons::{LightDungeon, ShadowDungeon, DungeonProgress};
                                        let portal = gs.dungeons.portal.as_ref().map(|p| p.can_fight).unwrap_or(false);
                                        let dng_next = gs.dungeons.next_free_fight;
                                        let open_count = {
                                            let mut c = 0u32;
                                            for d in LightDungeon::iter() {
                                                if let DungeonProgress::Open { .. } = gs.dungeons.progress(d) { c += 1; }
                                            }
                                            for d in ShadowDungeon::iter() {
                                                if let DungeonProgress::Open { .. } = gs.dungeons.progress(d) { c += 1; }
                                            }
                                            c
                                        };
                                        let pets_pvp = gs.pets.as_ref().and_then(|p| p.opponent.next_free_battle);
                                        let pets_exp = gs.pets.as_ref().and_then(|p| p.next_free_exploration);
                                        let hydra = gs.guild.as_ref().map(|g| (g.hydra.remaining_fights, g.hydra.next_battle));
                                        log::debug!(
                                            "Automation {:?}: post-update snapshot (queued) -> portal: {}, dng_next: {:?}, open_dng: {}, pets_pvp: {:?}, pets_exp: {:?}, hydra: {:?}",
                                            ident, portal, dng_next, open_count, pets_pvp, pets_exp, hydra
                                        );
                                    }
                                    lock.put_session(r.1);
                                    Message::PlayerPolled { ident }
                                }
                                Err(e) => {
                                    log::error!("Automation {:?}: queued {:?} failed: {:?}", ident, queued_cmd_for_log, e);
                                    Message::PlayerCommandFailed {
                                        ident,
                                        session: r.1,
                                        attempt: 0,
                                    }
                                },
                            },
                        );
                    }
                }
            }
            Message::PlayerSetMaxUndergroundLvl { ident, lvl } => {
                let Some(server) = self.servers.get_mut(&ident.server_id)
                else {
                    return Command::none();
                };
                let Some(account) = server.accounts.get_mut(&ident.account)
                else {
                    return Command::none();
                };
                let Some(si) = &mut account.underworld_info else {
                    return Command::none();
                };
                si.max_level = lvl;
                return self.update_best(ident, false);
            }
            Message::UpdateResult(should_update) => {
                self.should_update = should_update;
            }
            Message::SetAutoPoll(new_val) => {
                self.config.auto_poll = new_val;
                _ = self.config.write();
            }
            Message::AdvancedLevelRestrict(val) => {
                self.config.show_crawling_restrict = val;
                _ = self.config.write();
            }
            Message::CrawlerSetMinMax { server, min, max } => {
                let Some(server) = self.servers.get_mut(&server) else {
                    return Command::none();
                };
                if let CrawlingStatus::Crawling { que, .. } = &server.crawling {
                    let mut que = que.lock().unwrap();
                    que.min_level = min.max(1);
                    que.max_level = max.max(min).min(9999);

                    debug!(
                        "Changed MinMax to {}/{}",
                        que.min_level, que.max_level
                    );
                    let mut to_remove = IntSet::default();
                    for (lvl, _) in
                        que.lvl_skipped_accounts.range(0..que.min_level)
                    {
                        to_remove.insert(*lvl);
                    }
                    for (lvl, _) in
                        que.lvl_skipped_accounts.range(que.max_level + 1..)
                    {
                        to_remove.insert(*lvl);
                    }
                    for lvl in to_remove {
                        let Some(mut todo) =
                            que.lvl_skipped_accounts.remove(&lvl)
                        else {
                            continue;
                        };
                        que.todo_accounts.append(&mut todo);
                    }
                }
            }
            Message::ShowClasses(val) => {
                self.config.show_class_icons = val;
                _ = self.config.write();
            }
            Message::NextCLICrawling => {
                let Some(cli) = &mut self.cli_crawling else {
                    return Command::none();
                };
                let pb = cli.mbp.add(ProgressBar::new_spinner());

                let Some(url) = cli.todo_servers.pop() else {
                    cli.active -= 1;
                    if cli.active == 0 {
                        pb.println("Finished Crawling all servers");
                        pb.finish_and_clear();
                        std::process::exit(0);
                    }
                    pb.finish_and_clear();
                    return Command::none();
                };
                let threads = cli.threads;
                return match self.force_init_crawling(&url, threads, pb.clone())
                {
                    Some(s) => s,
                    None => {
                        pb.println(format!(
                            "Could not init crawling on: {url}"
                        ));
                        pb.finish_and_clear();
                        return Command::perform(async {}, |_| {
                            Message::NextCLICrawling
                        });
                    }
                };
            }
            Message::CrawlAllRes {
                servers,
                concurrency,
            } => {
                let Some(cli) = &mut self.cli_crawling else {
                    return Command::none();
                };
                let Some(servers) = servers else {
                    _ = cli.mbp.println("Could not fetch server list");
                    std::process::exit(1);
                };
                cli.todo_servers = servers;
                let mut res = vec![];
                for _ in 0..concurrency {
                    res.push(Command::perform(async {}, |_| {
                        Message::NextCLICrawling
                    }))
                }
                return Command::batch(res);
            }
            Message::FontLoaded(_) => {}
            Message::PlayerRelogDelay { ident, session } => {
                let Some(server) = self.servers.get_mut(&ident.server_id)
                else {
                    return Command::none();
                };
                let Some(account) = server.accounts.get_mut(&ident.account)
                else {
                    return Command::none();
                };

                let mut lock = account.status.lock().unwrap();
                lock.put_session(session);
                drop(lock);
            }
            Message::SSOImportAuto { ident } => {
                let i_name = ident.name.to_lowercase();
                let i_server = ServerIdent::new(&ident.server);

                let pos = self.login_state.import_que.iter().position(|char| {
                    let server = ServerIdent::new(char.server_url().as_str());
                    let name = char.username().to_lowercase();
                    server == i_server && name == i_name
                });
                let Some(pos) = pos else {
                    return Command::none();
                };
                let account = self.login_state.import_que.remove(pos);
                return self.login(account, false, PlayerAuth::SSO, true);
            }
            Message::SetOverviewSelected { ident, val } => {
                let View::Overview { selected, action } =
                    &mut self.current_view
                else {
                    return Command::none();
                };
                *action = None;
                if val {
                    for v in ident {
                        selected.insert(v);
                    }
                } else {
                    for v in ident {
                        selected.remove(&v);
                    }
                }
            }
            Message::ConfigSetAutoLogin { name, server, nv } => {
                let Some(config) = self.config.get_char_conf_mut(&name, server)
                else {
                    return Command::none();
                };
                config.login = nv;
                _ = self.config.write();
            }
            Message::ConfigSetAutoBattle { name, server, nv } => {
                let Some(config) = self.config.get_char_conf_mut(&name, server)
                else {
                    return Command::none();
                };
                config.auto_battle = nv;
                _ = self.config.write();
            }
            Message::SetBlacklistThr(nv) => {
                self.config.blacklist_threshold = nv.max(1);
                _ = self.config.write();
            }
            Message::AutoLureIdle => {}
            Message::AutoLurePossible { ident } => {
                let refetch = self.update_best(ident, true);

                let Some(server) = self.servers.0.get_mut(&ident.server_id)
                else {
                    return Command::none();
                };
                let Some(account) = server.accounts.get_mut(&ident.account)
                else {
                    return Command::none();
                };

                let CrawlingStatus::Crawling { .. } = &server.crawling else {
                    return Command::none();
                };

                let mut status = account.status.lock().unwrap();
                let AccountStatus::Idle(_, gs) = &*status else {
                    return refetch;
                };

                let Some(0..=4) = gs.underworld.as_ref().map(|a| a.lured_today)
                else {
                    return refetch;
                };

                let Some(mut session) = status.take_session("Luring") else {
                    return refetch;
                };

                let Some(ui) = &account.underworld_info else {
                    status.put_session(session);
                    return refetch;
                };

                let total_len = ui.best.len();
                let new_len = ui.best.iter().filter(|a| !a.is_old()).count();

                if total_len == 0 || (new_len as f32 / total_len as f32) < 0.9 {
                    status.put_session(session);
                    return refetch;
                }

                let Some(target) =
                    ui.best.iter().find(|a| !a.is_old()).cloned()
                else {
                    status.put_session(session);
                    return refetch;
                };
                drop(status);
                info!("Auto Underworld attack {ident}");
                let fight = Command::perform(
                    async move {
                        let cmd = sf_api::command::Command::UnderworldAttack {
                            player_id: target.uid,
                        };
                        let resp = session.send_command(&cmd).await;
                        (resp, session)
                    },
                    move |r| match r.0 {
                        Ok(resp) => Message::PlayerLureResult {
                            ident,
                            session: r.1,
                            against: LureTarget {
                                uid: target.uid,
                                name: target.name,
                            },
                            resp: Box::new(resp),
                        },
                        Err(_) => Message::PlayerCommandFailed {
                            ident,
                            session: r.1,
                            attempt: 0,
                        },
                    },
                );

                return Command::batch([refetch, fight]);
            }
            Message::ConfigSetAutoLure { name, server, nv } => {
                let Some(config) = self.config.get_char_conf_mut(&name, server)
                else {
                    return Command::none();
                };
                config.auto_lure = nv;
                _ = self.config.write();
            }

            // NEW automation config handlers
            Message::ConfigSetAutoTavern { name, server, nv } => {
                let Some(cfg) = self.config.get_char_conf_mut(&name, server) else {
                    return Command::none();
                };
                cfg.auto_tavern = nv;
                _ = self.config.write();
            }
            Message::ConfigSetAutoExpeditions { name, server, nv } => {
                let Some(cfg) = self.config.get_char_conf_mut(&name, server) else {
                    return Command::none();
                };
                cfg.auto_expeditions = nv;
                _ = self.config.write();
            }
            Message::ConfigSetAutoDungeons { name, server, nv } => {
                let Some(cfg) = self.config.get_char_conf_mut(&name, server) else {
                    return Command::none();
                };
                cfg.auto_dungeons = nv;
                _ = self.config.write();
            }
            Message::ConfigSetAutoPets { name, server, nv } => {
                let Some(cfg) = self.config.get_char_conf_mut(&name, server) else {
                    return Command::none();
                };
                cfg.auto_pets = nv;
                _ = self.config.write();
            }
            Message::ConfigSetAutoGuild { name, server, nv } => {
                let Some(cfg) = self.config.get_char_conf_mut(&name, server) else {
                    return Command::none();
                };
                cfg.auto_guild = nv;
                _ = self.config.write();
            }
            Message::ConfigSetAutoGuildAcceptDefense { name, server, nv } => {
                let Some(cfg) = self.config.get_char_conf_mut(&name, server) else {
                    return Command::none();
                };
                cfg.auto_guild_accept_defense = nv;
                _ = self.config.write();
            }
            Message::ConfigSetAutoGuildAcceptAttack { name, server, nv } => {
                let Some(cfg) = self.config.get_char_conf_mut(&name, server) else {
                    return Command::none();
                };
                cfg.auto_guild_accept_attack = nv;
                _ = self.config.write();
            }
            Message::ConfigSetAutoGuildHydra { name, server, nv } => {
                let Some(cfg) = self.config.get_char_conf_mut(&name, server) else {
                    return Command::none();
                };
                cfg.auto_guild_hydra = nv;
                _ = self.config.write();
            }
            Message::ConfigSetMissionStrategy { name, server, nv } => {
                let Some(cfg) = self.config.get_char_conf_mut(&name, server) else {
                    return Command::none();
                };
                cfg.mission_strategy = nv;
                _ = self.config.write();
            }
            Message::ConfigSetAutoBuyBeerMushrooms { name, server, nv } => {
                let Some(cfg) = self.config.get_char_conf_mut(&name, server) else {
                    return Command::none();
                };
                cfg.auto_buy_beer_mushrooms = nv;
                _ = self.config.write();
            }
            Message::ConfigSetMaxMushroomsBeer { name, server, nv } => {
                let Some(cfg) = self.config.get_char_conf_mut(&name, server) else {
                    return Command::none();
                };
                cfg.max_mushrooms_beer = nv;
                _ = self.config.write();
            }
            Message::ConfigSetMaxMushroomsDungeonSkip { name, server, nv } => {
                let Some(cfg) = self.config.get_char_conf_mut(&name, server) else {
                    return Command::none();
                };
                cfg.max_mushrooms_dungeon_skip = nv;
                _ = self.config.write();
            }
            Message::ConfigSetMaxMushroomsPetSkip { name, server, nv } => {
                let Some(cfg) = self.config.get_char_conf_mut(&name, server) else {
                    return Command::none();
                };
                cfg.max_mushrooms_pet_skip = nv;
                _ = self.config.write();
            }

            Message::AutoLure { ident, state } => {
                let Some(server) = self.servers.0.get_mut(&ident.server_id)
                else {
                    return Command::none();
                };
                let Some(player) = server.accounts.get_mut(&ident.account)
                else {
                    return Command::none();
                };

                let Some(si) = &mut player.underworld_info else {
                    return Command::none();
                };

                si.auto_lure = state;
            }
            Message::CopyBestLures { ident } => {
                let Some(server) = self.servers.0.get_mut(&ident.server_id)
                else {
                    return Command::none();
                };
                let Some(player) = server.accounts.get_mut(&ident.account)
                else {
                    return Command::none();
                };

                let Some(si) = &mut player.underworld_info else {
                    return Command::none();
                };

                let mut res = format!(
                    "Best lure targets on {}. Max Lvl = {}\n",
                    server.ident.url, si.max_level
                );

                for a in &si.best {
                    if a.is_old() {
                        continue;
                    }
                    _ = res.write_fmt(format_args!(
                        "lvl: {:3}, items: {}, name: {}\n",
                        a.level,
                        a.equipment.len(),
                        a.name,
                    ));
                }

                return iced::clipboard::write(res);
            }
            Message::SetAction(a) => {
                let View::Overview { action, .. } = &mut self.current_view
                else {
                    return Command::none();
                };
                *action = a;
            }
            Message::MultiAction { action } => {
                let View::Overview {
                    action: ac,
                    selected,
                } = &mut self.current_view
                else {
                    return Command::none();
                };
                let targets = match ac {
                    Some(ActionSelection::Multi) => {
                        selected.iter().copied().collect()
                    }
                    Some(ActionSelection::Character(c)) => vec![*c],
                    None => return Command::none(),
                };

                *ac = None;

                let messages = targets
                    .into_iter()
                    .map(|a| match action {
                        OverviewAction::Logout => {
                            Message::RemoveAccount { ident: a }
                        }
                        OverviewAction::AutoBattle(nv) => Message::AutoBattle {
                            ident: a,
                            state: nv,
                        },
                    })
                    .map(|a| Command::perform(async {}, move |_| a));

                return Command::batch(messages);
            }
        }
        Command::none()
    }
}