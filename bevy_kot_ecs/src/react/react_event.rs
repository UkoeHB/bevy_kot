//local shortcuts

//third-party shortcuts

//standard shortcuts
use std::sync::Arc;

//-------------------------------------------------------------------------------------------------------------------

/// Data sent to event reactors.
/// - Data can only be accessed immutably.
pub struct ReactEvent<E: Send + Sync + 'static>
{
    data: Arc<E>,
}

impl<E: Send + Sync + 'static> ReactEvent<E>
{
    pub fn new(data: E) -> Self
    {
        Self{ data: Arc::new(data) }
    }

    pub fn get(&self) -> &E { &self.data }
}

impl<E: Send + Sync + 'static> Clone for ReactEvent<E> { fn clone(&self) -> Self { Self{ data: self.data.clone() } } }

//-------------------------------------------------------------------------------------------------------------------
