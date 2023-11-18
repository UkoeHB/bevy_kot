//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::ecs::system::SystemParamItem;
use bevy::ecs::system::lifetimeless::SRes;
use bevy::prelude::*;

//standard shortcuts
use std::marker::PhantomData;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Debug, Default)]
pub struct MouseLButton<U: LunexUi, C: LunexCursor>
{
    _phantom: PhantomData<(U, C)>,
}

impl<U: LunexUi, C: LunexCursor> InteractionSource for MouseLButton<U, C>
{
    type SourceParam = SRes<Input<MouseButton>>;
    type LunexUi     = U;
    type LunexCursor = C;

    fn just_clicked(&self, source: &SystemParamItem<SRes<Input<MouseButton>>>) -> bool
    {
        source.just_pressed(MouseButton::Left)
    }
    fn is_clicked(&self, source: &SystemParamItem<SRes<Input<MouseButton>>>) -> bool
    {
        source.pressed(MouseButton::Left)
    }
    fn just_unclicked(&self, source: &SystemParamItem<SRes<Input<MouseButton>>>) -> bool
    {
        source.just_released(MouseButton::Left)
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Debug, Default)]
pub struct MouseRButton<U: LunexUi, C: LunexCursor>
{
    _phantom: PhantomData<(U, C)>,
}

impl<U: LunexUi, C: LunexCursor> InteractionSource for MouseRButton<U, C>
{
    type SourceParam = SRes<Input<MouseButton>>;
    type LunexUi     = U;
    type LunexCursor = C;

    fn just_clicked(&self, source: &SystemParamItem<SRes<Input<MouseButton>>>) -> bool
    {
        source.just_pressed(MouseButton::Right)
    }
    fn is_clicked(&self, source: &SystemParamItem<SRes<Input<MouseButton>>>) -> bool
    {
        source.pressed(MouseButton::Right)
    }
    fn just_unclicked(&self, source: &SystemParamItem<SRes<Input<MouseButton>>>) -> bool
    {
        source.just_released(MouseButton::Right)
    }
}

//-------------------------------------------------------------------------------------------------------------------
