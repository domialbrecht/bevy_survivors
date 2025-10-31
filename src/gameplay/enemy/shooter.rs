use std::{f32::consts::PI, time::Duration};

use bevy::{prelude::*, time::common_conditions::on_timer};

use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;

use crate::{
    AppSystems,
    animation::{AnimationIndices, AnimationTimer},
    gameplay::{
        Health, Speed,
        enemy::{
            AbilityDamage, DamageCooldown, Enemy, EnemyProjectile, EnemyType, KnockbackDirection,
            ProjectileOf, ProjectileSpeed, Ranged, SPAWN_RADIUS,
        },
        player::{Direction, Player, PlayerHitEvent},
        spells::{Cooldown, Damage, Knockback, Range},
    },
    screens::Screen,
};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        spawn_shooter
            .run_if(on_timer(Duration::from_millis(2000)))
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::Update),
    );

    app.add_observer(shooter_attack);
    app.add_observer(shooter_projectile_hit);
}

#[derive(Component)]
#[require(
    EnemyType::Shooter,
    Ranged,
    Health(10.),
    Speed(100.),
    Knockback(0.0),
    KnockbackDirection(Direction(Vec3 {
        x: 0.,
        y: 0.,
        z: 0.,
    })),
    //Meele hit
    DamageCooldown(Timer::from_seconds(0.5, TimerMode::Repeating)),
    //Shoot cd
    Cooldown(Timer::from_seconds(2.0,TimerMode::Once)),
    Damage(1.0),
    AbilityDamage(5.0),
    Range(200.0),
    ProjectileSpeed(125.),
)]
pub(crate) struct Shooter;

#[derive(Event)]
pub(crate) struct ShooterAttackEvent(pub Entity);

#[derive(Event)]
pub(crate) struct ShooterProjectileHitEvent {
    pub projectile: Entity,
    pub source: Entity,
}

fn spawn_shooter(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    player_query: Query<&Transform, With<Player>>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    shooter_q: Query<&Shooter>,
) -> Result {
    let player_pos = player_query.single()?;

    let random_angle: f32 = rng.random_range(0.0..(2. * PI));
    let random_radius: f32 = rng.random_range(0.0..10.);
    let offset_x = (SPAWN_RADIUS + random_radius) * f32::sin(random_angle);
    let offset_y = (SPAWN_RADIUS + random_radius) * f32::cos(random_angle);

    let enemy_pos_x = player_pos.translation.x + offset_x;
    let enemy_pos_y = player_pos.translation.y + offset_y;

    let mut shooter_count = shooter_q.iter().count();
    shooter_count += 1;

    let texture = asset_server.load("enemies/necromancer_.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(24), 8, 5, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { first: 0, last: 3 };

    commands.spawn((
        Name::new(format!("Shooter {shooter_count}")),
        Enemy,
        Shooter,
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
    ));

    Ok(())
}

fn shooter_attack(
    trigger: On<ShooterAttackEvent>,
    shooter_q: Query<&Transform, With<Shooter>>,
    player_q: Query<&Transform, With<Player>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) -> Result {
    let shooter = trigger.0;
    let player_pos = player_q.single()?.translation.truncate();

    let Ok(transform) = shooter_q.get(shooter) else {
        return Ok(());
    };

    let shooter_pos = transform.translation.truncate();
    let direction = (player_pos - shooter_pos).normalize();
    let angle = direction.y.atan2(direction.x);

    commands.spawn((
        Sprite {
            image: asset_server.load("enemies/shooter_bullet.png"),
            ..default()
        },
        Transform {
            translation: transform.translation,
            rotation: Quat::from_rotation_z(angle),
            ..default()
        },
        EnemyProjectile,
        ProjectileOf(shooter),
        Direction(direction.extend(0.0)),
    ));

    Ok(())
}

fn shooter_projectile_hit(
    trigger: On<ShooterProjectileHitEvent>,
    shooter_q: Query<&AbilityDamage, With<Shooter>>,
    mut commands: Commands,
) {
    let projectile = trigger.projectile;
    let shooter = trigger.source;

    let Ok(damage) = shooter_q.get(shooter) else {
        return;
    };

    commands.trigger(PlayerHitEvent { dmg: damage.0 });

    commands.entity(projectile).despawn();
}
