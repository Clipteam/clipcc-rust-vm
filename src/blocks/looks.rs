use std::time::{Duration, Instant};

use generational_arena::Index;

use crate::*;
// looks_say
pub fn looks_say(ctx: &mut BlockContext) -> BlockResult {
    let text = ctx.stack.arguments.get(0);
    if let Some(text) = text {
        println!("{}", text);
        BlockResult::Resolved(None)
    } else {
        BlockResult::ResolveArgument(0)
    }
}

pub fn looks_sayforsecs(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_need_wait_refresh(|ctx| {
        if let Some((i, t)) = ctx.stack.block_data.downcast_ref::<(Instant, f64)>() {
            return if i.elapsed().as_secs_f64() >= *t {
                BlockResult::Resolved(None)
            } else {
                let wait_time = Duration::from_secs_f64(*t) - i.elapsed();
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
            };
        }
        ctx.acquire_args(2, |ctx| {
            let text = ctx.arg(0);
            let arg_time = ctx.arg(1).to_number();
            println!("{}", text);
            if arg_time <= 0. || arg_time.is_nan() {
                BlockResult::Resolved(None)
            } else {
                ctx.stack.block_data = Box::new((Instant::now(), arg_time));
                BlockResult::Pending
            }
        })
    })
}

pub fn looks_show(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_need_wait_refresh(|ctx| {
        ctx.running_target_mut().visible = true;
        // println!("Show {}", ctx.running_target().name);
        end()
    })
}

pub fn looks_hide(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_need_wait_refresh(|ctx| {
        ctx.running_target_mut().visible = false;
        // println!("Hide {}", ctx.running_target().name);
        end()
    })
}

pub fn looks_setsizeto(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_need_wait_refresh(|ctx| {
        ctx.acquire_args(1, |ctx| {
            let size = ctx.arg(0).to_number();
            ctx.running_target_mut().size = size.max(0.);
            // println!("Set size to {} -> {}", size, ctx.running_target().size);
            end()
        })
    })
}

pub fn looks_costume(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(1, |ctx| ret(ctx.arg(0).to_owned()))
}

pub fn looks_changesizeby(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_need_wait_refresh(|ctx| {
        ctx.acquire_args(1, |ctx| {
            let current_size = ctx.running_target_mut().size;
            let size = ctx.arg(0).to_number();
            ctx.running_target_mut().size = (size + current_size).max(0.);
            // println!("Change size by {} -> {}", size, ctx.running_target().size);
            end()
        })
    })
}

pub fn looks_switchbackdropto(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_need_wait_refresh(|ctx| {
        ctx.acquire_args(1, |ctx| {
            let costume_len = ctx.stage().costumes.len();
            match ctx.arg(0) {
                BlockValue::String(costume_name) => {
                    if let Some(index) = ctx
                        .stage()
                        .costumes
                        .iter()
                        .position(|x| x.as_str() == costume_name.as_str())
                    {
                        ctx.running_stage_mut().current_costume = index;
                    }
                }
                BlockValue::Number(costume_index) => {
                    let index = costume_index.max(0.) as usize;
                    ctx.running_stage_mut().current_costume = index.min(costume_len - 1);
                }
                _ => {}
            }
            end()
        })
    })
}

pub fn looks_backdrops(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(1, |ctx| ret(ctx.arg(0).to_owned()))
}

pub fn looks_switchcostumeto(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_need_wait_refresh(|ctx| {
        ctx.acquire_args(1, |ctx| {
            let costume_len = ctx.target().costumes.len();
            match ctx.arg(0) {
                BlockValue::String(costume_name) => {
                    if let Some(index) = ctx
                        .target()
                        .costumes
                        .iter()
                        .position(|x| x.as_str() == costume_name.as_str())
                    {
                        ctx.running_target_mut().current_costume = index;
                    }
                }
                BlockValue::Number(costume_index) => {
                    let index = costume_index.max(0.) as usize;
                    ctx.running_target_mut().current_costume = index.min(costume_len - 1);
                }
                _ => {}
            }
            end()
        })
    })
}

pub fn looks_nextcostume(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_need_wait_refresh(|ctx| {
        ctx.running_target_mut().current_costume =
            (ctx.running_target().current_costume + 1) % ctx.target().costumes.len();
        end()
    })
}

fn move_running_targets(ctx: &mut BlockContext, rtid: Index, delta: isize) {
    let mut running_targets = ctx
        .running_targets
        .iter()
        .filter(|x| x.1.target_id != ctx.stage_id)
        .map(|x| x.0)
        .collect::<Vec<_>>();
    running_targets.sort_by(|a, b| {
        let a_rt = &ctx.running_targets[*a];
        let b_rt = &ctx.running_targets[*b];
        b_rt.layer_order.cmp(&a_rt.layer_order)
    });
    let old_index = running_targets.iter().position(|x| x == &rtid).unwrap();
    running_targets.remove(old_index);
    let new_index = old_index as isize + delta;
    let new_index = if new_index > running_targets.len() as isize {
        running_targets.len() as isize
    } else {
        new_index
    };
    let new_index = if new_index <= 0 {
        0usize
    } else {
        new_index as usize
    };
    running_targets.insert(new_index, rtid);
    for (i, rtid) in running_targets.into_iter().enumerate() {
        ctx.running_targets[rtid].layer_order = i + 1;
    }
}

pub fn looks_gotofrontback(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_need_wait_refresh(|ctx| {
        ctx.acquire_args(1, |ctx| {
            let front_back = ctx.arg(0).to_string();
            if front_back.as_str() == "front" {
                move_running_targets(
                    ctx,
                    ctx.running_target_id,
                    ctx.running_targets.len() as isize,
                );
            } else {
                move_running_targets(
                    ctx,
                    ctx.running_target_id,
                    -(ctx.running_targets.len() as isize),
                );
            }
            end()
        })
    })
}
pub fn looks_goforwardbackwardlayers(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_need_wait_refresh(|ctx| {
        ctx.acquire_args(2, |ctx| {
            let forward_backward = ctx.arg(0).to_string();
            let offset = ctx.arg(1).to_number();
            if forward_backward.as_str() == "forward" {
                move_running_targets(ctx, ctx.running_target_id, offset as isize);
            } else {
                move_running_targets(ctx, ctx.running_target_id, -offset as isize);
            }
            end()
        })
    })
}
pub fn looks_size(ctx: &mut BlockContext) -> BlockResult {
    ret(ctx.running_target().size)
}

pub fn looks_costumenumbername(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(1, |ctx| {
        let number_name = ctx.arg(0).to_string();
        if number_name.as_str() == "number" {
            ret(ctx.running_target().current_costume)
        } else {
            ret(ctx.target().costumes[ctx.running_target().current_costume].to_owned())
        }
    })
}

pub fn looks_backdropnumbername(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(1, |ctx| {
        let number_name = ctx.arg(0).to_string();
        if number_name.as_str() == "number" {
            ret(ctx.running_stage().current_costume)
        } else {
            ret(ctx.stage().costumes[ctx.running_stage().current_costume].to_owned())
        }
    })
}
