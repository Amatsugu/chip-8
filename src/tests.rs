#[cfg(test)]
mod tests {
	use std::f32::consts::E;

	use crate::chip8::Chip8;

	#[test]
	fn jump() {
		let mut emu = Chip8::new();

		emu.load_code(vec![0x13, 0x45]).tick();

		assert_eq!(emu.program_counter, 0x345);
	}

	#[test]
	fn call() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0x23, 0x55]).tick();

		assert_eq!(emu.program_counter, 0x355);
		assert_eq!(emu.stack_pointer, 1);
		assert_eq!(emu.stack[emu.stack_pointer], 0x200);
	}

	#[test]
	fn skip_eq() {
		let mut emu = Chip8::new();
		emu.registers[0x3] = 0x33;

		//Skip
		emu.load_code(vec![0x33, 0x33]);
		emu.tick();
		assert_eq!(emu.program_counter, 0x200 + 2);

		//Dont Skip
		emu.load_code(vec![0x33, 0x35]).tick();
		assert_eq!(emu.program_counter, 0x200 + 1);
	}

	#[test]
	fn skip_ne() {
		let mut emu = Chip8::new();
		emu.registers[0x3] = 0x33;

		//Dont Skip
		emu.load_code(vec![0x43, 0x33]);
		emu.tick();
		assert_eq!(emu.program_counter, 0x200 + 1);

		//Skip
		emu.load_code(vec![0x43, 0x35]).tick();
		assert_eq!(emu.program_counter, 0x200 + 2);
	}

	#[test]
	fn skip_eq2() {
		let mut emu = Chip8::new();
		emu.registers[0x3] = 0x33;
		emu.registers[0x4] = 0x33;

		//Skip
		emu.load_code(vec![0x53, 0x40]);
		emu.tick();
		assert_eq!(emu.program_counter, 0x200 + 2);

		//Dont Skip
		emu.load_code(vec![0x53, 0x20]).tick();
		assert_eq!(emu.program_counter, 0x200 + 1);
	}
}
