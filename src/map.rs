use crate::prelude::*;

use mw_common::grid::map::CompactMapCoordExt;
use mw_common::game::{ProdState, MineKind, MapDataInit, TileKind, MapDescriptor};
use mw_common::plid::PlayerId;
use mw_common::grid::*;

use crate::assets::TileAssets;
use crate::AppGlobalState;

use self::tileid::CoordTileids;

#[cfg(feature = "gfx_sprites")]
mod gfx_sprites;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(SystemLabel)]
pub enum MapLabels {
    /// Anything that sends MapEvents should come before this
    ApplyEvents,
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MapEvent>();
        app.add_system(
            setup_map
                .track_progress()
                .run_in_state(AppGlobalState::GameLoading)
        );
        app.add_exit_system(AppGlobalState::InGame, despawn_with_recursive::<MapCleanup>);
        #[cfg(feature = "dev")]
        app.add_system(debug_mapevents.label(MapLabels::ApplyEvents));
        #[cfg(feature = "gfx_sprites")]
        app.add_plugin(gfx_sprites::MapGfxSpritesPlugin);
    }
}

pub struct MaxViewBounds(pub f32);

#[derive(Component)]
struct MapCleanup;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapEventKind {
    Owner {
        plid: PlayerId,
    },
    Digit {
        digit: u8,
    },
    Mine {
        kind: Option<MineKind>,
    },
    Road {
        state: Option<ProdState>,
    },
    Explosion {
        kind: MineKind,
    },
    MineActive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapEvent {
    /// coordinate to update
    pub c: Pos,
    /// which view in a multi-view setup is the event for
    pub plid: PlayerId,
    /// the event
    pub kind: MapEventKind,
}

fn debug_mapevents(
    mut er_map: EventReader<MapEvent>,
) {
    for ev in er_map.iter() {
        debug!("{:?}", ev);
    }
}

fn setup_map(
    mut commands: Commands,
    descriptor: Option<Res<MapDescriptor>>,
    data_hex: Option<Res<MapDataInit<Hex>>>,
    data_sq: Option<Res<MapDataInit<Sq>>>,
    mut done: Local<bool>,
) -> Progress {
    let descriptor = if let Some(descriptor) = descriptor {
        // reset for new game
        if descriptor.is_changed() {
            *done = false;
        }

        descriptor
    } else {
        return false.into();
    };

    if *done {
        return true.into();
    }

    match descriptor.topology {
        Topology::Hex => {
            let data = if let Some(data) = data_hex {
                data
            } else {
                return false.into();
            };
            setup_map_topology::<Hex>(&mut commands, &*data);
        }
        Topology::Sq => {
            let data = if let Some(data) = data_sq {
                data
            } else {
                return false.into();
            };
            setup_map_topology::<Sq>(&mut commands, &*data);
        }
        _ => unimplemented!(),
    }
    *done = true;
    debug!("Setup tile entities for map: {:?}", descriptor);

    true.into()
}

struct TileEntityIndex<C: CompactMapCoordExt>(MapData<C, Entity>);

struct PlidView {
}

/// Per-tile component: the map coordinates
#[derive(Debug, Clone, Copy, Component)]
struct TileCoord(Pos);
/// Per-tile component: the minesweeper digit
#[derive(Debug, Clone, Copy, Component)]
struct TileDigit(u8);
/// Per-tile component: the PlayerId of the owner
#[derive(Debug, Clone, Copy, Component)]
struct TileOwner(PlayerId);
/// Per-tile component: visibility (fog of war) state
#[derive(Debug, Clone, Copy, Component)]
struct TileVisible(bool);

#[derive(Bundle)]
struct TileBundle {
    kind: TileKind,
    coord: TileCoord,
    digit: TileDigit,
    owner: TileOwner,
    vis: TileVisible,
}

fn setup_map_topology<C: CoordTileids + CompactMapCoordExt>(
    commands: &mut Commands,
    data: &MapDataInit<C>,
) {
    let mut tile_index = TileEntityIndex::<C>(MapData::new(data.map.size(), Entity::from_raw(0)));

    commands.insert_resource(MaxViewBounds(C::TILE_OFFSET.x.min(C::TILE_OFFSET.y) * data.map.size() as f32));
    for (c, init) in data.map.iter() {
        let tile_e = commands.spawn_bundle(
            TileBundle {
                kind: init.kind,
                coord: TileCoord(c.into()),
                digit: TileDigit(0),
                owner: TileOwner(PlayerId::Spectator),
                vis: TileVisible(false),
            })
            .insert(MapCleanup).id();

        tile_index.0[c] = tile_e;
    }

    commands.insert_resource(tile_index);

    commands.remove_resource::<MapDataInit<C>>();
}

pub mod tileid {
    #![allow(dead_code)]

    use crate::prelude::*;
    use bevy::math::const_vec2;
    use mw_common::grid::*;

    pub trait CoordTileids: Coord {
        const TILE_OFFSET: Vec2;
        const TILEID_LAND: usize;
        const TILEID_CURSOR: usize;
        const TILEID_ROADS: &'static [usize];
    }

    impl CoordTileids for Hex {
        const TILE_OFFSET: Vec2 = const_vec2!([224.0, 256.0]);
        const TILEID_LAND: usize = 0o1;
        const TILEID_CURSOR: usize = 0o0;
        const TILEID_ROADS: &'static [usize] = &[0o60, 0o61, 0o62, 0o63, 0o64, 0o65];
    }

    impl CoordTileids for Sq {
        const TILE_OFFSET: Vec2 = const_vec2!([224.0, 224.0]);
        const TILEID_LAND: usize = 0o11;
        const TILEID_CURSOR: usize = 0o10;
        const TILEID_ROADS: &'static [usize] = &[0o70, 0o71, 0o72, 0o73];
    }

    pub const ITEM_MINE: usize = 0o4;
    pub const ITEM_DECOY: usize = 0o5;
    pub const EXPLODE_MINE: usize = 0o14;
    pub const EXPLODE_DECOY: usize = 0o15;
    pub const MINE_ACTIVE: usize = 0o24;

    pub const GEO_WATER: usize = 0o20;
    pub const GEO_FERTILE: usize = 0o21;
    pub const GEO_MOUNTAIN: usize = 0o22;

    pub const LANDMARK_CITY: usize = 0o40;
    pub const LANDMARK_TOWER: usize = 0o41;
    pub const DECAL_SKULL: usize = 0o50;

    pub const DIGITS: [usize; 8] = [0, 0o51, 0o52, 0o53, 0o54, 0o55, 0o56, 0o57];
}
