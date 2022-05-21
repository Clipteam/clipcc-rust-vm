use crate::*;

pub fn procedures_definition(_ctx: &mut BlockContext) -> BlockResult {
    end()
}

/// ClipCC
pub fn procedures_definition_return(_ctx: &mut BlockContext) -> BlockResult {
    end()
}

pub struct ProcedureArguments(pub Vec<BlockValue>);
pub struct ReturnProcedureArguments(pub Vec<BlockValue>);

pub fn procedures_call(ctx: &mut BlockContext) -> BlockResult {
    if ctx.arg_len() >= ctx.get_block().arguments.len() {
        return BlockResult::Resolved(None);
    }
    ctx.acquire_args(ctx.get_block().arguments.len() - 1, |ctx| {
        let bid = match ctx.get_block().arguments.last() {
            Some(BlockValue::BlockId(bid)) => BlockResult::PushStack(*bid),
            _ => BlockResult::Resolved(None),
        };
        if let BlockResult::PushStack(bid) = bid {
            ctx.stack.block_data = Box::new(ProcedureArguments(ctx.stack.arguments.to_owned()));
            BlockResult::PushStack(bid)
        } else {
            BlockResult::Resolved(None)
        }
    })
}

/// ClipCC
pub fn procedures_call_return(ctx: &mut BlockContext) -> BlockResult {
    if ctx.arg_len() >= ctx.get_block().arguments.len() {
        return BlockResult::Resolved(None);
    }
    ctx.acquire_args(ctx.get_block().arguments.len() - 1, |ctx| {
        let bid = match ctx.get_block().arguments.last() {
            Some(BlockValue::BlockId(bid)) => BlockResult::PushStack(*bid),
            _ => BlockResult::Resolved(None),
        };
        if let BlockResult::PushStack(bid) = bid {
            ctx.stack.block_data =
                Box::new(ReturnProcedureArguments(ctx.stack.arguments.to_owned()));
            BlockResult::PushStack(bid)
        } else {
            BlockResult::Resolved(None)
        }
    })
}

pub fn argument_reporter_string_number(ctx: &mut BlockContext) -> BlockResult {
    if ctx.arg_len() > 0 {
        return BlockResult::Resolved(Some(ctx.arg(0).to_owned()));
    } else {
        let param_id = ctx.get_block().arguments[0].to_number() as usize;
        BlockResult::ResolveProcedureArgument(param_id)
    }
}

pub fn argument_reporter_boolean(ctx: &mut BlockContext) -> BlockResult {
    if ctx.arg_len() > 1 {
        return match ctx.arg(1) {
            BlockValue::Undefined => ret(0),
            other => ret(other.to_owned()),
        };
    }
    ctx.acquire_args(1, |ctx| {
        let param_id = ctx.arg(0).to_number() as usize;
        BlockResult::ResolveProcedureArgument(param_id)
    })
}

/// ClipCC
pub fn procedures_return(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(1, |ctx| {
        let value = ctx.arg(0).to_owned();
        BlockResult::ReturnProcedure(value)
    })
}
