use std::{f32::consts::PI, time::Duration};

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;

use crate::{
    AppSystems,
    animation::{AnimationIndices, AnimationTimer},
    gameplay::{
        Health, Speed,
        enemy::{DamageCooldown, Enemy, KnockbackDirection, Meele, SPAWN_RADIUS},
        player::{Direction, Player},
        spells::Knockback,
    },
    screens::Screen,
};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        spawn_walker
            .run_if(on_timer(Duration::from_millis(2000)))
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::Update),
    );
}

#[derive(Component)]
#[require(
    Meele,
    Health(10.),
    Speed(50.),
    DamageCooldown,
    Transform,
    KnockbackDirection(Direction(Vec3 {
            x: 0.,
            y: 0.,
            z: 0.,
        })),
    Knockback(0.0),
    Enemy,
)]
#[derive(Reflect)]
pub(crate) struct Walker;

fn spawn_walker(
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, With<Player>>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) -> Result {
    let player_pos = player_query.single()?;

    let random_angle: f32 = rng.random_range(0.0..(2. * PI));
    let random_radius: f32 = rng.random_range(0.0..10.);
    let offset_x = (SPAWN_RADIUS + random_radius) * f32::sin(random_angle);
    let offset_y = (SPAWN_RADIUS + random_radius) * f32::cos(random_angle);

    let enemy_pos_x = player_pos.translation.x + offset_x;
    let enemy_pos_y = player_pos.translation.y + offset_y;

    let texture = asset_server.load("enemies/skeleton_.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(24), 7, 7, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { first: 0, last: 3 };

    commands.spawn((
        Name::new("Default Enemy"),
        Walker,
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first,
            },
        ),
        Transform::from_xyz(enemy_pos_x, enemy_pos_y, 0.).with_scale(Vec3::splat(32.0 / 22.0)),
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        DamageCooldown(Timer::from_seconds(0.5, TimerMode::Repeating)),
    ));

    Ok(())
}
