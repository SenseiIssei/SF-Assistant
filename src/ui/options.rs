use iced::{
    Alignment, Element, Length,
    widget::{checkbox, column, text, row, pick_list},
};

use crate::{
    config::{Config, MissionStrategy},
    message::Message,
    player::AccountInfo,
    server::ServerInfo,
};

pub fn view_options<'a>(
    player: &'a AccountInfo,
    og_server: &'a ServerInfo,
    config: &'a Config,
) -> Element<'a, Message> {
    let config = config.get_char_conf(&player.name, og_server.ident.id);

    let Some(config) = config else {
        return text(
            "Use 'Remember me' during login to store player configurations",
        )
        .size(20)
        .into();
    };

    let mut all = column!().spacing(20).width(Length::Fixed(360.0));

    all = all.push(
        checkbox("Automatically login on startup", config.login).on_toggle(
            |nv| Message::ConfigSetAutoLogin {
                name: player.name.clone(),
                server: og_server.ident.id,
                nv,
            },
        ),
    );

    all = all.push(
        checkbox("Enable auto-battle on login", config.auto_battle).on_toggle(
            |nv| Message::ConfigSetAutoBattle {
                name: player.name.clone(),
                server: og_server.ident.id,
                nv,
            },
        ),
    );

    all = all.push(
        checkbox("Enable auto-lure on login", config.auto_lure).on_toggle(
            |nv| Message::ConfigSetAutoLure {
                name: player.name.clone(),
                server: og_server.ident.id,
                nv,
            },
        ),
    );

    all = all.push(text("Automation").size(18));

    all = all.push(
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
        ]
        .spacing(16),
    );

    all = all.push(
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
        ]
        .spacing(16),
    );

    all = all.push(
        row![
            checkbox("Guild", config.auto_guild).on_toggle(|nv| Message::ConfigSetAutoGuild {
                name: player.name.clone(),
                server: og_server.ident.id,
                nv,
            }),
        ]
        .spacing(16),
    );

    let strategies = &[
        MissionStrategy::Shortest,
        MissionStrategy::MostGold,
        MissionStrategy::BestGoldPerMinute,
        MissionStrategy::BestXpPerMinute,
        MissionStrategy::Smartest,
    ];

    all = all.push(
        row![
            text("Mission strategy").width(Length::Fixed(150.0)),
            pick_list(
                strategies.to_vec(),
                Some(config.mission_strategy),
                {
                    let name = player.name.clone();
                    let server = og_server.ident.id;
                    move |nv| Message::ConfigSetMissionStrategy { name: name.clone(), server, nv }
                }
            )
            .width(Length::Fixed(200.0))
        ]
        .spacing(12)
        .align_items(Alignment::Center),
    );

    // Reserve mushrooms removed: we save all mushrooms by default and only spend if a specific budget is enabled

    column!(all)
        .padding(20)
        .height(Length::Fill)
        .width(Length::Fill)
        .align_items(Alignment::Center)
        .into()
}