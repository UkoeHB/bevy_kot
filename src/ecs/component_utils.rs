//local shortcuts

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Returns `true` if component added to entity and entity did not previously have the component.
pub fn try_add_component_to_entity<C: Component>(world: &mut World, entity: Entity, component: C) -> bool
{
    let Some(mut entity_mut) = world.get_entity_mut(entity) else { return false; };
    if entity_mut.contains::<C>() { return false; }
    entity_mut.insert(component);
    true
}

//-------------------------------------------------------------------------------------------------------------------

/// Returns `Some([component])` if component removed from entity.
pub fn try_remove_component_from_entity<C: Component>(world: &mut World, entity: Entity) -> Option<C>
{
    let Some(mut entity_mut) = world.get_entity_mut(entity) else { return None; };
    if !entity_mut.contains::<C>() { return None; }
    entity_mut.take::<C>()
}

//-------------------------------------------------------------------------------------------------------------------

/// Returns `true` if the component was added-to or updated on the entity.
/// - Returns `false` if the value wouldn't change.
/// - Does not trigger change detection unless the component is added or modified.
pub fn try_set_component<C: Component + Eq>(world: &mut World, entity: Entity, component: C) -> bool
{
    // try to get the entity
    let Some(mut entity_mut) = world.get_entity_mut(entity) else { return false; };

    // get or insert the value
    let Some(mut existing_component) = entity_mut.get_mut::<C>()
    else { entity_mut.insert(component); return true; };

    // update if value is new
    if *(existing_component.bypass_change_detection()) == component { return false; }
    *existing_component = component;
    true
}

//-------------------------------------------------------------------------------------------------------------------

/// Returns `true` if the component was updated on the entity with a different value.
/// - Returns `false` if the component did not previously exist on the entity, or if the value wouldn't change.
/// - Does not trigger change detection unless the component is modified.
pub fn try_update_component_if_different<C: Component + Eq>(world: &mut World, entity: Entity, component: C) -> bool
{
    let Some(mut entity_mut) = world.get_entity_mut(entity) else { return false; };
    let Some(mut existing_component) = entity_mut.get_mut::<C>() else { return false; };
    if *(existing_component.bypass_change_detection()) == component { return false; }
    *existing_component = component;
    true
}

//-------------------------------------------------------------------------------------------------------------------
