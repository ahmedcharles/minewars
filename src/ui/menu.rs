use crate::{prelude::*, locale::L10nKey, assets::UiAssets, settings::NeedsSettingsSet};

use super::tooltip::InfoText;

mod mainmenu;

pub(super) struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            mainmenu::MainMenuPlugin,
        ));
        app.add_systems(Update, (
            menu_butt_interact_visual.in_set(NeedsSettingsSet),
        ));
    }
}

/// Marker for the area where top bar items / buttons can be placed when in a menu
#[derive(Component)]
struct MenuTopBar;

/// Marker for the area where a menu can display its main content
#[derive(Component)]
struct MenuContainer;

/// Marker for menu buttons
#[derive(Component)]
struct MenuButton;

fn spawn_menu_butt(
    commands: &mut Commands,
    uiassets: &UiAssets,
    settings: &AllSettings,
    behavior: OnClick,
    text: &'static str,
    info_text: &'static str,
    enabled: bool,
) -> Entity {
    let color_init = if enabled {
        settings.ui.color_menu_button
    } else {
        settings.ui.color_menu_button_inactive
    };

    let color_text = if enabled {
        settings.ui.color_text
    } else {
        settings.ui.color_text_inactive
    };

    let butt = commands.spawn((
        MenuButton,
        behavior,
        InfoText(info_text.to_owned()),
        ButtonBundle {
            background_color: BackgroundColor(color_init),
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(4.0)),
                margin: UiRect::all(Val::Px(4.0)),
                flex_grow: 1.0,
                flex_shrink: 0.0,
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();

    let text = commands.spawn((
        L10nKey(text.to_owned()),
        TextBundle {
            text: Text::from_section(
                text,
                TextStyle {
                    color: color_text,
                    font_size: 32.0 * settings.ui.text_scale,
                    font: uiassets.font.clone(),
                },
            ),
            ..Default::default()
        },
    )).id();

    commands.entity(butt).push_children(&[text]);

    if !enabled {
        commands.entity(butt).insert(UiDisabled);
    }

    butt
}

fn menu_butt_interact_visual(
    settings: Res<AllSettings>,
    mut query: Query<(
        &Interaction, &mut BackgroundColor,
    ), (
        Changed<Interaction>, With<MenuButton>, Without<UiDisabled>,
    )>,
) {
    for (interaction, mut color) in query.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(settings.ui.color_menu_button_selected);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(settings.ui.color_menu_button_selected);
            }
            Interaction::None => {
                *color = BackgroundColor(settings.ui.color_menu_button);
            }
        }
    }
}
