/*//local shortcuts
use bevy_kot::prelude::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowTheme};
use bevy_lunex::prelude::*;

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
        entity,
        (right_edge_min_rel, right_edge_max_rel)
    ))           : In<(Entity, (f32, f32))>,
    mut commands : Commands,
    cursor_pos   : CursorPos<MainMouseCursor>,
    widgets      : Query<&Widget>,
    ui           : Query<&UiTree<MainUI>>,  //todo: InFocusedWindow
){
    // slider entity
    let Some(mut entity_commands) = commands.get_entity(entity) else { return; };
    let Ok(slider_widget) = widgets.get(entity) else { return; };

    // widget start position
    let Ok(ui) = ui.get_single() else { return; };
    let Ok(widget_branch) = slider_widget.fetch(&ui) else { return; };
    let widget_start_pos = widget_branch.container_get().position_get().get_pos(Vec2::default());
    let LayoutPackage::Relative(ref layout) = widget_branch.container_get().layout_get() else { return; };
    let widget_start_displacement_1_x = layout.absolute_1.x;
    let widget_start_displacement_2_x = layout.absolute_2.x;

    // cursor start position
    let Some(cpos_screen) = cursor_pos.get_screen() else { return; };

    // denormalize edge min/max to absolute coordinates
    let right_edge_min = right_edge_min_rel * ui.width / 100.0;
    let right_edge_max = right_edge_max_rel * ui.width / 100.0;

    // overwrite any existing drag state
    entity_commands.insert(
            SliderDragState{
                    right_edge_min,
                    right_edge_max,
                    drag_start_x   : cpos_screen.x,
                    widget_start_x : widget_start_pos.x,
                    widget_start_displacement_1_x,
                    widget_start_displacement_2_x
                }
        );
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn drag_slider(
    In(entity) : In<Entity>,
    cursor_pos : CursorPos<MainMouseCursor>,
    mut ui     : Query<&mut UiTree<MainUI>>,  //todo: InFocusedWindow
    slider     : Query<(&Widget, &SliderDragState)>,
){
    // slider
    let Ok((slider_widget, drag_state)) = slider.get(entity) else { return; };

    // ui
    let Ok(mut ui) = ui.get_single_mut() else { return; };

    // get new position for widget
    let Some(cpos_screen) = cursor_pos.get_screen() else { return; };
    let drag_diff = cpos_screen.x - drag_state.drag_start_x;
    let new_widget_x = (drag_state.widget_start_x + drag_diff).max(drag_state.right_edge_min).min(drag_state.right_edge_max);
    let new_widget_x_diff = new_widget_x - drag_state.widget_start_x;

    // update widget position
    let Ok(widget_branch) = slider_widget.fetch_mut(&mut ui) else { return; };
    let LayoutPackage::Relative(ref mut layout) = widget_branch.container_get_mut().layout_get_mut() else { return; };
    layout.absolute_1.x = drag_state.widget_start_displacement_1_x + new_widget_x_diff;
    layout.absolute_2.x = drag_state.widget_start_displacement_2_x + new_widget_x_diff;
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn add_button_rect(ui: &mut UiBuilder<MainUI>, area: &Widget, color: Color)
{
    let image = ImageElementBundle::new(
            area,
            ImageParams::center()
                .with_width(Some(100.))
                .with_height(Some(100.))
                .with_color(color),
            ui.asset_server.load("example_button_rect.png"),
            Vec2::new(250.0, 142.0)
        );
    ui.commands().spawn(image);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut ui: UiBuilder<MainUI>)
{
    // root widget
    let root = relative_widget(ui.tree(), "root", (0., 100.), (0., 100.));

    // slider bar widget
    let slider_x_left = 30.0;
    let slider_x_right = 70.0;
    let slider_width = slider_x_right - slider_x_left;
    let slider_bar = relative_widget(ui.tree(), root.end("slider_bar"), (slider_x_left, slider_x_right), (49., 51.));

    // slider bar image tied to slider bar
    add_button_rect(&mut ui, &slider_bar, Color::BLACK);

    // button widget
    let button_edge_right = 25.0;
    let button_edge_right_max = button_edge_right + slider_width;
    let button_x_range_rel = (button_edge_right, button_edge_right_max);
    let button = relative_widget(ui.tree(), root.end("button"), (button_edge_right, 35.0), (45., 55.));

    // default button image tied to button
    let default_widget = make_overlay(ui.tree(), &button, "default", true);
    add_button_rect(&mut ui, &default_widget, Color::GRAY);

    // pressed button image tied to button
    let pressed_widget = make_overlay(ui.tree(), &button, "pressed", false);
    add_button_rect(&mut ui, &pressed_widget, Color::DARK_GRAY);  //tint when pressed

    // slider button home zone covers screen and is positioned slightly above slider button
    let slider_button_home_zone = make_overlay(ui.tree(), &root, "", true);
    let zone_depth = button.fetch(ui.tree()).unwrap().get_depth() + 0.01;
    slider_button_home_zone.fetch_mut(ui.tree()).unwrap().set_depth(zone_depth);

    // button entity
    let mut entity_commands = ui.commands().spawn_empty();
    let entity = entity_commands.id();

    // button as interactive element
    InteractiveElementBuilder::new()
        .with_default_widget(default_widget)
        .with_pressed_widget(pressed_widget)
        .with_press_home_zone(slider_button_home_zone)
        .press_on_click()
        .unpress_on_unclick_home_or_away()
        .abort_press_if_obstructed()
        .on_startpress(move |world: &mut World| syscall(world, (entity, button_x_range_rel), initialize_slider_drag))
        .on_press_home(move |world: &mut World| syscall(world, entity, drag_slider))
        .build::<MouseLButtonMain>(&mut entity_commands, button.clone())
        .unwrap();
    entity_commands.insert(UIInteractionBarrier::<MainUI>::default());
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup(mut commands: Commands, window: Query<Entity, (With<Window>, With<PrimaryWindow>)>)
{
    // prepare 2D camera
    commands.spawn(
            Camera2dBundle{ transform: Transform{ translation: Vec3 { x: 0., y: 0., z: 1000. }, ..default() }, ..default() }
        );

    // make lunex cursor
    commands.spawn((Cursor::new(0.0), Transform::default(), MainMouseCursor));

    // prepare lunex ui tree
    commands.insert_resource(StyleStackRes::<MainUI>::default());
    let tree = UiTree::<MainUI>::new("ui");

    let window = window.single();
    commands.entity(window).insert((tree, Transform::default(), Size::default()));
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
        .add_plugins(LunexUiPlugin2D::<MainUI>(std::marker::PhantomData::default()))
        //.add_plugins(UIDebugOverlayPlugin)
        .add_plugins(ReactPlugin)
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .register_interaction_source(MouseLButtonMain::default())
        .add_systems(PreStartup, setup)
        .add_systems(Startup, build_ui)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------
*/ fn main() {}