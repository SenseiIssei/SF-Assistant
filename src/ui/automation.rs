use iced::{
    Alignment, Element, Length,
    widget::{checkbox, column, text, row, pick_list, container, button, horizontal_space, slider},
    theme,
};

use crate::{
    config::{Config, MissionStrategy, ExpeditionRewardPriority},
    message::Message,
    player::{AccountInfo, AccountStatus},
    server::ServerInfo,
};

pub fn view_automation<'a>(
    player: &'a AccountInfo,
    og_server: &'a ServerInfo,
    config: &'a Config,
) -> Element<'a, Message> {
    // Access current GameState for live info
    let lock = player.status.lock().unwrap();
    let gs = match &*lock {
        AccountStatus::LoggingIn => {
            return text("Logging in").size(20).into();
        }
        AccountStatus::Idle(_, gs) => gs,
        AccountStatus::Busy(gs, _) => gs,
        AccountStatus::FatalError(err) => {
            return text(format!("Error: {err}")).size(20).into();
        }
        AccountStatus::LoggingInAgain => {
            return text("Logging in again").size(20).into();
        }
    };

    let config = config.get_char_conf(&player.name, og_server.ident.id);

    let Some(config) = config else {
        return text("Use 'Remember me' during login to store player configurations")
            .size(20)
            .into();
    };

    let strategies = &[
        MissionStrategy::Shortest,
        MissionStrategy::MostGold,
        MissionStrategy::BestGoldPerMinute,
        MissionStrategy::BestXpPerMinute,
        MissionStrategy::Smartest,
    ];

    let header = row![
        text("Automation").size(24),
        horizontal_space(),
        button(text("Run now"))
            .on_press(Message::RunAutomationTick { ident: player.ident })
            .style(theme::Button::Primary),
        button(text("Back")).style(theme::Button::Secondary).on_press(Message::ViewSettings),
    ];

    // Live status
    use chrono::Local;
    let now = Local::now();
    let quest_status = match &gs.tavern.current_action {
        sf_api::gamestate::tavern::CurrentAction::Quest { busy_until, .. } => {
            let secs = (*busy_until - now).num_seconds().max(0);
            text(format!("Quest ends in {}m {}s", secs / 60, secs % 60))
        }
        sf_api::gamestate::tavern::CurrentAction::Expedition => text("In expedition").into(),
        _ => text("Tavern idle").into(),
    };

    let extra_beer = gs
        .character
        .equipment
        .has_enchantment(sf_api::gamestate::items::Enchantment::ThirstyWanderer) as u8;
    let beer_cap = 10 + extra_beer;
    let thirst = gs.tavern.thirst_for_adventure_sec.max(0);

    // Left column: toggles and configuration
    let mut left = column![]
        .spacing(16)
        .width(Length::Fixed(420.0));

    left = left.push(text("Automations").size(18));
    left = left.push(
        row![
            checkbox("Auto battle", config.auto_battle).on_toggle(|nv| Message::ConfigSetAutoBattle {
                name: player.name.clone(),
                server: og_server.ident.id,
                nv,
            }),
            checkbox("Auto lure", config.auto_lure).on_toggle(|nv| Message::ConfigSetAutoLure {
                name: player.name.clone(),
                server: og_server.ident.id,
                nv,
            }),
        ]
        .spacing(24)
    );
    left = left.push(
        row![
            checkbox("Tavern", config.auto_tavern).on_toggle(|nv| Message::ConfigSetAutoTavern {
                name: player.name.clone(),
                server: og_server.ident.id,
                nv,
            }),
            checkbox("Expeditions", config.auto_expeditions).on_toggle(|nv| Message::ConfigSetAutoExpeditions {
                name: player.name.clone(),
                server: og_server.ident.id,
                nv,
            }),
        ].spacing(24)
    );
    left = left.push(
        row![
            checkbox("Dungeons", config.auto_dungeons).on_toggle(|nv| Message::ConfigSetAutoDungeons {
                name: player.name.clone(),
                server: og_server.ident.id,
                nv,
            }),
            checkbox("Pets", config.auto_pets).on_toggle(|nv| Message::ConfigSetAutoPets {
                name: player.name.clone(),
                server: og_server.ident.id,
                nv,
            }),
        ].spacing(24)
    );
    left = left.push(
        row![
            checkbox("Guild", config.auto_guild).on_toggle(|nv| Message::ConfigSetAutoGuild {
                name: player.name.clone(),
                server: og_server.ident.id,
                nv,
            }),
        ]
        .spacing(24)
    );
    if config.auto_guild {
        left = left.push(
            column![
                row![
                    checkbox("Accept defense", config.auto_guild_accept_defense).on_toggle(|nv| Message::ConfigSetAutoGuildAcceptDefense {
                        name: player.name.clone(),
                        server: og_server.ident.id,
                        nv,
                    }),
                    checkbox("Accept attack", config.auto_guild_accept_attack).on_toggle(|nv| Message::ConfigSetAutoGuildAcceptAttack {
                        name: player.name.clone(),
                        server: og_server.ident.id,
                        nv,
                    }),
                ].spacing(24),
                row![
                    checkbox("Hydra battle", config.auto_guild_hydra).on_toggle(|nv| Message::ConfigSetAutoGuildHydra {
                        name: player.name.clone(),
                        server: og_server.ident.id,
                        nv,
                    }),
                ],
            ]
            .spacing(12)
            .padding(6)
        );
    }
    left = left.push(text("Strategy").size(18));
    left = left.push(
        row![
            text("Mission strategy").width(Length::Fixed(160.0)),
            pick_list(
                strategies.to_vec(),
                Some(config.mission_strategy),
                {
                    let name = player.name.clone();
                    let server = og_server.ident.id;
                    move |nv| Message::ConfigSetMissionStrategy { name: name.clone(), server, nv }
                }
            )
            .width(Length::Fixed(220.0))
        ]
        .spacing(12)
        .align_items(Alignment::Center),
    );
    // Expedition options
    left = left.push(text("Expeditions").size(18));
    left = left.push(
        row![
            checkbox("Use glasses to skip waits", config.use_glasses_for_expeditions)
                .on_toggle(|nv| Message::ConfigSetUseExpeditionGlasses {
                    name: player.name.clone(),
                    server: og_server.ident.id,
                    nv,
                }),
        ]
        .spacing(12)
    );
    let reward_presets = [
        ExpeditionRewardPriority::MushroomsGoldEggs,
        ExpeditionRewardPriority::GoldMushroomsEggs,
        ExpeditionRewardPriority::EggsMushroomsGold,
    ];
    left = left.push(
        row![
            text("Reward priority").width(Length::Fixed(160.0)),
            pick_list(
                reward_presets,
                Some(config.expedition_reward_priority),
                {
                    let name = player.name.clone();
                    let server = og_server.ident.id;
                    move |nv| Message::ConfigSetExpeditionRewardPriority { name: name.clone(), server, nv }
                }
            )
            .width(Length::Fixed(220.0))
        ]
        .spacing(12)
        .align_items(Alignment::Center),
    );
    // Reserve mushrooms removed: we save all mushrooms by default and only spend if a specific budget is enabled
    left = left.push(text("Tavern options").size(18));
    left = left.push(
        row![
            checkbox("Buy beer with mushrooms", config.auto_buy_beer_mushrooms).on_toggle(|nv| Message::ConfigSetAutoBuyBeerMushrooms {
                name: player.name.clone(),
                server: og_server.ident.id,
                nv,
            }),
            checkbox("Use glasses to skip waits", config.use_glasses_for_tavern).on_toggle(|nv| Message::ConfigSetUseTavernGlasses {
                name: player.name.clone(),
                server: og_server.ident.id,
                nv,
            }),
        ]
        .spacing(12)
    );

    // Mushroom budgets
    left = left.push(text("Mushroom budgets").size(18));
    left = left.push(
        row![
            text("Beer (per day)").width(Length::Fixed(160.0)),
            slider(0..=50, config.max_mushrooms_beer, {
                let name = player.name.clone();
                let server = og_server.ident.id;
                move |nv| Message::ConfigSetMaxMushroomsBeer { name: name.clone(), server, nv }
            })
            .width(Length::Fixed(220.0)),
            text(config.max_mushrooms_beer.to_string()),
        ]
        .spacing(12)
        .align_items(Alignment::Center),
    );
    left = left.push(
        row![
            text("Dungeon skip").width(Length::Fixed(160.0)),
            slider(0..=50, config.max_mushrooms_dungeon_skip, {
                let name = player.name.clone();
                let server = og_server.ident.id;
                move |nv| Message::ConfigSetMaxMushroomsDungeonSkip { name: name.clone(), server, nv }
            })
            .width(Length::Fixed(220.0)),
            text(config.max_mushrooms_dungeon_skip.to_string()),
        ]
        .spacing(12)
        .align_items(Alignment::Center),
    );
    left = left.push(
        row![
            text("Pet skip").width(Length::Fixed(160.0)),
            slider(0..=50, config.max_mushrooms_pet_skip, {
                let name = player.name.clone();
                let server = og_server.ident.id;
                move |nv| Message::ConfigSetMaxMushroomsPetSkip { name: name.clone(), server, nv }
            })
            .width(Length::Fixed(220.0)),
            text(config.max_mushrooms_pet_skip.to_string()),
        ]
        .spacing(12)
        .align_items(Alignment::Center),
    );

    // Right column: live info and timers
    let mut right = column![].spacing(12).width(Length::Fixed(520.0));

    let next_free = match gs.arena.next_free_fight {
        Some(t) if t > chrono::Local::now() => {
            let mins = (t - chrono::Local::now()).num_minutes();
            text(format!("Next free fight in {}m", mins.max(0))).size(16)
        }
        _ => text("Free fight possible").size(16),
    };
    right = right.push(next_free);

    right = right.push(text("Tavern status").size(18));
    right = right.push(
        row![text("Tavern status:"), quest_status].spacing(12)
    );

    right = right.push(text("Live timers").size(18));
    right = right.push(column![
        row![text("Thirst left:"), text(format!("{}s", thirst))].spacing(8),
        row![text("Beer today:"), text(format!("{}/{}", gs.tavern.beer_drunk, beer_cap))].spacing(8),
        row![text("Mushroom skip allowed:"), text(if gs.tavern.mushroom_skip_allowed { "Yes" } else { "No" })].spacing(8),
    ].spacing(6));

    // Pets timers
    right = right.push(text("Pets").size(18));
    if let Some(pets) = &gs.pets {
        use sf_api::gamestate::unlockables::{HabitatType, HabitatExploration};
        use sf_api::misc::EnumMapGet;
        use strum::IntoEnumIterator;
        let pvp = match pets.opponent.next_free_battle {
            Some(t) if t > now => {
                let secs = (t - now).num_seconds().max(0);
                format!("in {}m {}s", secs / 60, secs % 60)
            }
            _ => "ready".into(),
        };
        let explore = match pets.next_free_exploration {
            Some(t) if t > now => {
                let secs = (t - now).num_seconds().max(0);
                format!("in {}m {}s", secs / 60, secs % 60)
            }
            _ => "ready".into(),
        };
        let mut pet_detail = column![
            row![text("Next pet PvP:"), text(pvp)].spacing(8),
            row![text("Next pet exploration:"), text(explore)].spacing(8),
        ]
        .spacing(6);

        for hab in HabitatType::iter() {
            let h = pets.habitats.get(hab);
            let best = h.pets.iter().max_by_key(|p| p.level).map(|p| p.level).unwrap_or(0);
            let (state, enemy_pos) = match &h.exploration {
                HabitatExploration::Exploring { fights_won, .. } => ("Exploring", *fights_won + 1),
                _ => ("Idle/Unknown", 0),
            };
            pet_detail = pet_detail.push(
                row![
                    text(format!("{:?}", hab)).width(Length::Fixed(100.0)),
                    text(format!("best lvl {}", best)).width(Length::Fixed(90.0)),
                    text(format!("{}{}", state, if enemy_pos>0 { format!(" (enemy {})", enemy_pos) } else { String::new() })).width(Length::Fill),
                ]
                .spacing(8)
            );
        }

        right = right.push(pet_detail);
    } else {
        right = right.push(text("Pets feature not unlocked").size(14));
    }

    // Dungeons/Portal
    right = right.push(text("Dungeons").size(18));
    {
        let mut lines = column![];
        if let Some(portal) = &gs.dungeons.portal {
            let s = if portal.can_fight { "Portal: available" } else { "Portal: done for today" };
            lines = lines.push(text(s));
        }
        use strum::IntoEnumIterator;
        use sf_api::gamestate::dungeons::{LightDungeon, ShadowDungeon, DungeonProgress};
        if let DungeonProgress::Open { finished } = gs.dungeons.progress(LightDungeon::Tower) {
            lines = lines.push(text(format!("Tower: next level {}", finished + 1)));
        }
        if let Some(ld) = LightDungeon::iter().filter(|d| !matches!(d, LightDungeon::Tower)).find(|&d| matches!(gs.dungeons.progress(d), DungeonProgress::Open { .. })) {
            lines = lines.push(text(format!("Light dungeon: {:?} open", ld))); 
        } else {
            lines = lines.push(text("No light dungeon open"));
        }
        if let Some(sd) = ShadowDungeon::iter().find(|&d| matches!(gs.dungeons.progress(d), DungeonProgress::Open { .. })) {
            lines = lines.push(text(format!("Shadow dungeon: {:?} open", sd)));
        } else {
            lines = lines.push(text("No shadow dungeon open"));
        }
        if let Some(t) = gs.dungeons.next_free_fight {
            let now = chrono::Local::now();
            let s = if t > now { let secs = (t - now).num_seconds().max(0); format!("Next dungeon fight in {}m {}s", secs/60, secs%60) } else { "Next dungeon fight: ready".into() };
            lines = lines.push(text(s));
        }
        right = right.push(lines.spacing(6));
    }

    // Guild hydra timer
    if let Some(guild) = &gs.guild {
        let hydra = match (guild.hydra.remaining_fights, guild.hydra.next_battle) {
            (rf, Some(t)) if t > now => {
                let secs = (t - now).num_seconds().max(0);
                format!("Hydra: {} fights, next in {}m {}s", rf, secs / 60, secs % 60)
            }
            (rf, _) => format!("Hydra: {} fights, ready", rf),
        };
        let auto_def = if config.auto_guild_accept_defense { "on" } else { "off" };
        let auto_att = if config.auto_guild_accept_attack { "on" } else { "off" };
        right = right.push(column![
            text("Guild").size(18),
            text(hydra),
            row![text("Join defense (auto):"), text(auto_def)].spacing(8),
            row![text("Join attack (auto):"), text(auto_att)].spacing(8),
            text("Opponents: attack/defense details not available").size(12).style(theme::Text::Color(iced::Color::from_rgb8(130,130,130))),
        ].spacing(6));
    }

    let body = column![
        header,
        row![
            left,
            horizontal_space(),
            right
        ]
        .spacing(24)
        .align_items(Alignment::Start)
    ]
    .spacing(16)
    .width(Length::Fill);

    container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Top)
        .padding(20)
        .into()
}