//! A module to test all the projects comes from offical llk/scratch-vm repo.

use std::time::Instant;

#[test]
fn test_all() {
    use crate::blocks::*;
    use crate::*;

    fn run_test(file: &str) -> Vec<String> {
        let mut vm = crate::sb3_loader::create_vm_from_sb3(file);
        let timer = Instant::now();
        vm.start_flag();
        while !vm.is_idle() || timer.elapsed().as_secs() < 10 {
            vm.step();
        }
        return if let Some(x) = vm
            .global_data
            .get("SAY_TEXT")
            .and_then(|x| x.downcast_ref::<Vec<String>>())
        {
            x.to_owned()
        } else if timer.elapsed().as_secs() > 10 {
            vec!["fail test time out".to_owned()]
        } else {
            vec![]
        };
    }

    fn collect_say(ctx: &mut BlockContext) -> BlockResult {
        let text = ctx.stack.arguments.get(0);
        if let Some(text) = text {
            println!("{}", text);
            if let Some(x) = ctx
                .global_data
                .get_mut("SAY_TEXT")
                .and_then(|x| x.downcast_mut::<Vec<String>>())
            {
                x.push(text.to_string());
            } else {
                ctx.global_data
                    .insert("SAY_TEXT".to_string(), Box::new(vec![text.to_string()]));
            }
            BlockResult::Resolved(None)
        } else {
            BlockResult::ResolveArgument(0)
        }
    }

    register_block(
        "looks_say",
        BlockInfo {
            block_function: collect_say,
            arguments: vec![(ArgType::Input, "MESSAGE".into())],
        },
    );

    let test_dir = format!("{}/test", env!("CARGO_MANIFEST_DIR"));
    let files = std::fs::read_dir(test_dir).unwrap();
    let mut tested = 0;
    let mut failed = false;
    for file in files.flatten() {
        tested += 1;
        let file_path = file.path();
        if file_path.extension().unwrap_or_default() == "sb3" {
            println!("Testing {} {}", tested, file_path.to_str().unwrap());
            let results = run_test(file_path.to_str().unwrap());
            for result in results {
                if result.starts_with("fail ") {
                    failed = true;
                    println!("{}", result);
                }
            }
        }
    }
    if failed {
        panic!("Some tests failed");
    }
}
