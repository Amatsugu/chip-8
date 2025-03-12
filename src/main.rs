pub mod chip8;
pub mod tests;

use std::{env, fs};

use chip8::Chip8;

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() < 2 {
		println!("No file provided");
		return;
	}
	let path = &args[1];
	let file = fs::read(path);

	let bytes = file.expect("Failed to read file");

	let mut emu = Chip8::new();
	emu.load_code(bytes);
	emu.run(9000);

	emu.print_display();

	println!("PC: {}", emu.program_counter);
}
