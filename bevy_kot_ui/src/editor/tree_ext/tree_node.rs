//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use std::marker::PhantomData;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component)]
pub struct TreeNode<Ui: LunexUi>(PhantomData<Ui>);

impl Default for TreeNode<Ui> { fn default() -> Self { Self(PhantomData::default()) } }

//-------------------------------------------------------------------------------------------------------------------
