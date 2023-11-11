//local shortcuts

//third-party shortcuts
use bevy::ecs::event::Event;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(Event)]
pub(crate) struct ReactEventInner<E: Send + Sync + 'static>(pub(crate) E);

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemParam)]
pub struct ReactEvents<'w, 's, E: Send + Sync + 'static>
{
    reader: EventReader<'w, 's, ReactEventInner<E>>,
}

impl<'w, 's, E: Send + Sync + 'static> ReactEvents<'w, 's, E>
{
    pub fn read(&mut self) -> impl Iterator<Item = &E> + '_
    {
        self.reader.iter().map(|e| &e.0)
    }

    pub fn next(&mut self) -> Option<&E>
    {
        self.iter().next()
    }

    pub fn is_empty(&self) -> bool
    {
        self.reader.is_empty()
    }

    pub fn len(&self) -> usize
    {
        self.reader.len()
    }

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
