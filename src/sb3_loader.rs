use crate::blocks::get_blockinfo;
use crate::*;
use json::*;
use std::collections::HashSet;
use std::io::Read;
use std::path::Path;
use zip::*;

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

pub fn load_project(vm: &mut VirtualMachine, project: &JsonValue) {
    let mut unknown_opcodes = HashSet::new();
    fn json_value_to_block_value(v: &JsonValue) -> BlockValue {
        if v.is_string() {
            BlockValue::String(v.as_str().unwrap().to_owned())
        } else if v.is_number() {
            BlockValue::Number(v.as_f64().unwrap())
        } else if v.is_boolean() {
            BlockValue::Boolean(v[1].as_bool().unwrap_or(false))
        } else {
            dbg!(v);
            unimplemented!();
        }
    }
    fn setup_target(
        target: &mut Target,
        target_json: &JsonValue,
        unknown_opcodes: &mut HashSet<String>,
    ) {
        // BlockId, ArgumentIds, ArgumentNames
        type ProcBlockMap = HashMap<String, (BlockId, Vec<String>, Vec<String>)>;
        let mut procedures_block: ProcBlockMap = HashMap::with_capacity(32);
        fn register_procedure(
            procedures_block: &mut ProcBlockMap,
            block_id: BlockId,
            blocks: &JsonValue,
            block: &JsonValue,
        ) {
            // procedures_prototype
            if let Some(prototype_bid) = block["inputs"]["custom_block"][1].as_str() {
                let proccode = blocks[prototype_bid]["mutation"]["proccode"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned();
                let argumentids = blocks[prototype_bid]["mutation"]["argumentids"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned();
                let argumentnames = blocks[prototype_bid]["mutation"]["argumentnames"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned();
                if !(proccode.is_empty() || argumentids.is_empty() || argumentnames.is_empty()) {
                    let argumentids = json::parse(&argumentids)
                        .unwrap()
                        .members()
                        .filter_map(|v| v.as_str())
                        .map(|x| x.to_owned())
                        .collect::<Vec<String>>();
                    let argumentnames = json::parse(&argumentnames)
                        .unwrap()
                        .members()
                        .filter_map(|v| v.as_str())
                        .map(|x| x.to_owned())
                        .collect::<Vec<String>>();
                    procedures_block.insert(proccode, (block_id, argumentids, argumentnames));
                }
            }
        }
        target.name = target_json["name"].as_str().unwrap().to_owned();
        target.x = target_json["x"].as_f64().unwrap_or(0.);
        target.y = target_json["y"].as_f64().unwrap_or(0.);
        target.size = target_json["size"].as_f64().unwrap_or(100.);
        target.visible = target_json["visible"].as_bool().unwrap_or(true);
        target.direction = target_json["direction"].as_f64().unwrap_or(90.);
        target.current_costume = target_json["currentCostume"].as_usize().unwrap_or(0);
        target.layer_order = target_json["layerOrder"].as_usize().unwrap_or(0);
        target.costumes = target_json["costumes"]
            .members()
            .map(|v| v["name"].as_str().unwrap().to_owned())
            .collect::<Vec<String>>();
        target.sounds = target_json["sounds"]
            .members()
            .map(|v| v["name"].as_str().unwrap().to_owned())
            .collect::<Vec<String>>();
        // variables
        for (_vid, variable) in target_json["variables"].entries() {
            let name = variable[0].as_str().unwrap().to_owned();
            let value: BlockValue = json_value_to_block_value(&variable[1]);
            target.variables.insert(name, value);
        }
        // lists
        for (_lid, list) in target_json["lists"].entries() {
            let name = list[0].as_str().unwrap().to_owned();
            let mut value = Vec::with_capacity(list[1].len());
            for v in list[1].members() {
                value.push(json_value_to_block_value(v));
            }
            target.lists.insert(name, value);
        }
        fn parse_input_to_block_value(blocks: &mut Arena<Block>, input: &JsonValue) -> BlockValue {
            if input.is_string() {
                // Block id, is should be parsed ahead of time
                BlockValue::Undefined
            } else if input.is_array() {
                if input.len() >= 2 {
                    if let Some(id) = input[0].as_u8() {
                        match id {
                            4 | 5 | 6 | 7 | 8 | 9 | 10 => match &input[1] {
                                JsonValue::Number(_v) => input[1].as_f64().unwrap_or(0.0).into(),
                                JsonValue::String(v) => BlockValue::String(v.to_owned()),
                                JsonValue::Short(v) => BlockValue::String(v.as_str().to_owned()),
                                JsonValue::Boolean(v) => BlockValue::Boolean(*v),
                                _ => BlockValue::Undefined,
                            },
                            11 => match &input[2] {
                                JsonValue::String(v) => BlockValue::String(v.to_owned()),
                                JsonValue::Short(v) => BlockValue::String(v.as_str().to_owned()),
                                _ => BlockValue::Undefined,
                            },
                            12 | 13 => {
                                // Variable or List
                                // Create a virtual block
                                let vid = match &input[2] {
                                    JsonValue::String(v) => v.to_owned(),
                                    JsonValue::Short(v) => v.as_str().to_owned(),
                                    _ => "".into(),
                                };
                                if vid.is_empty() {
                                    BlockValue::Undefined
                                } else {
                                    match id {
                                        12 => {
                                            // Variable
                                            let bid = blocks.alloc_with_id(|id| Block {
                                                self_id: id,
                                                arguments: vec![vid.into()],
                                                block_function: crate::core_blocks::data_variable,
                                                next: None,
                                                opcode: "data_variable",
                                                toplevel: false,
                                                #[cfg(debug_assertions)]
                                                block_id: "[Auto Generated]".into(),
                                            });
                                            BlockValue::BlockId(bid)
                                        }
                                        13 => {
                                            // List
                                            let bid = blocks.alloc_with_id(|id| Block {
                                                self_id: id,
                                                arguments: vec![vid.into()],
                                                block_function:
                                                    crate::core_blocks::data_listcontents,
                                                next: None,
                                                opcode: "data_variable",
                                                toplevel: false,
                                                #[cfg(debug_assertions)]
                                                block_id: "[Auto Generated]".into(),
                                            });
                                            BlockValue::BlockId(bid)
                                        }
                                        _ => BlockValue::Undefined,
                                    }
                                }
                            }
                            _ => BlockValue::Undefined,
                        }
                    } else {
                        BlockValue::Undefined
                    }
                } else {
                    BlockValue::Undefined
                }
            } else {
                BlockValue::Undefined
            }
        }
        fn parse_block(
            target_json: &JsonValue,
            block_meta: &JsonValue,
            blocks: &mut id_arena::Arena<Block>,
            prev_id: Option<BlockId>,
            procedures_block: &ProcBlockMap,
            procedure: &str,
            block_id: &str,
            unknown_opcodes: &mut HashSet<String>,
        ) -> BlockId {
            let mut prev_id = prev_id;
            let mut block_meta = block_meta;
            let mut top_block = None;
            loop {
                let opcode = block_meta["opcode"].as_str().unwrap_or_default().to_owned();
                if let Some(opcode_meta) = get_blockinfo(opcode.as_str()) {
                    let bid = blocks.alloc_with_id(|id| Block {
                        self_id: id,
                        arguments: vec![],
                        block_function: opcode_meta.block_function,
                        next: None,
                        opcode: string_to_static_str(block_meta["opcode"].as_str().unwrap().to_owned()),
                        toplevel: block_meta["topLevel"].as_bool().unwrap_or(false),
                        #[cfg(debug_assertions)]
                        block_id: block_id.into(),
                    });
                    if top_block.is_none() {
                        top_block = Some(bid);
                    }
                    if let Some(prev_id) = prev_id {
                        if let Some(b) = blocks.get_mut(prev_id) {
                            b.next = Some(bid)
                        }
                    }
                    let arguments = opcode_meta
                        .arguments
                        .iter()
                        .map(|(arg_type, arg_name)| match arg_type {
                            blocks::ArgType::Input => {
                                let input = &block_meta["inputs"][arg_name][1];
                                if input.is_string() {
                                    // Block Id
                                    let block_id = input.as_str().unwrap();
                                    let block_meta = &target_json["blocks"][block_id];
                                    BlockValue::BlockId(parse_block(
                                        target_json,
                                        block_meta,
                                        blocks,
                                        None,
                                        procedures_block,
                                        procedure,
                                        block_id,
                                        unknown_opcodes,
                                    ))
                                } else {
                                    parse_input_to_block_value(blocks, input)
                                }
                            }
                            blocks::ArgType::Field => {
                                let value = &block_meta["fields"][arg_name][0];
                                let id = &block_meta["fields"][arg_name][1];
                                if id.is_string() {
                                    id.as_str().unwrap().to_owned().into()
                                } else {
                                    json_value_to_block_value(value)
                                }
                            }
                        })
                        .collect();
                    blocks.get_mut(bid).unwrap().arguments = arguments;
                    if block_meta["next"].is_string() {
                        prev_id = Some(bid);
                        let next_meta_id = block_meta["next"].as_str().unwrap();
                        block_meta = &target_json["blocks"][next_meta_id];
                    } else {
                        return top_block.unwrap();
                    }
                } else if opcode.as_str() == "argument_reporter_string_number"
                    || opcode.as_str() == "argument_reporter_boolean"
                {
                    let value = block_meta["fields"]["VALUE"][0]
                        .as_str()
                        .unwrap_or_default();
                    let argument = if let Some(procmeta) = procedures_block.get(procedure) {
                        if let Some((argid, _)) = procmeta
                            .2
                            .iter()
                            .enumerate()
                            .find(|x| x.1.as_str() == value)
                        {
                            argid.into()
                        } else {
                            usize::MAX.into()
                        }
                    } else {
                        usize::MAX.into()
                    };
                    let bid = blocks.alloc_with_id(|bid| Block {
                        self_id: bid,
                        toplevel: false,
                        arguments: vec![argument],
                        opcode: if opcode.as_str() == "argument_reporter_string_number" {
                            "argument_reporter_string_number"
                        } else {
                            "argument_reporter_boolean"
                        },
                        block_function: if opcode.as_str() == "argument_reporter_string_number" {
                            core_blocks::argument_reporter_string_number
                        } else {
                            core_blocks::argument_reporter_boolean
                        },
                        #[cfg(debug_assertions)]
                        block_id: block_id.into(),
                        next: None,
                    });
                    return bid;
                } else if opcode.as_str() == "procedures_call" {
                    let proccode = block_meta["mutation"]["proccode"]
                        .as_str()
                        .unwrap_or_default();
                    if let Some(procmeta) = procedures_block.get(proccode) {
                        let bid = blocks.alloc_with_id(|id| Block {
                            self_id: id,
                            arguments: vec![],
                            block_function: core_blocks::procedures_call,
                            next: None,
                            opcode: "procedures_call",
                            toplevel: block_meta["topLevel"].as_bool().unwrap_or(false),
                            #[cfg(debug_assertions)]
                            block_id: block_id.into(),
                        });
                        if top_block.is_none() {
                            top_block = Some(bid);
                        }
                        if let Some(prev_id) = prev_id {
                            if let Some(b) = blocks.get_mut(prev_id) {
                                b.next = Some(bid)
                            }
                        }
                        let arguments = procmeta
                            .1
                            .iter()
                            .map(|arg_id| {
                                let input = &block_meta["inputs"][arg_id][1];
                                if input.is_string() {
                                    // Block Id
                                    let block_id = input.as_str().unwrap();
                                    let block_meta = &target_json["blocks"][block_id];
                                    BlockValue::BlockId(parse_block(
                                        target_json,
                                        block_meta,
                                        blocks,
                                        None,
                                        procedures_block,
                                        procedure,
                                        block_id,
                                        unknown_opcodes,
                                    ))
                                } else {
                                    parse_input_to_block_value(blocks, input)
                                }
                            })
                            .chain(Some(BlockValue::BlockId(procmeta.0)))
                            .collect();
                        blocks.get_mut(bid).unwrap().arguments = arguments;
                        if block_meta["next"].is_string() {
                            prev_id = Some(bid);
                            let next_meta_id = block_meta["next"].as_str().unwrap();
                            block_meta = &target_json["blocks"][next_meta_id];
                        } else {
                            return top_block.unwrap();
                        }
                    } else if block_meta["next"].is_string() {
                        let next_meta_id = block_meta["next"].as_str().unwrap();
                        block_meta = &target_json["blocks"][next_meta_id];
                    } else {
                        return blocks.alloc_with_id(|id| Block {
                            block_function: crate::blocks::noop,
                            opcode: "",
                            arguments: vec![],
                            self_id: id,
                            next: None,
                            toplevel: false,
                            #[cfg(debug_assertions)]
                            block_id: block_id.into(),
                        });
                    }
                } else if block_meta["next"].is_string() {
                    // println!("WARN: Unknown opcode: {}", opcode);
                    unknown_opcodes.insert(opcode.clone());
                    let next_meta_id = block_meta["next"].as_str().unwrap();
                    block_meta = &target_json["blocks"][next_meta_id];
                } else {
                    // println!("WARN: Unknown opcode: {}", opcode);
                    unknown_opcodes.insert(opcode.clone());
                    return blocks.alloc_with_id(|id| Block {
                        block_function: crate::blocks::noop,
                        opcode: "",
                        arguments: vec![],
                        self_id: id,
                        next: None,
                        toplevel: false,
                        #[cfg(debug_assertions)]
                        block_id: block_id.into(),
                    });
                }
            }
        }

        // Register Procedure Blocks
        for (block_meta_id, block_meta) in target_json["blocks"].entries() {
            if block_meta["topLevel"].as_bool().unwrap_or(false)
                && (block_meta["opcode"].as_str().unwrap_or("") == "procedures_definition"
                    || block_meta["opcode"].as_str().unwrap_or("")
                        == "procedures_return_definition")
            {
                let block_id = target.blocks.alloc_with_id(|id| Block {
                    self_id: id,
                    toplevel: true,
                    arguments: vec![],
                    opcode: match block_meta["opcode"].as_str().unwrap_or("") {
                        "procedures_definition" => "procedures_definition",
                        "procedures_return_definition" => "procedures_return_definition",
                        _ => unreachable!(),
                    },
                    block_function: match block_meta["opcode"].as_str().unwrap_or("") {
                        "procedures_definition" => core_blocks::procedures_definition,
                        "procedures_return_definition" => core_blocks::procedures_definition_return,
                        _ => unreachable!(),
                    },
                    next: None,
                    #[cfg(debug_assertions)]
                    block_id: block_meta_id.into(),
                });

                register_procedure(
                    &mut procedures_block,
                    block_id,
                    &target_json["blocks"],
                    block_meta,
                );
            }
        }

        // Blocks
        for (block_meta_id, block_meta) in target_json["blocks"].entries() {
            if block_meta["topLevel"].as_bool().unwrap_or(false) {
                let opcode = block_meta["opcode"].as_str().unwrap_or_default().to_owned();
                match opcode.as_str() {
                    "procedures_definition" | "procedures_return_definition" => {
                        let proccode = target_json["blocks"][block_meta["inputs"]["custom_block"]
                            [1]
                        .as_str()
                        .unwrap_or_default()]["mutation"]["proccode"]
                            .as_str()
                            .unwrap_or_default()
                            .to_owned();
                        if !proccode.is_empty() {
                            let procmeta = procedures_block.get(&proccode).unwrap();
                            let blockid = procmeta.0;
                            let block_meta = &target_json["blocks"]
                                [block_meta["next"].as_str().unwrap_or_default()];
                            parse_block(
                                target_json,
                                block_meta,
                                &mut target.blocks,
                                Some(blockid),
                                &procedures_block,
                                proccode.as_str(),
                                block_meta_id,
                                unknown_opcodes,
                            );
                        }
                    }
                    "" => {}
                    _ => {
                        parse_block(
                            target_json,
                            block_meta,
                            &mut target.blocks,
                            None,
                            &procedures_block,
                            "",
                            block_meta_id,
                            unknown_opcodes,
                        );
                    }
                }
            }
        }
    }
    for target_json in project["targets"].members() {
        if target_json["isStage"].as_bool().unwrap() {
            let stage = vm.targets.get_mut(vm.stage_id).unwrap();
            setup_target(stage, target_json, &mut unknown_opcodes);
            vm.resync_stage();
        } else {
            let mut target = Target::default();
            setup_target(&mut target, target_json, &mut unknown_opcodes);
            vm.new_target(target);
        }
    }
    let mut unknown_opcodes = unknown_opcodes.into_iter().collect::<Vec<_>>();
    unknown_opcodes.sort();
    for unknown_opcode in unknown_opcodes {
        println!("WARN: Unknown opcode: {}", unknown_opcode);
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = createVMFromSb3Data))]
pub fn create_vm_from_sb3_data(data: &[u8]) -> VirtualMachine {
    let mut vm = VirtualMachine::default();
    let r = std::io::Cursor::new(data);
    let mut file = ZipArchive::new(r).unwrap();
    let mut project_file = file.by_name("project.json").unwrap();
    let mut json_file = String::with_capacity(project_file.size() as _);
    project_file.read_to_string(&mut json_file).unwrap();
    let project = json::parse(&json_file).unwrap();
    load_project(&mut vm, &project);
    vm
}
pub fn create_vm_from_sb3(file_path: impl AsRef<Path>) -> VirtualMachine {
    let mut vm = VirtualMachine::default();
    let r = std::fs::OpenOptions::new()
        .read(true)
        .open(file_path.as_ref())
        .unwrap();
    let mut file = ZipArchive::new(r).unwrap();
    let mut project_file = file.by_name("project.json").unwrap();
    let mut json_file = String::with_capacity(project_file.size() as _);
    project_file.read_to_string(&mut json_file).unwrap();
    let project = json::parse(&json_file).unwrap();
    load_project(&mut vm, &project);
    vm
}
