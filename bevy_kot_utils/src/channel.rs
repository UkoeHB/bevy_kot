//local shortcuts

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Multi-producer sender.
#[derive(Component, Resource, Clone, Debug)]
pub struct Sender<T>
{
    sender: crossbeam::channel::Sender<T>
}

impl<T> Sender<T>
{
    fn new(sender: crossbeam::channel::Sender<T>) -> Sender<T>
    {
        Sender{ sender }
    }

    pub fn send(&self, message: T) -> Result<(), crossbeam::channel::SendError<T>>
    {
        self.sender.send(message)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Multi-consumer receiver.
#[derive(Component, Resource, Clone, Debug)]
pub struct Receiver<T>
{
    receiver: crossbeam::channel::Receiver<T>
}

impl<T> Receiver<T>
{
    fn new(receiver: crossbeam::channel::Receiver<T>) -> Receiver<T>
    {
        Receiver{ receiver }
    }

    pub fn try_recv(&self) -> Option<T>
    {
        let Ok(msg) = self.receiver.try_recv() else { return None; };
        Some(msg)
    }

    pub fn len(&self) -> usize
    {
        self.receiver.len()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Create an unbounded multi-producer/multi-consumer channel.
///
/// Uses an unbounded `crossbeam` channel internally.
pub fn new_channel<T>() -> (Sender<T>, Receiver<T>)
{
    let (channel_sender, channel_receiver) = crossbeam::channel::unbounded::<T>();
    return (Sender::<T>::new(channel_sender), Receiver::<T>::new(channel_receiver));
}

//-------------------------------------------------------------------------------------------------------------------
