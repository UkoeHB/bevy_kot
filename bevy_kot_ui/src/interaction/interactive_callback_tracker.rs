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
    cache: HashMap<Entity, Vec<SysName>>,
}

impl InteractiveCallbackTracker
{
    pub(crate) fn add(&mut self, entity: Entity, sys_name: SysName)
    {
        self.cache.entry(entity).or_default().push(sys_name);
    }

    pub(crate) fn remove(&mut self, entity: Entity) -> Vec<SysName>
    {
        self.cache.remove(&entity).unwrap_or_else(|| Vec::default())
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
    mut cache2   : Option<ResMut<IdMappedSystems<bool, ()>>>,
){
    despawns.iter().for_each(
            |entity|
            {
                let sys_names = tracker.remove(entity);
                for sys_name in sys_names
                {
                    if let Some(cache1) = &mut cache1 { cache1.revoke_sysname(sys_name); }
                    if let Some(cache2) = &mut cache2 { cache2.revoke_sysname(sys_name); }
                }
            }
        );
}

//-------------------------------------------------------------------------------------------------------------------
