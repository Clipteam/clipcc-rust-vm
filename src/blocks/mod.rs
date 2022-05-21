use crate::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;

pub mod controls;
pub mod data;
pub mod events;
pub mod looks;
pub mod motions;
pub mod operators;
pub mod procedures;
pub mod sensing;

pub fn noop(_ctx: &mut BlockContext) -> BlockResult {
    BlockResult::Resolved(None)
}

pub fn just_return(ctx: &mut BlockContext) -> BlockResult {
    debug_assert_eq!(
        ctx.get_block().arguments.len(),
        1,
        "Just return argument must be only one!"
    );
    debug_assert!(
        !ctx.get_block().arguments[0].is_block(),
        "Just return argument must not be a block!"
    );
    BlockResult::Resolved(Some(ctx.get_block().arguments[0].to_owned()))
}

pub enum ArgType {
    Input,
    Field,
}

pub struct BlockInfo {
    pub block_function: BlockFunction,
    pub arguments: Vec<(ArgType, String)>,
}

pub fn register_block(opcode: &'static str, block_info: BlockInfo) {
    unsafe {
        BLOCKS.insert(opcode, block_info);
    }
}

pub fn get_blockinfo(opcode: &str) -> Option<&BlockInfo> {
    unsafe { BLOCKS.get(opcode) }
}

pub static mut BLOCKS: Lazy<HashMap<&str, BlockInfo>> = Lazy::new(|| {
    let mut h = HashMap::new();

    h.insert(
        "motion_movesteps",
        BlockInfo {
            block_function: crate::core_blocks::motion_movesteps,
            arguments: vec![(ArgType::Input, "STEPS".into())],
        },
    );
    h.insert(
        "motion_turnright",
        BlockInfo {
            block_function: crate::core_blocks::motion_turnright,
            arguments: vec![(ArgType::Input, "DEGREES".into())],
        },
    );
    h.insert(
        "motion_turnleft",
        BlockInfo {
            block_function: crate::core_blocks::motion_turnleft,
            arguments: vec![(ArgType::Input, "DEGREES".into())],
        },
    );
    h.insert(
        "motion_pointindirection",
        BlockInfo {
            block_function: crate::core_blocks::motion_pointindirection,
            arguments: vec![(ArgType::Input, "DIRECTION".into())],
        },
    );
    h.insert(
        "motion_pointtowards",
        BlockInfo {
            block_function: crate::core_blocks::motion_pointtowards,
            arguments: vec![(ArgType::Input, "TOWARDS".into())],
        },
    );
    h.insert(
        "motion_pointtowards_menu",
        BlockInfo {
            block_function: crate::core_blocks::motion_pointtowards_menu,
            arguments: vec![(ArgType::Field, "TOWARDS".into())],
        },
    );
    h.insert(
        "motion_gotoxy",
        BlockInfo {
            block_function: crate::core_blocks::motion_gotoxy,
            arguments: vec![(ArgType::Input, "X".into()), (ArgType::Input, "Y".into())],
        },
    );
    h.insert(
        "motion_goto",
        BlockInfo {
            block_function: crate::core_blocks::motion_goto,
            arguments: vec![(ArgType::Input, "TO".into())],
        },
    );
    h.insert(
        "motion_goto_menu",
        BlockInfo {
            block_function: crate::core_blocks::motion_goto_menu,
            arguments: vec![(ArgType::Field, "TO".into())],
        },
    );
    h.insert(
        "motion_glidesecstoxy",
        BlockInfo {
            block_function: crate::core_blocks::motion_glidesecstoxy,
            arguments: vec![
                (ArgType::Input, "SECS".into()),
                (ArgType::Input, "X".into()),
                (ArgType::Input, "Y".into()),
            ],
        },
    );
    h.insert(
        "motion_glideto",
        BlockInfo {
            block_function: crate::core_blocks::motion_glideto,
            arguments: vec![
                (ArgType::Input, "SECS".into()),
                (ArgType::Input, "TO".into()),
            ],
        },
    );
    h.insert(
        "motion_glideto_menu",
        BlockInfo {
            block_function: crate::core_blocks::motion_glideto_menu,
            arguments: vec![
                (ArgType::Field, "TO".into()),
            ],
        },
    );
    h.insert(
        "motion_changexby",
        BlockInfo {
            block_function: crate::core_blocks::motion_changexby,
            arguments: vec![(ArgType::Input, "DX".into())],
        },
    );
    h.insert(
        "motion_setx",
        BlockInfo {
            block_function: crate::core_blocks::motion_setx,
            arguments: vec![(ArgType::Input, "X".into())],
        },
    );
    h.insert(
        "motion_changeyby",
        BlockInfo {
            block_function: crate::core_blocks::motion_changeyby,
            arguments: vec![(ArgType::Input, "DY".into())],
        },
    );
    h.insert(
        "motion_sety",
        BlockInfo {
            block_function: crate::core_blocks::motion_sety,
            arguments: vec![(ArgType::Input, "Y".into())],
        },
    );
    h.insert(
        "motion_ifonedgebounce",
        BlockInfo {
            block_function: crate::core_blocks::motion_ifonedgebounce,
            arguments: vec![],
        },
    );
    h.insert(
        "motion_setrotationstyle",
        BlockInfo {
            block_function: crate::core_blocks::motion_setrotationstyle,
            arguments: vec![(ArgType::Field, "STYLE".into())],
        },
    );
    h.insert(
        "motion_xposition",
        BlockInfo {
            block_function: crate::core_blocks::motion_xposition,
            arguments: vec![],
        },
    );
    h.insert(
        "motion_yposition",
        BlockInfo {
            block_function: crate::core_blocks::motion_yposition,
            arguments: vec![],
        },
    );
    h.insert(
        "motion_direction",
        BlockInfo {
            block_function: crate::core_blocks::motion_direction,
            arguments: vec![],
        },
    );
    //    h.insert("motion_scroll_right", BlockInfo { block_function: crate::core_blocks::motion_scroll_right, arguments: vec![(ArgType::Input, "DISTANCE".into())] });
    //    h.insert("motion_scroll_up", BlockInfo { block_function: crate::core_blocks::motion_scroll_up, arguments: vec![(ArgType::Input, "DISTANCE".into())] });
    //    h.insert("motion_align_scene", BlockInfo { block_function: crate::core_blocks::motion_align_scene, arguments: vec![(ArgType::Field, "ALIGNMENT".into())] });
    //    h.insert("motion_xscroll", BlockInfo { block_function: crate::core_blocks::motion_xscroll, arguments: vec![] });
    //    h.insert("motion_yscroll", BlockInfo { block_function: crate::core_blocks::motion_yscroll, arguments: vec![] });
    h.insert(
        "looks_sayforsecs",
        BlockInfo {
            block_function: crate::core_blocks::looks_sayforsecs,
            arguments: vec![
                (ArgType::Input, "MESSAGE".into()),
                (ArgType::Input, "SECS".into()),
            ],
        },
    );
    h.insert(
        "looks_say",
        BlockInfo {
            block_function: crate::core_blocks::looks_say,
            arguments: vec![(ArgType::Input, "MESSAGE".into())],
        },
    );
    h.insert(
        "looks_thinkforsecs",
        BlockInfo {
            block_function: crate::core_blocks::looks_sayforsecs,
            arguments: vec![
                (ArgType::Input, "MESSAGE".into()),
                (ArgType::Input, "SECS".into()),
            ],
        },
    );
    h.insert(
        "looks_think",
        BlockInfo {
            block_function: crate::core_blocks::looks_say,
            arguments: vec![(ArgType::Input, "MESSAGE".into())],
        },
    );
    h.insert(
        "looks_show",
        BlockInfo {
            block_function: crate::core_blocks::looks_show,
            arguments: vec![],
        },
    );
    h.insert(
        "looks_hide",
        BlockInfo {
            block_function: crate::core_blocks::looks_hide,
            arguments: vec![],
        },
    );
    // h.insert("looks_hideallsprites", BlockInfo { block_function: crate::core_blocks::looks_hideallsprites, arguments: vec![] });
    h.insert(
        "looks_switchcostumeto",
        BlockInfo {
            block_function: crate::core_blocks::looks_switchcostumeto,
            arguments: vec![(ArgType::Input, "COSTUME".into())],
        },
    );
    h.insert(
        "looks_costume",
        BlockInfo {
            block_function: crate::core_blocks::looks_costume,
            arguments: vec![(ArgType::Field, "COSTUME".into())],
        },
    );
    h.insert(
        "looks_nextcostume",
        BlockInfo {
            block_function: crate::core_blocks::looks_nextcostume,
            arguments: vec![],
        },
    );
    h.insert(
        "looks_switchbackdropto",
        BlockInfo {
            block_function: crate::core_blocks::looks_switchbackdropto,
            arguments: vec![(ArgType::Input, "BACKDROP".into())],
        },
    );
    h.insert(
        "looks_backdrops",
        BlockInfo {
            block_function: crate::core_blocks::looks_backdrops,
            arguments: vec![(ArgType::Field, "BACKDROP".into())],
        },
    );
    // h.insert("looks_changeeffectby", BlockInfo { block_function: crate::core_blocks::looks_changeeffectby, arguments: vec![(ArgType::Field, "EFFECT".into()), (ArgType::Input, "CHANGE".into())] });
    // h.insert("looks_seteffectto", BlockInfo { block_function: crate::core_blocks::looks_seteffectto, arguments: vec![(ArgType::Field, "EFFECT".into()), (ArgType::Input, "VALUE".into())] });
    // h.insert("looks_cleargraphiceffects", BlockInfo { block_function: crate::core_blocks::looks_cleargraphiceffects, arguments: vec![] });
    h.insert(
        "looks_changesizeby",
        BlockInfo {
            block_function: crate::core_blocks::looks_changesizeby,
            arguments: vec![(ArgType::Input, "CHANGE".into())],
        },
    );
    h.insert(
        "looks_setsizeto",
        BlockInfo {
            block_function: crate::core_blocks::looks_setsizeto,
            arguments: vec![(ArgType::Input, "SIZE".into())],
        },
    );
    // h.insert("looks_changestretchby", BlockInfo { block_function: crate::core_blocks::looks_changestretchby, arguments: vec![(ArgType::Input, "CHANGE".into())] });
    // h.insert("looks_setstretchto", BlockInfo { block_function: crate::core_blocks::looks_setstretchto, arguments: vec![(ArgType::Input, "STRETCH".into())] });
    h.insert(
        "looks_gotofrontback",
        BlockInfo {
            block_function: crate::core_blocks::looks_gotofrontback,
            arguments: vec![(ArgType::Field, "FRONT_BACK".into())],
        },
    );
    h.insert(
        "looks_goforwardbackwardlayers",
        BlockInfo {
            block_function: crate::core_blocks::looks_goforwardbackwardlayers,
            arguments: vec![
                (ArgType::Field, "FORWARD_BACKWARD".into()),
                (ArgType::Input, "NUM".into()),
            ],
        },
    );
    h.insert(
        "looks_costumenumbername",
        BlockInfo {
            block_function: crate::core_blocks::looks_costumenumbername,
            arguments: vec![(ArgType::Input, "NUMBER_NAME".into())],
        },
    );
    h.insert(
        "looks_costumenumbernamemenu",
        BlockInfo {
            block_function: crate::core_blocks::looks_costumenumbername,
            arguments: vec![(ArgType::Field, "NUMBER_NAME".into())],
        },
    );
    h.insert(
        "looks_backdropnumbername",
        BlockInfo {
            block_function: crate::core_blocks::looks_backdropnumbername,
            arguments: vec![(ArgType::Input, "NUMBER_NAME".into())],
        },
    );
    h.insert(
        "looks_backdropnumbernamemenu",
        BlockInfo {
            block_function: crate::core_blocks::looks_backdropnumbername,
            arguments: vec![(ArgType::Field, "NUMBER_NAME".into())],
        },
    );
    h.insert(
        "looks_size",
        BlockInfo {
            block_function: crate::core_blocks::looks_size,
            arguments: vec![],
        },
    );
    // h.insert("looks_switchbackdroptoandwait", BlockInfo { block_function: crate::core_blocks::looks_switchbackdroptoandwait, arguments: vec![(ArgType::Input, "BACKDROP".into())] });
    // h.insert("looks_nextbackdrop", BlockInfo { block_function: crate::core_blocks::looks_nextbackdrop, arguments: vec![] });
    // h.insert("looks_backdropnumbername", BlockInfo { block_function: crate::core_blocks::looks_backdropnumbername, arguments: vec![] });
    // h.insert("sound_play", BlockInfo { block_function: crate::core_blocks::sound_play, arguments: vec![(ArgType::Input, "SOUND_MENU".into())] });
    // h.insert("sound_playuntildone", BlockInfo { block_function: crate::core_blocks::sound_playuntildone, arguments: vec![(ArgType::Input, "SOUND_MENU".into())] });
    // h.insert("sound_stopallsounds", BlockInfo { block_function: crate::core_blocks::sound_stopallsounds, arguments: vec![] });
    // h.insert("music_playDrumForBeats", BlockInfo { block_function: crate::core_blocks::music_playDrumForBeats, arguments: vec![(ArgType::Input, "DRUM".into()), (ArgType::Input, "BEATS".into())] });
    // h.insert("music_midiPlayDrumForBeats", BlockInfo { block_function: crate::core_blocks::music_midiPlayDrumForBeats, arguments: vec![(ArgType::Input, "DRUM".into()), (ArgType::Input, "BEATS".into())] });
    // h.insert("music_restForBeats", BlockInfo { block_function: crate::core_blocks::music_restForBeats, arguments: vec![(ArgType::Input, "BEATS".into())] });
    // h.insert("music_playNoteForBeats", BlockInfo { block_function: crate::core_blocks::music_playNoteForBeats, arguments: vec![(ArgType::Input, "NOTE".into()), (ArgType::Input, "BEATS".into())] });
    // h.insert("music_setInstrument", BlockInfo { block_function: crate::core_blocks::music_setInstrument, arguments: vec![(ArgType::Input, "INSTRUMENT".into())] });
    // h.insert("music_midiSetInstrument", BlockInfo { block_function: crate::core_blocks::music_midiSetInstrument, arguments: vec![(ArgType::Input, "INSTRUMENT".into())] });
    // h.insert("sound_changevolumeby", BlockInfo { block_function: crate::core_blocks::sound_changevolumeby, arguments: vec![(ArgType::Input, "VOLUME".into())] });
    // h.insert("sound_setvolumeto", BlockInfo { block_function: crate::core_blocks::sound_setvolumeto, arguments: vec![(ArgType::Input, "VOLUME".into())] });
    // h.insert("sound_volume", BlockInfo { block_function: crate::core_blocks::sound_volume, arguments: vec![] });
    // h.insert("music_changeTempo", BlockInfo { block_function: crate::core_blocks::music_changeTempo, arguments: vec![(ArgType::Input, "TEMPO".into())] }); */
    // h.insert("music_setTempo", BlockInfo { block_function: crate::core_blocks::music_setTempo, arguments: vec![(ArgType::Input, "TEMPO".into())] });
    // h.insert("music_getTempo", BlockInfo { block_function: crate::core_blocks::music_getTempo, arguments: vec![] });
    // h.insert("pen_clear", BlockInfo { block_function: crate::core_blocks::pen_clear, arguments: vec![] });
    // h.insert("pen_stamp", BlockInfo { block_function: crate::core_blocks::pen_stamp, arguments: vec![] });
    // h.insert("pen_penDown", BlockInfo { block_function: crate::core_blocks::pen_penDown, arguments: vec![] });
    // h.insert("pen_penUp", BlockInfo { block_function: crate::core_blocks::pen_penUp, arguments: vec![] });
    // h.insert("pen_setPenColorToColor", BlockInfo { block_function: crate::core_blocks::pen_setPenColorToColor, arguments: vec![(ArgType::Input, "COLOR".into())] });
    // h.insert("pen_changePenHueBy", BlockInfo { block_function: crate::core_blocks::pen_changePenHueBy, arguments: vec![(ArgType::Input, "HUE".into())] });
    // h.insert("pen_setPenHueToNumber", BlockInfo { block_function: crate::core_blocks::pen_setPenHueToNumber, arguments: vec![(ArgType::Input, "HUE".into())] });
    // h.insert("pen_changePenShadeBy", BlockInfo { block_function: crate::core_blocks::pen_changePenShadeBy, arguments: vec![(ArgType::Input, "SHADE".into())] });
    // h.insert("pen_setPenShadeToNumber", BlockInfo { block_function: crate::core_blocks::pen_setPenShadeToNumber, arguments: vec![(ArgType::Input, "SHADE".into())] });
    // h.insert("pen_changePenSizeBy", BlockInfo { block_function: crate::core_blocks::pen_changePenSizeBy, arguments: vec![(ArgType::Input, "SIZE".into())] });
    // h.insert("pen_setPenSizeTo", BlockInfo { block_function: crate::core_blocks::pen_setPenSizeTo, arguments: vec![(ArgType::Input, "SIZE".into())] });
    // h.insert("videoSensing_videoOn", BlockInfo { block_function: crate::core_blocks::videoSensing_videoOn, arguments: vec![(ArgType::Input, "ATTRIBUTE".into()), (ArgType::Input, "SUBJECT".into())] });
    h.insert(
        "event_whenflagclicked",
        BlockInfo {
            block_function: crate::core_blocks::event_whenflagclicked,
            arguments: vec![],
        },
    );
    h.insert(
        "event_whenkeypressed",
        BlockInfo {
            block_function: crate::core_blocks::event_whenkeypressed,
            arguments: vec![(ArgType::Field, "KEY_OPTION".into())],
        },
    );
    h.insert(
        "event_whenthisspriteclicked",
        BlockInfo {
            block_function: crate::core_blocks::event_whenthisspriteclicked,
            arguments: vec![],
        },
    );
    h.insert(
        "event_whenbackdropswitchesto",
        BlockInfo {
            block_function: crate::core_blocks::event_whenbackdropswitchesto,
            arguments: vec![(ArgType::Field, "BACKDROP".into())],
        },
    );
    h.insert(
        "event_whenbroadcastreceived",
        BlockInfo {
            block_function: crate::core_blocks::event_whenbroadcastreceived,
            arguments: vec![(ArgType::Field, "BROADCAST_OPTION".into())],
        },
    );
    h.insert(
        "event_broadcast",
        BlockInfo {
            block_function: crate::core_blocks::event_broadcast,
            arguments: vec![(ArgType::Input, "BROADCAST_INPUT".into())],
        },
    );
    h.insert(
        "event_broadcastandwait",
        BlockInfo {
            block_function: crate::core_blocks::event_broadcastandwait,
            arguments: vec![(ArgType::Input, "BROADCAST_INPUT".into())],
        },
    );
    h.insert(
        "control_wait",
        BlockInfo {
            block_function: crate::core_blocks::control_wait,
            arguments: vec![(ArgType::Input, "DURATION".into())],
        },
    );
    h.insert(
        "control_repeat",
        BlockInfo {
            block_function: crate::core_blocks::control_repeat,
            arguments: vec![
                (ArgType::Input, "TIMES".into()),
                (ArgType::Input, "SUBSTACK".into()),
            ],
        },
    );
    h.insert(
        "control_forever",
        BlockInfo {
            block_function: crate::core_blocks::control_forever,
            arguments: vec![(ArgType::Input, "SUBSTACK".into())],
        },
    );
    h.insert(
        "control_if",
        BlockInfo {
            block_function: crate::core_blocks::control_if,
            arguments: vec![
                (ArgType::Input, "CONDITION".into()),
                (ArgType::Input, "SUBSTACK".into()),
            ],
        },
    );
    h.insert(
        "control_if_else",
        BlockInfo {
            block_function: crate::core_blocks::control_if_else,
            arguments: vec![
                (ArgType::Input, "CONDITION".into()),
                (ArgType::Input, "SUBSTACK".into()),
                (ArgType::Input, "SUBSTACK2".into()),
            ],
        },
    );
    h.insert(
        "control_wait_until",
        BlockInfo {
            block_function: crate::core_blocks::control_wait_until,
            arguments: vec![(ArgType::Input, "CONDITION".into())],
        },
    );
    h.insert(
        "control_repeat_until",
        BlockInfo {
            block_function: crate::core_blocks::control_repeat_until,
            arguments: vec![
                (ArgType::Input, "CONDITION".into()),
                (ArgType::Input, "SUBSTACK".into()),
            ],
        },
    );
    h.insert(
        "control_while",
        BlockInfo {
            block_function: crate::core_blocks::control_while,
            arguments: vec![
                (ArgType::Input, "CONDITION".into()),
                (ArgType::Input, "SUBSTACK".into()),
            ],
        },
    );
    h.insert(
        "control_for_each",
        BlockInfo {
            block_function: crate::core_blocks::control_for_each,
            arguments: vec![
                (ArgType::Field, "VARIABLE".into()),
                (ArgType::Input, "VALUE".into()),
                (ArgType::Input, "SUBSTACK".into()),
            ],
        },
    );
    h.insert(
        "control_stop",
        BlockInfo {
            block_function: crate::core_blocks::control_stop,
            arguments: vec![(ArgType::Field, "STOP_OPTION".into())],
        },
    );
    h.insert(
        "control_start_as_clone",
        BlockInfo {
            block_function: crate::core_blocks::control_start_as_clone,
            arguments: vec![],
        },
    );
    h.insert(
        "control_create_clone_of",
        BlockInfo {
            block_function: crate::core_blocks::control_create_clone_of,
            arguments: vec![(ArgType::Input, "CLONE_OPTION".into())],
        },
    );
    h.insert(
        "control_create_clone_of_menu",
        BlockInfo {
            block_function: crate::core_blocks::control_create_clone_of_menu,
            arguments: vec![(ArgType::Field, "CLONE_OPTION".into())],
        },
    );
    h.insert(
        "control_delete_this_clone",
        BlockInfo {
            block_function: crate::core_blocks::control_delete_this_clone,
            arguments: vec![],
        },
    );
    h.insert(
        "control_get_counter",
        BlockInfo {
            block_function: crate::core_blocks::control_get_counter,
            arguments: vec![],
        },
    );
    h.insert(
        "control_incr_counter",
        BlockInfo {
            block_function: crate::core_blocks::control_incr_counter,
            arguments: vec![],
        },
    );
    h.insert(
        "control_clear_counter",
        BlockInfo {
            block_function: crate::core_blocks::control_clear_counter,
            arguments: vec![],
        },
    );
    h.insert(
        "control_all_at_once",
        BlockInfo {
            block_function: crate::core_blocks::control_all_at_once,
            arguments: vec![(ArgType::Input, "SUBSTACK".into())],
        },
    );
    // h.insert("sensing_touchingobject", BlockInfo { block_function: crate::core_blocks::sensing_touchingobject, arguments: vec![(ArgType::Input, "TOUCHINGOBJECTMENU".into())] });
    // h.insert("sensing_touchingcolor", BlockInfo { block_function: crate::core_blocks::sensing_touchingcolor, arguments: vec![(ArgType::Input, "COLOR".into())] });
    // h.insert("sensing_coloristouchingcolor", BlockInfo { block_function: crate::core_blocks::sensing_coloristouchingcolor, arguments: vec![(ArgType::Input, "COLOR".into()), (ArgType::Input, "COLOR2".into())] });
    h.insert(
        "sensing_distanceto",
        BlockInfo {
            block_function: crate::core_blocks::sensing_distanceto,
            arguments: vec![(ArgType::Input, "DISTANCETOMENU".into())],
        },
    );
    h.insert(
        "sensing_distancetomenu",
        BlockInfo {
            block_function: crate::core_blocks::sensing_distancetomenu,
            arguments: vec![(ArgType::Field, "DISTANCETOMENU".into())],
        },
    );
    h.insert(
        "sensing_askandwait",
        BlockInfo {
            block_function: crate::core_blocks::sensing_askandwait,
            arguments: vec![(ArgType::Input, "QUESTION".into())],
        },
    );
    h.insert(
        "sensing_answer",
        BlockInfo {
            block_function: crate::core_blocks::sensing_answer,
            arguments: vec![],
        },
    );
    // h.insert("sensing_keypressed", BlockInfo { block_function: crate::core_blocks::sensing_keypressed, arguments: vec![(ArgType::Input, "KEY_OPTION".into())] });
    // h.insert("sensing_mousedown", BlockInfo { block_function: crate::core_blocks::sensing_mousedown, arguments: vec![] });
    // h.insert("sensing_mousex", BlockInfo { block_function: crate::core_blocks::sensing_mousex, arguments: vec![] });
    // h.insert("sensing_mousey", BlockInfo { block_function: crate::core_blocks::sensing_mousey, arguments: vec![] });
    // h.insert("sensing_loudness", BlockInfo { block_function: crate::core_blocks::sensing_loudness, arguments: vec![] });
    // h.insert("sensing_loud", BlockInfo { block_function: crate::core_blocks::sensing_loud, arguments: vec![] });
    // h.insert("videoSensing_videoToggle", BlockInfo { block_function: crate::core_blocks::videoSensing_videoToggle, arguments: vec![(ArgType::Input, "VIDEO_STATE".into())] });
    // h.insert("videoSensing_setVideoTransparency", BlockInfo { block_function: crate::core_blocks::videoSensing_setVideoTransparency, arguments: vec![(ArgType::Input, "TRANSPARENCY".into())] });
    h.insert(
        "sensing_timer",
        BlockInfo {
            block_function: crate::core_blocks::sensing_timer,
            arguments: vec![],
        },
    );
    h.insert(
        "sensing_resettimer",
        BlockInfo {
            block_function: crate::core_blocks::sensing_resettimer,
            arguments: vec![],
        },
    );
    h.insert(
        "sensing_of",
        BlockInfo {
            block_function: crate::core_blocks::sensing_of,
            arguments: vec![
                (ArgType::Field, "PROPERTY".into()),
                (ArgType::Input, "OBJECT".into()),
            ],
        },
    );
    h.insert(
        "sensing_of_object_menu",
        BlockInfo {
            block_function: crate::core_blocks::sensing_of_object_menu,
            arguments: vec![(ArgType::Field, "OBJECT".into())],
        },
    );
    h.insert("sensing_current", BlockInfo { block_function: crate::core_blocks::sensing_current, arguments: vec![(ArgType::Field, "CURRENTMENU".into())] });
    h.insert("sensing_dayssince2000", BlockInfo { block_function: crate::core_blocks::sensing_dayssince2000, arguments: vec![] });
    h.insert("sensing_username", BlockInfo { block_function: crate::core_blocks::sensing_username, arguments: vec![] });
    h.insert("sensing_userid", BlockInfo { block_function: crate::core_blocks::sensing_userid, arguments: vec![] });
    h.insert(
        "operator_add",
        BlockInfo {
            block_function: crate::core_blocks::operator_add,
            arguments: vec![
                (ArgType::Input, "NUM1".into()),
                (ArgType::Input, "NUM2".into()),
            ],
        },
    );
    h.insert(
        "operator_subtract",
        BlockInfo {
            block_function: crate::core_blocks::operator_subtract,
            arguments: vec![
                (ArgType::Input, "NUM1".into()),
                (ArgType::Input, "NUM2".into()),
            ],
        },
    );
    h.insert(
        "operator_multiply",
        BlockInfo {
            block_function: crate::core_blocks::operator_multiply,
            arguments: vec![
                (ArgType::Input, "NUM1".into()),
                (ArgType::Input, "NUM2".into()),
            ],
        },
    );
    h.insert(
        "operator_divide",
        BlockInfo {
            block_function: crate::core_blocks::operator_divide,
            arguments: vec![
                (ArgType::Input, "NUM1".into()),
                (ArgType::Input, "NUM2".into()),
            ],
        },
    );
    h.insert(
        "operator_random",
        BlockInfo {
            block_function: crate::core_blocks::operator_random,
            arguments: vec![
                (ArgType::Input, "FROM".into()),
                (ArgType::Input, "TO".into()),
            ],
        },
    );
    h.insert(
        "operator_lt",
        BlockInfo {
            block_function: crate::core_blocks::operator_lt,
            arguments: vec![
                (ArgType::Input, "OPERAND1".into()),
                (ArgType::Input, "OPERAND2".into()),
            ],
        },
    );
    h.insert(
        "operator_equals",
        BlockInfo {
            block_function: crate::core_blocks::operator_equals,
            arguments: vec![
                (ArgType::Input, "OPERAND1".into()),
                (ArgType::Input, "OPERAND2".into()),
            ],
        },
    );
    h.insert(
        "operator_gt",
        BlockInfo {
            block_function: crate::core_blocks::operator_gt,
            arguments: vec![
                (ArgType::Input, "OPERAND1".into()),
                (ArgType::Input, "OPERAND2".into()),
            ],
        },
    );
    h.insert(
        "operator_and",
        BlockInfo {
            block_function: crate::core_blocks::operator_and,
            arguments: vec![
                (ArgType::Input, "OPERAND1".into()),
                (ArgType::Input, "OPERAND2".into()),
            ],
        },
    );
    h.insert(
        "operator_or",
        BlockInfo {
            block_function: crate::core_blocks::operator_or,
            arguments: vec![
                (ArgType::Input, "OPERAND1".into()),
                (ArgType::Input, "OPERAND2".into()),
            ],
        },
    );
    h.insert(
        "operator_not",
        BlockInfo {
            block_function: crate::core_blocks::operator_not,
            arguments: vec![(ArgType::Input, "OPERAND".into())],
        },
    );
    h.insert(
        "operator_join",
        BlockInfo {
            block_function: crate::core_blocks::operator_join,
            arguments: vec![
                (ArgType::Input, "STRING1".into()),
                (ArgType::Input, "STRING2".into()),
            ],
        },
    );
    h.insert(
        "operator_letter_of",
        BlockInfo {
            block_function: crate::core_blocks::operator_letter_of,
            arguments: vec![
                (ArgType::Input, "LETTER".into()),
                (ArgType::Input, "STRING".into()),
            ],
        },
    );
    h.insert(
        "operator_contains",
        BlockInfo {
            block_function: crate::core_blocks::operator_contains,
            arguments: vec![
                (ArgType::Input, "STRING1".into()),
                (ArgType::Input, "STRING2".into()),
            ],
        },
    );
    h.insert(
        "operator_length",
        BlockInfo {
            block_function: crate::core_blocks::operator_length,
            arguments: vec![(ArgType::Input, "STRING".into())],
        },
    );
    h.insert(
        "operator_mod",
        BlockInfo {
            block_function: crate::core_blocks::operator_mod,
            arguments: vec![
                (ArgType::Input, "NUM1".into()),
                (ArgType::Input, "NUM2".into()),
            ],
        },
    );
    h.insert(
        "operator_round",
        BlockInfo {
            block_function: crate::core_blocks::operator_round,
            arguments: vec![(ArgType::Input, "NUM".into())],
        },
    );
    h.insert(
        "operator_mathop",
        BlockInfo {
            block_function: crate::core_blocks::operator_mathop,
            arguments: vec![
                (ArgType::Field, "OPERATOR".into()),
                (ArgType::Input, "NUM".into()),
            ],
        },
    );
    h.insert(
        "data_variable",
        BlockInfo {
            block_function: crate::core_blocks::data_variable,
            arguments: vec![(ArgType::Field, "VARIABLE".into())],
        },
    );
    h.insert(
        "data_variable",
        BlockInfo {
            block_function: crate::core_blocks::data_variable,
            arguments: vec![(ArgType::Field, "VARIABLE".into())],
        },
    );
    h.insert(
        "data_setvariableto",
        BlockInfo {
            block_function: crate::core_blocks::data_setvariableto,
            arguments: vec![
                (ArgType::Field, "VARIABLE".into()),
                (ArgType::Input, "VALUE".into()),
            ],
        },
    );
    h.insert(
        "data_changevariableby",
        BlockInfo {
            block_function: crate::core_blocks::data_changevariableby,
            arguments: vec![
                (ArgType::Field, "VARIABLE".into()),
                (ArgType::Input, "VALUE".into()),
            ],
        },
    );
    // h.insert("data_showvariable", BlockInfo { block_function: crate::core_blocks::data_showvariable, arguments: vec![(ArgType::Field, "VARIABLE".into())] });
    // h.insert("data_hidevariable", BlockInfo { block_function: crate::core_blocks::data_hidevariable, arguments: vec![(ArgType::Field, "VARIABLE".into())] });
    h.insert(
        "data_listcontents",
        BlockInfo {
            block_function: crate::core_blocks::data_listcontents,
            arguments: vec![(ArgType::Field, "LIST".into())],
        },
    );
    h.insert(
        "data_addtolist",
        BlockInfo {
            block_function: crate::core_blocks::data_addtolist,
            arguments: vec![
                (ArgType::Input, "ITEM".into()),
                (ArgType::Field, "LIST".into()),
            ],
        },
    );
    h.insert(
        "data_deleteoflist",
        BlockInfo {
            block_function: crate::core_blocks::data_deleteoflist,
            arguments: vec![
                (ArgType::Input, "INDEX".into()),
                (ArgType::Field, "LIST".into()),
            ],
        },
    );
    h.insert(
        "data_deletealloflist",
        BlockInfo {
            block_function: crate::core_blocks::data_deletealloflist,
            arguments: vec![(ArgType::Field, "LIST".into())],
        },
    );
    h.insert(
        "data_insertatlist",
        BlockInfo {
            block_function: crate::core_blocks::data_insertatlist,
            arguments: vec![
                (ArgType::Input, "ITEM".into()),
                (ArgType::Input, "INDEX".into()),
                (ArgType::Field, "LIST".into()),
            ],
        },
    );
    h.insert(
        "data_replaceitemoflist",
        BlockInfo {
            block_function: crate::core_blocks::data_replaceitemoflist,
            arguments: vec![
                (ArgType::Input, "INDEX".into()),
                (ArgType::Field, "LIST".into()),
                (ArgType::Input, "ITEM".into()),
            ],
        },
    );
    h.insert(
        "data_itemoflist",
        BlockInfo {
            block_function: crate::core_blocks::data_itemoflist,
            arguments: vec![
                (ArgType::Input, "INDEX".into()),
                (ArgType::Field, "LIST".into()),
            ],
        },
    );
    h.insert(
        "data_lengthoflist",
        BlockInfo {
            block_function: crate::core_blocks::data_lengthoflist,
            arguments: vec![(ArgType::Field, "LIST".into())],
        },
    );
    h.insert(
        "data_itemnumoflist",
        BlockInfo {
            block_function: crate::core_blocks::data_itemnumoflist,
            arguments: vec![
                (ArgType::Field, "LIST".into()),
                (ArgType::Input, "ITEM".into()),
            ],
        },
    );
    h.insert(
        "data_listcontainsitem",
        BlockInfo {
            block_function: crate::core_blocks::data_listcontainsitem,
            arguments: vec![
                (ArgType::Field, "LIST".into()),
                (ArgType::Input, "ITEM".into()),
            ],
        },
    );
    // h.insert("data_showlist", BlockInfo { block_function: crate::core_blocks::data_showlist, arguments: vec![(ArgType::Field, "LIST".into())] });
    // h.insert("data_hidelist", BlockInfo { block_function: crate::core_blocks::data_hidelist, arguments: vec![(ArgType::Field, "LIST".into())] });
    // h.insert("procedures_definition", BlockInfo { block_function: crate::core_blocks::procedures_definition, arguments: vec![] });
    // h.insert("argument_reporter_string_number", BlockInfo { block_function: crate::core_blocks::argument_reporter_string_number, arguments: vec![(ArgType::Field, "VALUE".into())] });
    // h.insert("procedures_call", BlockInfo { block_function: crate::core_blocks::procedures_call, arguments: vec![] });
    h
});
