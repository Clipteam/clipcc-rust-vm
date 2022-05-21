use crate::*;

#[derive(Clone)]
pub struct Block {
    pub self_id: BlockId,
    pub toplevel: bool,
    pub arguments: Vec<BlockValue>,
    pub opcode: &'static str,
    pub block_function: BlockFunction,
    pub next: Option<BlockId>,
    #[cfg(debug_assertions)]
    pub block_id: String,
}

pub type BlockId = Id<Block>;

impl std::fmt::Debug for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Block")
            .field("self_id", &self.self_id)
            .field("toplevel", &self.toplevel)
            .field("arguments", &self.arguments)
            .field("opcode", &self.opcode)
            .field("next", &self.next)
            .finish()
    }
}
