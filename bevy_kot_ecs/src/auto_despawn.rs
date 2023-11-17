//local shortcuts
use bevy_kot_utils::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use std::sync::Arc;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

struct AutoDespawnSignalInner
{
    entity: Entity,
    sender: Sender<Entity>,
}

impl Drop for AutoDespawnSignalInner
{
    fn drop(&mut self)
    {
        let _ = self.sender.send(self.entity);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn auto_despawn(mut commands: Commands, despawns: Res<AutoDespawn>)
{
    while let Some(entity) = despawns.try_recv()
    {
        let Some(mut entity_commands) = commands.get_entity(entity) else { continue; };
        entity_commands.despawn();
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Creates [`AutoDespawnSignal`]s.
#[derive(Resource, Clone)]
pub struct AutoDespawn
{
    sender: Sender<Entity>,
    receiver: Receiver<Entity>,
}

impl AutoDespawn
{
    fn new() -> Self
    {
        let (sender, receiver) = new_channel();
        Self{ sender, receiver }
    }

    /// Get an RAII auto despawn signal for the given `entity`.
    ///
    /// When the last copy of the signal is dropped, the entity will be despawned in the `Last` schedule.
    pub fn signal(&self, entity: Entity) -> AutoDespawnSignal
    {
        AutoDespawnSignal::new(entity, self.sender.clone())
    }

    fn try_recv(&self) -> Option<Entity>
    {
         self.receiver.try_recv()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// RAII handle to a despawn signal.
///
/// The signal can be cloned. When the last copy is dropped, the entity will be despawned in the `Last` schedule.
pub struct AutoDespawnSignal(Arc<AutoDespawnSignalInner>);

impl AutoDespawnSignal
{
    fn new(entity: Entity, sender: Sender<Entity>) -> Self
    {
        Self(Arc::new(AutoDespawnSignalInner{ entity, sender }))
    }
}

impl Clone for AutoDespawnSignal
{
    fn clone(&self) -> Self { Self(self.0.clone()) }
}

//-------------------------------------------------------------------------------------------------------------------

/// Extends the `App` API with a method to set up auto despawning.
pub trait AutoDespawnAppExt
{
    /// Set up auto despawning. Can be added to multiple plugins without conflict.
    fn setup_auto_despawn(&mut self) -> &mut Self;
}

impl AutoDespawnAppExt for App
{
    fn setup_auto_despawn(&mut self) -> &mut Self
    {
        if self.world.contains_resource::<AutoDespawn>() { return self; }
        self.insert_resource(AutoDespawn::new())
            .add_systems(Last, auto_despawn)
    }
}

//-------------------------------------------------------------------------------------------------------------------
