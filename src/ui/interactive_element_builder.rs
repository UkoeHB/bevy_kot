//local shortcuts
use crate::ecs::*;
use super::*;

//third-party shortcuts
use bevy::ecs::system::{Command, EntityCommands};
use bevy::prelude::*;
use bevy_lunex::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn check_press_invariants(builder: &InteractiveElementBuilder) -> Result<(), InteractiveElementBuilderError>
{
    // check press_away consistency
    if builder.abort_press_on_press_away &&
        (
            builder.unpress_on_press_away ||
            builder.press_away_start_callback.is_some() ||
            builder.press_away_always_callback.is_some() ||
            builder.press_away_if_present_callback.is_some() ||
            builder.press_away_if_not_present_callback.is_some()

        )
    { return Err(InteractiveElementBuilderError::InconsistentPressAway); }

    // check unclick consistency
    if builder.unpress_on_unclick_home &&
        !(
            builder.unpress_on_unclick_away     ||
            builder.unpress_on_press_away        ||
            builder.abort_press_on_unclick_away
        )
    { return Err(InteractiveElementBuilderError::InconsistentUnPressUnclick); }

    if builder.unpress_on_unclick_away && !builder.unpress_on_unclick_home
    { return Err(InteractiveElementBuilderError::InconsistentUnPressUnclick); }

    // check if we will press and unpress the element
    let press_on_x   = builder.element_is_pressable();
    let unpress_on_x = builder.unpress_on_unclick_home || builder.unpress_on_unclick_away || builder.unpress_on_press_away;
    if press_on_x && unpress_on_x { return Ok(()); }
    if press_on_x && !unpress_on_x { return Err(InteractiveElementBuilderError::MissingPressReleaser); }
    if !press_on_x && unpress_on_x { return Err(InteractiveElementBuilderError::MissingPressActivator); }

    // we won't press the element, so there shouldn't be any press interaction pieces
    let error = Err(InteractiveElementBuilderError::MissingPressActivator);

    if builder.widget_pack.pressed_widget.is_some() { return error; }
    if builder.widget_pack.pressed_selected_widget.is_some() { return error; }
    if builder.widget_pack.hover_pressed_widget.is_some() { return error; }
    if builder.widget_pack.hover_pressed_selected_widget.is_some() { return error; }
    if builder.press_home_zone.is_some() { return error; }
    if builder.abort_press_on_unclick_away { return error; }
    if builder.abort_press_on_press_away { return error; }
    if builder.abort_press_on_press_away_if_not_present { return error; }
    if builder.select_on_press_start { return error; }
    if builder.select_on_unpress { return error; }
    if builder.no_hover_on_pressed { return error; }
    if builder.no_hover_on_pressed_selected { return error; }
    if builder.on_unclick_callback.is_some() { return error; }
    if builder.startpress_callback.is_some() { return error; }
    if builder.press_away_start_callback.is_some() { return error; }
    if builder.press_away_always_callback.is_some() { return error; }
    if builder.press_away_if_present_callback.is_some() { return error; }
    if builder.press_away_if_not_present_callback.is_some() { return error; }
    if builder.unpress_callback.is_some() { return error; }
    if builder.abortpress_callback.is_some() { return error; }

    Ok(())
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn check_select_invariants(builder: &InteractiveElementBuilder) -> Result<(), InteractiveElementBuilderError>
{
    // check if we will select the element
    // note: we assume the element owner will be in charge of deselecting, so no need to check if element will auto-deselect
    if builder.element_is_selectable() { return Ok(()); }

    // we won't select the element, so there shouldn't be any select interaction pieces
    let error = Err(InteractiveElementBuilderError::MissingSelectActivator);

    if builder.widget_pack.selected_widget.is_some() { return error; }
    if builder.with_select_toggling { return error; }
    if builder.no_hover_on_selected { return error; }
    if builder.no_hover_on_pressed_selected { return error; }
    if builder.select_callback.is_some() { return error; }
    if builder.deselect_callback.is_some() { return error; }

    Ok(())
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn check_hover_invariants(builder: &InteractiveElementBuilder) -> Result<(), InteractiveElementBuilderError>
{
    // check if we will hover the element
    // note: unhovering is automatic
    if builder.element_is_hoverable() { return Ok(()); }

    // we won't hover the element, so there shouldn't be any hover interaction pieces
    let error = Err(InteractiveElementBuilderError::MissingHoverReason);

    if builder.no_hover_on_pressed { return error; }
    if builder.no_hover_on_selected { return error; }
    if builder.no_hover_on_pressed_selected { return error; }

    Ok(())
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn toggle_widget_pack_visibility(
    ui          : &mut UiTree,
    on_widget   : Option<Widget>,
    widget_pack : InteractiveElementWidgetPack,
) -> Result<(), ()>
{
    // invisibility setter
    let mut try_set_invisible =
        |widget: Option<Widget>| -> Result<(), ()>
        {
            if widget == on_widget { return Ok(()); }
            let Some(widget) = widget else { return Ok(()); };
            widget.fetch_mut(ui).or(Err(()))?.set_visibility(false);
            Ok(())
        };

    // turn off all widgets in the widget pack not equal to the on widget
    // - we ignore errors in case some of the widgets have been removed from the UI tree but others remain
    let _ = try_set_invisible(widget_pack.default_widget);
    let _ = try_set_invisible(widget_pack.pressed_widget);
    let _ = try_set_invisible(widget_pack.selected_widget);
    let _ = try_set_invisible(widget_pack.hovered_widget);
    let _ = try_set_invisible(widget_pack.pressed_selected_widget);
    let _ = try_set_invisible(widget_pack.hover_selected_widget);
    let _ = try_set_invisible(widget_pack.hover_pressed_widget);
    let _ = try_set_invisible(widget_pack.hover_pressed_selected_widget);

    // set widget visibility on
    if let Some(on_widget) = on_widget { on_widget.fetch_mut(ui).or(Err(()))?.set_visibility(true); }
    Ok(())
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn update_interactive_element_visibility<U: LunexUI>(
    In((entity, mut pack)) : In<(Entity, InteractiveElementWidgetPack)>,
    pressed                : Query<&Pressed>,
    selected               : Query<(), With<Selected>>,
    hovered                : Query<(), With<Hovered>>,
    mut uis                : Query<&mut UiTree, With<U>>,
) -> Result<(), ()>
{
    // find the correct widget to activate
    // - match ordering is based on designed precedence
    // - 'is pressed' is true only if pressed on Home (todo: consider variants for 'pressed home' vs 'pressed away')
    let is_pressed  = matches!(pressed.get(entity), Ok(&Pressed::Home));
    let is_selected = selected.get(entity).is_ok();
    let is_hovered  = hovered.get(entity).is_ok();

    let widget = match (is_hovered, is_pressed, is_selected)
    {
        (true, true, true)   if pack.hover_pressed_selected_widget.is_some() => pack.hover_pressed_selected_widget.take(),
        (false, true, true)  if pack.pressed_selected_widget.is_some() => pack.pressed_selected_widget.take(),
        (true, false, true)  if pack.hover_selected_widget.is_some()   => pack.hover_selected_widget.take(),
        (true, true, false)  if pack.hover_pressed_widget.is_some()    => pack.hover_pressed_widget.take(),
        (false, true, false) if pack.pressed_widget.is_some()  => pack.pressed_widget.take(),
        (false, false, true) if pack.selected_widget.is_some() => pack.selected_widget.take(),
        (true, false, false) if pack.hovered_widget.is_some()  => pack.hovered_widget.take(),
        (_, _, _)            if pack.default_widget.is_some()  => pack.default_widget.take(),
        _ => None,
    };

    // toggle visibility
    let mut ui = uis.get_single_mut().or(Err(()))?;  //todo: uis.get_mut(widgets.get(entity).ui_entity())
    toggle_widget_pack_visibility(&mut ui, widget, pack)?;
    Ok(())
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Returns `true` when the element is allowed to be hovered.
fn hover_is_allowed(
    element_entity               : Entity,
    no_hover_on_selected         : bool,
    no_hover_on_pressed          : bool,
    no_hover_on_pressed_selected : bool,
    selected                     : &Query<(), With<Selected>>,
    pressed                      : &Query<(), With<Pressed>>,
) -> bool
{
    if no_hover_on_selected { if selected.contains(element_entity) { return false; } }
    if no_hover_on_pressed  { if pressed.contains(element_entity)  { return false; } }

    if no_hover_on_pressed_selected && !(no_hover_on_selected || no_hover_on_pressed)
    {
        if selected.contains(element_entity) && pressed.contains(element_entity)
        { return false; }
    }

    true
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Returns `true` when the element is allowed to be hovered.
fn hover_is_allowed_with_world(
    world                        : &mut World,
    element_entity               : Entity,
    no_hover_on_selected         : bool,
    no_hover_on_pressed          : bool,
    no_hover_on_pressed_selected : bool,
) -> bool
{
    if !(no_hover_on_selected || no_hover_on_pressed || no_hover_on_pressed_selected) { return true; }
    let Some(entity_ref) = world.get_entity(element_entity) else { return false; };

    if no_hover_on_selected { if entity_ref.contains::<Selected>() { return false; } }
    if no_hover_on_pressed  { if entity_ref.contains::<Pressed>()  { return false; } }

    if no_hover_on_pressed_selected && !(no_hover_on_selected || no_hover_on_pressed)
    {
        if entity_ref.contains::<Selected>() && entity_ref.contains::<Pressed>()
        { return false; }
    }

    true
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn hover_fixer(
    In((
        element_entity,
        no_hover_on_selected,
        no_hover_on_pressed,
        no_hover_on_pressed_selected,
    ))           : In<(Entity, bool, bool, bool)>,
    mut commands : Commands,
    selected     : Query<(), With<Selected>>,
    pressed      : Query<(), With<Pressed>>,
    hovered      : Query<&Callback<OnUnHover>, With<Hovered>>,
){
    // We assume at least one of the 'no hover' booleans is true, so there is no perf reason to check booleans first.
    let Ok(unhover_callback) = hovered.get(element_entity) else { return; };

    if hover_is_allowed(
            element_entity,
            no_hover_on_selected,
            no_hover_on_pressed,
            no_hover_on_pressed_selected,
            &selected,
            &pressed,
        )
    { return; }

    commands.add(unhover_callback.clone());
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn prepare_hover_fixer(
    need_hover                   : bool,
    element_entity               : Entity,
    no_hover_on_selected         : bool,
    no_hover_on_pressed          : bool,
    no_hover_on_pressed_selected : bool
) -> impl Fn(&mut World) -> () + Clone + Send + Sync + 'static
{
    let maybe_need_fix = need_hover && (no_hover_on_selected || no_hover_on_pressed || no_hover_on_pressed_selected);

    move |world: &mut World|
    {
        if !maybe_need_fix { return; }
        syscall(
                world,
                (
                        element_entity,
                        no_hover_on_selected,
                        no_hover_on_pressed,
                        no_hover_on_pressed_selected,
                ),
                hover_fixer
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn maybe_build_action_start_press<H, V>(
    need_press                   : bool,
    entity_commands              : &mut EntityCommands,
    element_entity               : Entity,
    select_on_press_start        : bool,
    no_hover_on_pressed          : bool,
    no_hover_on_pressed_selected : bool,
    hover_fixer                  : &H,
    startpress_callback          : Option<CallbackWith<(), Vec2>>,
    update_widget_visibility     : &V,
)
where
    H: Fn(&mut World) -> () + Clone + Send + Sync + 'static,
    V: Fn(&mut World) -> () + Clone + Send + Sync + 'static 
{
    // check if action is needed
    if !need_press { return; }

    let hover_fixer = if no_hover_on_pressed || no_hover_on_pressed_selected { Some(hover_fixer.clone()) } else { None };
    let vis_updater = update_widget_visibility.clone();

    // callback
    let press_start_callback = CallbackWith::<StartPress, Vec2>::new(
            move |world: &mut World, cpos_world: Vec2|
            {
                // try to add Pressed::Home to entity
                if !try_add_component_to_entity(world, element_entity, Pressed::Home) { return; }

                // [option] action: select
                if select_on_press_start { let _ = try_callback::<Select>(world, element_entity); }

                // fix hover state
                if let Some(hover_fixer) = &hover_fixer { hover_fixer(world); }

                // invoke user-defined callback
                if let Some(cb) = &startpress_callback { cb.call_with(cpos_world).apply(world); }

                // update visibility
                vis_updater(world);
            }
        );

    // insert callback
    entity_commands.insert(press_start_callback);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn maybe_build_action_unpress<H, V>(
    need_press                   : bool,
    entity_commands              : &mut EntityCommands,
    element_entity               : Entity,
    select_on_unpress            : bool,
    no_hover_on_pressed          : bool,
    no_hover_on_pressed_selected : bool,
    hover_fixer                  : &H,
    unpress_callback             : Option<CallbackWith<(), Vec2>>,
    update_widget_visibility     : &V,
)
where
    H: Fn(&mut World) -> () + Clone + Send + Sync + 'static,
    V: Fn(&mut World) -> () + Clone + Send + Sync + 'static 
{
    // check if action is needed
    if !need_press { return; }

    let hover_fixer = if no_hover_on_pressed || no_hover_on_pressed_selected { Some(hover_fixer.clone()) } else { None };
    let vis_updater = update_widget_visibility.clone();

    // callback
    let unpress_callback = CallbackWith::<UnPress, Vec2>::new(
            move |world: &mut World, cpos_world: Vec2|
            {
                // try to remove `Pressed` from entity
                let Some(_) = try_remove_component_from_entity::<Pressed>(world, element_entity) else { return; };

                // [option] action: select
                if select_on_unpress { let _ = try_callback::<Select>(world, element_entity); }

                // fix hover state
                if let Some(hover_fixer) = &hover_fixer { hover_fixer(world); }

                // invoke user-defined callback
                if let Some(cb) = &unpress_callback { cb.call_with(cpos_world).apply(world); }

                // update visibility
                vis_updater(world);
            }
        );

    // insert callback
    entity_commands.insert(unpress_callback);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn maybe_build_action_abort_press<H, V>(
    need_press                   : bool,
    entity_commands              : &mut EntityCommands,
    element_entity               : Entity,
    no_hover_on_pressed          : bool,
    no_hover_on_pressed_selected : bool,
    hover_fixer                  : &H,
    abortpress_callback          : Option<CallbackWith<(), Vec2>>,
    update_widget_visibility     : &V,
)
where
    H: Fn(&mut World) -> () + Clone + Send + Sync + 'static,
    V: Fn(&mut World) -> () + Clone + Send + Sync + 'static 
{
    // check if action is needed
    // - note: if we have press/unpress callbacks then we also need an abort callback in case of window focus changes
    if !need_press { return; }

    let hover_fixer = if no_hover_on_pressed || no_hover_on_pressed_selected { Some(hover_fixer.clone()) } else { None };
    let vis_updater = update_widget_visibility.clone();

    // callback
    let abort_press_callback = CallbackWith::<AbortPress, Vec2>::new(
            move |world: &mut World, cpos_world: Vec2|
            {
                // try to remove `Pressed` from entity
                let Some(_) = try_remove_component_from_entity::<Pressed>(world, element_entity) else { return; };

                // fix hover state
                if let Some(hover_fixer) = &hover_fixer { hover_fixer(world); }

                // invoke user-defined callback
                if let Some(cb) = &abortpress_callback { cb.call_with(cpos_world).apply(world); }

                // update visibility
                vis_updater(world);
            }
        );

    // insert callback
    entity_commands.insert(abort_press_callback);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn maybe_build_action_select<H, V>(
    need_select                  : bool,
    entity_commands              : &mut EntityCommands,
    element_entity               : Entity,
    with_select_toggling         : bool,
    no_hover_on_selected         : bool,
    no_hover_on_pressed_selected : bool,
    hover_fixer                  : &H,
    select_callback              : Option<Callback<()>>,
    update_widget_visibility     : &V,
)
where
    H: Fn(&mut World) -> () + Clone + Send + Sync + 'static,
    V: Fn(&mut World) -> () + Clone + Send + Sync + 'static 
{
    // check if action is needed
    if !need_select { return; }

    let hover_fixer = if no_hover_on_selected || no_hover_on_pressed_selected { Some(hover_fixer.clone()) } else { None };
    let vis_updater = update_widget_visibility.clone();

    // callback
    let press_start_callback = Callback::<Select>::new(
            move |world: &mut World|
            {
                // try to select the entity
                if !try_add_component_to_entity(world, element_entity, Selected)
                {
                    // if we are toggling select, then, since selecting failed, deselect
                    if with_select_toggling { let _ = try_callback::<Deselect>(world, element_entity); }
                    return;
                }

                // fix hover state
                if let Some(hover_fixer) = &hover_fixer { hover_fixer(world); }

                // invoke user-defined callback
                if let Some(cb) = &select_callback { cb.clone().apply(world); }

                // update visibility
                vis_updater(world);
            }
        );

    // insert callback
    entity_commands.insert(press_start_callback);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn maybe_build_action_deselect<H, V>(
    need_select                  : bool,
    entity_commands              : &mut EntityCommands,
    element_entity               : Entity,
    no_hover_on_selected         : bool,
    no_hover_on_pressed_selected : bool,
    hover_fixer                  : &H,
    deselect_callback            : Option<Callback<()>>,
    update_widget_visibility     : &V,
)
where
    H: Fn(&mut World) -> () + Clone + Send + Sync + 'static,
    V: Fn(&mut World) -> () + Clone + Send + Sync + 'static 
{
    // check if action is needed
    if !need_select { return; }

    let hover_fixer = if no_hover_on_selected || no_hover_on_pressed_selected { Some(hover_fixer.clone()) } else { None };
    let vis_updater = update_widget_visibility.clone();

    // callback
    let press_start_callback = Callback::<Deselect>::new(
            move |world: &mut World|
            {
                // try to remove `Selected` from entity
                let Some(_) = try_remove_component_from_entity::<Selected>(world, element_entity) else { return; };

                // fix hover state
                if let Some(hover_fixer) = &hover_fixer { hover_fixer(world); }

                // invoke user-defined callback
                if let Some(cb) = &deselect_callback { cb.clone().apply(world); }

                // update visibility
                vis_updater(world);
            }
        );

    // insert callback
    entity_commands.insert(press_start_callback);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn maybe_build_responder_on_click(
    entity_commands   : &mut EntityCommands,
    element_entity    : Entity,
    press_on_click    : bool,
    select_on_click   : bool,
    on_click_callback : Option<CallbackWith<(), Vec2>>,
){
    // check if responder is needed
    if !(press_on_click || select_on_click || on_click_callback.is_some()) { return; }

    // callback
    let on_click_callback = CallbackWith::<OnClick, Vec2>::new(
            move |world: &mut World, cpos_world: Vec2|
            {
                // [option] action: start press
                if press_on_click { let _ = try_callback_with::<StartPress, Vec2>(world, element_entity, cpos_world); }

                // [option] action: select
                if select_on_click { let _ = try_callback::<Select>(world, element_entity); }

                // invoke user-defined callback
                if let Some(cb) = &on_click_callback { cb.call_with(cpos_world).apply(world); }
            }
        );

    // insert callback
    entity_commands.insert(on_click_callback);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn maybe_build_responder_on_click_hold(
    entity_commands       : &mut EntityCommands,
    element_entity        : Entity,
    press_on_clickhold    : bool,
    on_clickhold_callback : Option<CallbackWith<(), Vec2>>,
){
    // check if responder is needed
    if !(press_on_clickhold || on_clickhold_callback.is_some()) { return; }

    // callback
    let on_click_hold_callback = CallbackWith::<OnClickHold, Vec2>::new(
            move |world: &mut World, cpos_world: Vec2|
            {
                // [option] action: start press
                if press_on_clickhold { let _ = try_callback_with::<StartPress, Vec2>(world, element_entity, cpos_world); }

                // invoke user-defined callback
                if let Some(cb) = &on_clickhold_callback { cb.call_with(cpos_world).apply(world); }
            }
        );

    // insert callback
    entity_commands.insert(on_click_hold_callback);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn maybe_build_responder_on_click_hold_home<H, V>(
    need_press                   : bool,
    entity_commands              : &mut EntityCommands,
    element_entity               : Entity,
    no_hover_on_pressed          : bool,
    no_hover_on_pressed_selected : bool,
    hover_fixer                  : &H,
    press_home_start_callback    : Option<CallbackWith<(), Vec2>>,
    press_home_callback          : Option<CallbackWith<(), Vec2>>,
    update_widget_visibility     : &V,
)
where
    H: Fn(&mut World) -> () + Clone + Send + Sync + 'static,
    V: Fn(&mut World) -> () + Clone + Send + Sync + 'static 
{
    // check if responder is needed
    if !need_press { return; }

    let hover_fixer = if no_hover_on_pressed || no_hover_on_pressed_selected { Some(hover_fixer.clone()) } else { None };
    let vis_updater = update_widget_visibility.clone();

    // callback
    let on_click_hold_home_callback = CallbackWith::<OnClickHoldHome, Vec2>::new(
            move |world: &mut World, cpos_world: Vec2|
            {
                // invoke user-defined callback: press home (always)
                if let Some(cb) = &press_home_callback { cb.call_with(cpos_world).apply(world); }

                // try to update `Pressed` component to `Pressed::Home`
                // - we leave if already in `Pressed::Home` because the remaining work is only needed when transitioning
                // - we also leave if not `Pressed`, since this callback is intended for already-pressed elements
                if !try_update_component_if_different(world, element_entity, Pressed::Home) { return; }

                // fix hover state
                if let Some(hover_fixer) = &hover_fixer { hover_fixer(world); }

                // invoke user-defined callback: press home (start)
                if let Some(cb) = &press_home_start_callback { cb.call_with(cpos_world).apply(world); }

                // update visibility
                vis_updater(world);
            }
        );

    // insert callback
    entity_commands.insert(on_click_hold_home_callback);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn maybe_build_responder_on_click_hold_away<H, V>(
    need_press                               : bool,
    entity_commands                          : &mut EntityCommands,
    element_entity                           : Entity,
    abort_press_on_press_away                : bool,
    abort_press_on_press_away_if_not_present : bool,
    unpress_on_press_away                    : bool,
    no_hover_on_pressed                      : bool,
    no_hover_on_pressed_selected             : bool,
    hover_fixer                              : &H,
    press_away_start_callback                : Option<CallbackWith<(), Vec2>>,
    press_away_always_callback               : Option<CallbackWith<(), Vec2>>,
    press_away_if_present_callback           : Option<CallbackWith<(), Vec2>>,
    press_away_if_not_present_callback       : Option<CallbackWith<(), Vec2>>,
    update_widget_visibility                 : &V,
)
where
    H: Fn(&mut World) -> () + Clone + Send + Sync + 'static,
    V: Fn(&mut World) -> () + Clone + Send + Sync + 'static 
{
    // check if responder is needed
    if !need_press { return; }

    let hover_fixer = if no_hover_on_pressed || no_hover_on_pressed_selected { Some(hover_fixer.clone()) } else { None };
    let vis_updater = update_widget_visibility.clone();

    // callback
    let on_click_hold_away_callback = CallbackWith::<OnClickHoldAway, (Vec2, bool)>::new(
            move | world: &mut World, (cpos_world, is_present): (Vec2, bool) |
            {
                // [option] action: abort press
                if abort_press_on_press_away
                { let _ = try_callback_with::<AbortPress, Vec2>(world, element_entity, cpos_world); return; }

                // [option] action: abort press
                // - note: must check this before unpress_on_press_away (they are overlapping options)
                if abort_press_on_press_away_if_not_present && !is_present
                { let _ = try_callback_with::<AbortPress, Vec2>(world, element_entity, cpos_world); return; }

                // [option] action: unpress
                if unpress_on_press_away
                { let _ = try_callback_with::<UnPress, Vec2>(world, element_entity, cpos_world); return; }

                // try to update `Pressed` component to `Pressed::Away`
                let press_away_started = try_update_component_if_different(world, element_entity, Pressed::Away);
                if press_away_started
                {
                    // fix hover state
                    if let Some(hover_fixer) = &hover_fixer { hover_fixer(world); }

                    // invoke user-defined callback: press away start
                    if let Some(cb) = &press_away_start_callback { cb.call_with(cpos_world).apply(world); }
                }

                // invoke user-defined callback: press away (always)
                if let Some(cb) = &press_away_always_callback { cb.call_with(cpos_world).apply(world); }

                // invoke user-defined callback: press away (if present)
                if is_present
                {
                    if let Some(cb) = &press_away_if_present_callback { cb.call_with(cpos_world).apply(world); }
                }
                // invoke user-defined callback: press away (if not present)
                else
                {
                    if let Some(cb) = &press_away_if_not_present_callback { cb.call_with(cpos_world).apply(world); }
                }

                // update visibility
                if press_away_started { vis_updater(world); }
            }
        );

    // insert callback
    entity_commands.insert(on_click_hold_away_callback);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn maybe_build_responder_on_unclick(
    need_press                  : bool,
    entity_commands             : &mut EntityCommands,
    element_entity              : Entity,
    unpress_on_unclick_home     : bool,
    abort_press_on_unclick_away : bool,
    unpress_on_unclick_away     : bool,
    on_unclick_callback         : Option<CallbackWith<(), (Vec2, bool)>>,
){
    // check if responder is needed
    if !need_press { return; }

    // callback
    let on_unclick_callback = CallbackWith::<OnUnClick, (Vec2, bool)>::new(
            move | world: &mut World, (cpos_world, unclick_on_home): (Vec2, bool) |
            {
                // invoke user-defined callback
                if let Some(cb) = &on_unclick_callback { cb.call_with((cpos_world, unclick_on_home)).apply(world); }

                if unclick_on_home
                {
                    // [option] action: unpress
                    if unpress_on_unclick_home
                    { let _ = try_callback_with::<UnPress, Vec2>(world, element_entity, cpos_world); }
                }
                else
                {
                    // [option] action: abort press
                    if abort_press_on_unclick_away
                    { let _ = try_callback_with::<AbortPress, Vec2>(world, element_entity, cpos_world); }

                    // [option] action: unpress
                    if unpress_on_unclick_away
                    { let _ = try_callback_with::<UnPress, Vec2>(world, element_entity, cpos_world); }
                }
            }
        );

    // insert callback
    entity_commands.insert(on_unclick_callback);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn maybe_build_responder_on_hover<V>(
    need_hover                   : bool,
    entity_commands              : &mut EntityCommands,
    element_entity               : Entity,
    no_hover_on_selected         : bool,
    no_hover_on_pressed          : bool,
    no_hover_on_pressed_selected : bool,
    select_on_hover_start        : bool,
    on_hover_start_callback      : Option<CallbackWith<(), Vec2>>,
    on_hover_callback            : Option<CallbackWith<(), Vec2>>,
    update_widget_visibility     : &V,
)
where
    V: Fn(&mut World) -> () + Clone + Send + Sync + 'static 
{
    // check if responder is needed
    if !need_hover { return; }

    let vis_updater = update_widget_visibility.clone();

    // callback
    let on_hover_callback = CallbackWith::<OnHover, Vec2>::new(
            move |world: &mut World, cpos_world: Vec2|
            {
                // check if hovering is allowed
                if !hover_is_allowed_with_world(
                        world,
                        element_entity,
                        no_hover_on_selected,
                        no_hover_on_pressed,
                        no_hover_on_pressed_selected,
                    )
                { return; }

                // try to start hovering the entity
                let started_hovering = try_add_component_to_entity(world, element_entity, Hovered);
                if started_hovering
                {
                    // [option] action: select
                    if select_on_hover_start { let _ = try_callback::<Select>(world, element_entity); }

                    // invoke user-defined callback: hover start
                    if let Some(cb) = &on_hover_start_callback { cb.call_with(cpos_world).apply(world); }
                }

                // invoke user-defined callback: hover
                if let Some(cb) = &on_hover_callback { cb.call_with(cpos_world).apply(world); }

                // update visibility
                if started_hovering { vis_updater(world); }
            }
        );

    // insert callback
    entity_commands.insert(on_hover_callback);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn maybe_build_responder_on_unhover<V>(
    need_hover               : bool,
    entity_commands          : &mut EntityCommands,
    element_entity           : Entity,
    on_unhover_callback      : Option<Callback<()>>,
    update_widget_visibility : &V,
)
where
    V: Fn(&mut World) -> () + Clone + Send + Sync + 'static 
{
    // check if responder is needed
    if !need_hover { return; }

    let vis_updater = update_widget_visibility.clone();

    // callback
    let on_unhover_callback = Callback::<OnUnHover>::new(
            move |world: &mut World|
            {
                // try to remove `Hovered` from entity
                let Some(_) = try_remove_component_from_entity::<Hovered>(world, element_entity) else { return; };

                // invoke user-defined callback
                if let Some(cb) = &on_unhover_callback { cb.clone().apply(world); }

                // update visibility
                vis_updater(world);
            }
        );

    // insert callback
    entity_commands.insert(on_unhover_callback);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InteractiveElementBuilderError
{
    /// Missing `press_on_*`.
    MissingPressActivator,
    /// Missing `unpress_on_*`.
    MissingPressReleaser,
    /// Missing `select_on_*`.
    MissingSelectActivator,
    /// Setting `abort_press_on_press_away` is active in addition to something that uses `press_away`.
    InconsistentPressAway,
    /// An `unpress_on_unclick_*` setting is active but is inconsistent with other settings.
    InconsistentUnPressUnclick,
    /// A `no_hover_on_*` setting is active but no reason to track hovers was given.
    MissingHoverReason,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Default)]
struct InteractiveElementWidgetPack
{
    default_widget                : Option<Widget>,
    pressed_widget                : Option<Widget>,
    selected_widget               : Option<Widget>,
    hovered_widget                : Option<Widget>,
    pressed_selected_widget       : Option<Widget>,
    hover_selected_widget         : Option<Widget>,
    hover_pressed_widget          : Option<Widget>,
    hover_pressed_selected_widget : Option<Widget>,
}

/// Builder for an interactive UI element.
///
/// **Explanation**
/// An interactive element is a UI widget that responds to clicks and hovers. A click sequence is composed of 'on click' ->
/// 'click hold' -> [optional: 'click hold away from element'] -> 'unclick' (on or off the element). A hover sequence
/// is composed of 'hover start' -> 'hovering' -> 'unhover'. The element can respond to clicks/hovers by entering a 'pressed'
/// state, which is temporary, or a 'selected' state, which may persist indefinitely. The pressed state has two modes:
/// `Home`, which occurs when the clicker is above the 'press home zone' (this equals the element by default, but may be
/// customized), and `Away`, which occurs
/// when the clicker is away from that zone. The components `Hovered`, `Pressed`, and `Selected` will be added to, updated
/// on, and removed from the element entity automatically.
///
/// The interactive element builder allows
/// you to decide when the pressed state is entered, exited, and aborted, when the selected state is entered, and whether
/// hovering is detected when pressed and/or selected. Exiting
/// the selected state is user-defined unless you opt for select toggling. The builder also allows callbacks to
/// be inserted into the element's state-change and response events, and widgets may be associated with all
/// permutations of `Hovered`/`Pressed::Home`/`Selected`. The visibility of those widgets will automatically toggle
/// as the element moves through state changes.
///
/// Interactive elements are closely tied to `InteractionSource`s. To consume an element builder you must specify the
/// interaction source that you want to control the element. If you register that source with
/// `register_interaction_source<[source]>`, then the built-in interaction pipeline will automatically connect your source
/// with any elements associated with that source. If you don't want the built-in pipeline, then you can implement your
/// own using the element's callbacks. If you want multiple interaction sources to control the same element, then you
/// can add an `InteractiveElement<[source]>` component to the element entity for each additional source. Note that
/// an interaction source will only work for an element if the element's parent UI tree has the source's `LunexUI` tag.
///
/// When it comes to deciding which element in a stack of overlapping elements should respond to an interaction source
/// event, we use 'interactive element targeting' based on a {`LunexUI`, `LunexCursor`} pair. Only the highest element
/// with a specific targeting will be interacted with by sources that use that targeting. You can add targeting to any
/// entity with `EnableInteractiveElementTargeting`, and if that entity has a Lunex widget then it will contribute to
/// targeting selection. If you want overlapping elements to interact with the same source, you can make two sources that
/// target two different cursors. The two cursors can then be tied to the same hardware source (e.g. the mouse). You
/// then enable/disable the cursors according to the conditions that allow overlapping interaction.
///
/// **Built-in `InteractionSource`s**
/// - `MouseLButton`: {clicks: mouse left button, hovers: mouse pointer}
/// - `MouseRButton`: {clicks: mouse right button, hovers: mouse pointer}
///
/// **Implementation comments**
/// - Interactivity takes the form of a set of 'action' and 'response' callbacks which are added to the element entity.
///   The action callbacks are all related to pressing and selecting. The response callbacks are used
///   by the UI backend to respond to clicker events in the environment (click/clickhold/unclick/hover/unhover).
///   The UI backend will use the element's press home zone to make decisions about press/press-away/unpress/press-abort
///   actions (e.g. `unpress_on_unclick` will unpress if unclick occurs above the press home zone, but will abort press if
///   unclick occurs away from the press home zone).
///   The response callbacks internally invoke action
///   callbacks as needed. We include all callbacks as public components so the user may access them for custom
///   workflows. Note that only the `Callback<Deselect>` action callback needs to be accessed for most normal use-cases.
/// - Only callbacks that have an effect are added to the element. For example, if you only specify `on_click_callback`
///   and `with_default_widget`, then there will be no action callbacks for pressing or selecting, and no response
///   callbacks for clickholding, unclicking, or hovering.
/// - If you manually remove or replace an action callback from the element entity, the change will be correctly
///   recognized by all response callbacks that use it. Adding previously-absent action callbacks will NOT be recognized,
///   since we avoid trying to invoke those for efficiency.
/// - Element widget visibility is typically updated at the end of built-in callbacks, but only if that callback may have
///   changed visibility. There are several cases where a user-defined callback may be invoked with no
///   visibility update afterward, which mean a user-defined change to visibility won't be registered immediately.
///   (todo: add option to enhance visibility updates? always update visibility + add visibility-change callback)
///
/// **Notes**
/// - All callbacks take the world position of the cursor at the time the callback is invoked.
/// - Deselection will only be automatic if `with_select_toggling` is set. Deselect the element manually with
///   `Callback<Deselect>`, which will be added by the builder if you specify a `select_on_*` setting.
/// - The pressed widget variants will not show if in state `Pressed::Away`.
/// - A `on_unclick_callback` can only be added to the builder if a press activator and deactivator are specified, since
///   unclicking only makes sense in the context of pressing (and otherwise it would be non-obvious what is being unclicked
///   between the element and the press away zone).
///
#[derive(Clone, Default)]
pub struct InteractiveElementBuilder
{
    widget_pack                              : InteractiveElementWidgetPack,

    press_home_zone                          : Option<Widget>,
    press_on_click                           : bool,
    press_on_clickhold                       : bool,

    unpress_on_unclick_home                  : bool,
    unpress_on_unclick_away                  : bool,
    unpress_on_press_away                    : bool,

    abort_press_on_unclick_away              : bool,
    abort_press_on_press_away                : bool,
    abort_press_on_press_away_if_not_present : bool,

    select_on_click                          : bool,
    select_on_press_start                    : bool,
    select_on_unpress                        : bool,
    select_on_hover_start                    : bool,

    with_select_toggling                     : bool,

    no_hover_on_pressed                      : bool,
    no_hover_on_selected                     : bool,
    no_hover_on_pressed_selected             : bool,

    on_click_callback                        : Option<CallbackWith<(), Vec2>>,
    on_clickhold_callback                    : Option<CallbackWith<(), Vec2>>,
    on_unclick_callback                      : Option<CallbackWith<(), (Vec2, bool)>>,
    on_hover_start_callback                  : Option<CallbackWith<(), Vec2>>,
    on_hover_callback                        : Option<CallbackWith<(), Vec2>>,
    on_unhover_callback                      : Option<Callback<()>>,

    startpress_callback                      : Option<CallbackWith<(), Vec2>>,
    press_home_start_callback                : Option<CallbackWith<(), Vec2>>,
    press_home_callback                      : Option<CallbackWith<(), Vec2>>,
    press_away_start_callback                : Option<CallbackWith<(), Vec2>>,
    press_away_always_callback               : Option<CallbackWith<(), Vec2>>,
    press_away_if_present_callback           : Option<CallbackWith<(), Vec2>>,
    press_away_if_not_present_callback       : Option<CallbackWith<(), Vec2>>,
    unpress_callback                         : Option<CallbackWith<(), Vec2>>,
    abortpress_callback                      : Option<CallbackWith<(), Vec2>>,

    select_callback                          : Option<Callback<()>>,
    deselect_callback                        : Option<Callback<()>>,
}

impl InteractiveElementBuilder
{
    /// New empty builder.
    pub fn new() -> InteractiveElementBuilder
    {
        InteractiveElementBuilder::default()
    }

    /// Add widget that is visible in the default state of the element.
    pub fn with_default_widget(mut self, widget: Widget) -> Self
    {
        self.widget_pack.default_widget = Some(widget);
        self
    }

    /// Add widget that is visible when the element is `Pressed::Home`.
    pub fn with_pressed_widget(mut self, widget: Widget) -> Self
    {
        self.widget_pack.pressed_widget = Some(widget);
        self
    }

    /// Add widget that is visible when the element is `Hovered`.
    pub fn with_hovered_widget(mut self, widget: Widget) -> Self
    {
        self.widget_pack.hovered_widget = Some(widget);
        self
    }

    /// Add widget that is visible when the element is `Selected`.
    pub fn with_selected_widget(mut self, widget: Widget) -> Self
    {
        self.widget_pack.selected_widget = Some(widget);
        self
    }

    /// Add widget that is visible when the element is `Pressed::Home` and `Selected`.
    /// - Takes precedence over `with_hover_pressed_widget` and `with_hover_selected_widget`.
    pub fn with_pressed_selected_widget(mut self, widget: Widget) -> Self
    {
        self.widget_pack.pressed_selected_widget = Some(widget);
        self
    }

    /// Add widget that is visible when the element is `Hovered` and `Pressed::Home`.
    pub fn with_hover_pressed_widget(mut self, widget: Widget) -> Self
    {
        self.widget_pack.hover_pressed_widget = Some(widget);
        self
    }

    /// Add widget that is visible when the element is `Hovered` and `Selected`.
    pub fn with_hover_selected_widget(mut self, widget: Widget) -> Self
    {
        self.widget_pack.hover_selected_widget = Some(widget);
        self
    }

    /// Add widget that is visible when the element is `Hovered`, `Pressed::Home`, and `Selected`.
    /// - Takes precedence over all other widget visibility cases.
    pub fn with_hover_pressed_selected_widget(mut self, widget: Widget) -> Self
    {
        self.widget_pack.hover_pressed_selected_widget = Some(widget);
        self
    }

    /// Add widget for zone where, if the element is pressed and the clicker is in the zone, then the element will be
    /// in state `Pressed::Home`. If pressed and outside the zone, the element will be in state `Pressed::Away`.
    /// - By default the 'press home zone' equals the element widget.
    pub fn with_press_home_zone(mut self, widget: Widget) -> Self
    {
        self.press_home_zone = Some(widget);
        self
    }

    /// Press the element when a click is detected on the element (i.e. just clicked).
    /// - Disables setting `press_on_clickhold`.
    pub fn press_on_click(mut self) -> Self
    {
        self.press_on_click     = true;
        self.press_on_clickhold = false;
        self
    }

    /// Press the element when a click or click hold is detected on the element.
    /// - Enables settings `press_on_click` and `press_on_clickhold`.
    /// - Note: We don't expose `press_on_clickhold` directly. In *most* cases a click-hold will occur immediately
    ///         after a click, but very rarely a click and unclick may happen within the same tick, in which case
    ///         click-hold won't be detected (which would be unexpected). To avoid that. we require 'press on click' and
    ///         'press on click-hold' to go together.
    /// Example: Suppose you have a row of buttons and want to click on a button then drag across other buttons and
    ///          release on the one you want to select. That would use `press_on_click_or_hold`, `unpress on unclick`,
    ///          and `abort_press_on_press_away`.
    pub fn press_on_click_or_hold(mut self) -> Self
    {
        self.press_on_click     = true;
        self.press_on_clickhold = true;
        self
    }

    /// Unpress the element when an unclick is detected on the element's press home zone, or abort the press
    /// if unclick is detected away from the press home zone.
    pub fn unpress_on_unclick_home_and_abort_on_unclick_away(mut self) -> Self
    {
        self.unpress_on_unclick_home     = true;
        self.abort_press_on_unclick_away = true;
        self
    }

    /// Unpress the element when an unclick is detected anywhere.
    /// - This option is not recommended, since the results are likely to be counterintuitive. Most of the time you
    ///   want `unpress_on_unclick_home_and_abort_on_unclick_away`.
    pub fn unpress_on_unclick_home_or_away(mut self) -> Self
    {
        self.unpress_on_unclick_home = true;
        self.unpress_on_unclick_away = true;
        self
    }

    /// Unpress the element when a click hold is detected away from the element's press home zone or if unclick is
    /// detected on the press home zone. Prefer `unpress_on_press_away_recommended` unless you have a specific use-case.
    /// - Disables setting `abort_press_on_press_away`.
    /// - If `abort_press_on_press_away_if_not_present` is set then this only takes effect when the element is
    ///   present (visible and the click hold does not occur above an interaction barrier higher than the
    ///   press home zone).
    /// - If this is not paired with `abort_press_on_press_away_if_not_present`,
    ///   then UI changes like pop-ups may spuriously register as 'unpress' events.
    pub fn unpress_on_press_away(mut self) -> Self
    {
        self.unpress_on_unclick_home     = true;
        self.unpress_on_press_away        = true;
        self.abort_press_on_unclick_away = false;
        self.abort_press_on_press_away    = false;
        self
    }

    /// Unpress the element when a click hold is detected away from the element's press home zone or if unclick is
    /// detected on the press home zone. Also abort press if click hold is detected when the element is not present.
    /// - Invokes `unpress_on_press_away()` and `abort_press_on_press_away_if_not_present()`.
    /// Example: Suppose you want to hold an item with the mouse and drag it over another item to show a pop-up
    ///          that displays what happens when they combine. To achieve this you can use two overlapping cursors: the
    ///          main mouse cursor and a secondary item-held mouse cursor. Let the item in hand have an interactive element
    ///          for the main cursor and use `press_on_click`, `unpress_on_unclick_recommended`, `startpress_callback`,
    ///          and `unpress_callback`, where the press-start callback activates the secondary mouse cursor and the
    ///          unpress callback deactivates it (in addition to grabbing/releasing the item). Let the second item have
    ///          an interactive element for the secondary cursor and use `press_on_clickhold`,
    ///          `unpress_on_press_away_recommended`, `with_hovered_pressed_widget`, `press_home_start_callback`,
    ///          where the hovered-pressed widget displays the pop-up and the callback edits the pop-up contents.
    pub fn unpress_on_press_away_recommended(self) -> Self
    {
        self.unpress_on_press_away()
            .abort_press_on_press_away_if_not_present()
    }

    /// Abort press when a click hold is detected away from the press home zone.
    /// - Disables settings `unpress_on_unclick_away`, `unpress_on_press_away`, and
    ///   `abort_press_on_press_away_if_not_present`.
    /// - Unpresses the element WITHOUT invoking any unpress callbacks. Will invoke `abortpress_callback`.
    pub fn abort_press_on_press_away(mut self) -> Self
    {
        self.unpress_on_unclick_away                 = false;
        self.unpress_on_press_away                    = false;
        self.abort_press_on_press_away                = true;
        self.abort_press_on_press_away_if_not_present = false;
        self
    }

    /// Abort press when a click hold is detected away from the press home zone and the press home zone is not present
    /// (it is invisible or the event occurs above an interaction barrier higher than the press home zone).
    /// - Disables setting `abort_press_on_press_away`.
    /// - Unpresses the element WITHOUT invoking any unpress callbacks. Will invoke `abortpress_callback`.
    pub fn abort_press_on_press_away_if_not_present(mut self) -> Self
    {
        self.abort_press_on_press_away                = false;
        self.abort_press_on_press_away_if_not_present = true;
        self
    }

    /// Select the element when a click is detected on the element (i.e. just clicked).
    /// - Disables settings `select_on_press_start`, `select_on_unpress`, and `select_on_hover_start`.
    pub fn select_on_click(mut self) -> Self
    {
        self.select_on_click       = true;
        self.select_on_press_start = false;
        self.select_on_unpress     = false;
        self.select_on_hover_start = false;
        self
    }

    /// Select the element when the element was just pressed.
    /// - Disables settings `select_on_click`, `select_on_unpress`, and `select_on_hover_start`.
    pub fn select_on_press_start(mut self) -> Self
    {
        self.select_on_click       = false;
        self.select_on_press_start = true;
        self.select_on_unpress     = false;
        self.select_on_hover_start = false;
        self
    }

    /// Select the element when the element was just unpressed.
    /// - Disables settings `select_on_click`, `select_on_press_start`, `select_on_hover_start`.
    pub fn select_on_unpress(mut self) -> Self
    {
        self.select_on_click       = false;
        self.select_on_press_start = false;
        self.select_on_unpress     = true;
        self.select_on_hover_start = false;
        self
    }

    /// Select the element when the element just started being hovered.
    /// - Disables settings `select_on_click`, `select_on_press_start`, `select_on_unpress`.
    pub fn select_on_hover_start(mut self) -> Self
    {
        self.select_on_click       = false;
        self.select_on_press_start = false;
        self.select_on_unpress     = false;
        self.select_on_hover_start = true;
        self
    }

    /// Selecting the element will toggle between selecting/deselecting the element.
    pub fn with_select_toggling(mut self) -> Self
    {
        self.with_select_toggling = true;
        self
    }

    /// Do not hover the element when it is `Pressed::{Home/Away}`.
    pub fn no_hover_on_pressed(mut self) -> Self
    {
        self.no_hover_on_pressed = true;
        self
    }

    /// Do not hover the element when it is `Selected`.
    pub fn no_hover_on_selected(mut self) -> Self
    {
        self.no_hover_on_selected = true;
        self
    }

    /// Do not hover the element when it is `Pressed::{Home/Away}` and `Selected`.
    pub fn no_hover_on_pressed_selected(mut self) -> Self
    {
        self.no_hover_on_pressed_selected = true;
        self
    }

    /// Callback invoked when a click is detected on the element (i.e. just clicked).
    /// - Takes the world position of the cursor.
    pub fn on_click_callback(mut self, callback: impl Fn(&mut World, Vec2) -> () + Send + Sync + 'static) -> Self
    {
        self.on_click_callback = Some(CallbackWith::<(), Vec2>::new(callback));
        self
    }

    /// Callback invoked when click hold is detected on the element.
    /// - Takes the world position of the cursor.
    /// - Invoked every tick while true.
    pub fn on_clickhold_callback(mut self, callback: impl Fn(&mut World, Vec2) -> () + Send + Sync + 'static) -> Self
    {
        self.on_clickhold_callback = Some(CallbackWith::<(), Vec2>::new(callback));
        self
    }

    /// Callback invoked when an unclick is detected and the press home zone is pressed.
    /// - Takes the world position of the cursor.
    /// - Takes a bool indicating if the clicker was above the press home zone when the unclick occurred.
    /// - WARNING: You must specify a press activator and deactivator to use this, since unclicks only make sense in the
    ///            context of pressing the element.
    pub fn on_unclick_callback(
        mut self,
        callback: impl Fn(&mut World, (Vec2, bool)) -> () + Send + Sync + 'static
    ) -> Self
    {
        self.on_unclick_callback = Some(CallbackWith::<(), (Vec2, bool)>::new(callback));
        self
    }

    /// Callback invoked when the element just started being hovered.
    /// - Takes the world position of the cursor.
    pub fn on_hover_start_callback(
        mut self,
        callback: impl Fn(&mut World, Vec2) -> () + Send + Sync + 'static
    ) -> Self
    {
        self.on_hover_start_callback = Some(CallbackWith::<(), Vec2>::new(callback));
        self
    }

    /// Callback invoked when the element is being hovered.
    /// - Takes the world position of the cursor.
    /// - Invoked every tick while true.
    pub fn on_hover_callback(
        mut self,
        callback: impl Fn(&mut World, Vec2) -> () + Send + Sync + 'static
    ) -> Self
    {
        self.on_hover_callback = Some(CallbackWith::<(), Vec2>::new(callback));
        self
    }

    /// Callback invoked when the element stops being hovered.
    pub fn on_unhover_callback(
        mut self,
        callback: impl Fn(&mut World) -> () + Send + Sync + 'static
    ) -> Self
    {
        self.on_unhover_callback = Some(Callback::<()>::new(callback));
        self
    }

    /// Callback invoked when the element is just pressed.
    /// - Takes the world position of the cursor.
    pub fn startpress_callback(
        mut self,
        callback: impl Fn(&mut World, Vec2) -> () + Send + Sync + 'static
    ) -> Self
    {
        self.startpress_callback = Some(CallbackWith::<(), Vec2>::new(callback));
        self
    }

    /// Callback invoked when the element just transitioned to `Pressed::Home`.
    /// - Takes the world position of the cursor.
    pub fn press_home_start_callback(
        mut self,
        callback: impl Fn(&mut World, Vec2) -> () + Send + Sync + 'static
    ) -> Self
    {
        self.press_home_start_callback = Some(CallbackWith::<(), Vec2>::new(callback));
        self
    }

    /// Callback invoked when the element is in state `Pressed::Home`.
    /// - Takes the world position of the cursor.
    /// - Invoked every tick while true.
    pub fn press_home_callback(
        mut self,
        callback: impl Fn(&mut World, Vec2) -> () + Send + Sync + 'static
    ) -> Self
    {
        self.press_home_callback = Some(CallbackWith::<(), Vec2>::new(callback));
        self
    }

    /// Callback invoked when the element just transitioned to `Pressed::Away`.
    /// - Takes the world position of the cursor.
    pub fn press_away_start_callback(
        mut self,
        callback: impl Fn(&mut World, Vec2) -> () + Send + Sync + 'static
    ) -> Self
    {
        self.press_away_start_callback = Some(CallbackWith::<(), Vec2>::new(callback));
        self
    }

    /// Callback invoked when the element is in state `Pressed::Away`.
    /// - Takes the world position of the cursor.
    /// - Invoked every tick while true.
    pub fn press_away_always_callback(
        mut self,
        callback: impl Fn(&mut World, Vec2) -> () + Send + Sync + 'static
    ) -> Self
    {
        self.press_away_always_callback = Some(CallbackWith::<(), Vec2>::new(callback));
        self
    }

    /// Callback invoked when the element is in state `Pressed::Away` if the element is present (is visible and the
    /// event does not occur above an interaction barrier higher than the press home zone).
    /// - Takes the world position of the cursor.
    /// - Invoked every tick while true.
    pub fn press_away_if_present_callback(
        mut self,
        callback: impl Fn(&mut World, Vec2) -> () + Send + Sync + 'static
    ) -> Self
    {
        self.press_away_if_present_callback = Some(CallbackWith::<(), Vec2>::new(callback));
        self
    }

    /// Callback invoked when the element is in state `Pressed::Away` if the element is not present (is invisible or the
    /// event occurs above an interaction barrier higher than the press home zone).
    /// - Takes the world position of the cursor.
    /// - Invoked every tick while true.
    pub fn press_away_if_not_present_callback(
        mut self,
        callback: impl Fn(&mut World, Vec2) -> () + Send + Sync + 'static
    ) -> Self
    {
        self.press_away_if_not_present_callback = Some(CallbackWith::<(), Vec2>::new(callback));
        self
    }

    /// Callback invoked when the element is unpressed.
    /// - Takes the world position of the cursor.
    pub fn unpress_callback(
        mut self,
        callback: impl Fn(&mut World, Vec2) -> () + Send + Sync + 'static
    ) -> Self
    {
        self.unpress_callback = Some(CallbackWith::<(), Vec2>::new(callback));
        self
    }

    /// Callback invoked when press is aborted on the element.
    /// - Takes the world position of the cursor.
    pub fn abortpress_callback(
        mut self,
        callback: impl Fn(&mut World, Vec2) -> () + Send + Sync + 'static
    ) -> Self
    {
        self.abortpress_callback = Some(CallbackWith::<(), Vec2>::new(callback));
        self
    }

    /// Callback invoked when the element is selected.
    /// - Takes the world position of the cursor.
    pub fn select_callback(
        mut self,
        callback: impl Fn(&mut World) -> () + Send + Sync + 'static
    ) -> Self
    {
        self.select_callback = Some(Callback::<()>::new(callback));
        self
    }

    /// Callback invoked when the element is deselected.
    pub fn deselect_callback(
        mut self,
        callback: impl Fn(&mut World) -> () + Send + Sync + 'static
    ) -> Self
    {
        self.deselect_callback = Some(Callback::<()>::new(callback));
        self
    }

    /// Consume the builder to build interactivity into the element.
    /// - The builder returns an error if the build configuration is incomplete or inconsistent (e.g. a press activator
    ///   was specified but no press deactivator).
    /// - If you want multiple interaction sources, add `InteractiveElement<[source]>::default()` bundles to the element
    ///   for each additional source. Use the `DisableElementInteractionSource` and `DisableInteractiveElementTargeting`
    ///   commands to disable a source or targeting on an element. Note that if an element's parent UI does not have a
    ///   copy of a source's `LunexUI` tag, then the source is unlikely to work as intended.
    pub fn build<S: InteractionSource>(
        mut self,
        entity_commands : &mut EntityCommands,
        element_widget  : Widget,
    ) -> Result<(), InteractiveElementBuilderError>
    {
        // check invariants
        check_press_invariants(&self)?;
        check_select_invariants(&self)?;
        check_hover_invariants(&self)?;

        // check which handlers we need
        let need_press  = self.element_is_pressable();
        let need_select = self.element_is_selectable();
        let need_hover  = self.element_is_hoverable();

        // define the widget that will handle press-away events
        let press_home_zone = self.press_home_zone.unwrap_or(element_widget.clone());

        // spawn initial components on the element
        entity_commands.insert(
                (
                    element_widget.clone(),
                    InteractiveElement::<S>::default(),
                )
            );
        if need_press { entity_commands.insert(PressHomeZone(press_home_zone)); }


        // prepare visibility updater
        let element_entity = entity_commands.id();
        let widget_pack    = self.widget_pack;

        let update_widget_visibility =
            move |world: &mut World|
            {
                syscall(world, (), apply_deferred);  //make sure any side-effects have resolved
                if let Err(_) = syscall(
                        world,
                        (element_entity, widget_pack.clone()),
                        update_interactive_element_visibility::<S::LunexUI>,
                    )
                { tracing::warn!(?element_entity, ?element_widget, "failed updating element visibility"); }
            };

        // prepare hover fixer (unhovers the element if a 'no hover' condition is met)
        //todo: how to avoid making this if it's not needed?
        let hover_fixer = prepare_hover_fixer(
                need_hover,
                element_entity,
                self.no_hover_on_selected,
                self.no_hover_on_pressed,
                self.no_hover_on_pressed_selected,
            );


        // action: start press
        //need_press
        //add Pressed::Home component (leave if Pressed already exists)
        //[option] action: select
        //hover fixer
        //callback: on press start
        //update visibility
        maybe_build_action_start_press(
                need_press,
                entity_commands,
                element_entity,
                self.select_on_press_start,
                self.no_hover_on_pressed,
                self.no_hover_on_pressed_selected,
                &hover_fixer,
                self.startpress_callback.take(),
                &update_widget_visibility,
            );

        // action: unpress
        //need_press
        //remove Pressed component (leave if nothing removed)
        //[option] action: select
        //hover fixer
        //callback: on unpress
        //update visibility
        maybe_build_action_unpress(
                need_press,
                entity_commands,
                element_entity,
                self.select_on_unpress,
                self.no_hover_on_pressed,
                self.no_hover_on_pressed_selected,
                &hover_fixer,
                self.unpress_callback.take(),
                &update_widget_visibility,
            );

        // action: abort press
        //need_press (note: if need_press then always need this action in case of window focus change)
        //remove Pressed component (leave if nothing removed)
        //hover fixer
        //callback: on abort press
        //update visibility
        maybe_build_action_abort_press(
                need_press,
                entity_commands,
                element_entity,
                self.no_hover_on_pressed,
                self.no_hover_on_pressed_selected,
                &hover_fixer,
                self.abortpress_callback.take(),
                &update_widget_visibility,
            );

        // action: select
        //need_select
        //if toggle select and selected: action: on deselect, then leave
        //add Selected component (leave if already exists)
        //hover fixer
        //callback: on select
        //update visibility
        maybe_build_action_select(
                need_select,
                entity_commands,
                element_entity,
                self.with_select_toggling,
                self.no_hover_on_selected,
                self.no_hover_on_pressed_selected,
                &hover_fixer,
                self.select_callback.take(),
                &update_widget_visibility,
            );

        // action: deselect
        //need_select
        //remove Selected component (leave if nothing removed)
        //hover fixer
        //callback: on deselect
        //update visibility
        maybe_build_action_deselect(
                need_select,
                entity_commands,
                element_entity,
                self.no_hover_on_selected,
                self.no_hover_on_pressed_selected,
                &hover_fixer,
                self.deselect_callback.take(),
                &update_widget_visibility,
            );


        // responder: on click
        //[option] action: start press
        //[option] action: select
        //callback: on click
        maybe_build_responder_on_click(
                entity_commands,
                element_entity,
                self.press_on_click,
                self.select_on_click,
                self.on_click_callback.take(),
            );

        // responder: on click hold
        //[option] action: start press
        //callback: on click hold
        maybe_build_responder_on_click_hold(
                entity_commands,
                element_entity,
                self.press_on_clickhold,
                self.on_clickhold_callback.take(),
            );

        // responder: on click hold home w/ Pressed component
        //need_press
        //callback: press home
        //set Pressed component to Home (leave if not pressed or no change)
        //hover fixer
        //callback: press home start
        //update visibility
        maybe_build_responder_on_click_hold_home(
                need_press,
                entity_commands,
                element_entity,
                self.no_hover_on_pressed,
                self.no_hover_on_pressed_selected,
                &hover_fixer,
                self.press_home_start_callback.take(),
                self.press_home_callback.take(),
                &update_widget_visibility,
            );

        // responder: on click hold away w/ Pressed component
        //need_press
        //[option: always]: action: abort press, then leave
        //[option: not present]: action: abort press, then leave
        //[option] action: unpress, then leave
        // - always
        //-try change Pressed component to Away (don't leave, we need to do callbacks after this)
        // hover fixer
        // callback: press away start
        //callback: press away always
        // - if present
        //callback: press away if present
        // - if not present
        //callback: press away if not present
        // - always (end)
        //-if changed Pressed component
        // update visibility
        maybe_build_responder_on_click_hold_away(
                need_press,
                entity_commands,
                element_entity,
                self.abort_press_on_press_away,
                self.abort_press_on_press_away_if_not_present,
                self.unpress_on_press_away,
                self.no_hover_on_pressed,
                self.no_hover_on_pressed_selected,
                &hover_fixer,
                self.press_away_start_callback.take(),
                self.press_away_always_callback.take(),
                self.press_away_if_present_callback.take(),
                self.press_away_if_not_present_callback.take(),
                &update_widget_visibility,
            );

        // responder: on unclick w/ Pressed component {INFO: on/off home zone widget}
        //need_press
        //callback: on unclick
        //-if on home zone widget
        // [option] action: unpress
        //-else
        // [option] action: abort press
        // [option] action: unpress
        maybe_build_responder_on_unclick(
                need_press,
                entity_commands,
                element_entity,
                self.unpress_on_unclick_home,
                self.abort_press_on_unclick_away,
                self.unpress_on_unclick_away,
                self.on_unclick_callback.take(),
            );

        // responder: on hover
        //need_hover
        //check if hover may start (don't check if unhover is needed)
        //-try add Hovered component
        // [option]: action: select
        // callback: on hover start
        //callback: on hover
        //-if added hovered component
        // update visibility
        maybe_build_responder_on_hover(
                need_hover,
                entity_commands,
                element_entity,
                self.no_hover_on_selected,
                self.no_hover_on_pressed,
                self.no_hover_on_pressed_selected,
                self.select_on_hover_start,
                self.on_hover_start_callback.take(),
                self.on_hover_callback.take(),
                &update_widget_visibility,
            );

        // responder: on unhover w/ Hovered component
        //need_hover
        //remove Hovered component (leave if not removed)
        //callback: on unhover
        //update visibility
        maybe_build_responder_on_unhover(
                need_hover,
                entity_commands,
                element_entity,
                self.on_unhover_callback.take(),
                &update_widget_visibility,
            );

        Ok(())
    }

    fn element_is_pressable(&self) -> bool
    {
        self.press_on_click ||
        self.press_on_clickhold
    }

    fn element_is_selectable(&self) -> bool
    {
        self.select_on_click       ||
        self.select_on_press_start ||
        self.select_on_unpress     ||
        self.select_on_hover_start
    }

    fn element_is_hoverable(&self) -> bool
    {
        self.widget_pack.hovered_widget.is_some()                ||
        self.widget_pack.hover_selected_widget.is_some()         ||
        self.widget_pack.hover_pressed_widget.is_some()          ||
        self.widget_pack.hover_pressed_selected_widget.is_some() ||
        self.select_on_hover_start                               ||
        self.on_hover_start_callback.is_some()                   ||
        self.on_hover_callback.is_some()                         ||
        self.on_unhover_callback.is_some()
    }
}

//-------------------------------------------------------------------------------------------------------------------
