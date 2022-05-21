use crate::*;

pub fn operator_add(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(2, |ctx| {
        let left = ctx.arg(0).to_number();
        let right = ctx.arg(1).to_number();
        BlockResult::Resolved(Some((left + right).into()))
    })
}

pub fn operator_subtract(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(2, |ctx| {
        let left = ctx.arg(0).to_number();
        let right = ctx.arg(1).to_number();
        BlockResult::Resolved(Some((left - right).into()))
    })
}

pub fn operator_multiply(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(2, |ctx| {
        let left = ctx.arg(0).to_number();
        let right = ctx.arg(1).to_number();
        BlockResult::Resolved(Some((left * right).into()))
    })
}

pub fn operator_divide(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(2, |ctx| {
        let left = ctx.arg(0).to_number();
        let right = ctx.arg(1).to_number();
        BlockResult::Resolved(Some((left / right).into()))
    })
}

pub fn operator_random(ctx: &mut BlockContext) -> BlockResult {
    use rand::prelude::*;
    ctx.acquire_args(2, |ctx| {
        let mut from = ctx.arg(0).to_number();
        let mut to = ctx.arg(1).to_number();
        if from > to {
            std::mem::swap(&mut from, &mut to);
        }
        if from == to {
            return BlockResult::Resolved(Some(from.into()));
        }
        let result = if from.trunc() == from && to.trunc() == to {
            unsafe { THREAD_RND.gen_range((from as isize)..(to as isize)) as f64 }
        } else {
            unsafe { THREAD_RND.gen_range(from..to) }
        };
        BlockResult::Resolved(Some(result.into()))
    })
}

pub fn operator_lt(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(2, |ctx| {
        let left = ctx.arg(0);
        let right = ctx.arg(1);
        if let Some(r) = (left).partial_cmp(right) {
            ret(r == std::cmp::Ordering::Less)
        } else {
            ret(false)
        }
    })
}

pub fn operator_equals(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(2, |ctx| {
        let left = ctx.arg(0);
        let right = ctx.arg(1);
        ret(left.eq(right))
    })
}

pub fn operator_gt(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(2, |ctx| {
        let left = ctx.arg(0);
        let right = ctx.arg(1);
        if let Some(r) = left.partial_cmp(right) {
            ret(r == std::cmp::Ordering::Greater)
        } else {
            ret(false)
        }
    })
}

pub fn operator_and(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(2, |ctx| {
        let left = ctx.arg(0).to_boolean();
        let right = ctx.arg(1).to_boolean();
        BlockResult::Resolved(Some((left && right).into()))
    })
}

pub fn operator_or(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(2, |ctx| {
        let left = ctx.arg(0).to_boolean();
        let right = ctx.arg(1).to_boolean();
        BlockResult::Resolved(Some((left || right).into()))
    })
}

pub fn operator_not(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(1, |ctx| {
        let value = ctx.arg(0).to_boolean();
        BlockResult::Resolved(Some((!value).into()))
    })
}

pub fn operator_join(ctx: &mut BlockContext) -> BlockResult {
    if let Some(string0) = ctx.stack.arguments.get(0) {
        if let Some(string1) = ctx.stack.arguments.get(1) {
            let string0 = string0.to_string();
            let string1 = string1.to_string();
            BlockResult::Resolved(Some(format!("{}{}", string0, string1).into()))
        } else {
            BlockResult::ResolveArgument(1)
        }
    } else {
        BlockResult::ResolveArgument(0)
    }
}

pub fn operator_letter_of(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(2, |ctx| {
        let letter = ctx.arg(0).to_number() - 1.;
        let string = ctx.arg(1).to_string();
        if letter < 0. || letter >= string.len() as f64 {
            return BlockResult::Resolved(Some("".into()));
        }
        let letter = letter.trunc() as usize;
        ret(string
            .chars().nth(letter)
            .map(|c| c.to_string())
            .unwrap_or_default())
    })
}

pub fn operator_contains(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(2, |ctx| {
        let string1 = ctx.arg(0).to_string().to_lowercase();
        let string2 = ctx.arg(1).to_string().to_lowercase();
        ret(string1.contains(&string2))
    })
}

pub fn operator_length(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(1, |ctx| {
        let string = ctx.arg(0).to_string();
        BlockResult::Resolved(Some(string.len().into()))
    })
}

pub fn operator_mod(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(2, |ctx| {
        let n = ctx.arg(0).to_number();
        let m = ctx.arg(1).to_number();
        let mut r = n % m;
        if r / m < 0.0 {
            r += m
        }
        BlockResult::Resolved(Some(r.into()))
    })
}

pub fn operator_round(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(2, |ctx| {
        let num = ctx.arg(0).to_number();
        BlockResult::Resolved(Some(num.round().into()))
    })
}

pub fn operator_mathop(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(2, |ctx| {
        let op = ctx.arg(0).to_string().to_lowercase();
        let num = ctx.arg(1).to_number();
        BlockResult::Resolved(Some(
            match op.as_str() {
                "abs" => num.abs(),
                "floor" => num.floor(),
                "ceiling" => num.ceil(),
                "sqrt" => num.sqrt(),
                "sin" => (num.to_radians().sin() * 10.).trunc() / 10.,
                "cos" => (num.to_radians().cos() * 10.).trunc() / 10.,
                "tan" => num.tan(),
                "asin" => num.asin().to_degrees(),
                "acos" => num.acos().to_degrees(),
                "atan" => num.atan().to_degrees(),
                "ln" => num.ln(),
                "log" => num.log10(),
                "e ^" => num.exp(),
                "10 ^" => (10f64).powf(num),
                _ => 0.,
            }
            .into(),
        ))
    })
}
