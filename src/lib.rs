use bevy::prelude::*;
use bevy_enhanced_input::EnhancedInputPlugin;
use bevy_rand::{plugin::EntropyPlugin, prelude::WyRand};
use bevy_seedling::prelude::*;
use gameplay::player::Player;
use screens::Screen;

mod animation;
mod audio;
#[cfg(feature = "dev")]
mod dev_tools;
mod gameplay;
mod screens;
mod widgets;

pub fn plugin(app: &mut App) {
    app.configure_sets(
        Update,
        (AppSystems::RecordInput, AppSystems::Update).chain(),
    );

    app.add_systems(Startup, spawn_camera);

    app.add_systems(Update, update_camera.run_if(in_state(Screen::Gameplay)));

    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Window {
                    title: "bevy survivor".to_string(),
                    fit_canvas_to_parent: true,
                    ..default()
                }
                .into(),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
        EnhancedInputPlugin,
        EntropyPlugin::<WyRand>::default(),
        SeedlingPlugin::default(),
    ));

    app.add_plugins((animation::plugin, gameplay::plugin, audio::plugin));

    #[cfg(feature = "dev")]
    app.add_plugins(dev_tools::plugin);

    app.add_plugins(screens::plugin);
}

const ENEMY_SIZE: f32 = 30.0;
const PLAYER_SIZE: f32 = 30.0;
const SPELL_SIZE: f32 = 16.0;
const XP_GAIN_GEM: f32 = 10.;

/// How quickly should the camera snap to the desired location.
const CAMERA_DECAY_RATE: f32 = 2.;

/// High-level groupings of systems for the app in the `Update` schedule.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AppSystems {
    /// Record player input.
    RecordInput,
    /// Do everything else
    Update,
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        Projection::from(OrthographicProjection {
            scale: 0.75,
            ..OrthographicProjection::default_2d()
        }),
        // Render all UI to this camera.
        // Not strictly necessary since we only use one camera,
        // but if we don't use this component, our UI will disappear as soon
        // as we add another camera. This includes indirect ways of adding cameras like using
        // [ui node outlines](https://bevyengine.org/news/bevy-0-14/#ui-node-outline-gizmos)
        // for debugging. So it's good to have this here for future-proofing.
        IsDefaultUiCamera,
    ));
}

/// Update the camera position by tracking the player.
fn update_camera(
    mut camera: Single<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
    time: Res<Time>,
) {
    let Vec3 { x, y, .. } = player.translation;
    let direction = Vec3::new(x, y, camera.translation.z);

    // Applies a smooth effect to camera movement using stable interpolation
    // between the camera position and the player position on the x and y axes.
    camera
        .translation
        .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());
}
