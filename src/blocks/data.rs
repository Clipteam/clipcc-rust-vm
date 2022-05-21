use crate::*;

enum ListIndex {
    Invalid,
    All,
    Index(usize),
}

fn to_list_index(index: &BlockValue, len: usize, accept_all: bool) -> ListIndex {
    use rand::prelude::*;
    match index {
        BlockValue::String(index) => match index.as_str() {
            "all" => {
                if accept_all {
                    ListIndex::All
                } else {
                    ListIndex::Invalid
                }
            }
            "last" => {
                if len > 0 {
                    ListIndex::Index(len - 1)
                } else {
                    ListIndex::Invalid
                }
            }
            "random" | "any" => {
                if len > 0 {
                    let index = unsafe { THREAD_RND.gen_range(0..len) };
                    ListIndex::Index(index)
                } else {
                    ListIndex::Invalid
                }
            }
            _ => ListIndex::Invalid,
        },
        BlockValue::Number(index) => {
            if *index >= 1. && *index <= len as f64 {
                ListIndex::Index(index.floor() as usize)
            } else {
                ListIndex::Invalid
            }
        }
        _ => ListIndex::Invalid,
    }
}

pub fn data_variable(ctx: &mut BlockContext) -> BlockResult {
    if let Some(name) = ctx.stack.arguments.get(0) {
        match name {
            BlockValue::Undefined => BlockResult::Resolved(Some(BlockValue::Undefined)),
            other => {
                let name = other.to_string();
                if let Some(v) = ctx.running_target().variables.get(&name) {
                    BlockResult::Resolved(Some(v.to_owned()))
                } else if let Some(v) = ctx.running_stage().variables.get(&name) {
                    BlockResult::Resolved(Some(v.to_owned()))
                } else {
                    BlockResult::Resolved(Some(BlockValue::Undefined))
                }
            }
        }
    } else {
        BlockResult::ResolveArgument(0)
    }
}

pub fn data_listcontents(ctx: &mut BlockContext) -> BlockResult {
    fn is_all_single_letters(v: &Vec<BlockValue>) -> bool {
        for v in v.iter() {
            match v {
                BlockValue::String(v) => {
                    if v.len() == 1 {
                        continue;
                    } else {
                        return false;
                    }
                }
                _ => return false,
            }
        }
        true
    }
    if let Some(name) = ctx.stack.arguments.get(0) {
        match name {
            BlockValue::Undefined => BlockResult::Resolved(Some(BlockValue::Undefined)),
            other => {
                let name = other.to_string();
                if let Some(v) = ctx.running_target().lists.get(&name) {
                    let isallsingleletters = is_all_single_letters(v);
                    let result = if isallsingleletters {
                        v.iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<String>>()
                            .join("")
                    } else {
                        v.iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<String>>()
                            .join(" ")
                    };
                    BlockResult::Resolved(Some(result.into()))
                } else if let Some(v) = ctx.running_stage().lists.get(&name) {
                    let isallsingleletters = is_all_single_letters(v);
                    let result = if isallsingleletters {
                        v.iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<String>>()
                            .join("")
                    } else {
                        v.iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<String>>()
                            .join(" ")
                    };
                    BlockResult::Resolved(Some(result.into()))
                } else {
                    BlockResult::Resolved(Some(BlockValue::Undefined))
                }
            }
        }
    } else {
        BlockResult::ResolveArgument(0)
    }
}

pub fn data_changevariableby(ctx: &mut BlockContext) -> BlockResult {
    let arg_len = ctx.stack.arguments.len();
    if arg_len < 2 {
        return BlockResult::ResolveArgument(arg_len);
    }
    let name = &ctx.stack.arguments[0];
    let value = &ctx.stack.arguments[1];
    match name {
        BlockValue::Undefined => BlockResult::Resolved(Some(BlockValue::Undefined)),
        other => {
            let name = other.to_string();
            let value = value.to_number();
            let variable = ctx.get_variable_mut(&name);
            *variable = (variable.to_number() + value).into();
            BlockResult::Resolved(None)
        }
    }
}

pub fn data_setvariableto(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(2, |ctx| {
        let name = ctx.arg(0).to_string();
        let value = ctx.arg(1).to_owned();
        let variable = ctx.get_variable_mut(&name);
        *variable = value;
        BlockResult::Resolved(None)
    })
}

pub fn data_itemoflist(ctx: &mut BlockContext) -> BlockResult {
    let arg_len = ctx.stack.arguments.len();
    if arg_len < 2 {
        return BlockResult::ResolveArgument(arg_len);
    }
    let index = ctx.stack.arguments[0].to_number() as usize;
    let list = ctx.stack.arguments[1].to_string();
    if let Some(list) = ctx.running_target().lists.get(&list) {
        BlockResult::Resolved(list.get(index).cloned())
    } else {
        BlockResult::Resolved(None)
    }
}

pub fn data_addtolist(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(2, |ctx| {
        let item = ctx.arg(0).to_owned();
        let name = ctx.arg(1).to_string();
        let list = ctx.get_list_mut(&name);
        list.push(item);
        BlockResult::Resolved(None)
    })
}

pub fn data_deleteoflist(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(2, |ctx| {
        let index = ctx.arg(0).to_owned();
        let name = ctx.arg(1).to_string();
        let list = ctx.get_list_mut(&name);
        match to_list_index(&index, list.len(), true) {
            ListIndex::Index(index) => {
                if index < list.len() {
                    list.remove(index);
                }
            }
            ListIndex::All => {
                list.clear();
            }
            ListIndex::Invalid => {}
        }
        BlockResult::Resolved(None)
    })
}

pub fn data_deletealloflist(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(1, |ctx| {
        let name = ctx.arg(0).to_string();
        let list = ctx.get_list_mut(&name);
        list.clear();
        end()
    })
}

pub fn data_insertatlist(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(3, |ctx| {
        let item = ctx.arg(0).to_owned();
        let index = ctx.arg(1).to_owned();
        let name = ctx.arg(2).to_string();
        let list = ctx.get_list_mut(&name);
        match to_list_index(&index, list.len(), false) {
            ListIndex::Index(index) => {
                list.insert(index, item);
            }
            _ => {}
        }
        BlockResult::Resolved(None)
    })
}

pub fn data_replaceitemoflist(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(3, |ctx| {
        let name = ctx.arg(1).to_string();
        let list_len = ctx.get_list(&name).len();
        let item = ctx.arg(2).to_owned();
        let index = ctx.arg(0);
        match to_list_index(index, list_len, false) {
            ListIndex::Index(index) => {
                let list = ctx.get_list_mut(&name);
                while index >= list.len() {
                    list.push(BlockValue::Undefined);
                }
                list[index] = item;
            }
            _ => {}
        }
        BlockResult::Resolved(None)
    })
}

pub fn data_lengthoflist(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(1, |ctx| {
        let name = ctx.arg(0).to_string();
        let len = ctx.get_list(&name).len();
        BlockResult::Resolved(Some(len.into()))
    })
}

pub fn data_itemnumoflist(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(2, |ctx| {
        let name = ctx.arg(0).to_string();
        let item = ctx.arg(1);
        let list = ctx.get_list(&name);
        if let Some(index) = list.iter().position(|x| x == item) {
            ret(index + 1)
        } else {
            ret(0usize)
        }
    })
}

pub fn data_listcontainsitem(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(2, |ctx| {
        let name = ctx.arg(0).to_string();
        let value = ctx.arg(1);
        for item in ctx.get_list(&name).iter() {
            if item == value {
                return ret(true);
            }
        }
        ret(false)
    })
}
