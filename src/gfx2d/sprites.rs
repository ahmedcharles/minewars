use crate::assets::GameAssets;
use crate::camera::GridCursor;
use crate::camera::GridCursorSet;
use crate::prelude::*;

use mw_common::grid::*;
use mw_common::game::*;
use mw_app::map::*;

use super::Gfx2dSet;
use super::Gfx2dTileSetupSet;
use super::TilemapInitted;
use super::fancytint;

pub struct Gfx2dSpritesPlugin;

impl Plugin for Gfx2dSpritesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            (
                setup_base_tile::<Hex>
                    .in_set(MapTopologySet(Topology::Hex)),
                setup_base_tile::<Sq>
                    .in_set(MapTopologySet(Topology::Sq)),
            )
                .in_set(Gfx2dTileSetupSet)
                .run_if(not(resource_exists::<TilemapInitted>())),
            (
                tile_kind,
                tile_owner,
            )
                .run_if(resource_exists::<TilemapInitted>()),
        ).in_set(Gfx2dSet::Sprites));
        app.add_systems(OnEnter(AppState::InGame), (
            setup_cursor,
        ));
        app.add_systems(Update, (
            (
                cursor_sprite::<Hex>
                    .in_set(MapTopologySet(Topology::Hex)),
                cursor_sprite::<Sq>
                    .in_set(MapTopologySet(Topology::Sq)),
            )
                .after(GridCursorSet),
        ).in_set(Gfx2dSet::Any));
    }
}

#[derive(Component)]
struct CursorSprite;

#[derive(Bundle)]
struct CursorSpriteBundle {
    sprite: SpriteSheetBundle,
    pos: MwTilePos,
    marker: CursorSprite,
}

fn setup_cursor(
    mut commands: Commands,
    gass: Res<GameAssets>,
    mapdesc: Res<MapDescriptor>,
) {
    let i = match mapdesc.topology {
        Topology::Hex => super::sprite::TILES6 + super::sprite::TILE_CURSOR,
        Topology::Sq => super::sprite::TILES4 + super::sprite::TILE_CURSOR,
    };
    commands.spawn((
        CursorSpriteBundle {
            sprite: SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index: i,
                    ..Default::default()
                },
                texture_atlas: gass.sprites.clone(),
                transform: Transform::from_xyz(0.0, 0.0, super::zpos::CURSOR),
                ..Default::default()
            },
            pos: MwTilePos(Pos::origin()),
            marker: CursorSprite,
        },
    ));
}

fn cursor_sprite<C: Coord>(
    mut q: Query<(&mut Transform, &mut MwTilePos), With<CursorSprite>>,
    crs: Res<GridCursor>,
) {
    if !crs.is_changed() {
        return;
    }
    let (mut xf, mut pos) = q.single_mut();
    *pos = MwTilePos(crs.0);

    let (width, height) = match C::TOPOLOGY {
        Topology::Hex => (
            super::sprite::WIDTH6, super::sprite::HEIGHT6,
        ),
        Topology::Sq => (
            super::sprite::WIDTH4, super::sprite::HEIGHT4,
        ),
    };
    let trans = C::from(crs.0).translation();
    xf.translation = Vec3::new(trans.x * width, trans.y * height, super::zpos::CURSOR);
}

fn setup_base_tile<C: MapTileIndexCoord>(
    world: &mut World,
) {
    let texture_atlas = world.resource::<GameAssets>().sprites.clone();
    let index = world.remove_resource::<MapTileIndex>().unwrap();
    let (sprite_index, width, height) = match C::TOPOLOGY {
        Topology::Hex => (
            super::sprite::TILES6 + super::sprite::TILE_WATER,
            super::sprite::WIDTH6, super::sprite::HEIGHT6,
        ),
        Topology::Sq => (
            super::sprite::TILES4 + super::sprite::TILE_WATER,
            super::sprite::WIDTH4, super::sprite::HEIGHT4,
        ),
    };
    for (c, &e) in C::get_mapdata(&index).iter() {
        let trans = c.translation();
        world.entity_mut(e).insert(SpriteSheetBundle {
            texture_atlas: texture_atlas.clone(),
            sprite: TextureAtlasSprite {
                color: Color::WHITE,
                index: sprite_index,
                ..Default::default()
            },
            transform: Transform::from_xyz(trans.x * width, trans.y * height, super::zpos::TILE),
            ..Default::default()
        });
    }
    world.insert_resource(index);
    world.insert_resource(TilemapInitted);
}

fn tile_kind(
    desc: Res<MapDescriptor>,
    mut q: Query<(&mut TextureAtlasSprite, &TileKind, &MwTilePos), Changed<TileKind>>,
    q_pos: Query<&TileKind>,
    index: Res<MapTileIndex>,
) {
    let i_base = match desc.topology {
        Topology::Hex => super::sprite::TILES6,
        Topology::Sq => super::sprite::TILES4,
    };

    for (mut spr, kind, pos) in &mut q {
        spr.index = i_base + match kind {
            TileKind::Water => super::sprite::TILE_WATER,
            TileKind::Foundation => super::sprite::TILE_FOUNDATION,
            TileKind::Regular => super::sprite::TILE_LAND,
            TileKind::Fertile => super::sprite::TILE_FERTILE,
            TileKind::Forest => super::sprite::TILE_FOREST,
            TileKind::Mountain => super::sprite::TILE_MTN,
            TileKind::Destroyed => super::sprite::TILE_DEAD,
        };
        if *kind == TileKind::Water {
            let a = match desc.topology {
                // PERF: getting the tile kind via an ECS query is very slow
                // this should probably get its data from the View instead
                Topology::Hex => fancytint(
                    desc.size,
                    Hex::from(pos.0),
                    |c| q_pos
                        .get(index.get_pos(c))
                        .map(|d| *d)
                        .unwrap_or(TileKind::Water)
                ),
                Topology::Sq => fancytint(
                    desc.size,
                    Sq::from(pos.0),
                    |c| q_pos
                        .get(index.get_pos(c))
                        .map(|d| *d)
                        .unwrap_or(TileKind::Water)
                ),
            };
            spr.color.set_a(a);
        } else {
            spr.color.set_a(1.0);
        }
    }
}

fn tile_owner(
    settings: Res<AllSettings>,
    mut q: Query<(&mut TextureAtlasSprite, &TileOwner), Changed<TileOwner>>,
) {
    for (mut spr, owner) in &mut q {
        spr.color = settings.player_colors.visible[owner.0.i()].into();
    }
}
