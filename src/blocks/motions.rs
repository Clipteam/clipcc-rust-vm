use std::time::Instant;



use crate::*;

pub fn motion_movesteps(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_need_wait_refresh(|ctx| {
        ctx.acquire_args(1, |ctx| {
            let steps = ctx.arg(0).to_number();
            let direction = 90. - ctx.running_target().direction;
            let direction = direction.to_radians();
            let dx = steps * direction.cos();
            let dy = steps * direction.sin();
            ctx.running_target_mut().x += dx;
            ctx.running_target_mut().y += dy;
            end()
        })
    })
}

fn wrap_clamp(n: f64, min: f64, max: f64) -> f64 {
    let range = (max - min) + 1.;
    n - ((n - min) / range).floor() * range
}

pub fn motion_turnright(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_need_wait_refresh(|ctx| {
        ctx.acquire_args(1, |ctx| {
            let degrees = ctx.arg(0).to_number();
            let direction = ctx.running_target().direction;
            ctx.running_target_mut().direction = wrap_clamp(direction + degrees, -179., 180.);
            end()
        })
    })
}
pub fn motion_turnleft(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_need_wait_refresh(|ctx| {
        ctx.acquire_args(1, |ctx| {
            let degrees = ctx.arg(0).to_number();
            let direction = ctx.running_target().direction;
            ctx.running_target_mut().direction = wrap_clamp(direction - degrees, -179., 180.);
            end()
        })
    })
}
pub fn motion_pointindirection(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(1, |ctx| {
        let degrees = ctx.arg(0).to_number();
        ctx.running_target_mut().direction = wrap_clamp(degrees, -179., 180.);
        end()
    })
}

pub fn motion_pointtowards_menu(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(1, |ctx| ret(ctx.arg(0).to_owned()))
}

pub fn motion_pointtowards(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(1, |ctx| {
        let towards = ctx.arg(0).to_string();
        match towards.as_str() {
            "_mouse_" => {}
            "_random_" => {}
            _target_name => {}
        }
        end()
    })
}
pub fn motion_gotoxy(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_need_wait_refresh(|ctx| {
        ctx.acquire_args(2, |ctx| {
            let x = ctx.arg(0).to_number();
            let y = ctx.arg(1).to_number();
            ctx.running_target_mut().x = x;
            ctx.running_target_mut().y = y;
            end()
        })
    })
}

pub fn motion_goto(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_need_wait_refresh(|ctx| {
        ctx.acquire_args(1, |ctx| {
            let target_name = ctx.arg(0).to_string();
            if let Some((x, y)) = get_target_xy(ctx, target_name.as_str()) {
                ctx.running_target_mut().x = x;
                ctx.running_target_mut().y = y;
            }
            end()
        })
    })
}

pub fn motion_goto_menu(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(1, |ctx| ret(ctx.arg(0).to_owned()))
}
type GlideData = (Instant, f64, f64, f64, f64, f64);
pub fn motion_glidesecstoxy(ctx: &mut BlockContext) -> BlockResult {
    if ctx.stack.block_data.is::<GlideData>() {
        let (i, time, x0, y0, x1, y1) = ctx
            .stack
            .block_data
            .downcast_ref::<GlideData>()
            .cloned()
            .unwrap();
        let d = i.elapsed();
        let mut rt = ctx.running_target_mut();
        if d.as_secs_f64() >= time {
            rt.x = x1;
            rt.y = y1;
            end()
        } else {
            let t = d.as_secs_f64() / time;
            rt.x = x0 + (x1 - x0) * t;
            rt.y = y0 + (y1 - y0) * t;
            pending()
        }
    } else {
        ctx.acquire_args(3, |ctx| {
            let secs = ctx.arg(0).to_number();
            let x1 = ctx.arg(1).to_number();
            let y1 = ctx.arg(2).to_number();
            let x0 = ctx.running_target().x;
            let y0 = ctx.running_target().y;
            let i = Instant::now();
            ctx.stack.block_data = Box::new((i, secs, x0, y0, x1, y1));
            pending()
        })
    }
}
pub fn motion_glideto(ctx: &mut BlockContext) -> BlockResult {
    if ctx.stack.block_data.is::<GlideData>() {
        let (i, time, x0, y0, x1, y1) = ctx
            .stack
            .block_data
            .downcast_ref::<GlideData>()
            .cloned()
            .unwrap();
        let d = i.elapsed();
        let mut rt = ctx.running_target_mut();
        if d.as_secs_f64() >= time {
            rt.x = x1;
            rt.y = y1;
            end()
        } else {
            let t = d.as_secs_f64() / time;
            rt.x = x0 + (x1 - x0) * t;
            rt.y = y0 + (y1 - y0) * t;
            pending()
        }
    } else {
        ctx.acquire_args(2, |ctx| {
            let secs = ctx.arg(0).to_number();
            let to = ctx.arg(1).to_string();
            if let Some((x1, y1)) = get_target_xy(ctx, to.as_str()) {
                let x0 = ctx.running_target().x;
                let y0 = ctx.running_target().y;
                let i = Instant::now();
                ctx.stack.block_data = Box::new((i, secs, x0, y0, x1, y1));
                pending()
            } else {
                end()
            }
        })
    }
}
pub fn motion_glideto_menu(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(1, |ctx| ret(ctx.arg(0).to_owned()))
}

pub fn motion_changexby(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_need_wait_refresh(|ctx| {
        ctx.acquire_args(1, |x| {
            let dx = x.arg(0).to_number();
            x.target_mut().x += dx;
            end()
        })
    })
}
pub fn motion_setx(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_need_wait_refresh(|ctx| {
        ctx.acquire_args(1, |ctx| {
            let x = ctx.arg(0).to_number();
            ctx.running_target_mut().x = x;
            end()
        })
    })
}
pub fn motion_changeyby(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_need_wait_refresh(|ctx| {
        ctx.acquire_args(1, |ctx| {
            let dy = ctx.arg(0).to_number();
            ctx.running_target_mut().y += dy;
            end()
        })
    })
}
pub fn motion_sety(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_need_wait_refresh(|ctx| {
        ctx.acquire_args(1, |ctx| {
            let y = ctx.arg(0).to_number();
            ctx.running_target_mut().y = y;
            end()
        })
    })
}
pub fn motion_ifonedgebounce(_ctx: &mut BlockContext) -> BlockResult {
    end()
}
pub fn motion_setrotationstyle(_ctx: &mut BlockContext) -> BlockResult {
    end()
}
pub fn motion_xposition(ctx: &mut BlockContext) -> BlockResult {
    ret(ctx.running_target().x)
}
pub fn motion_yposition(ctx: &mut BlockContext) -> BlockResult {
    ret(ctx.running_target().y)
}
pub fn motion_direction(ctx: &mut BlockContext) -> BlockResult {
    ret(ctx.running_target().direction)
}
