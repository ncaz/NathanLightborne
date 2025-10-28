use std::time::Duration;

use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};

use crate::asset::LoadResource;

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<BgmTracks>();
        app.load_resource::<BgmTracks>();
        app.add_observer(handle_change_bgm_event);
        app.add_systems(Update, fade_bgm);
    }
}

#[derive(Component, Default)]
pub struct BgmMarker;

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct BgmTracks {
    #[dependency]
    mustnt_stop: Handle<AudioSource>,
    #[dependency]
    light_in_the_dark: Handle<AudioSource>,
    #[dependency]
    cutscene_1_draft: Handle<AudioSource>,
    #[dependency]
    level_select: Handle<AudioSource>,
}

impl FromWorld for BgmTracks {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        Self {
            mustnt_stop: asset_server.load("music/Mustn't Stop - M2 Version.mp3"),
            light_in_the_dark: asset_server.load("music/A Light in the Dark - Two Loops.mp3"),
            cutscene_1_draft: asset_server.load("music/lightborne cutscene 1 draft 2.mp3"),
            level_select: asset_server.load("music/main_menu.wav"),
        }
    }
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum BgmTrack {
    MustntStop,
    LightInTheDark,
    Cutscene1Draft,
    LevelSelect,
    #[default]
    None,
}

pub const BGM_VOLUME: Volume = Volume::Linear(0.8);

/// Fades out all other bgm tracks, and spawns the selected track
#[derive(Event)]
pub struct ChangeBgmEvent(pub BgmTrack);

pub fn handle_change_bgm_event(
    event: On<ChangeBgmEvent>,
    mut commands: Commands,
    q_active_tracks: Query<(Entity, &AudioSink), With<BgmMarker>>,
    tracks: Res<BgmTracks>,
    mut current_bgm: Local<BgmTrack>,
) {
    const BGM_FADE_DURATION: Duration = Duration::from_secs(3);

    for (track, sink) in q_active_tracks.iter() {
        commands.entity(track).insert((
            Fade::new(BGM_FADE_DURATION, sink.volume(), Volume::Linear(0.0)),
            FadeSettings::Despawn,
        ));
    }

    let source = match event.0 {
        BgmTrack::MustntStop => tracks.mustnt_stop.clone(),
        BgmTrack::LightInTheDark => tracks.light_in_the_dark.clone(),
        BgmTrack::Cutscene1Draft => tracks.cutscene_1_draft.clone(),
        BgmTrack::LevelSelect => tracks.level_select.clone(),
        BgmTrack::None => return,
    };

    commands.spawn((
        AudioPlayer::new(source),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::Linear(0.0),
            ..default()
        },
        Fade::new(BGM_FADE_DURATION, Volume::Linear(0.0), BGM_VOLUME),
        BgmMarker,
    ));

    *current_bgm = event.0;
}

#[derive(Component, Default, Debug, PartialEq, Eq)]
pub enum FadeSettings {
    Despawn,
    #[default]
    Continue,
}

#[derive(Component)]
#[require(FadeSettings)]
pub struct Fade {
    timer: Timer,
    from: Volume,
    to: Volume,
}

impl Fade {
    pub fn new(duration: Duration, from: Volume, to: Volume) -> Self {
        Self {
            timer: Timer::new(duration, TimerMode::Once),
            from,
            to,
        }
    }
}

fn fade_bgm(
    mut commands: Commands,
    mut audio_sink: Query<(&mut AudioSink, Entity, &mut Fade, &FadeSettings)>,
    time: Res<Time>,
    global_volume: Res<GlobalVolume>,
) {
    for (mut audio, entity, mut fade, fade_settings) in audio_sink.iter_mut() {
        fade.timer.tick(time.delta());
        let progress = fade.timer.elapsed_secs() / fade.timer.duration().as_secs_f32();
        audio.set_volume(fade.from.fade_towards(fade.to, progress) * global_volume.volume);
        if !fade.timer.just_finished() {
            continue;
        }

        // make sure its actually the end vol
        audio.set_volume(fade.to * global_volume.volume);

        match fade_settings {
            FadeSettings::Continue => {
                commands.entity(entity).remove::<Fade>();
            }
            FadeSettings::Despawn => {
                commands.entity(entity).despawn();
            }
        }
    }
}
