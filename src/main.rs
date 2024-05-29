mod modules;

use modules::debugger::*;
use modules::vm::*;
use modules::other::*;

fn main() {
    if check_debugger() {
        println!("debugger detected :3");
        std::process::exit(1);
    } else {
        println!("no debugger detected :3");
    }
    if check_vm() {
        println!("running inside a virtual machine 3:<");
        std::process::exit(1);
    } else {
        println!("not running inside a virtual machine :3");
    }
    if check_network() {
        println!("network did something :3");
        std::process::exit(1);
    } else {
        println!("network check didnt do anything :(");
    }
}
