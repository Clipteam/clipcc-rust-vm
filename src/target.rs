use std::str::FromStr;

use generational_arena::Index;

use crate::{
    blocks::get_blockinfo,
    *,
};

/// Current rotation style.
#[derive(Debug, Clone, Copy)]
pub enum RotationStyle {
    /// Rotation style for "all around"/spinning.
    AllAround,
    /// Rotation style for "left-right"/flipping.
    LeftRight,
    /// Rotation style for "no rotation."
    None,
}

impl FromStr for RotationStyle {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "all around" => Ok(Self::AllAround),
            "left-right" => Ok(Self::LeftRight),
            "don't rotate" => Ok(Self::None),
            _ => Ok(Self::AllAround),
        }
    }
}

impl Default for RotationStyle {
    fn default() -> Self {
        Self::AllAround
    }
}

#[derive(Default, Debug, Clone)]
pub struct Effects {
    pub color: f64,
    pub fisheye: f64,
    pub whirl: f64,
    pub pixelate: f64,
    pub mosaic: f64,
    pub brightness: f64,
    pub ghost: f64,
}

#[derive(Debug, Clone)]
pub struct Target {
    /// Human-readable name for this sprite (and all clones).
    pub name: String,
    /// Shared blocks object for all clones of sprite.
    pub blocks: Arena<Block>,
    /// Dictionary of variables and their values for this target.
    /// Key is the variable id.
    pub variables: BTreeMap<String, BlockValue>,
    /// Dictionary of variables and their list values for this target.
    /// Key is the variable id.
    pub lists: BTreeMap<String, Vec<BlockValue>>,
    /// Scratch X coordinate. Currently should range from -240 to 240.
    pub x: f64,
    /// Scratch Y coordinate. Currently should range from -180 to 180.
    pub y: f64,
    /// Scratch direction. Currently should range from -179 to 180.
    pub direction: f64,
    /// Whether the rendered target is currently visible.
    pub visible: bool,
    /// Size of rendered target as a percent of costume size.
    pub size: f64,
    /// Currently selected costume index.
    pub current_costume: usize,
    /// Costume names.
    pub costumes: Vec<String>,
    /// Costume names.
    pub sounds: Vec<String>,
    /// Current layer order index.
    pub layer_order: usize,
    /// Current rotation style.
    pub rotation_style: RotationStyle,
    /// Loudness for sound playback for this target, as a percentage.
    pub volume: f64,
    /// Current tempo (used by the music extension).
    /// This property is global to the project and stored in the stage.
    pub tempo: f64,
}

impl Default for Target {
    fn default() -> Self {
        Self {
            direction: 90.,
            visible: true,
            size: 100.,
            tempo: 60.,
            volume: 100.,
            layer_order: 0,
            name: String::new(),
            blocks: Arena::new(),
            variables: BTreeMap::new(),
            lists: BTreeMap::new(),
            x: 0.0,
            y: 0.0,
            current_costume: 0,
            costumes: Vec::new(),
            sounds: Vec::new(),
            rotation_style: RotationStyle::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RunningTarget {
    pub target_id: TargetId,
    pub name: String,
    pub is_clone: bool,
    /// Dictionary of variables and their values for this target.
    /// Key is the variable id.
    pub variables: BTreeMap<String, BlockValue>,
    /// Dictionary of variables and their list values for this target.
    /// Key is the variable id.
    pub lists: BTreeMap<String, Vec<BlockValue>>,
    /// Scratch X coordinate. Currently should range from -240 to 240.
    pub x: f64,
    /// Scratch Y coordinate. Currently should range from -180 to 180.
    pub y: f64,
    /// Scratch direction. Currently should range from -179 to 180.
    pub direction: f64,
    /// Whether the rendered target is currently visible.
    pub visible: bool,
    /// Size of rendered target as a percent of costume size.
    pub size: f64,
    /// Currently selected costume index.
    pub current_costume: usize,
    /// Current rotation style.
    pub rotation_style: RotationStyle,
    /// Loudness for sound playback for this target, as a percentage.
    pub volume: f64,
    /// Current tempo (used by the music extension).
    /// This property is global to the project and stored in the stage.
    pub tempo: f64,
    /// Current layer order index.
    pub layer_order: usize,
}

impl Target {
    pub fn make_target(&self, target_id: TargetId, is_clone: bool) -> RunningTarget {
        RunningTarget {
            target_id,
            name: self.name.to_owned(),
            is_clone,
            variables: self.variables.to_owned(),
            lists: self.lists.to_owned(),
            x: self.x,
            y: self.y,
            size: self.size,
            visible: self.visible,
            direction: self.direction,
            current_costume: self.current_costume,
            rotation_style: self.rotation_style,
            layer_order: self.layer_order,
            volume: self.volume,
            tempo: self.tempo,
        }
    }
    pub fn new_block(
        &mut self,
        opcode: &str,
        parent: Option<BlockId>,
        arguments: &[BlockValue],
    ) -> BlockId {
        let func = get_blockinfo(opcode)
            .map(|a| a.block_function)
            .unwrap_or_else(|| blocks::noop);
        let bid = self.blocks.alloc_with_id(|bid| Block {
            self_id: bid,
            toplevel: parent.is_none(),
            arguments: arguments.to_vec(),
            opcode: string_to_static_str(opcode.to_owned()),
            block_function: func,
            next: None,
            #[cfg(debug_assertions)]
            block_id: "[Host Generated]".into(),
        });
        if let Some(parent) = parent {
            if let Some(parent_block) = self.blocks.get_mut(parent) {
                parent_block.next = Some(bid);
            }
        }
        bid
    }
}

pub type TargetId = Id<Target>;
pub type RunningTargetId = Index;
