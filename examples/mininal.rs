

fn main() {
    let args = std::env::args();
    let file_path = args.map(|a| a.as_str().to_owned()).collect::<Vec<String>>();
    if file_path.len() > 1 {
        if let Some(file_path) = file_path.last() {
            let mut vm = clipcc_rust_vm::sb3_loader::create_vm_from_sb3(&file_path);
            vm.start_flag();
            while !vm.is_idle() {
                vm.step();
            }
        } else {
            println!("ScratchRust - DEMO by SteveXMH");
            println!("Usage: scrust [FILE]");
        }
    } else {
        println!("ScratchRust - DEMO by SteveXMH");
        println!("Usage: scrust [FILE]");
    }
}
