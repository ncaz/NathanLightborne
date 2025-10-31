use avian2d::{math::*, prelude::*};
use bevy::{ecs::query::Has, prelude::*};

use crate::{
    game::{lyra::Lyra, LevelSystems},
    shared::PlayState,
};

/// The number of [`FixedUpdate`] steps the player can jump for after pressing the spacebar.
const SHOULD_JUMP_TICKS: isize = 8;
/// The number of [`FixedUpdate`] steps the player can jump for after falling off an edge.
const COYOTE_TIME_TICKS: isize = 5;
/// The number of [`FixedUpdate`] steps the player should receive upward velocity for.
const JUMP_BOOST_TICKS: isize = 2;

/// Max player horizontal velocity.
const PLAYER_MAX_H_VEL: f32 = 1.5;
/// Max player vertical velocity.
const PLAYER_MAX_Y_VEL: f32 = 5.;
/// The positive y velocity added to the player every jump boost tick.
const PLAYER_JUMP_VEL: f32 = 2.2;
/// The x velocity added to the player when A/D is held.
const PLAYER_MOVE_VEL: f32 = 0.4;
/// The y velocity subtracted from the player due to gravity.
const PLAYER_GRAVITY: f32 = 0.15;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MovementAction>();
        app.add_systems(Update, keyboard_input.in_set(LevelSystems::Input));
        app.add_systems(
            FixedUpdate,
            update_grounded
                .before(movement)
                .in_set(LevelSystems::Simulation),
        );
        app.add_systems(FixedUpdate, movement.in_set(LevelSystems::Simulation));
        app.add_systems(
            PhysicsSchedule,
            kinematic_controller_collisions
                .in_set(NarrowPhaseSystems::Last)
                .run_if(in_state(PlayState::Playing)),
        );
        app.add_systems(OnExit(PlayState::Playing), cache_linear_vel);
        app.add_systems(OnEnter(PlayState::Playing), res_linear_vel);
    }
}

#[derive(Component)]
pub struct CachedLinearVelocity(pub Vector);

pub fn cache_linear_vel(
    lyra: Single<(&mut LinearVelocity, &mut CachedLinearVelocity), With<Lyra>>,
) {
    let (mut linvel, mut cache) = lyra.into_inner();
    cache.0 = linvel.0;
    linvel.0 = Vec2::ZERO;
}

pub fn res_linear_vel(lyra: Single<(&mut LinearVelocity, &mut CachedLinearVelocity), With<Lyra>>) {
    let (mut linvel, mut cache) = lyra.into_inner();
    linvel.0 = cache.0;
    cache.0 = Vec2::ZERO;
}

/// A [`Message`] written for a movement input action.
#[derive(Message)]
pub enum MovementAction {
    Move(Scalar),
    Jump,
    JumpCut,
    Crouch,
    Stand,
}

/// A marker component indicating that an entity is using a character controller.
#[derive(Component)]
pub struct CharacterController;

/// A marker component indicating that an entity is on the ground.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;

/// A bundle that contains components for character movement.
#[derive(Component, Default)]
pub struct MovementInfo {
    pub should_jump_ticks: isize,
    pub coyote_time_ticks: isize,
    pub jump_boost_ticks: isize,
    pub crouched: bool,
}

pub fn keyboard_input(
    mut movement_writer: MessageWriter<MovementAction>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    let horizontal = right as i8 - left as i8;
    let direction = horizontal as Scalar;

    if direction != 0.0 {
        movement_writer.write(MovementAction::Move(direction));
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        movement_writer.write(MovementAction::Jump);
    }
    if keyboard_input.just_released(KeyCode::Space) {
        movement_writer.write(MovementAction::JumpCut);
    }
    if keyboard_input.just_pressed(KeyCode::ShiftLeft) {
        movement_writer.write(MovementAction::Crouch);
    }
    if keyboard_input.just_released(KeyCode::ShiftLeft) {
        movement_writer.write(MovementAction::Stand);
    }
}

pub fn update_grounded(
    mut commands: Commands,
    mut query: Query<(Entity, &ShapeHits), With<CharacterController>>,
) {
    for (entity, hits) in &mut query {
        let is_grounded = hits.iter().next().is_some();
        if is_grounded {
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

pub fn movement(
    time: Res<Time>,
    mut movement_reader: MessageReader<MovementAction>,
    mut controllers: Query<(
        &mut MovementInfo,
        &mut LinearVelocity,
        &ShapeHits,
        Has<Grounded>,
    )>,
) {
    let delta = time.delta_secs() * 64.;
    for (mut movement_info, mut linear_velocity, shape_hits, is_grounded) in &mut controllers {
        if is_grounded {
            movement_info.coyote_time_ticks = COYOTE_TIME_TICKS;
        }

        let mut moved = false;
        let mut crouch = false;
        for event in movement_reader.read() {
            match event {
                MovementAction::Move(direction) => {
                    linear_velocity.x += *direction * PLAYER_MOVE_VEL * 64. * delta;
                    moved = true;
                }
                MovementAction::Jump => {
                    movement_info.should_jump_ticks = SHOULD_JUMP_TICKS;
                }
                MovementAction::JumpCut => {
                    if linear_velocity.y > 0. {
                        linear_velocity.y /= 3.;
                        movement_info.jump_boost_ticks = 0;
                        movement_info.should_jump_ticks = 0;
                    }
                }
                MovementAction::Crouch => crouch = true,
                MovementAction::Stand => crouch = false,
            }
        }

        if movement_info.should_jump_ticks > 0 && movement_info.coyote_time_ticks > 0 {
            movement_info.jump_boost_ticks = JUMP_BOOST_TICKS;
        }

        let too_close = shape_hits.iter().any(|hit| hit.distance < 0.25);
        if movement_info.jump_boost_ticks > 0 {
            linear_velocity.y = PLAYER_JUMP_VEL * 64.;
        } else if too_close && linear_velocity.y < 0.5 {
            linear_velocity.y = 0.45;
        } else if is_grounded && linear_velocity.y < 0.5 {
            linear_velocity.y = 0.;
        } else {
            linear_velocity.y -= PLAYER_GRAVITY * 64. * delta;
        }

        linear_velocity.y = linear_velocity
            .y
            .clamp(-PLAYER_MAX_Y_VEL * 64., PLAYER_MAX_Y_VEL * 64.);

        let crouch_modif = if crouch { 0.5 } else { 1.0 };
        linear_velocity.x = linear_velocity.x.clamp(
            -PLAYER_MAX_H_VEL * 64. * crouch_modif,
            PLAYER_MAX_H_VEL * 64. * crouch_modif,
        );

        if !moved {
            linear_velocity.x *= 0.6;
            if linear_velocity.x.abs() < 0.1 {
                linear_velocity.x = 0.;
            }
        }

        movement_info.should_jump_ticks -= 1;
        movement_info.jump_boost_ticks -= 1;
        movement_info.coyote_time_ticks -= 1;
    }
}

/// Kinematic bodies do not get pushed by collisions by default,
/// so it needs to be done manually.
///
/// This system handles collision response for kinematic character controllers
/// by pushing them along their contact normals by the current penetration depth,
/// and applying velocity corrections in order to snap to slopes, slide along walls,
/// and predict collisions using speculative contacts.
#[allow(clippy::type_complexity)]
fn kinematic_controller_collisions(
    collisions: Collisions,
    bodies: Query<&RigidBody>,
    collider_rbs: Query<&ColliderOf, Without<Sensor>>,
    mut character_controllers: Query<
        (&mut Position, &mut LinearVelocity),
        (With<RigidBody>, With<CharacterController>),
    >,
    time: Res<Time>,
) {
    // Iterate through collisions and move the kinematic body to resolve penetration
    for contacts in collisions.iter() {
        // Get the rigid body entities of the colliders (colliders could be children)
        let Ok([&ColliderOf { body: rb1 }, &ColliderOf { body: rb2 }]) =
            collider_rbs.get_many([contacts.collider1, contacts.collider2])
        else {
            continue;
        };

        // Get the body of the character controller and whether it is the first
        // or second entity in the collision.
        let is_first: bool;

        let character_rb: RigidBody;
        let is_other_dynamic: bool;

        let (mut position, mut linear_velocity) =
            if let Ok(character) = character_controllers.get_mut(rb1) {
                is_first = true;
                character_rb = *bodies.get(rb1).unwrap();
                is_other_dynamic = bodies.get(rb2).is_ok_and(|rb| rb.is_dynamic());
                character
            } else if let Ok(character) = character_controllers.get_mut(rb2) {
                is_first = false;
                character_rb = *bodies.get(rb2).unwrap();
                is_other_dynamic = bodies.get(rb1).is_ok_and(|rb| rb.is_dynamic());
                character
            } else {
                continue;
            };

        // This system only handles collision response for kinematic character controllers.
        if !character_rb.is_kinematic() {
            continue;
        }

        // Iterate through contact manifolds and their contacts.
        // Each contact in a single manifold shares the same contact normal.
        for manifold in contacts.manifolds.iter() {
            let normal = if is_first {
                -manifold.normal
            } else {
                manifold.normal
            };

            let mut deepest_penetration: Scalar = Scalar::MIN;
            for contact in manifold.points.iter() {
                if contact.penetration > 0.0 {
                    position.0 += normal * (contact.penetration + 0.02);
                }
                deepest_penetration = deepest_penetration.max(contact.penetration);
            }

            // For now, this system only handles velocity corrections for collisions against static geometry.
            if is_other_dynamic {
                continue;
            }

            if deepest_penetration <= 0.0 {
                // The character is not yet intersecting the other object,
                // but the narrow phase detected a speculative collision.
                //
                // We need to push back the part of the velocity
                // that would cause penetration within the next frame.
                let normal_speed = linear_velocity.dot(normal);

                // Don't apply an impulse if the character is moving away from the surface.
                if normal_speed >= 0.0 {
                    continue;
                }

                // Compute the impulse to apply.
                let impulse_magnitude =
                    normal_speed - (deepest_penetration / time.delta_secs_f64().adjust_precision());
                let mut impulse = impulse_magnitude * normal;

                // Apply the impulse differently depending on the slope angle.
                impulse.y = impulse.y.max(0.0);
                linear_velocity.0 -= impulse;
            }
        }
    }
}
