//local shortcuts
use super::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_lunex::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Test if a cursor intersects with a widget.
/// - Returns `Ok(Some(widget_depth))` on success (for use in `LunexCursor` methods).
pub fn cursor_intersects_widget(
    cursor_lunex_position : Vec2,
    ui                    : &UiTree,
    widget                : &Widget,
    depth_limit           : Option<f32>,
    widget_depth          : f32,
) -> Result<Option<f32>, ()>
{
    // check if the widget is lower than the depth limit
    if let Some(depth_limit) = depth_limit { if depth_limit > widget_depth { return Ok(None); } }

    // check if the cursor is within the widget area
    match widget.contains_position(&ui, &cursor_lunex_position)
    {
        Ok(true)  => Ok(Some(widget_depth)),
        Ok(false) => Ok(None),
        _         => Err(()),
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Make a widget that exactly overlaps its parent widget.
/// - Panics if unable to create a widget (mostly likely because the widget name already exists in the tree with the
///   specified parent).
pub fn make_overlay(ui: &mut UiTree, parent: &Widget, overlay_name: &str, visible_by_default: bool) -> Widget
{
    // make overlay
    let overlay = Widget::create(
            ui,
            parent.end(overlay_name),
            RelativeLayout{
                relative_1: Vec2 { x: 0., y: 0. },
                relative_2: Vec2 { x: 100., y: 100. },
                ..Default::default()
            }
        ).unwrap();

    // set default visibility
    overlay.fetch_mut(ui).unwrap().set_visibility(visible_by_default);

    overlay
}

//-------------------------------------------------------------------------------------------------------------------

/// Toggle between two sets of widgets.
//todo: handle multiple uis (pass in UI entity)
pub fn toggle_ui_visibility<U: LunexUI, const ON: usize, const OFF: usize>(
    In((_, on_widgets, off_widgets)) : In<(U, [Widget; ON], [Widget; OFF])>,
    mut uis                          : Query<&mut UiTree, With<U>>,
){
    // get target ui
    let Ok(mut ui) = uis.get_single_mut() else { tracing::error!("multiple uis detected in toggle ui vis"); return; };

    // set widget visibility: on
    for on_widget in on_widgets
    {
        let Ok(on_widget_branch) = on_widget.fetch_mut(&mut ui) else { continue; };
        on_widget_branch.set_visibility(true);
    }

    // set widget visibility: off
    for off_widget in off_widgets
    {
        let Ok(off_widget_branch) = off_widget.fetch_mut(&mut ui) else { continue; };
        off_widget_branch.set_visibility(false);
    }
}

//-------------------------------------------------------------------------------------------------------------------
