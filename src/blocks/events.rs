use crate::*;

pub fn event_whenflagclicked(_ctx: &mut BlockContext) -> BlockResult {
    end()
}

pub fn event_whenkeypressed(_ctx: &mut BlockContext) -> BlockResult {
    end()
}

pub fn event_whenthisspriteclicked(_ctx: &mut BlockContext) -> BlockResult {
    end()
}

pub fn event_whenbackdropswitchesto(_ctx: &mut BlockContext) -> BlockResult {
    end()
}

pub fn event_whenbroadcastreceived(_ctx: &mut BlockContext) -> BlockResult {
    end()
}

pub fn event_broadcast(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(1, |ctx| {
        // println!("Broadcast {}", ctx.arg(0).to_string());
        BlockResult::Boardcast(ctx.arg(0).to_string())
    })
}

pub fn event_broadcastandwait(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(1, |ctx| {
        // println!("Broadcast Wait {}", ctx.arg(0).to_string());
        BlockResult::BoardcastAndWait(ctx.arg(0).to_string())
    })
}
