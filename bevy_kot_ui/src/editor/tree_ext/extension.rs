//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_lunex::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn find_new_tree_nodes<Ui: LunexUi>(
    mut tree_ext : ResMut<TreeExt<Ui>>,
    new_nodes    : Query<(Entity, &LinePosition, &Widget), Added<TreeNode<Ui>>>
){
    for (new_node, line_pos, widget) in new_nodes.iter()
    {
        tree_ext.active_nodes.insert(new_node, TreeNodeMeta::new(*line_pos, String::new(widget.path())));
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn find_dead_tree_nodes<Ui: LunexUi>(
    mut tree_ext   : ResMut<TreeExt<Ui>>,
    mut dead_nodes : RemovedComponents<TreeNode<Ui>>,
){
    for dead_node in dead_nodes.read()
    {
        tree_ext.active_nodes.remove(&dead_node);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Implements [`EditorWindowExtension`] for UI trees.
//todo: more efficient mapping strategy for rebuilding the tree preview
#[derive(ReactResource, Default)]
pub struct TreeExt<Ui: LunexUi>
{
    //todo: registered tree extensions; used to populate node editor from node meta extension data when a node is selected
    active_nodes: HashMap<Entity, TreeNodeMeta>,
    active_node_paths: HashMap<String, Entity>,  //need to check for entries to remove when the ui tree updates?
    //todo: last selected node; used to rebuild the node editor?
}

impl<Ui: LunexUi> TreeExt<Ui>
{
    pub fn get_node_mut(&mut self, entity: Entity) -> Option<&mut TreeNodeMeta>
    {
        self.active_nodes.get_mut(&entity)
    }
}

impl<Ui: LunexUi> Default for TreeExt<Ui>
{
    fn default() -> Self
    {
        Self{ active_nodes: HashMap::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet)]
pub struct TreeExtSystemSet<Ui: LunexUi>;

/// Adds the tree extension to the app.
pub fn TreeExtPlugin<Ui: LunexUi>;

impl<Ui: LunexUi> Plugin for TreeExtPlugin<Ui>
{
    fn build(app: &mut App)
    {
        app.insert_react_resource(TreeExt::<Ui>::default())
            .add_systems(PreUpdate,
                (
                    find_new_tree_nodes::<Ui>,
                    find_dead_tree_nodes::<Ui>,
                ).in_set(TreeExtSystemSet)
            );

        //rebuild tree ext display if the ui tree changed
        // - in update schedule so subextensions can update first?
        // - make sure if there are no display changes it won't trigger ui tree change detection

        //rebuild tree ext display if the tree ext changed
        // - in update schedule so subextensions can update first?
    }
}

//-------------------------------------------------------------------------------------------------------------------

/*

- editor uses its own lunex UI

- tree nodes as entities
    - tree node data component
    - tree node extension data components
        - can be mutated by extensions
- tree widget visualization entities
    - visualization builder
        - plain widgets: these are not interactable
        - tree nodes: these are interactable
            - input
                - tree node entity
                - registered tree node extensions
                - implicit: tree node editor resource, ability to toggle the tree node editor window
            - tree node extensions can add callbacks to the node element (e.g. layout extension shows debug lines on hover)
        - modify the node appearance
            - decide widget appearance by querying extensions
                - base: basic text
                - node: bolded text
                - other: custom
            - when widget is dirty: mark with asterisk or italicize?
                - system: in postupdate/last, query all extensions on the current-selected node; mark dirty if at least one
                extension is dirty
    - cleanup when mirrored widget dies
        - despawn vis entity
        - remove visualization widget from ui tree
        - remove vis widget from tree extension
- tree extension
    - spawns new visualization entities when new tree widgets are detected
        - ignore "editor/tree_ext/tree_vis/*" widgets to avoid recursive mirroring
    - manually tracks all vis widgets in order to identify the location for new vis entities
    - tree layout
        - each branch is placed within a widget box, branch boxes stack on each other and shrink to fit their insides

*/

