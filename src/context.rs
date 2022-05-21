use crate::*;

#[derive(Debug)]
pub struct BlockContext<'a> {
    pub global_data: &'a mut HashMap<String, Box<dyn Any + Send + Sync>>,
    pub stack: &'a mut Stack,
    pub stage_id: TargetId,
    pub running_stage_id: generational_arena::Index,
    pub target_id: TargetId,
    pub running_target_id: generational_arena::Index,
    pub targets: &'a mut Arena<Target>,
    pub running_targets: &'a mut generational_arena::Arena<RunningTarget>,
}

struct RefreshStackData(Option<BlockValue>, usize);

impl BlockContext<'_> {
    #[inline(always)]
    pub fn is_stage(&self) -> bool {
        self.running_stage_id == self.running_target_id
    }
    #[inline(always)]
    pub fn arg_len(&self) -> usize {
        self.stack.arguments.len()
    }
    #[inline(always)]
    pub fn stage(&self) -> &Target {
        &self.targets[self.stage_id]
    }
    #[inline(always)]
    pub fn stage_mut(&mut self) -> &mut Target {
        &mut self.targets[self.stage_id]
    }
    #[inline(always)]
    pub fn running_stage_mut(&mut self) -> &mut RunningTarget {
        &mut self.running_targets[self.running_stage_id]
    }
    #[inline(always)]
    pub fn running_stage(&self) -> &RunningTarget {
        &self.running_targets[self.running_stage_id]
    }
    #[inline(always)]
    pub fn target(&self) -> &Target {
        &self.targets[self.target_id]
    }
    #[inline(always)]
    pub fn target_mut(&mut self) -> &mut Target {
        &mut self.targets[self.target_id]
    }
    #[inline(always)]
    pub fn running_target(&self) -> &RunningTarget {
        &self.running_targets[self.running_target_id]
    }
    #[inline(always)]
    pub fn running_target_mut(&mut self) -> &mut RunningTarget {
        &mut self.running_targets[self.running_target_id]
    }
    #[inline(always)]
    pub fn get_block(&self) -> &Block {
        self.target().blocks.get(self.stack.block_id).unwrap()
    }
    #[inline(always)]
    pub fn get_block_mut(&mut self) -> &mut Block {
        let bid = self.stack.block_id;
        self.target_mut().blocks.get_mut(bid).unwrap()
    }
    #[inline(always)]
    pub fn arg(&self, index: usize) -> &BlockValue {
        &self.stack.arguments[index]
    }
    #[inline(always)]
    pub fn arg_proto(&self, index: usize) -> &BlockValue {
        &self.get_block().arguments[index]
    }
    #[inline(always)]
    pub fn get_list(&self, name: &str) -> &Vec<BlockValue> {
        if let Some(v) = self.running_target().lists.get(name) {
            v
        } else if let Some(v) = self.running_stage().lists.get(name) {
            v
        } else {
            static EMPTY_VEC: Vec<BlockValue> = vec![];
            &EMPTY_VEC
        }
    }
    #[inline(always)]
    pub fn get_list_mut(&mut self, name: &str) -> &mut Vec<BlockValue> {
        {
            if self.running_target_mut().lists.get_mut(name).is_some() {
                return self.running_target_mut().lists.get_mut(name).unwrap();
            }
        }
        {
            if self.running_stage_mut().lists.get_mut(name).is_some() {
                return self.running_stage_mut().lists.get_mut(name).unwrap();
            }
        }
        {
            self.running_target_mut()
                .lists
                .insert(name.to_owned(), vec![]);
            self.running_target_mut().lists.get_mut(name).unwrap()
        }
    }
    #[inline(always)]
    pub fn get_variable(&self, name: &str) -> &BlockValue {
        if let Some(v) = self.running_target().variables.get(name) {
            v
        } else if let Some(v) = self.running_stage().variables.get(name) {
            v
        } else {
            &BlockValue::Undefined
        }
    }
    #[inline(always)]
    pub fn get_variable_mut(&mut self, name: &str) -> &mut BlockValue {
        if self.running_target_mut().variables.get_mut(name).is_some() {
            self.running_target_mut().variables.get_mut(name).unwrap()
        } else if self.running_stage_mut().variables.get_mut(name).is_some() {
            self.running_stage_mut().variables.get_mut(name).unwrap()
        } else {
            self.running_target_mut()
                .variables
                .insert(name.to_owned(), BlockValue::Undefined);
            self.running_target_mut().variables.get_mut(name).unwrap()
        }
    }
    #[inline(always)]
    pub fn acquire_args(
        &mut self,
        args: usize,
        callback: impl FnOnce(&mut Self) -> BlockResult,
    ) -> BlockResult {
        let len = self.arg_len();
        if len >= args {
            callback(self)
        } else {
            BlockResult::ResolveArgument(len)
        }
    }

    #[inline(always)]
    pub fn is_stage_dirty(&self) -> bool {
        if let Some(stage_frame) = self
            .global_data
            .get("STAGE_FRAME")
            .and_then(|x| x.downcast_ref::<usize>())
        {
            if let Some(v) = self.stack.block_data.downcast_ref::<RefreshStackData>() {
                stage_frame <= &v.1
            } else {
                false
            }
        } else {
            false
        }
    }
    /// 标记等待舞台更新，配合 `is_stage_dirty` 使用将模块等待到舞台更新后继续执行
    #[inline(always)]
    pub fn acquire_need_wait_refresh(
        &mut self,
        callback: impl FnOnce(&mut Self) -> BlockResult,
    ) -> BlockResult {
        if let Some(result) = self.stack.block_data.downcast_ref::<RefreshStackData>() {
            if self.is_stage_dirty() {
                pending()
            } else {
                BlockResult::Resolved(result.0.to_owned())
            }
        } else {
            match callback(self) {
                BlockResult::Resolved(value) => {
                    if let Some(stage_frame) = self
                        .global_data
                        .get("STAGE_FRAME")
                        .and_then(|x| x.downcast_ref::<usize>())
                    {
                        self.stack.block_data = Box::new(RefreshStackData(value, *stage_frame));
                    } else {
                        self.stack.block_data = Box::new(RefreshStackData(value, 0));
                    }
                    pending()
                }
                other => other,
            }
        }
    }
    #[inline(always)]
    pub fn end(&self) -> BlockResult {
        BlockResult::Resolved(None)
    }
    #[inline(always)]
    pub fn ret(&self, v: impl Into<BlockValue>) -> BlockResult {
        BlockResult::Resolved(Some(v.into()))
    }
}
