//local shortcuts
use crate::*;
use bevy_kot_ecs::*;

//third-party shortcuts
use bevy::ecs::system::StaticSystemParam;
use bevy::prelude::*;
use bevy_lunex::prelude::*;

//standard shortcuts
use std::fmt::Debug;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug)]
struct InteractionSourceInfoPack
{
    just_clicked      : bool,
    is_clicked        : bool,
    just_unclicked    : bool,
    target_is_hovered : bool,
    depth_limit       : Option<f32>,
    targeted          : Option<Entity>,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

//todo: how to handle UiTrees occluding each other?
//todo: custom system param filter to get only widgets associated with ui in focused window ?? UIInFocusedWindow<LunexUI>
fn try_get_interaction_source_info_pack<S: InteractionSource>(
    ui              : Query<&UiTree, With<S::LunexUI>>,  //todo: InFocusedWindow
    cursor_pos      : CursorPos<S::LunexCursor>,
    source          : Res<S>,
    source_param    : StaticSystemParam<S::SourceParam>,
    barrier_param   : StaticSystemParam<<<S as InteractionSource>::LunexCursor as LunexCursor>::BarrierParam>,
    element_param   : StaticSystemParam<<<S as InteractionSource>::LunexCursor as LunexCursor>::ElementParam>,
    home_zone_param : StaticSystemParam<<<S as InteractionSource>::LunexCursor as LunexCursor>::HomeZoneParam>,
    barrier_widgets : Query<
        (Entity, &Widget),
        Or<(With<UIInteractionBarrier<S::LunexUI>>, With<InteractionBarrier<S::LunexUI, S::LunexCursor>>)>
    >,
    unpressed_elements: Query<
        (Entity, &Widget),
        (Without<Pressed>, With<ElementInteractionTargeter<S::LunexUI, S::LunexCursor>>)
    >,
    pressed_elements: Query<
        (Entity, &Widget, &PressHomeZone),
        (With<Pressed>, With<ElementInteractionTargeter<S::LunexUI, S::LunexCursor>>)
    >,
) -> Option<InteractionSourceInfoPack>
{
    // check that ui and cursor are available
    // - note that the cursor is more likely to be unavailable than the ui tree
    if cursor_pos.available() || ui.is_empty() { return None; };

    let Ok(ui) = ui.get_single()
    else { tracing::error!("multiple uis with the same tag detected in the focused window"); return None; };

    // get positions
    let Some(cpos_world) = cursor_pos.get() else { tracing::error!("unable to access the cursor position"); return None; };
    let cpos_lunex = cpos_world.as_lunex(ui.offset);

    // find top-most barrier widget under the cursor
    let mut depth_limit: Option<f32> = None;

    for (entity, widget) in barrier_widgets.iter()
    {
        // check visibility
        let Ok(widget_branch) = widget.fetch(&ui) else { continue; };
        if !widget_branch.is_visible() { continue; }

        // check if barrier widget intersects with the cursor
        let Ok(Some(widget_depth)) = S::LunexCursor::cursor_intersects_barrier(
                cpos_world,
                cpos_lunex,
                ui,
                widget,
                entity,
                depth_limit,
                widget_branch.get_depth(),
                &barrier_param,
            ) else { continue; };

        depth_limit = Some(widget_depth);
    }

    // get highest unpressed element under the cursor
    let mut top_unpressed: Option<Entity> = None;
    let mut target_limit = depth_limit;

    for (entity, widget) in unpressed_elements.iter()
    {
        // check visibility
        let Ok(widget_branch) = widget.fetch(ui) else { continue; };
        if !widget_branch.is_visible() { continue; }

        // check if element widget intersects with the cursor
        let Ok(Some(widget_depth)) = S::LunexCursor::cursor_intersects_element(
                cpos_world,
                cpos_lunex,
                ui,
                widget,
                entity,
                target_limit,
                widget_branch.get_depth(),
                &element_param,
            ) else { continue; };

        top_unpressed = Some(entity);
        target_limit  = Some(widget_depth);
    }

    // get highest pressed element whose press home zone is under the cursor
    let mut top_pressed: Option<Entity> = None;

    for (entity, _, press_home_zone) in pressed_elements.iter()
    {
        // check visibility
        let Ok(widget_branch) = press_home_zone.0.fetch(ui) else { continue; };
        if !widget_branch.is_visible() { continue; }

        // check if press home zone widget intersects with the cursor
        let Ok(Some(widget_depth)) = S::LunexCursor::cursor_intersects_press_home_zone(
                cpos_world,
                cpos_lunex,
                ui,
                &press_home_zone.0,
                entity,
                target_limit,
                widget_branch.get_depth(),
                &home_zone_param,
            ) else { continue; };

        top_pressed  = Some(entity);
        target_limit = Some(widget_depth);
    }

    // set final target
    let targeted;
    let target_is_hovered;

    match top_pressed
    {
        Some(pressed_entity) =>
        {
            // target: pressed entity
            targeted = Some(pressed_entity);

            // get widget and branch
            let Ok((_, widget, _)) = pressed_elements.get(pressed_entity)
            else { tracing::error!("pressed entity is missing"); return None; };

            let Ok(widget_branch) = widget.fetch(ui)
            else { tracing::error!("pressed entity's widget branch is missing"); return None; };

            // check if target's element is hovered
            if let Ok(Some(_)) = S::LunexCursor::cursor_intersects_element(
                    cpos_world,
                    cpos_lunex,
                    ui,
                    widget,
                    pressed_entity,
                    None,  //no depth limit
                    widget_branch.get_depth(),
                    &element_param,
                ) { target_is_hovered = true; }
            else { target_is_hovered = false; }
        }
        None =>
        {
            targeted          = top_unpressed;
            target_is_hovered = top_unpressed.is_some();
        }
    }

    // assemble the info pack
    Some(InteractionSourceInfoPack{
            just_clicked   : source.just_clicked(&source_param),
            is_clicked     : source.is_clicked(&source_param),
            just_unclicked : source.just_unclicked(&source_param),
            target_is_hovered,
            depth_limit,
            targeted,
        })
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_just_clicked<S: InteractionSource>(
    In(info_pack) : In<InteractionSourceInfoPack>,
    mut commands  : Commands,
    widgets       : Query<
        &Callback<OnClick>,
        (With<ElementInteractionTargeter<S::LunexUI, S::LunexCursor>>, With<ElementInteractionSource<S>>)
    >,
){
    // check the source was just clicked
    if !info_pack.just_clicked { return; }

    // check if the target is hovered (just-clicked only applies to the element, not the press home zone)
    if !info_pack.target_is_hovered { return; }

    // check if there are any widgets to click
    if widgets.is_empty() { return; }

    // see if the targeted entity has a callback
    let Some(highest_entity) = info_pack.targeted else { return; };
    let Ok(callback) = widgets.get(highest_entity) else { return; };

    // queue the callback
    commands.add(callback.clone());
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_is_clicked<S: InteractionSource>(
    In(info_pack) : In<InteractionSourceInfoPack>,
    mut commands  : Commands,
    widgets       : Query<
        &Callback<OnClickHold>,
        (With<ElementInteractionTargeter<S::LunexUI, S::LunexCursor>>, With<ElementInteractionSource<S>>)
    >,
){
    // check the source is clicked
    if !info_pack.is_clicked { return; }

    // check if the target is hovered (is-clicked only applies to the element, not the press home zone)
    if !info_pack.target_is_hovered { return; }

    // check if there are any widgets to click
    if widgets.is_empty() { return; }

    // see if the targeted entity has a callback
    let Some(highest_entity) = info_pack.targeted else { return; };
    let Ok(callback) = widgets.get(highest_entity) else { return; };

    // queue the callback
    commands.add(callback.clone());
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_is_clicked_home<S: InteractionSource>(
    In(info_pack) : In<InteractionSourceInfoPack>,
    mut commands  : Commands,
    widgets       : Query<
        &Callback<OnClickHoldHome>,
        (With<Pressed>, With<ElementInteractionTargeter<S::LunexUI, S::LunexCursor>>, With<ElementInteractionSource<S>>)
    >,
){
    // check the source is clicked
    if !info_pack.is_clicked { return; }

    // note: is-clicked-home applies to the press home zone

    // check if there are any widgets with callbacks
    if widgets.is_empty() { return; }

    // see if the targeted entity has a callback
    let Some(highest_entity) = info_pack.targeted else { return; };
    let Ok(callback) = widgets.get(highest_entity) else { return; };

    // queue the callback
    commands.add(callback.clone());
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_is_clicked_away<S: InteractionSource>(
    In(info_pack) : In<InteractionSourceInfoPack>,
    mut commands  : Commands,
    ui            : Query<&UiTree, With<S::LunexUI>>,  //todo: InFocusedWindow
    widgets       : Query<
        (Entity, &Widget, &CallbackWith<OnClickHoldAway, bool>),
        (With<Pressed>, With<ElementInteractionTargeter<S::LunexUI, S::LunexCursor>>, With<ElementInteractionSource<S>>)
    >,
){
    // check the source is clicked
    if !info_pack.is_clicked { return; }

    // check if there are any widgets with callbacks
    if widgets.is_empty() { return; }

    // get the ui
    let Ok(ui) = ui.get_single() else { tracing::error!("ui is missing"); return; };

    // find pressed widgets away from the cursor
    for (entity, widget, callback) in widgets.iter()
    {
        // skip widgets targeted by the cursor
        if info_pack.targeted == Some(entity) { continue; };

        // skip widgets not associated with the current UI
        let Ok(widget_branch) = widget.fetch(&ui) else { continue; };

        // check if the widget is present (visible and above the cursor's interaction barrier)
        let is_present =
            widget_branch.is_visible() &&
            'x : {
                if let Some(top) = info_pack.depth_limit { if top > widget_branch.get_depth() { break 'x false } }
                true
            };

        // queue the callback
        commands.add(callback.call_with(is_present));
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_just_unclicked<S: InteractionSource>(
    In(info_pack)   : In<InteractionSourceInfoPack>,
    mut commands    : Commands,
    unclick_widgets : Query<
        (Entity, &CallbackWith<OnUnClick, bool>),
        (With<Pressed>, With<ElementInteractionTargeter<S::LunexUI, S::LunexCursor>>, With<ElementInteractionSource<S>>)
    >,
){
    // check the source was just unclicked
    if !info_pack.just_unclicked { return; }

    // check if there are any widgets to unclick
    if unclick_widgets.is_empty() { return; }

    // unclick all the widgets
    for (entity, unclick_callback) in unclick_widgets.iter()
    {
        // check if the target is under the cursor (it will be the press home zone)
        let under_cursor = info_pack.targeted == Some(entity);

        // queue the callback
        commands.add(unclick_callback.call_with(under_cursor));
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_hover<S: InteractionSource>(
    In(info_pack) : In<InteractionSourceInfoPack>,
    mut commands  : Commands,
    widgets       : Query<
        &Callback<OnHover>,
        (With<ElementInteractionTargeter<S::LunexUI, S::LunexCursor>>, With<ElementInteractionSource<S>>)
    >,
){
    // see if the targeted element is hovered by the cursor
    // - note: we cannot just check if the widgets with `OnHover` callbacks contain the targeted entity, because
    //         an element is only hovered if the **element** is hovered, not the press home zone (which may be the target)
    if !info_pack.target_is_hovered { return; }

    // check if there are any widgets with callbacks
    if widgets.is_empty() { return; }

    // see if the targeted entity has a callback
    let Some(highest_entity) = info_pack.targeted else { return; };
    let Ok(callback) = widgets.get(highest_entity) else { return; };

    // queue the callback
    commands.add(callback.clone());
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_unhover<S: InteractionSource>(
    In(info_pack) : In<InteractionSourceInfoPack>,
    mut commands  : Commands,
    widgets       : Query<
        (Entity, &Callback<OnUnHover>),
        (With<Hovered>, With<ElementInteractionTargeter<S::LunexUI, S::LunexCursor>>, With<ElementInteractionSource<S>>)
    >,
){
    // check if there are any widgets with callbacks
    if widgets.is_empty() { return; }

    // unhover any widgets that aren't targeted and hovered
    for (entity, callback) in widgets.iter()
    {
        // skip if targeted by the cursor and hovered
        if (info_pack.targeted == Some(entity)) && info_pack.target_is_hovered { continue; };

        // queue the callback
        commands.add(callback.clone());
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Poll the interaction source and handle its current state.
pub fn interaction_pipeline<S: InteractionSource>(world: &mut World)
{
    // try get source info pack
    // - if we cannot get an info pack then the pipeline for this source is disabled
    let Some(info_pack) = syscall(world, (), try_get_interaction_source_info_pack::<S>)
    else
    {
        //todo: clean up Pressed/Hovered if focused window changed (abort press and force unhover)
        //todo: clean up Pressed/Hovered if cursor is deactivated or despawned (abort press and force unhover)
        //(maybe do this in separate system for all sources?)
        return;
    };

    // [IF CLICKED]
    // handle source was just unclicked and there is an entity with `Pressed`
    // - if the source is clicked then we may have had an unclick -> click event sequence
    if info_pack.is_clicked && info_pack.just_unclicked { syscall(world, info_pack, handle_just_unclicked::<S>); }

    // handle source is clicked but positioned away from an entity with `Pressed`
    // - do this before other click checks so we can clean up widgets that are currently `Pressed` but need to be
    //   unpressed (this may be important for data dependencies in registered callbacks)
    if info_pack.is_clicked { syscall(world, info_pack, handle_is_clicked_away::<S>); }

    // handle source was just clicked
    // - only if source hits an element
    if info_pack.just_clicked && info_pack.targeted.is_some() && info_pack.target_is_hovered
    { syscall(world, info_pack, handle_just_clicked::<S>); }

    // handle source is clicked
    // - only if source hits an element
    if info_pack.is_clicked && info_pack.targeted.is_some() && info_pack.target_is_hovered
    { syscall(world, info_pack, handle_is_clicked::<S>); }

    // handle source is clicked and positioned on an entity with `Pressed`
    // - do this after 'just clicked' and 'is clicked' in case `Pressed` was added to an entity by one of them
    if info_pack.is_clicked && info_pack.targeted.is_some() { syscall(world, info_pack, handle_is_clicked_home::<S>); }

    // [IF NOT CLICKED]
    // handle source was just unclicked and there is an entity with `Pressed`
    // - if the source is not clicked then we may have had a click -> unclick event sequence
    if !info_pack.is_clicked && info_pack.just_unclicked { syscall(world, info_pack, handle_just_unclicked::<S>); }

    // handle hover (source is positioned over a widget)
    // - only if source hits an element
    // - do this after click handlers in case any of them remove `Pressed` or `Selected` from a hovered entity
    if info_pack.targeted.is_some() && info_pack.target_is_hovered { syscall(world, info_pack, handle_hover::<S>); }

    // handle unhover (source was positioned over a widget and now it is not)
    // - do this after checking 'hover' in case doing so caused an entity to become unhovered
    syscall(world, info_pack, handle_unhover::<S>);
}

//-------------------------------------------------------------------------------------------------------------------
