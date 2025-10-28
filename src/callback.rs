use bevy::prelude::*;

/// Event that animations, cutscenes, etc will use to perform callbacks
/// Pass the entity with an entity observer attached (the callback)
/// The animation/cutscene manager should trigger this event once finished
#[derive(EntityEvent)]
pub struct Callback {
    pub entity: Entity,
}
