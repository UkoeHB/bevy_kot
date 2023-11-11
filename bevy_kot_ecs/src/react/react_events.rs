//local shortcuts

//third-party shortcuts
use bevy::ecs::event::Event;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

struct ReactEventSync(u64);

impl FromWorld for ReactEventSync
{
    fn from_world(world: &mut World) -> Self
    {
        Self(world.resource::<ReactEventCounter>().0)
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default)]
pub(crate) struct ReactEventCounter(u64);

impl ReactEventCounter
{
    pub(crate) fn increment(&mut self) -> u64
    {
        self.0 += 1;
        self.0
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Event)]
pub(crate) struct ReactEventInner<E: Send + Sync + 'static>
{
    /// This event's id.
    pub(crate) event_id: u64,
    /// The event.
    pub(crate) event: E,
}

//-------------------------------------------------------------------------------------------------------------------

/// Provides access to react events of type `E`.
///
/// Will **not** return react events sent before the system that contains the `ReactEvents` param was intialized in
/// the world.
///
/// It is only recommended to use this inside systems registered as event reactors with [`ReactCommands`]. The behavior
/// is likely to be unexpected if used anywhere else.
#[derive(SystemParam)]
pub struct ReactEvents<'w, 's, E: Send + Sync + 'static>
{
    /// Event counter recording the last-seen react event. Used to prevent this param from exposing events sent before
    /// a system was registered.
    sync: Local<'s, ReactEventSync>,
    /// Reads events.
    reader: EventReader<'w, 's, ReactEventInner<E>>,
}

impl<'w, 's, E: Send + Sync + 'static> ReactEvents<'w, 's, E>
{
    /// Iterate over all currently-pending react events.
    ///
    /// It is recommended to instead use [`ReactEvents::next()`] once per invocation of an event reactor system, since
    /// event reactors are invoked once per event.
    pub fn iter(&mut self) -> impl Iterator<Item = &E> + '_
    {
        let floor = self.sync.0;
        self.reader
            .iter()
            .filter_map(
                move |e|
                {
                    if e.event_id <= floor { return None; }
                    Some(&e.event)
                }
            )
    }

    /// Get the next available event.
    pub fn next(&mut self) -> Option<&E>
    {
        self.iter().next()
    }

    /// Check if the events queue is empty.
    pub fn is_empty(&self) -> bool
    {
        self.reader.is_empty()
    }

    /// Get number of pending events.
    ///
    //todo: this is not accurate since we may need to ignore some events in the internal reader 
    /*
    pub fn len(&self) -> usize
    {
        self.reader.len()
    }
    */

    /// Clear all pending events in this reader.
    pub fn clear(&mut self)
    {
        self.reader.clear()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Extends the `World` API with reactive event methods.
pub trait ReactEventAppExt
{
    fn add_react_event<E: Send + Sync + 'static>(&mut self) -> &mut Self;
}

impl ReactEventAppExt for App
{
    fn add_react_event<E: Send + Sync + 'static>(&mut self) -> &mut Self
    {
        self.add_event::<ReactEventInner<E>>()
    }
}

//-------------------------------------------------------------------------------------------------------------------
