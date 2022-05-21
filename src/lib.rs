use generational_arena::Index;
use id_arena::*;
use retain_mut::RetainMut;
use std::any::Any;
use std::collections::{BTreeMap, HashMap};

mod block_value;
mod test_projects;
pub mod uid;
pub mod util;
pub(crate) use util::*;
pub mod blocks;
pub use block_value::BlockValue;
mod block;
mod context;
mod target;
pub use block::*;
pub use context::*;
pub use target::*;
pub mod core_blocks;
pub mod sb3_loader;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub static mut THREAD_RND: once_cell::sync::Lazy<rand::rngs::SmallRng> =
    once_cell::sync::Lazy::new(|| {
        use rand::prelude::*;
        rand::rngs::SmallRng::from_entropy()
    });

#[derive(Debug)]
pub enum TargetType {
    AllScripts,
    ThisScript,
    OtherScriptsInSprite,
}

#[derive(Debug)]
pub enum BlockResult {
    Pending,
    /// Resolve a argument and push it into the arguments.
    ResolveArgument(usize),
    Resolved(Option<BlockValue>),
    PushStack(BlockId),
    // 一些特殊指令
    ResolveProcedureArgument(usize),
    ReturnProcedure(BlockValue),
    Boardcast(String),
    BoardcastAndWait(String),
    StopScript(TargetType),
    CreateClone(String),
    DeleteThisClone,
}

pub type BlockFunction = fn(&mut BlockContext) -> BlockResult;

pub struct Stack {
    pub uid: usize,
    pub block_id: BlockId,
    pub block_function: BlockFunction,
    pub arguments: Vec<BlockValue>,
    pub block_data: Box<dyn Any + Send + Sync>,
}

impl Stack {
    pub fn require_resolve(&self, argument_id: usize) -> bool {
        matches!(&self.arguments[argument_id], BlockValue::BlockId(_))
    }
}

impl std::fmt::Debug for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Stack")
            .field("uid", &self.uid)
            .field("block_id", &self.block_id)
            .field("arguments", &self.arguments)
            .field("block_data", &self.block_data)
            .finish()
    }
}

impl Clone for Stack {
    fn clone(&self) -> Self {
        Self {
            uid: crate::uid::uid(),
            block_id: self.block_id,
            block_function: self.block_function,
            arguments: Vec::with_capacity(self.arguments.len()),
            block_data: Box::new(()),
        }
    }
}
static THREAD_ID_COUNTER: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);

#[derive(Debug, Clone)]
pub struct Thread {
    pub running_target_id: generational_arena::Index,
    pub thread_id: usize,
    pub awaiting_thread: Vec<usize>,
    pub stacks: Vec<Stack>,
}

impl Thread {
    pub fn new(running_target_id: generational_arena::Index) -> Self {
        Self {
            running_target_id,
            thread_id: THREAD_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            awaiting_thread: Vec::with_capacity(16),
            stacks: Vec::with_capacity(16),
        }
    }

    pub fn new_with_stacks(
        running_target_id: generational_arena::Index,
        stacks: Vec<Stack>,
    ) -> Self {
        Self {
            running_target_id,
            thread_id: THREAD_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            awaiting_thread: Vec::with_capacity(16),
            stacks,
        }
    }
}

#[derive(Debug)]
pub struct VirtualMachine {
    pub stage_id: TargetId,
    pub running_stage_id: generational_arena::Index,
    pub global_data: HashMap<String, Box<dyn Any + Send + Sync>>,
    pub targets: Arena<Target>,
    pub running_targets: generational_arena::Arena<RunningTarget>,
    pub threads: Vec<Thread>,
    pub new_clones: Vec<Index>,
    threads_removed: Vec<usize>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl VirtualMachine {
    
    pub(crate) fn new_target(&mut self, target: Target) -> (TargetId, generational_arena::Index) {
        let tid = self.targets.alloc(target);
        let rtid = self
            .running_targets
            .insert(self.targets[tid].make_target(tid, false));
        (tid, rtid)
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = resyncStage))]
    pub fn resync_stage(&mut self) {
        let stage = &self.targets[self.stage_id];
        let mut running_stage = &mut self.running_targets[self.running_stage_id];
        running_stage.variables = stage.variables.to_owned();
        running_stage.lists = stage.lists.to_owned();
    }
    
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = markStageRefreshed))]
    pub fn mark_stage_refreshed(&mut self) {
        if let Some(sf) = self
            .global_data
            .get_mut("STAGE_FRAME")
            .and_then(|x| x.downcast_mut::<usize>())
        {
            *sf += 1;
        } else {
            self.global_data
                .insert("STAGE_FRAME".to_owned(), Box::new(0usize));
        }
    }
}

impl Default for VirtualMachine {
    fn default() -> Self {
        let mut targets = Arena::with_capacity(1);
        let mut running_targets = generational_arena::Arena::with_capacity(1);
        let stage_id = targets.alloc(Target {
            name: "Stage".into(),
            ..Default::default()
        });
        let running_stage_id =
            running_targets.insert(targets[stage_id].make_target(stage_id, false));
        let mut global_data: HashMap<String, Box<dyn Any + Send + Sync>> =
            HashMap::with_capacity(16);
        global_data.insert("COUNTER".into(), Box::new(0usize)); // 计数器相关的模块只增不减
        global_data.insert("GLOBAL_TIMER".into(), Box::new(std::time::Instant::now()));
        Self {
            stage_id,
            global_data,
            targets,
            running_targets,
            running_stage_id,
            threads: Vec::with_capacity(16),
            threads_removed: Vec::with_capacity(16),
            new_clones: Vec::with_capacity(16),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl VirtualMachine {
    pub(crate) fn start_block(&mut self, target_id: TargetId, block_id: BlockId) {
        if let Some(target) = self.targets.get(target_id) {
            if let Some(block) = target.blocks.get(block_id) {
                if let Some((rtid, _running_target)) = self
                    .running_targets
                    .iter()
                    .find(|a| a.1.target_id == target_id && !a.1.is_clone)
                {
                    self.threads.push(Thread::new_with_stacks(
                        rtid,
                        vec![Stack {
                            uid: crate::uid::uid(),
                            block_id,
                            block_function: block.block_function,
                            arguments: Vec::with_capacity(block.arguments.len()),
                            block_data: Box::new(()),
                        }],
                    ));
                }
            }
        }
    }
    pub(crate) fn start_opcode_of_running_target(
        &mut self,
        opcode: &str,
        toplevel: bool,
        running_target_id: generational_arena::Index,
    ) {
        if let Some(target) = self.running_targets.get(running_target_id) {
            for (bid, block) in self.targets[target.target_id].blocks.iter() {
                if (!toplevel || block.toplevel) && block.opcode == opcode {
                    self.threads.push(Thread::new_with_stacks(
                        running_target_id,
                        vec![Stack {
                            uid: crate::uid::uid(),
                            block_id: bid,
                            block_function: block.block_function,
                            arguments: Vec::new(),
                            block_data: Box::new(()),
                        }],
                    ));
                }
            }
        }
    }

    pub(crate) fn start_topblock_if(&mut self, opcode: &str, callback: impl Fn(&Block) -> bool) {
        for (rtid, running_target) in self.running_targets.iter() {
            for (bid, block) in self.targets[running_target.target_id].blocks.iter() {
                if block.toplevel && block.opcode == opcode && callback(block) {
                    self.threads.push(Thread::new_with_stacks(
                        rtid,
                        vec![Stack {
                            uid: crate::uid::uid(),
                            block_id: bid,
                            block_function: block.block_function,
                            arguments: Vec::new(),
                            block_data: Box::new(()),
                        }],
                    ));
                }
            }
        }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = startOpcode))]
    pub fn start_opcode(&mut self, opcode: &str) {
        self.threads.clear();
        for (tid, target) in self.running_targets.iter_mut() {
            for (bid, block) in self.targets[target.target_id].blocks.iter() {
                if block.toplevel && block.opcode == opcode {
                    self.threads.push(Thread::new_with_stacks(
                        tid,
                        vec![Stack {
                            uid: crate::uid::uid(),
                            block_id: bid,
                            block_function: block.block_function,
                            arguments: Vec::new(),
                            block_data: Box::new(()),
                        }],
                    ));
                }
            }
        }
    }
    /// Clear all clones and threads, and then fire the flag event
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = startFlag))]
    pub fn start_flag(&mut self) {
        self.running_targets.retain(|_, a| !a.is_clone);
        self.start_opcode("event_whenflagclicked")
    }
    
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = isIdle))]
    pub fn is_idle(&self) -> bool {
        self.threads.is_empty()
    }
    pub(crate) fn run_until_idle(&mut self, max_time: std::time::Duration) -> Option<std::time::Duration> {
        let time = std::time::Instant::now();
        while !self.is_idle() && time.elapsed() < max_time {
            self.global_data.remove("MIN_WAIT_TIME");
            self.global_data.remove("WAITING_THREADS");
            self.step();
            if let Some(waiting_threads) = self
                .global_data
                .get("WAITING_THREADS")
                .and_then(|x| x.downcast_ref::<usize>())
            {
                if *waiting_threads >= self.threads.len() {
                    if let Some(min_wait_time) = self
                        .global_data
                        .get("MIN_WAIT_TIME")
                        .and_then(|x| x.downcast_ref::<std::time::Duration>())
                    {
                        return Some(*min_wait_time);
                    }
                }
            }
        }
        None
    }
    
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = printThreadStacks))]
    pub fn print_thread_stacks(&self) {
        println!("VMThreadStacks");
        for thread in &self.threads {
            let rt = self.running_targets.get(thread.running_target_id).unwrap();
            let t = self.targets.get(rt.target_id).unwrap();
            println!(
                "Thread {}: {:?} {:?} {}",
                thread.thread_id, thread.running_target_id, rt.target_id, rt.name
            );
            if rt.is_clone {
                println!("  Sprite[Clone] {}:", rt.name);
            } else {
                println!("  Sprite {}:", rt.name);
            }
            println!("    Position: {}, {}", rt.x, rt.y);
            println!("    Direction: {}", rt.direction);
            println!("    CurrentCostume: {}", rt.current_costume);
            for (i, stack) in thread.stacks.iter().enumerate() {
                println!("  Stack {}: {}", i, stack.uid);
                let block = t.blocks.get(stack.block_id).unwrap();
                #[cfg(debug_assertions)]
                println!(
                    "    Block {} {}: {:?}",
                    block.block_id, block.opcode, stack.block_id
                );
                #[cfg(not(debug_assertions))]
                println!("    Block {}: {:?}", block.opcode, stack.block_id);
                for arg in &stack.arguments {
                    println!("      Arg: {:?}", arg);
                }
            }
        }
        println!("VMThreadStacks End");
    }
    pub fn step(&mut self) {
        let mut should_stop_everything = false;
        let mut clone_list: Vec<generational_arena::Index> = Vec::with_capacity(16);
        let mut boardcast_list: Vec<(usize, String)> = Vec::with_capacity(16);
        let mut boardcast_wait_list: Vec<(usize, String)> = Vec::with_capacity(16);
        let mut boardcast_finished_list: Vec<usize> = Vec::with_capacity(16);
        self.threads_removed.clear();
        // Clean waiting threads
        for thread in &self.threads {
            if thread.awaiting_thread.is_empty() {
                continue;
            }
            let mut boardcast_finished = true;
            for awaiting_thread_id in &thread.awaiting_thread {
                if self
                    .threads
                    .iter()
                    .any(|x| &x.thread_id == awaiting_thread_id)
                {
                    boardcast_finished = false;
                    break;
                }
            }
            if boardcast_finished {
                boardcast_finished_list.push(thread.thread_id)
            }
        }
        for thread in &mut self.threads {
            if boardcast_finished_list.contains(&thread.thread_id) {
                thread.awaiting_thread.clear();
            }
        }
        RetainMut::retain_mut(
            &mut self.threads,
            |Thread {
                 running_target_id: tid,
                 thread_id,
                 awaiting_thread,
                 stacks,
             }| {
                if should_stop_everything {
                    return false;
                }
                if !awaiting_thread.is_empty() {
                    return true;
                }
                let target_id = self.running_targets.get(*tid).unwrap().target_id;

                let only_one_stack = stacks.len() == 1;
                loop {
                    let block_result = if let Some(stack) = stacks.last_mut() {
                        let block_function = stack.block_function;
                        if !self.running_targets.contains(*tid) {
                            return false;
                        }

                        let block_result = {
                            let mut ctx = BlockContext {
                                stack,
                                global_data: &mut self.global_data,
                                targets: &mut self.targets,
                                running_targets: &mut self.running_targets,
                                target_id,
                                running_stage_id: self.running_stage_id,
                                running_target_id: *tid,
                                stage_id: self.stage_id,
                            };
                            (block_function)(&mut ctx)
                        };
                        if self.targets.get(target_id).is_some() {
                            Some(block_result)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    if let Some(block_result) = block_result {
                        {
                            if let Some(target) = self.targets.get_mut(target_id) {
                                match block_result {
                                    BlockResult::Pending => return true,
                                    BlockResult::ResolveArgument(index) => {
                                        if let Some(stack) = stacks.last_mut() {
                                            if let Some(block) = target.blocks.get(stack.block_id) {
                                                if let Some(arg) = block.arguments.get(index) {
                                                    match arg {
                                                        BlockValue::BlockId(block_id) => {
                                                            if let Some(block) =
                                                                target.blocks.get(*block_id)
                                                            {
                                                                stacks.push(Stack {
                                                                    uid: crate::uid::uid(),
                                                                    block_id: *block_id,
                                                                    block_function: block
                                                                        .block_function,
                                                                    arguments: vec![],
                                                                    block_data: Box::new(()),
                                                                });
                                                                return true;
                                                            } else {
                                                                stack
                                                                    .arguments
                                                                    .push(BlockValue::Undefined);
                                                            }
                                                        }
                                                        other => {
                                                            stack.arguments.push(other.to_owned());
                                                        }
                                                    }
                                                } else {
                                                    stack.arguments.push(BlockValue::Undefined);
                                                }
                                            } else {
                                                stack.arguments.push(BlockValue::Undefined);
                                            }
                                        }
                                    }
                                    BlockResult::PushStack(bid) => {
                                        let block = target.blocks.get(bid).unwrap();
                                        stacks.push(Stack {
                                            uid: crate::uid::uid(),
                                            block_id: bid,
                                            block_function: block.block_function,
                                            arguments: Vec::with_capacity(block.arguments.len()),
                                            block_data: Box::new(()),
                                        });
                                        return true;
                                    }
                                    BlockResult::Resolved(result) => {
                                        if let Some(stack) = stacks.last_mut() {
                                            let block = target.blocks.get(stack.block_id).unwrap();
                                            if let Some(next) = block.next {
                                                // Drop the result and keep running next block
                                                let block = target.blocks.get(next).unwrap();
                                                stack.block_id = next;
                                                stack.block_data = Box::new(());
                                                stack.arguments.clear();
                                                stack.block_function = block.block_function;
                                                return true;
                                            } else {
                                                // Pop stack and push the result to the previous stack arguments
                                                if only_one_stack {
                                                    return false;
                                                } else {
                                                    stacks.pop();
                                                    let prev_stack = stacks.last_mut().unwrap();
                                                    prev_stack
                                                        .arguments
                                                        .push(result.unwrap_or_default());
                                                    return true;
                                                }
                                            }
                                        } else {
                                            return false;
                                        }
                                    }
                                    BlockResult::StopScript(stop_type) => match stop_type {
                                        TargetType::AllScripts => {
                                            should_stop_everything = true;
                                            return false;
                                        }
                                        TargetType::ThisScript => {
                                            self.threads_removed.push(*thread_id);
                                            return false;
                                        }
                                        TargetType::OtherScriptsInSprite => {
                                            // TODO
                                            return true;
                                        }
                                    },
                                    BlockResult::CreateClone(clone_option) => {
                                        if let Some(stack) = stacks.last_mut() {
                                            match clone_option.as_str() {
                                                "_myself_" => {
                                                    clone_list.push(*tid);
                                                }
                                                other => {
                                                    if let Some(tid) =
                                                        self.running_targets.iter().find(|a| {
                                                            !a.1.is_clone
                                                                && a.1.name.as_str() == other
                                                        })
                                                    {
                                                        clone_list.push(tid.0);
                                                    }
                                                }
                                            }
                                            // Same as Resolved
                                            let block = target.blocks.get(stack.block_id).unwrap();
                                            if let Some(next) = block.next {
                                                // Drop the result and keep running next block
                                                let block = target.blocks.get(next).unwrap();
                                                stack.block_id = next;
                                                stack.block_data = Box::new(());
                                                stack.arguments.clear();
                                                stack.block_function = block.block_function;
                                                return true;
                                            } else {
                                                // Pop stack and push the result to the previous stack arguments
                                                if only_one_stack {
                                                    return false;
                                                } else {
                                                    stacks.pop();
                                                    let prev_stack = stacks.last_mut().unwrap();
                                                    prev_stack
                                                        .arguments
                                                        .push(BlockValue::Undefined);
                                                    return true;
                                                }
                                            }
                                        }
                                    }
                                    BlockResult::DeleteThisClone => {
                                        if self.running_targets[*tid].is_clone {
                                            self.running_targets.remove(*tid);
                                        }
                                        return false;
                                    }
                                    BlockResult::ResolveProcedureArgument(i) => {
                                        let arg = if let Some(s) =
                                            stacks.iter().rev().find(|s| {
                                                self.targets[target_id].blocks[s.block_id].opcode
                                                    == "procedures_call"
                                                    && (
                                                        s.block_data.is::<core_blocks::ProcedureArguments>() ||
                                                        s.block_data.is::<core_blocks::ReturnProcedureArguments>()
                                                    )
                                            }) {
                                            if let Some(v) = s
                                                .block_data
                                                .downcast_ref::<core_blocks::ProcedureArguments>()
                                                .and_then(|x| x.0.get(i))
                                            {
                                                match v {
                                                    BlockValue::BlockId(_) => {
                                                        BlockValue::Number(0.0)
                                                    }
                                                    v => v.to_owned(),
                                                }
                                            } else {
                                                BlockValue::Number(0.0)
                                            }
                                        } else {
                                            BlockValue::Number(0.0)
                                        };
                                        if let Some(stack) = stacks.last_mut() {
                                            stack.arguments.push(arg);
                                        } else {
                                            return false;
                                        }
                                    }
                                    BlockResult::Boardcast(name) => {
                                        boardcast_list.push((*thread_id, name));
                                        if let Some(stack) = stacks.last_mut() {
                                            let block = target.blocks.get(stack.block_id).unwrap();
                                            if let Some(next) = block.next {
                                                // Drop the result and keep running next block
                                                let block = target.blocks.get(next).unwrap();
                                                stack.block_id = next;
                                                stack.block_data = Box::new(());
                                                stack.arguments.clear();
                                                stack.block_function = block.block_function;
                                                return true;
                                            } else {
                                                // Pop stack and push the result to the previous stack arguments
                                                if only_one_stack {
                                                    return false;
                                                } else {
                                                    stacks.pop();
                                                    let prev_stack = stacks.last_mut().unwrap();
                                                    prev_stack
                                                        .arguments
                                                        .push(BlockValue::Undefined);
                                                    return true;
                                                }
                                            }
                                        } else {
                                            return false;
                                        }
                                    }
                                    BlockResult::BoardcastAndWait(name) => {
                                        boardcast_wait_list.push((*thread_id, name));
                                        if let Some(stack) = stacks.last_mut() {
                                            let block = target.blocks.get(stack.block_id).unwrap();
                                            if let Some(next) = block.next {
                                                // Drop the result and keep running next block
                                                let block = target.blocks.get(next).unwrap();
                                                stack.block_id = next;
                                                stack.block_data = Box::new(());
                                                stack.arguments.clear();
                                                stack.block_function = block.block_function;
                                                return true;
                                            } else {
                                                // Pop stack and push the result to the previous stack arguments
                                                if only_one_stack {
                                                    return false;
                                                } else {
                                                    stacks.pop();
                                                    let prev_stack = stacks.last_mut().unwrap();
                                                    prev_stack
                                                        .arguments
                                                        .push(BlockValue::Undefined);
                                                    return true;
                                                }
                                            }
                                        } else {
                                            return false;
                                        }
                                    }
                                    BlockResult::ReturnProcedure(_result) => {
                                        let arg = if let Some(s) =
                                            stacks.iter().rev().find(|s| {
                                                self.targets[target_id].blocks[s.block_id].opcode
                                                    == "procedures_call"
                                                    && (
                                                        s.block_data.is::<core_blocks::ProcedureArguments>() ||
                                                        s.block_data.is::<core_blocks::ReturnProcedureArguments>()
                                                    )
                                            }) {
                                            if let Some(v) = s
                                                .block_data
                                                .downcast_ref::<core_blocks::ReturnProcedureArguments>()
                                                .and_then(|x| x.0.get(0))
                                            {
                                                match v {
                                                    BlockValue::BlockId(_) => {
                                                        BlockValue::Number(0.0)
                                                    }
                                                    v => v.to_owned(),
                                                }
                                            } else {
                                                BlockValue::Number(0.0)
                                            }
                                        } else {
                                            BlockValue::Number(0.0)
                                        };
                                        if let Some(stack) = stacks.last_mut() {
                                            stack.arguments.push(arg);
                                        } else {
                                            return false;
                                        }
                                    }
                                }
                            } else {
                                return false;
                            }
                        }
                    } else {
                        return false;
                    }
                }
            },
        );
        if should_stop_everything {
            self.threads.clear();
        } else if !self.threads_removed.is_empty() {
            for i in self.threads_removed.drain(..) {
                for (x, t) in self.threads.iter().enumerate() {
                    if t.thread_id == i {
                        self.threads.swap_remove(x);
                        break;
                    }
                }
            }
        }
        for (_tid, name) in boardcast_list {
            self.start_topblock_if("event_whenbroadcastreceived", |b| {
                b.arguments
                    .first()
                    .and_then(|x| {
                        if x.to_string().as_str() == name {
                            Some(())
                        } else {
                            None
                        }
                    })
                    .is_some()
            })
        }
        for (tid, name) in boardcast_wait_list {
            let before_len = self.threads.len();
            self.start_topblock_if("event_whenbroadcastreceived", |b| {
                b.arguments
                    .first()
                    .and_then(|x| {
                        if x.to_string().as_str() == name {
                            Some(())
                        } else {
                            None
                        }
                    })
                    .is_some()
            });
            let current_len = self.threads.len();
            let awaiting_thread = (before_len..current_len)
                .map(|x| self.threads[x].thread_id)
                .collect();
            if let Some(thread) = self.threads.iter_mut().find(|t| t.thread_id == tid) {
                thread.awaiting_thread = awaiting_thread;
            }
        }
        for rtid in clone_list.iter() {
            if let Some(rt) = self.running_targets.get(*rtid) {
                let mut new_clone = rt.clone();
                new_clone.is_clone = true;
                new_clone.layer_order = self.running_targets.len();
                let new_rtid = self.running_targets.insert(new_clone);
                self.new_clones.push(new_rtid);
                self.start_opcode_of_running_target("control_start_as_clone", true, new_rtid);
            }
        }
    }
}
