#[cfg(test)]
mod tests {

	use rand::Rng;

	use crate::chip8::{CHIP_DIGITS, Chip8};

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
	fn ret() {
		let mut emu = Chip8::new();

		emu.load_code(vec![0x22, 0x04, 0x00, 0xE0, 0x00, 0xEE]);
		emu.tick();
		emu.tick();

		assert_eq!(emu.program_counter, 0x200 + 2, "Did not return to the correct addr");
		assert_eq!(emu.stack_pointer, 0, "Stack pointer is incorrect");
	}

	#[test]
	fn skip_eq() {
		let mut emu = Chip8::new();
		emu.registers[0x3] = 0x33;

		//Skip
		emu.load_code(vec![0x33, 0x33]);
		emu.tick();
		assert_eq!(emu.program_counter, 0x200 + 4);

		//Dont Skip
		emu.load_code(vec![0x33, 0x35]).tick();
		assert_eq!(emu.program_counter, 0x200 + 2);
	}

	#[test]
	fn skip_ne() {
		let mut emu = Chip8::new();
		emu.registers[0x3] = 0x33;

		//Dont Skip
		emu.load_code(vec![0x43, 0x33]);
		emu.tick();
		assert_eq!(emu.program_counter, 0x200 + 2);

		//Skip
		emu.load_code(vec![0x43, 0x35]).tick();
		assert_eq!(emu.program_counter, 0x200 + 4);
	}

	#[test]
	fn skip_eq2() {
		let mut emu = Chip8::new();
		emu.registers[0x3] = 0x33;
		emu.registers[0x4] = 0x33;

		//Skip
		emu.load_code(vec![0x53, 0x40]);
		emu.tick();
		assert_eq!(emu.program_counter, 0x200 + 4);

		//Dont Skip
		emu.load_code(vec![0x53, 0x20]).tick();
		assert_eq!(emu.program_counter, 0x200 + 2);
	}

	#[test]
	fn set_register() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0x63, 0x40]).tick();

		assert_eq!(emu.registers[0x3], 0x40);
	}

	#[test]
	fn add() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0x72, 0x70]);
		emu.registers[0x2] = 0x01;
		emu.tick();

		assert_eq!(emu.registers[0x2], 0x01 + 0x70);
	}

	#[test]
	fn math_copy() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0x82, 0x70]);
		emu.registers[0x2] = 0x10;
		emu.registers[0x7] = 0x33;
		emu.tick();
		assert_eq!(emu.registers[0x7], 0x33, "Original Value Modified");
		assert_eq!(emu.registers[0x2], 0x33, "Value not copied");
	}

	#[test]
	fn math_bit_or() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0x82, 0x71]);
		emu.registers[0x2] = 0x10;
		emu.registers[0x7] = 0x33;
		emu.tick();
		assert_eq!(emu.registers[0x2], 0x10 | 0x33);
	}

	#[test]
	fn math_bit_and() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0x82, 0x72]);
		emu.registers[0x2] = 0x10;
		emu.registers[0x7] = 0x33;
		emu.tick();
		assert_eq!(emu.registers[0x2], 0x10 & 0x33);
	}

	#[test]
	fn math_bit_xor() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0x82, 0x73]);
		emu.registers[0x2] = 0x10;
		emu.registers[0x7] = 0x33;
		emu.tick();
		assert_eq!(emu.registers[0x2], 0x10 ^ 0x33);
	}

	#[test]
	fn math_bit_add() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0x82, 0x74]);
		emu.registers[0x2] = 0x10;
		emu.registers[0x7] = 0x33;
		emu.tick();
		assert_eq!(emu.registers[0x2], 0x10 + 0x33);
		assert_eq!(emu.registers[0xF], 0, "VF incorrectly set");
	}

	#[test]
	fn math_bit_add_carry() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0x82, 0x74]);
		emu.registers[0x2] = 0xF0;
		emu.registers[0x7] = 0x33;
		emu.tick();
		let r: u16 = (0xF0 + 0x33) & 0x00FF;
		assert_eq!(emu.registers[0x2], r as u8);
		assert_eq!(emu.registers[0xF], 1, "VF incorrectly set");
	}

	#[test]
	fn math_bit_sub() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0x82, 0x75]);
		emu.registers[0x2] = 0x10;
		emu.registers[0x7] = 0x33;
		emu.tick();
		assert_eq!(emu.registers[0x2], u8::wrapping_sub(0x10, 0x33));
		assert_eq!(emu.registers[0xF], 0, "VF incorrectly set");
	}
	#[test]
	fn math_bit_sub_borrow() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0x82, 0x75]);
		emu.registers[0x2] = 0xF0;
		emu.registers[0x7] = 0x33;
		emu.tick();
		assert_eq!(emu.registers[0x2], u8::wrapping_sub(0xF0, 0x33));
		assert_eq!(emu.registers[0xF], 1, "VF incorrectly set");
	}

	#[test]
	fn math_bit_shr() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0x82, 0x06]);
		emu.registers[0x2] = 0x10;
		emu.tick();
		assert_eq!(emu.registers[0x2], 0x10 >> 1);
		assert_eq!(emu.registers[0xF], 0);
	}

	#[test]
	fn math_bit_shr2() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0x82, 0x06]);
		emu.registers[0x2] = 0x11;
		emu.tick();
		assert_eq!(emu.registers[0x2], 0x11 >> 1);
		assert_eq!(emu.registers[0xF], 1);
	}

	#[test]
	fn math_bit_subn_borrow() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0x82, 0x77]);
		emu.registers[0x2] = 0x10;
		emu.registers[0x7] = 0x33;
		emu.tick();
		assert_eq!(emu.registers[0x2], u8::wrapping_sub(0x10, 0x33));
		assert_eq!(emu.registers[0xF], 1, "VF incorrectly set");
	}
	#[test]
	fn math_bit_subn() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0x82, 0x77]);
		emu.registers[0x2] = 0xF0;
		emu.registers[0x7] = 0x33;
		emu.tick();
		assert_eq!(emu.registers[0x2], u8::wrapping_sub(0xF0, 0x33));
		assert_eq!(emu.registers[0xF], 0, "VF incorrectly set");
	}

	#[test]
	fn math_bit_shl() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0x82, 0x0E]);
		emu.registers[0x2] = 0x78;
		emu.tick();
		assert_eq!(emu.registers[0x2], 0x78 << 1);
		assert_eq!(emu.registers[0xF], 0, "VF incorrectly set");
	}

	#[test]
	fn math_bit_shl2() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0x82, 0x0E]);
		emu.registers[0x2] = 0x83;
		emu.tick();
		assert_eq!(emu.registers[0x2], 0x83 << 1);
		assert_eq!(emu.registers[0xF], 1, "VF incorrectly set");
	}

	#[test]
	fn skip_ne2() {
		let mut emu = Chip8::new();
		emu.registers[0x3] = 0x33;
		emu.registers[0x4] = 0x33;

		//Skip
		emu.load_code(vec![0x93, 0x20]);
		emu.tick();
		assert_eq!(emu.program_counter, 0x200 + 4);

		//Dont Skip
		emu.load_code(vec![0x93, 0x40]).tick();
		assert_eq!(emu.program_counter, 0x200 + 2);
	}

	#[test]
	fn set_i() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0xA3, 0x20]);
		emu.tick();
		assert_eq!(emu.reg_i, 0x320);
	}

	#[test]
	fn jump_offset() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0xB3, 0x20]);
		emu.registers[0x0] = 0x4;
		emu.tick();
		assert_eq!(emu.program_counter, 0x320 + 0x4);
	}

	#[test]
	fn rand() {
		//todo replace with seeded rng
		let mut emu = Chip8::new();
		let mut rng = emu.rng.clone();
		emu.load_code(vec![0xC3, 0x20]);
		emu.tick();
		let r: u8 = rng.random();
		assert_eq!(emu.registers[0x3], r & 0x20);
	}

	#[test]
	fn draw_sprite() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0xD3, 0x11]);
		emu.registers[0x3] = 62;
		//Draw a line at (62,0) wrapping around
		emu.tick();
		let expected = (u64::rotate_left(CHIP_DIGITS[0] as u64, 58) as u128) << 64;
		println!("{:b}", emu.display[0]);
		assert_eq!(emu.display[0], expected);
		assert_eq!(emu.registers[0xF], 0, "VF incorrectly set");
	}

	#[test]
	fn draw_sprite_erase() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0xD3, 0x11, 0xD4, 0x11]);
		emu.registers[0x3] = 62;
		emu.registers[0x4] = 0;
		//Draw a line at (62,0) wrapping around
		emu.tick();
		emu.tick();
		let expected = (3458764513820540931 as u128) << 64;
		println!("{:b}", expected);
		println!("{:b}", emu.display[0]);
		assert_eq!(emu.display[0], expected);
		assert_eq!(emu.registers[0xF], 1, "VF incorrectly set");
	}

	#[test]
	fn draw_sprite_highres() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0xD3, 0x11]);
		emu.registers[0x3] = 126;
		emu.high_res = true;
		//Draw a line at (62,0) wrapping around
		emu.tick();
		let expected = u128::rotate_left(CHIP_DIGITS[0] as u128, 122);
		assert_eq!(emu.display[0], expected);
		assert_eq!(emu.registers[0xF], 0, "VF incorrectly set");
	}

	#[test]
	fn read_dt() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0xF3, 0x07]);
		emu.reg_dt = 0x3;
		emu.tick();
		assert_eq!(emu.registers[0x3], 0x3);
	}

	#[test]
	fn set_dt() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0xF3, 0x15]);
		emu.registers[0x3] = 0x3;
		emu.tick();
		assert_eq!(emu.reg_dt, 0x3);
	}

	#[test]
	fn set_st() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0xF3, 0x18]);
		emu.registers[0x3] = 0x3;
		emu.tick();
		assert_eq!(emu.reg_st, 0x3);
	}

	#[test]
	fn set_vi() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0xF3, 0x1E]);
		emu.registers[0x3] = 0x3;
		emu.tick();
		assert_eq!(emu.reg_i, 0x3);
	}

	#[test]
	fn set_vi_to_digit() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0xF3, 0x29]);
		emu.registers[0x3] = 0x3;
		emu.tick();
		assert_eq!(emu.reg_i, 0x3 * 5);
	}

	#[test]
	fn set_bcd() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0xF3, 0x33]);
		emu.registers[0x3] = 128;
		emu.reg_i = 0x300;
		emu.tick();
		assert_eq!(emu.ram[emu.reg_i as usize], 1, "Digit 1 is incorrect");
		assert_eq!(emu.ram[emu.reg_i as usize + 1], 2, "Digit 2 is incorrect");
		assert_eq!(emu.ram[emu.reg_i as usize + 2], 8, "Digit 3 is incorrect");
	}

	#[test]
	fn store() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0xF4, 0x55]);
		for i in 0..0x4 {
			emu.registers[i] = i as u8;
		}
		emu.registers[0x5] = 0x9;
		emu.reg_i = 0x300;
		emu.tick();
		for i in 0..0x4 {
			assert_eq!(emu.ram[0x300 + i], i as u8);
		}
		assert_eq!(emu.ram[emu.reg_i as usize + 0x5], 0, "Wrote too much");
		assert_eq!(emu.reg_i, 0x300 + 0x4 + 1, "Register I was not incremented");
	}

	#[test]
	fn read() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0xF4, 0x65]);
		emu.reg_i = 0x300;
		for i in 0..0x4 {
			emu.ram[emu.reg_i as usize + i] = i as u8;
		}
		emu.ram[emu.reg_i as usize + 0x5] = 0x9;
		emu.tick();
		for i in 0..0x3 {
			assert_eq!(emu.registers[i], i as u8);
		}
		assert_eq!(emu.registers[0x5], 0, "Wrote too much");
		assert_eq!(emu.reg_i, 0x300 + 0x4 + 1, "Register I was not incremented");
	}

	#[test]
	fn bcd_and_read() {
		let mut emu = Chip8::new();
		emu.load_code(vec![0xF2, 0x33, 0xF2, 0x65]);
		emu.registers[0x2] = 128;
		emu.reg_i = 0x300;
		emu.tick();
		emu.tick();
		
		assert_eq!(emu.registers[0x0], 1, "First Digit");
		assert_eq!(emu.registers[0x1], 2, "Second Digit");
		assert_eq!(emu.registers[0x2], 8, "Third Digit");
	}
}
