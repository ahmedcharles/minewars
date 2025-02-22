pub mod prelude {
    pub use mw_common::prelude::*;
    pub use iyes_bevy_extras::prelude::*;
    pub use leafwing_input_manager::prelude::*;
    pub use crate::appstate::*;
    pub use crate::settings::{AllSettings, NeedsSettingsSet};
    pub use crate::PROPRIETARY;
}

pub const PROPRIETARY: bool = cfg!(feature = "proprietary");

pub mod appstate;
pub mod camera;
pub mod input;
pub mod map;
pub mod player;
pub mod tool;
pub mod view;
pub mod settings;

pub mod bevyhost;

use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct GameEventSet;

pub struct MwCommonPlugin;

impl Plugin for MwCommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<mw_common::game::event::GameEvent>();
        app.add_plugins((
            appstate::AppStatesPlugin,
            camera::MwCameraPlugin,
            tool::ToolPlugin,
            input::InputPlugin,
            settings::SettingsPlugin,
            map::MapPlugin,
            view::GameViewPlugin,
        ));
    }
}
