//local shortcuts
use crate::*;
use bevy_kot_utils::*;

//third-party shortcuts

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

cfg_if::cfg_if! {
    if #[cfg(feature = "editor")] {
        pub struct TreeNodeContext
        {
            line_pos: LinePosition,
        }

        impl TreeNodeContext
        {
            pub fn new(line_pos: LinePosition) -> Self
            {
                Self{ line_pos }
            }
        }
    }

    if #[cfg(not(feature = "editor"))] {
        pub struct TreeNodeContext;
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Collects editor information for a UI tree node.
///
/// Intended for use in the editor's UI helper functions.
///
/// Example:
/// ```no_run
/// div(n!(), ui, move |ui| { ... });
/// ```
macro_rules! n
{
    () =>
    {
        #[cfg(feature = "editor")]
        {
            TreeNodeContext::new(LinePosition::new(file!(), line!()))
        }

        #[cfg(not(feature = "editor"))]
        {
            TreeNodeContext
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper for [`UiBuilder::div()`].
///
/// This is currently just a wrapper with no extra functionality, included for stylistic consistency.
pub fn div<Ui: LunexUi, R>(_: TreeNodeContext, ui: &mut UiBuilder<Ui>, div: impl FnOnce(&mut UiBuilder<Ui>) -> R)
{
    ui.div(div)
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper for [`UiBuilder::div_rel()`]. Adds the widget to the editor as a layout node.
pub fn div_rel<Ui: LunexUi, R>(
    _ctx    : TreeNodeContext,
    ui      : &mut UiBuilder<Ui>,
    path    : impl Borrow<str>,
    x_range : (f32, f32),
    y_range : (f32, f32),
    div     : impl FnOnce(&mut UiBuilder<Ui>, &Widget) -> (Widget, R)
){
    let (area, result) = ui.div_rel(path, x_range, y_range, div);

    #[cfg(feature = "editor")]
    {
        let area = area.clone();
        ui.commands().add(
                move |world: &mut World|
                {
                    syscall(world, (_ctx.line_pos, area), add_layout_node::<Ui>);
                }
            );
    }

    (area, result)
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper for [`relative_widget()`]. Adds the widget to the editor as a layout node.
pub fn rel<Ui: LunexUi>(
    _ctx    : TreeNodeContext,
    ui      : &mut UiBuilder<Ui>,
    path    : impl Borrow<str>,
    x_range : (f32, f32),
    y_range : (f32, f32)
) -> Widget
{
    let area = relative_widget(ui.tree(), path, x_range, y_range);

    #[cfg(feature = "editor")]
    {
        let area = area.clone();
        ui.commands().add(
                move |world: &mut World|
                {
                    syscall(world, (_ctx.line_pos, area), add_layout_node::<Ui>);
                }
            );
    }

    area
}

//-------------------------------------------------------------------------------------------------------------------
