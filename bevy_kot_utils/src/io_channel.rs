//local shortcuts

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Multi-producer sender.
#[derive(Component, Resource, Clone, Debug)]
pub struct IoSender<T>
{
    sender: async_channel::Sender<T>
}

impl<T> IoSender<T>
{
    fn new(sender: async_channel::Sender<T>) -> IoSender<T>
    {
        IoSender{ sender }
    }

    /// Send a message.
    ///
    /// Returns `Err` if the channel is closed.
    pub fn send(&self, message: T) -> Result<(), async_channel::TrySendError<T>>
    {
        self.sender.try_send(message)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Multi--consumer receiver.
#[derive(Component, Resource, Clone, Debug)]
pub struct IoReceiver<T>
{
    receiver: async_channel::Receiver<T>
}

impl<T> IoReceiver<T>
{
    fn new(receiver: async_channel::Receiver<T>) -> IoReceiver<T>
    {
        IoReceiver{ receiver }
    }

    /// Get the next available message.
    ///
    /// Returns `None` if the channel is closed.
    pub async fn recv(&mut self) -> Option<T>
    {
        self.receiver.recv().await.ok()
    }

    /// Get the next available message.
    ///
    /// Returns `None` if there are no available messages or the channel is closed.
    pub fn try_recv(&mut self) -> Option<T>
    {
        let Ok(msg) = self.receiver.try_recv() else { return None; };
        Some(msg)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Create an unbounded multi-producer/single-consumer IO channel.
///
/// Uses an unbounded `async_channel` internally.
pub fn new_io_channel<T>() -> (IoSender<T>, IoReceiver<T>)
{
    let (channel_sender, channel_receiver) = async_channel::unbounded::<T>();
    (IoSender::<T>::new(channel_sender), IoReceiver::<T>::new(channel_receiver))
}

//-------------------------------------------------------------------------------------------------------------------
