//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use bevy_fn_plugin::bevy_plugin;
use bevy_lunex::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// A marker for ImageBundles spawned by debug functions, ***NOT INTENDED*** to be used by user!
#[derive(Component)]
struct UIDebugOverlay;

/// Add ui debug overlay to existing widgets.
/// - We make overlay boxes with lines to reduce the effects of stretching.
fn setup_ui_debug_overlay(
    mut commands : Commands,
    asset_server : Res<AssetServer>,
    uis          : Query<&UiTree>,
){
    for ui in uis.iter()
    {
        for x in ui.collect_paths()
        {
            let widget = Widget::new(&x);
            match widget.fetch(ui)
            {
                Ok(..) =>
                {
                    commands.spawn(
                            (
                                ImageElementBundle::new(
                                    &widget,
                                    ImageParams::topleft().with_width(Some(100.0)),
                                    asset_server.load("ui_debug_horizontal.png"),
                                    Vec2::new(231.0, 1.0)
                                ),
                                UIDebugOverlay
                            )
                        );
                    commands.spawn(
                            (
                                ImageElementBundle::new(
                                    &widget,
                                    ImageParams::bottomleft().with_width(Some(100.0)),
                                    asset_server.load("ui_debug_horizontal.png"),
                                    Vec2::new(231.0, 1.0)
                                ),
                                UIDebugOverlay
                            )
                        );
                    commands.spawn(
                            (
                                ImageElementBundle::new(
                                    &widget,
                                    ImageParams::topleft().with_height(Some(100.0)),
                                    asset_server.load("ui_debug_vertical.png"),
                                    Vec2::new(1.0, 128.0)
                                ),
                                UIDebugOverlay
                            )
                        );
                    commands.spawn(
                            (
                                ImageElementBundle::new(
                                    &widget,
                                    ImageParams::topright().with_height(Some(100.0)),
                                    asset_server.load("ui_debug_vertical.png"),
                                    Vec2::new(1.0, 128.0)
                                ),
                                UIDebugOverlay
                            )
                        );
                }
                Err(_) => (),
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Update visibility of debug overlay sprites.
//todo: should be dynamic
fn update_ui_debug_overlay(
    uis                : Query<&UiTree>,
    mut debug_overlays : Query<(&mut Widget, &mut Transform, &UIDebugOverlay)>,
){
    let ui: &UiTree = uis.get_single().unwrap();
    for (widget, mut transform, _) in &mut debug_overlays
    {
        match widget.fetch(&ui)
        {
            Ok(branch) => transform.translation.z = branch.get_depth() + 400.0,
            Err(_) =>
            {
                transform.translation.x = -10000.0;
                transform.translation.y = -10000.0;
            }
        };
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Add outlines to all lunex widgets. (todo: only works if there is one UI tree)
#[bevy_plugin]
pub fn UIDebugOverlayPlugin(app: &mut App)
{
    app.add_systems(PostStartup, setup_ui_debug_overlay)
        .add_systems(PostUpdate, update_ui_debug_overlay);
}

//-------------------------------------------------------------------------------------------------------------------
