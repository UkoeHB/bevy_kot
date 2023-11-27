//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_lunex::prelude::*;

//standard shortcuts

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn find_new_tree_node_layouts<Ui: LunexUi>(
    mut tree_ext : ResMut<TreeExt<Ui>>,
    new_nodes    : Query<(Entity, &Widget), Added<TreeNodeLayout>>
){
    for (new_node, widget) in new_nodes.iter()
    {
        let Some(new_node) = tree_ext.get_node_mut(new_node) else { continue; };
        new_node.set_extension_data(TreeNodeLayoutData::new(widget.clone()));
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn find_dead_tree_node_layouts<Ui: LunexUi>(
    mut tree_ext   : ResMut<TreeExt<Ui>>,
    mut dead_nodes : RemovedComponents<TreeNodeLayout>,
){
    for dead_node in dead_nodes.read()
    {
        let Some(new_node) = tree_ext.get_node_mut(new_node) else { continue; };
        new_node.remove_extension_data::<TreeNodeLayoutData>();
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Implements [`TreeExtension`] and [`EditorWindowExtension`] for the layout of UI tree nodes.
#[derive(Default)]
pub struct TreeNodeLayoutExt<Ui: LunexUi>
{
    
}

impl<Ui: LunexUi> Default for TreeNodeLayoutExt<Ui>
{
    fn default() -> Self
    {
        Self{ }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet)]
pub struct TreeNodeLayoutExtSystemSet<Ui: LunexUi>;

/// Adds the tree extension to the app.
pub fn TreeNodeLayoutExtPlugin<Ui: LunexUi>;

impl<Ui: LunexUi> Plugin for TreeNodeLayoutExtPlugin<Ui>
{
    fn build(app: &mut App)
    {
        app.insert_resource(TreeNodeLayoutExt::<Ui>::default())
            .add_systems(PreUpdate,
                (
                    find_new_tree_node_layouts::<Ui>,
                    find_dead_tree_node_layouts::<Ui>,
                ).after(TreeExtSystemSet)
            );

        //update tree ext display if the ui tree changed
        // (make sure if there are no display changes it won't trigger ui tree change detection)
    }
}

//-------------------------------------------------------------------------------------------------------------------
