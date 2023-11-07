//local shortcuts
use bevy_kot::prelude::{*, builtin::*};

//third-party shortcuts
use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_lunex::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Style)]
struct StyleA
{
    color: Color,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Style)]
struct StyleB
{
    color: Color,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn spawn_box(ctx: &mut UiBuilderCtx<MainUI>, area: &Widget, color: Color)
{
    let image = ImageElementBundle::new(
            area,
            ImageParams::center()
                .with_width(Some(100.))
                .with_height(Some(100.))
                .with_color(color),
            ctx.asset_server.load("box.png"),
            Vec2::new(236.0, 139.0)
        );
    ctx.commands().spawn(image);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn section(ctx: &mut UiBuilderCtx<MainUI>, area: &Widget) -> (Widget, Widget)
{
    let widget_a = relative_widget(ctx.ui(), area.end("a"), (0., 50.), (25., 75.));
    spawn_box(ctx, &widget_a, ctx.get_style::<StyleA>().unwrap().color);

    let widget_b = relative_widget(ctx.ui(), area.end("b"), (50., 100.), (25., 75.));
    spawn_box(ctx, &widget_b, ctx.get_style::<StyleB>().unwrap().color);

    (widget_a, widget_b)
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut ctx: UiBuilderCtx<MainUI>)
{
    // add base styles
    ctx.add_style((StyleA{ color: Color::WHITE }, StyleB{ color: Color::BLACK }));

    // build ui tree
    let root = relative_widget(ctx.ui(), "root", (0., 100.), (0., 100.));
    ctx.div(&root, (), |ctx, area, _| {
        let (widget_a, widget_b) = section(ctx, &area);

        ctx.div(&widget_a, (), |ctx, area, _| {
            ctx.add_style(StyleA{ color: Color::BLUE });
            section(ctx, &area);
        });
        ctx.div(&widget_b, (), |ctx, area, _| {
            ctx.add_style(StyleA{ color: Color::ORANGE });
            ctx.add_style(StyleB{ color: Color::GREEN });
            section(ctx, &area);
        });
    });
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup(mut commands: Commands)
{
    // prepare 2D camera
    commands.spawn(
            Camera2dBundle{ transform: Transform{ translation: Vec3 { x: 0., y: 0., z: 1000. }, ..default() }, ..default() }
        );

    // make lunex cursor
    commands.spawn((Cursor::new(0.0), Transform::default()));

    // prepare lunex ui tree
    commands.insert_resource(StyleStackRes::<MainUI>::default());
    commands.spawn((UiTree::new("ui"), MainUI));
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
        .add_plugins(LunexUiPlugin)
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .add_systems(PreStartup, setup)
        .add_systems(Startup, build_ui)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------
