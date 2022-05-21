use crate::*;
use std::time::{Duration, Instant};

pub fn control_wait(ctx: &mut BlockContext) -> BlockResult {
    if let Some((i, d)) = ctx.stack.block_data.downcast_ref::<(Instant, Duration)>() {
        let e = i.elapsed();
        if e >= *d {
            BlockResult::Resolved(None)
        } else {
            let wait_time = *d - e;
            if let Some(wt) = ctx
                .global_data
                .get_mut("MIN_WAIT_TIME")
                .and_then(|x| x.downcast_mut::<Duration>())
            {
                if *wt > wait_time {
                    *wt = wait_time;
                }
            } else {
                ctx.global_data
                    .insert("MIN_WAIT_TIME".into(), Box::new(wait_time));
            }
            if let Some(wt) = ctx
                .global_data
                .get_mut("WAITING_THREADS")
                .and_then(|x| x.downcast_mut::<usize>())
            {
                *wt += 1;
            } else {
                ctx.global_data
                    .insert("WAITING_THREADS".into(), Box::new(1usize));
            }

            BlockResult::Pending
        }
    } else if let Some(arg_time) = ctx.stack.arguments.get(0) {
        let arg_time = arg_time.to_number();
        if arg_time <= 0. || arg_time.is_nan() {
            BlockResult::Resolved(None)
        } else {
            ctx.stack.block_data = Box::new((
                Instant::now(),
                Duration::from_millis((arg_time * 1000.) as u64),
            ));
            BlockResult::Pending
        }
    } else {
        BlockResult::ResolveArgument(0)
    }
}

pub fn control_repeat(ctx: &mut BlockContext) -> BlockResult {
    type BlockData = (usize, BlockId);
    if let Some((times, substack)) = ctx.stack.block_data.downcast_mut::<BlockData>() {
        ctx.stack.arguments.clear();
        if *times > 0 {
            *times -= 1;
            BlockResult::PushStack(*substack)
        } else {
            BlockResult::Resolved(None)
        }
    } else if let Some(times) = ctx.stack.arguments.get(0) {
        let times = times.to_number() as usize;
        if times > 0 {
            let substack: Option<BlockId> =
                if let Some(BlockValue::BlockId(bid)) = ctx.get_block().arguments.get(1) {
                    Some(*bid)
                } else {
                    None
                };
            if let Some(bid) = substack {
                ctx.stack.block_data = Box::new((times - 1, bid));
                BlockResult::PushStack(bid)
            } else {
                BlockResult::Resolved(None)
            }
        } else {
            BlockResult::Resolved(None)
        }
    } else {
        BlockResult::ResolveArgument(0)
    }
}

pub fn control_if_else(ctx: &mut BlockContext) -> BlockResult {
    match ctx.arg_len() {
        0 => BlockResult::ResolveArgument(ctx.arg_len()),
        1 => {
            let condition = &ctx.stack.arguments[0];
            if condition.to_boolean() {
                BlockResult::ResolveArgument(1)
            } else {
                BlockResult::ResolveArgument(2)
            }
        }
        _ => {
            BlockResult::Resolved(None)
        }
    }
}

pub fn control_if(ctx: &mut BlockContext) -> BlockResult {
    match ctx.arg_len() {
        0 => BlockResult::ResolveArgument(ctx.arg_len()),
        1 => {
            let condition = &ctx.stack.arguments[0];
            if condition.to_boolean() {
                BlockResult::ResolveArgument(1)
            } else {
                BlockResult::Resolved(None)
            }
        }
        _ => {
            BlockResult::Resolved(None)
        }
    }
}

pub fn control_forever(ctx: &mut BlockContext) -> BlockResult {
    ctx.stack.arguments.clear(); // 因为请求完参数之后可能有返回值被推入
    BlockResult::ResolveArgument(0)
}

pub fn control_all_at_once(ctx: &mut BlockContext) -> BlockResult {
    if ctx.arg_len() == 0 {
        BlockResult::ResolveArgument(0)
    } else {
        BlockResult::Resolved(None)
    }
}

pub fn control_create_clone_of_menu(ctx: &mut BlockContext) -> BlockResult {
    if ctx.arg_len() == 0 {
        BlockResult::ResolveArgument(0)
    } else {
        BlockResult::Resolved(Some(ctx.arg(0).to_owned()))
    }
}
pub fn control_get_counter(ctx: &mut BlockContext) -> BlockResult {
    if let Some(i) = ctx.global_data.get("COUNTER") {
        if let Some(i) = i.downcast_ref::<usize>() {
            BlockResult::Resolved(Some((*i).into()))
        } else {
            BlockResult::Resolved(Some(0.into()))
        }
    } else {
        BlockResult::Resolved(Some(0.into()))
    }
}

pub fn control_incr_counter(ctx: &mut BlockContext) -> BlockResult {
    if let Some(i) = ctx
        .global_data
        .get_mut("COUNTER")
        .and_then(|a| a.downcast_mut::<usize>())
    {
        *i += 1;
    } else {
        ctx.global_data.insert("COUNTER".into(), Box::new(1usize));
    }
    BlockResult::Resolved(None)
}

pub fn control_clear_counter(ctx: &mut BlockContext) -> BlockResult {
    ctx.global_data.insert("COUNTER".into(), Box::new(0usize));
    BlockResult::Resolved(None)
}
pub fn control_delete_this_clone(_ctx: &mut BlockContext) -> BlockResult {
    BlockResult::DeleteThisClone
}

pub fn control_start_as_clone(_ctx: &mut BlockContext) -> BlockResult {
    BlockResult::Resolved(None)
}

pub fn control_create_clone_of(ctx: &mut BlockContext) -> BlockResult {
    let arg_len = ctx.arg_len();
    if arg_len < 1 {
        return BlockResult::ResolveArgument(arg_len);
    }
    let clone_option = ctx.arg(0).to_string();
    BlockResult::CreateClone(clone_option)
}

pub fn control_repeat_until(ctx: &mut BlockContext) -> BlockResult {
    let arg_len = ctx.arg_len();
    if arg_len < 1 {
        return BlockResult::ResolveArgument(arg_len);
    } else if arg_len > 1 {
        ctx.stack.arguments.clear();
        return BlockResult::ResolveArgument(0);
    }
    let condition = ctx.arg(0);
    if !condition.to_boolean() {
        BlockResult::ResolveArgument(1)
    } else {
        BlockResult::Resolved(None)
    }
}

pub fn control_while(ctx: &mut BlockContext) -> BlockResult {
    let arg_len = ctx.arg_len();
    if arg_len < 1 {
        return BlockResult::ResolveArgument(arg_len);
    } else if arg_len > 1 {
        ctx.stack.arguments.clear();
        return BlockResult::ResolveArgument(0);
    }
    let condition = ctx.arg(0);
    if condition.to_boolean() {
        BlockResult::ResolveArgument(1)
    } else {
        BlockResult::Resolved(None)
    }
}

pub fn control_for_each(ctx: &mut BlockContext) -> BlockResult {
    type BlockData = (String, bool, usize, usize);
    if let Some((_, _, index, value)) = ctx.stack.block_data.downcast_mut::<BlockData>() {
        ctx.stack.arguments.clear();
        if index < value {
            *index += 1;
        } else {
            return BlockResult::Resolved(None);
        }
    } else {
        let arg_len = ctx.arg_len();
        if arg_len < 2 {
            return BlockResult::ResolveArgument(arg_len);
        }
        let name = ctx.arg(0).to_string();
        let is_stage = !ctx.running_stage().variables.contains_key(&name)
            && ctx.running_stage().variables.contains_key(&name);
        let value = ctx.arg(1).to_number() as usize;
        if value > 0 {
            if is_stage {
                if let Some(v) = ctx.running_stage_mut().variables.get_mut(&name) {
                    *v = 1.0.into();
                } else {
                    ctx.running_stage_mut()
                        .variables
                        .insert(name.to_owned(), 1.0.into());
                }
            } else if let Some(v) = ctx.running_target_mut().variables.get_mut(&name) {
                *v = 1.0.into();
            } else {
                ctx.running_target_mut()
                    .variables
                    .insert(name.to_owned(), 1.0.into());
            }
            ctx.stack.block_data = Box::new((name, is_stage, 1usize, value));
            return BlockResult::ResolveArgument(2);
        } else {
            return BlockResult::Resolved(None);
        }
    }
    if let Some((name, is_stage, index, _)) =
        ctx.stack.block_data.downcast_ref::<BlockData>().cloned()
    {
        if is_stage {
            ctx.running_stage_mut().variables.insert(name, index.into());
        } else {
            ctx.running_target_mut()
                .variables
                .insert(name, index.into());
        }
    }
    BlockResult::ResolveArgument(2)
}

pub fn control_wait_until(ctx: &mut BlockContext) -> BlockResult {
    if let Some(cond) = ctx.stack.arguments.get(0) {
        if cond.to_boolean() {
            BlockResult::Resolved(None)
        } else {
            ctx.stack.arguments.clear();
            BlockResult::Pending
        }
    } else {
        BlockResult::ResolveArgument(0)
    }
}

pub fn control_stop(ctx: &mut BlockContext) -> BlockResult {
    let arg_len = ctx.arg_len();
    if arg_len < 1 {
        return BlockResult::ResolveArgument(arg_len);
    }
    match ctx.stack.arguments[0].to_string().as_str() {
        "all" => BlockResult::StopScript(TargetType::AllScripts),
        "other scripts in sprite" | "other scripts in stage" => {
            BlockResult::StopScript(TargetType::OtherScriptsInSprite)
        }
        "this script" => BlockResult::StopScript(TargetType::ThisScript),
        _ => BlockResult::Resolved(None),
    }
}
