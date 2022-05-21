use rand::Rng;

use crate::*;

#[inline(always)]
pub fn pending() -> BlockResult {
    BlockResult::Pending
}

#[inline(always)]
pub fn end() -> BlockResult {
    BlockResult::Resolved(None)
}

#[inline(always)]
pub fn ret(v: impl Into<BlockValue>) -> BlockResult {
    BlockResult::Resolved(Some(v.into()))
}

pub fn get_target_xy(ctx: &BlockContext, target_name: &str) -> Option<(f64, f64)> {
    match target_name {
        "_mouse_" => {
            // TODO: get mouse position
            Some((0., 0.))
        }
        "_random_" => {
            let x = unsafe { THREAD_RND.gen_range(-0.5f64..0.5) };
            let y = unsafe { THREAD_RND.gen_range(-0.5f64..0.5) };
            let x = (x * 480.).round();
            let y = (y * 360.).round();
            Some((x, y))
        }
        target_name => {
            for (_, rt) in ctx.running_targets.iter() {
                if !rt.is_clone && rt.name == target_name {
                    return Some((rt.x, rt.y));
                }
            }
            None
        }
    }
}

pub(crate) fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}