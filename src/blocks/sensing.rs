use chrono::*;

use crate::*;
use std::sync::mpsc::*;
use std::{
    io::BufRead,
    sync::{atomic::AtomicBool, Arc, Mutex},
    time::Instant,
};

pub fn sensing_resettimer(ctx: &mut BlockContext) -> BlockResult {
    ctx.global_data
        .insert("GLOBAL_TIMER".into(), Box::new(Instant::now()));
    BlockResult::Resolved(None)
}

pub fn sensing_timer(ctx: &mut BlockContext) -> BlockResult {
    if let Some(timer) = ctx.global_data.get("GLOBAL_TIMER") {
        if let Some(timer) = timer.downcast_ref::<Instant>() {
            BlockResult::Resolved(Some(timer.elapsed().as_secs_f64().into()))
        } else {
            sensing_timer(ctx);
            BlockResult::Resolved(Some(0f64.into()))
        }
    } else {
        sensing_resettimer(ctx);
        BlockResult::Resolved(Some(0f64.into()))
    }
}

const SESING_ANSWER: &str = "SENSING_ANSWER";

fn start_ask(tx: Sender<String>) {
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut handle = stdin.lock();
        let mut answer = String::new();
        handle.read_line(&mut answer).unwrap();
        tx.send(
            answer
                .trim_end_matches(|x| x == '\n' || x == '\r')
                .to_owned(),
        )
        .unwrap();
    });
}

pub fn sensing_distanceto(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(1, |ctx| {
        if ctx.is_stage() {
            ret(10000f64)
        } else if let Some((x, y)) = get_target_xy(ctx, ctx.arg(0).to_string().as_str()) {
            let dx = x - ctx.running_target().x;
            let dy = y - ctx.running_target().y;
            ret((dx.powi(2) + dy.powi(2)).sqrt())
        } else {
            ret(10000f64)
        }
    })
}

pub fn sensing_distancetomenu(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(1, |ctx| ret(ctx.arg(0).to_owned()))
}

pub fn sensing_askandwait(ctx: &mut BlockContext) -> BlockResult {
    type IOLock = Arc<AtomicBool>;
    const SESING_ASK_LOCK: &str = "SESING_ASK_LOCK";
    ctx.acquire_args(1, |ctx| {
        let ask_msg = ctx.arg(0).to_string();
        if let Some(self_lock) = ctx.stack.block_data.downcast_ref::<Receiver<String>>() {
            if let Ok(answer) = self_lock.try_recv() {
                ctx.global_data
                    .insert(SESING_ANSWER.into(), Box::new(answer));
                if let Some(lock) = ctx.global_data.get_mut(SESING_ASK_LOCK) {
                    if let Some(lock) = lock.downcast_mut::<IOLock>() {
                        lock.store(false, std::sync::atomic::Ordering::SeqCst);
                    }
                }
                end()
            } else {
                pending()
            }
        } else if let Some(lock) = ctx.global_data.get_mut(SESING_ASK_LOCK) {
            if let Some(lock) = lock.downcast_mut::<IOLock>() {
                if lock.load(std::sync::atomic::Ordering::SeqCst) {
                    pending()
                } else {
                    println!("{}", ask_msg);
                    // Start a new thread to wait for the answer
                    lock.store(true, std::sync::atomic::Ordering::SeqCst);
                    let (tx, rx) = channel();
                    start_ask(tx);
                    ctx.stack.block_data = Box::new(Arc::new(Mutex::new(rx)));
                    pending()
                }
            } else {
                println!("{}", ask_msg);
                // Start a new thread to wait for the answer
                ctx.global_data.insert(
                    SESING_ASK_LOCK.into(),
                    Box::new(IOLock::new(AtomicBool::new(true))),
                );
                let (tx, rx) = channel();
                start_ask(tx);
                ctx.stack.block_data = Box::new(Arc::new(Mutex::new(rx)));
                pending()
            }
        } else {
            //
            ctx.global_data.insert(
                SESING_ASK_LOCK.into(),
                Box::new(IOLock::new(AtomicBool::new(true))),
            );
            println!("{}", ask_msg);
            let (tx, rx) = channel();
            start_ask(tx);
            ctx.stack.block_data = Box::new(Arc::new(Mutex::new(rx)));
            pending()
        }
    })
}

pub fn sensing_answer(ctx: &mut BlockContext) -> BlockResult {
    BlockResult::Resolved(
        if let Some(answer) = ctx
            .global_data
            .get(SESING_ANSWER)
            .and_then(|x| x.downcast_ref::<String>())
        {
            Some(answer.to_owned().into())
        } else {
            Default::default()
        },
    )
}

pub fn sensing_of_object_menu(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(1, |ctx| ret(ctx.arg(0).to_owned()))
}
pub fn sensing_of(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(2, |ctx| {
        let property = ctx.arg(0).to_string();
        let object = ctx.arg(1).to_string();
        match object.as_str() {
            "_stage_" => {
                let stage = ctx.stage();
                let rstage = ctx.running_stage();
                match property.as_str() {
                    "background #" => return ret(rstage.current_costume + 1),
                    "backdrop #" => return ret(rstage.current_costume + 1),
                    "backdrop name" => {
                        return ret(stage.costumes[rstage.current_costume].to_owned())
                    }
                    "volume" => return ret(rstage.volume),
                    variable => {
                        if let Some(value) = rstage.variables.get(variable) {
                            return ret(value.to_owned());
                        }
                    }
                }
            }
            target => {
                if let Some((_, rt)) = ctx
                    .running_targets
                    .iter()
                    .find(|x| !x.1.is_clone && x.1.name == target)
                {
                    let target = &ctx.targets[rt.target_id];
                    match property.as_str() {
                        "x position" => return ret(rt.x),
                        "y position" => return ret(rt.y),
                        "direction" => return ret(rt.direction),
                        "costume #" => return ret(rt.current_costume + 1),
                        "costume name" => {
                            return ret(target.costumes[rt.current_costume].to_owned())
                        }
                        "size" => return ret(rt.size),
                        "volume" => return ret(rt.volume),
                        variable => {
                            if let Some(value) = rt.variables.get(variable) {
                                return ret(value.to_owned());
                            }
                        }
                    }
                }
            }
        }
        ret(0)
    })
}

pub fn sensing_current(ctx: &mut BlockContext) -> BlockResult {
    ctx.acquire_args(1, |ctx| {
        let current_menu = ctx.arg(0).to_string().to_lowercase();
        let local = Local::now();
        ret(match current_menu.as_str() {
            "year" => local.year_ce().1,
            "month" => local.month(),
            "date" => local.day(),
            "dayofweek" => local.weekday().num_days_from_sunday() + 1,
            "hour" => local.hour(),
            "minute" => local.minute(),
            "second" => local.second(),
            _ => 0,
        })
    })
}
pub fn sensing_dayssince2000(_ctx: &mut BlockContext) -> BlockResult {
    // Get day since 2000 year
    // Should be float number
    let start = Utc.ymd(2000, 1, 1).and_hms(0, 0, 0);
    let local = Utc::now();
    let days = local.signed_duration_since(start).num_milliseconds();
    ret(days as f64 / 1000.0 / 60.0 / 60.0 / 24.0)
}
pub fn sensing_username(_ctx: &mut BlockContext) -> BlockResult {
    ret("")
}
pub fn sensing_userid(_ctx: &mut BlockContext) -> BlockResult {
    ret(0)
}
