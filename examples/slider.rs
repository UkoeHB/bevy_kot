//path aliases
use bevy_kot::ecs as kot_ecs;
use bevy_kot::ui as kot;
use bevy_kot::ui::builtin as kot_builtin;
use bevy_lunex::prelude as lunex;

//local shortcuts
use kot::RegisterInteractionSourceExt;

//third-party shortcuts
use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_lunex::prelude::AsLunexVec2;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// WARNING: Becomes invalid if the window is resized (which should not happen during a slider drag since the cursor
///          is occupied).
#[derive(Component, Debug)]
struct SliderDragState
{
    right_edge_min: f32,
    right_edge_max: f32,
    drag_start_x: f32,
    widget_start_x: f32,
    widget_start_displacement_1_x: f32,
    widget_start_displacement_2_x: f32,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn initialize_slider_drag(
    In((
        cpos_world,
        entity,
        (right_edge_min_rel, right_edge_max_rel)
    ))           : In<(Vec2, Entity, (f32, f32))>,
    mut commands : Commands,
    widgets      : Query<&lunex::Widget>,
    ui           : Query<&lunex::UiTree, With<kot_builtin::MainUI>>,  //todo: InFocusedWindow
){
    // slider entity
    let Some(mut entity_commands) = commands.get_entity(entity) else { return; };
    let Ok(slider_widget) = widgets.get(entity) else { return; };

    // widget start position
    let Ok(ui) = ui.get_single() else { return; };
    let Ok(widget_branch) = slider_widget.fetch(&ui) else { return; };
    let widget_start_pos = widget_branch.container_get().position_get().get_pos(Vec2::default());
    let lunex::LayoutPackage::Relative(ref layout) = widget_branch.container_get().layout_get() else { return; };
    let widget_start_displacement_1_x = layout.absolute_1.x;
    let widget_start_displacement_2_x = layout.absolute_2.x;

    // cursor start position
    let cpos_lunex = cpos_world.as_lunex(ui.offset);

    // denormalize edge min/max to absolute coordinates
    let right_edge_min = right_edge_min_rel * ui.width / 100.0;
    let right_edge_max = right_edge_max_rel * ui.width / 100.0;

    // overwrite any existing drag state
    entity_commands.insert(
            SliderDragState{
                    right_edge_min,
                    right_edge_max,
                    drag_start_x   : cpos_lunex.x,
                    widget_start_x : widget_start_pos.x,
                    widget_start_displacement_1_x,
                    widget_start_displacement_2_x
                }
        );
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn drag_slider(
    In((
        cpos_world,
        entity,
    ))     : In<(Vec2, Entity)>,
    mut ui : Query<&mut lunex::UiTree, With<kot_builtin::MainUI>>,  //todo: InFocusedWindow
    slider : Query<(&lunex::Widget, &SliderDragState)>,
){
    // slider
    let Ok((slider_widget, drag_state)) = slider.get(entity) else { return; };

    // ui
    let Ok(mut ui) = ui.get_single_mut() else { return; };

    // get new position for widget
    let cpos_lunex = cpos_world.as_lunex(ui.offset);
    let drag_diff = cpos_lunex.x - drag_state.drag_start_x;
    let new_widget_x = (drag_state.widget_start_x + drag_diff).max(drag_state.right_edge_min).min(drag_state.right_edge_max);
    let new_widget_x_diff = new_widget_x - drag_state.widget_start_x;

    // update widget position
    let Ok(widget_branch) = slider_widget.fetch_mut(&mut ui) else { return; };
    let lunex::LayoutPackage::Relative(ref mut layout) = widget_branch.container_get_mut().layout_get_mut() else { return; };
    layout.absolute_1.x = drag_state.widget_start_displacement_1_x + new_widget_x_diff;
    layout.absolute_2.x = drag_state.widget_start_displacement_2_x + new_widget_x_diff;
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup(mut commands: Commands, asset_server: Res<AssetServer>)
{
    // prepare 2D camera
    commands.spawn(
            Camera2dBundle{ transform: Transform{ translation: Vec3 { x: 0., y: 0., z: 1000. }, ..default() }, ..default() }
        );

    // make lunex cursor
    commands.spawn((lunex::Cursor::new(0.0), Transform::default(), kot_builtin::MainMouseCursor));

    // create lunex ui tree
    let mut ui = lunex::UiTree::new("ui");

    // root widget
    let root = lunex::Widget::create(
            &mut ui,
            "root",
            lunex::RelativeLayout::new()
                .with_rel_1(Vec2 { x: 0.0, y: 0.0 })
                .with_rel_2(Vec2 { x: 100.0, y: 100.0 })
        ).unwrap();

    // slider bar widget
    let slider_x_left = 30.0;
    let slider_x_right = 70.0;
    let slider_width = slider_x_right - slider_x_left;
    let slider_bar = lunex::Widget::create(
            &mut ui,
            root.end("slider_bar"),
            lunex::RelativeLayout::new()
                .with_rel_1(Vec2 { x: slider_x_left, y: 49.0 })
                .with_rel_2(Vec2 { x: slider_x_right, y: 51.0 })
        ).unwrap();

    // slider bar image tied to slider bar
    commands.spawn(
        lunex::ImageElementBundle::new(
                    &slider_bar,
                    lunex::ImageParams::center()
                        .with_width(Some(100.))
                        .with_height(Some(100.))
                        .with_color(Color::BLACK),
                    asset_server.load("example_button_rect.png"),
                    Vec2::new(250.0, 142.0)
                )
        );

    // button widget
    let button_edge_right = 25.0;
    let button_edge_right_max = button_edge_right + slider_width;
    let button_x_range_rel = (button_edge_right, button_edge_right_max);
    let button = lunex::Widget::create(
            &mut ui,
            root.end("button"),
            lunex::RelativeLayout::new()
                .with_rel_1(Vec2 { x: button_edge_right, y: 45.0 })
                .with_rel_2(Vec2 { x: 35.0, y: 55.0 })
        ).unwrap();

    // default button image tied to button
    let default_widget = kot::make_overlay(&mut ui, &button, "default", true);
    commands.spawn(
        lunex::ImageElementBundle::new(
                    &default_widget,
                    lunex::ImageParams::center()
                        .with_width(Some(100.))
                        .with_height(Some(100.))
                        .with_color(Color::GRAY),
                    asset_server.load("example_button_rect.png"),
                    Vec2::new(250.0, 142.0)
                )
        );

    // pressed button image tied to button
    let pressed_widget = kot::make_overlay(&mut ui, &button, "pressed", false);
    commands.spawn(
        lunex::ImageElementBundle::new(
                    &pressed_widget,
                    lunex::ImageParams::center()
                        .with_width(Some(100.))
                        .with_height(Some(100.))
                        .with_color(Color::DARK_GRAY),  //tint when pressed
                    asset_server.load("example_button_rect.png"),
                    Vec2::new(250.0, 142.0)
                )
        );

    // slider button home zone covers screen and is positioned slightly above slider button
    let slider_button_home_zone = kot::make_overlay(&mut ui, &root, "", true);
    let zone_depth = button.fetch(&ui).unwrap().get_depth() + 0.01;
    slider_button_home_zone.fetch_mut(&mut ui).unwrap().set_depth(zone_depth);

    // button entity
    let mut entity_commands = commands.spawn_empty();
    let entity = entity_commands.id();

    // button as interactive element
    kot::InteractiveElementBuilder::new()
        .with_default_widget(default_widget)
        .with_pressed_widget(pressed_widget)
        .with_press_home_zone(slider_button_home_zone)
        .press_on_click()
        .unpress_on_unclick_home_or_away()
        .abort_press_if_obstructed()
        .startpress_callback(
            move |world, cpos_world|
            kot_ecs::syscall(world, (cpos_world, entity, button_x_range_rel), initialize_slider_drag)
        )
        .press_home_callback(
            move |world, cpos_world|
            kot_ecs::syscall(world, (cpos_world, entity), drag_slider)
        )
        .build::<kot_builtin::MouseLButtonMain>(&mut entity_commands, button.clone())
        .unwrap();
    entity_commands.insert(kot::UIInteractionBarrier::<kot_builtin::MainUI>::default());

    // add ui tree to ecs
    commands.spawn((ui, kot_builtin::MainUI));
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn main()
{
    App::new()
        .add_plugins(
            bevy::DefaultPlugins.set(
                WindowPlugin{
                    primary_window: Some(Window{ window_theme: Some(WindowTheme::Dark), ..Default::default() }),
                    ..Default::default()
                }
            )
        )
        .add_plugins(lunex::LunexUiPlugin)
        //.add_plugins(kot::UIDebugOverlayPlugin)
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .register_interaction_source(kot_builtin::MouseLButtonMain::default())
        .add_systems(Startup, setup)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------
