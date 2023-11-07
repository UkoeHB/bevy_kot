//local shortcuts

//third-party shortcuts
use bevy::utils::all_tuples;

//standard shortcuts
use std::any::Any;
use std::sync::Arc;

//-------------------------------------------------------------------------------------------------------------------

/// Identifies style structs.
pub trait Style: Send + Sync + 'static {}

impl<S: Style> StyleBundle for S
{
    #[inline]
    fn get_styles(self, func: &mut impl FnMut(Arc<dyn Any + Send + Sync + 'static>))
    {
        func(Arc::new(self));
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Collection of styles.
///
/// All members of a style bundle must implement [`StyleBundle`]. You should implement [`Style`] on the root members of
/// a style bundle.
pub trait StyleBundle: Send + Sync + 'static
{
    /// Calls `func` on each value, in the order of this bundle's [`Style`]s.
    ///
    /// This passes type-erased ownership of the style values to `func`.
    fn get_styles(self, func: &mut impl FnMut(Arc<dyn Any + Send + Sync + 'static>));
}

//-------------------------------------------------------------------------------------------------------------------

// Implements [`StyleBundle`] for tuples.
macro_rules! tuple_impl
{
    ($($name: ident),*) =>
    {
        impl<$($name: StyleBundle),*> StyleBundle for ($($name,)*)
        {
            #[allow(unused_variables, unused_mut)]
            #[inline(always)]
            fn get_styles(self, func: &mut impl FnMut(Arc<dyn Any + Send + Sync + 'static>))
            {
                #[allow(non_snake_case)]
                let ($(mut $name,)*) = self;
                $(
                    $name.get_styles(&mut *func);
                )*
            }
        }
    }
}

all_tuples!(tuple_impl, 0, 15, B);

//-------------------------------------------------------------------------------------------------------------------
