//local shortcuts
use crate::*;
use bevy_kot_ecs::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use std::collections::HashMap;

//-------------------------------------------------------------------------------------------------------------------

/// Tracks interactive callbacks in order to clean them up when interactive elements are despawned.
#[derive(Resource)]
pub(crate) struct InteractiveCallbackTracker
{
    cache: HashMap<Entity, Vec<SysId>>,
}

impl InteractiveCallbackTracker
{
    pub(crate) fn add(&mut self, entity: Entity, sys_id: SysId)
    {
        self.cache.entry(entity).or_default().push(sys_id);
    }

    pub(crate) fn remove(&mut self, entity: Entity) -> Vec<SysId>
    {
        self.cache.remove(&entity).unwrap_or(Vec::default())
    }
}

impl Default for InteractiveCallbackTracker
{
    fn default() -> Self { Self{ cache: HashMap::default() } }
}

//-------------------------------------------------------------------------------------------------------------------

/// Revoke callbacks for despawned interactive elements.
pub(crate) fn cleanup_interactive_callbacks(
    mut despawns : RemovedComponents<InteractiveElementTag>,
    mut tracker  : ResMut<InteractiveCallbackTracker>,
    mut cache1   : Option<ResMut<IdMappedSystems<(), ()>>>,
    mut cache2   : Option<ResMut<IdMappedSystems<Vec2, ()>>>,
    mut cache3   : Option<ResMut<IdMappedSystems<(Vec2, bool), ()>>>,
){
    despawns.iter().for_each(
            |entity|
            {
                let sys_ids = tracker.remove(entity);
                for sys_id in sys_ids
                {
                    if let Some(cache1) = &mut cache1 { cache1.revoke_sysid(sys_id); }
                    if let Some(cache2) = &mut cache2 { cache2.revoke_sysid(sys_id); }
                    if let Some(cache3) = &mut cache3 { cache3.revoke_sysid(sys_id); }
                }
            }
        );
}

//-------------------------------------------------------------------------------------------------------------------
