use std::time::SystemTime;

use rand::prelude::*;

pub struct Chip8 {
	pub program_counter: usize,
	pub stack_pointer: usize,
	pub registers: [u8; 16],
	pub reg_i: u16,
	pub ram: Vec<u8>,
	pub stack: [u16; 16],

	pub keys: [bool; 16],
	pub display: [u128; 64],

	pub reg_st: u8,
	pub reg_dt: u8,
	pub is_halted: bool,
	pub rng: ThreadRng,
	pub high_res: bool,

	pub need_draw: bool,

	timer: SystemTime,
}

impl Default for Chip8 {
	fn default() -> Self {
		Self {
			program_counter: Default::default(),
			stack_pointer: Default::default(),
			registers: Default::default(),
			reg_i: Default::default(),
			ram: Default::default(),
			stack: Default::default(),
			display: [0; 64],
			reg_st: Default::default(),
			reg_dt: Default::default(),
			is_halted: Default::default(),
			rng: rand::rng(),
			need_draw: false,
			high_res: false,
			keys: Default::default(),
			timer: SystemTime::now(),
		}
	}
}

pub const CHIP_DIGITS: [u8; 16 * 5] = [
	0xF0, 0x90, 0x90, 0x90, 0xF0, //0
	0x20, 0x60, 0x20, 0x20, 0x70, //1
	0xF0, 0x10, 0xF0, 0x80, 0xF0, //2
	0xF0, 0x10, 0xF0, 0x10, 0xF0, //3
	0x90, 0x90, 0xF0, 0x10, 0x10, //4
	0xF0, 0x80, 0xF0, 0x10, 0xF0, //5
	0xF0, 0x80, 0xF0, 0x90, 0xF0, //6
	0xF0, 0x10, 0x20, 0x40, 0x40, //7
	0xF0, 0x90, 0xF0, 0x90, 0xF0, //8
	0xF0, 0x90, 0xF0, 0x10, 0xF0, //9
	0xF0, 0x90, 0xF0, 0x90, 0x90, //A
	0xE0, 0x90, 0xE0, 0x90, 0xE0, //B
	0xF0, 0x80, 0x80, 0x80, 0xF0, //C
	0xE0, 0x90, 0x90, 0x90, 0xE0, //D
	0xF0, 0x80, 0xF0, 0x80, 0xF0, //E
	0xF0, 0x80, 0xF0, 0x80, 0x80, //F
];

impl Chip8 {
	pub fn new() -> Self {
		let mem = Self::init();
		return Chip8 {
			ram: mem,
			..Default::default()
		};
	}

	pub fn init() -> Vec<u8> {
		let mut mem = Vec::with_capacity(4096);
		for i in 0..mem.capacity() {
			if i < CHIP_DIGITS.len() {
				mem.push(CHIP_DIGITS[i]);
			} else {
				mem.push(0);
			}
		}
		return mem;
	}

	pub fn load_code(&mut self, code: Vec<u8>) -> &mut Self {
		self.program_counter = 0x200;
		self.load(code);
		return self;
	}

	pub fn load_code_eti(&mut self, code: Vec<u8>) -> &mut Self {
		self.program_counter = 0x600;
		self.load(code);
		return self;
	}

	fn load(&mut self, code: Vec<u8>) {
		println!("Loading Program, Length: {}", code.len());
		for i in 0..code.len() {
			self.ram[i + self.program_counter] = code[i]
		}
	}

	pub fn print_display(&self) {
		if self.high_res {
			self.print_display_high_res();
		} else {
			self.print_display_low_res();
		}
	}
	fn print_display_low_res(&self) {
		for row in self.display.iter().take(32) {
			let d = format!("{:0>64}", format!("{:b}", row >> 64))
				.replace("0", " ")
				.replace("1", "#");
			println!("{}", d);
		}
	}

	fn print_display_high_res(&self) {
		for row in self.display {
			let d = format!("{:0>128}", format!("{:b}", row))
				.replace("0", " ")
				.replace("1", "#");
			println!("{}", d);
		}
	}

	// fn zero_registers(&mut self) -> &mut Self {
	// 	self.registers = [0; 16];
	// 	self.reg_st = 0;
	// 	self.reg_dt = 0;
	// 	self.reg_i = 0;
	// 	return self;
	// }

	pub fn start(&mut self) {
		while !self.is_halted {
			self.tick();
			if self.need_draw {
				self.need_draw = false;
				print!("\x1B[2J\x1B[1;1H");
				self.print_display();
			}
		}
	}

	pub fn run(&mut self, ticks: usize) {
		for _ in 0..ticks {
			if self.program_counter >= self.ram.len() {
				println!("Done!");
				self.is_halted = true;
				break;
			}
			self.tick();
		}
	}

	pub fn tick(&mut self) {
		self.process_instructions();
		if let Ok(el) = self.timer.elapsed() {
			if el.as_millis() > 16 {
				self.process_timers();
				self.timer = SystemTime::now();
			}
		}
		self.program_counter += 2;
	}

	fn process_timers(&mut self) {
		if self.reg_dt > 0 {
			self.reg_dt -= 1;
		}
		if self.reg_st > 0 {
			self.reg_st -= 1;
		}
	}

	fn process_instructions(&mut self) {
		let b1 = &self.ram[self.program_counter];
		let b2 = &self.ram[self.program_counter + 1];
		let instruction = ((*b1 as u16) << 8) + (*b2 as u16);

		let g = b1 >> 4;

		#[cfg(feature = "print")]
		print!("{:#x}: ", instruction);

		match g {
			0x0 => self.instruction_zero(instruction),
			0x1 => self.instruction_jump(instruction),
			0x2 => self.instruction_call(instruction),
			0x3 => self.instruction_skip(instruction),
			0x4 => self.instruction_skip_ne(instruction),
			0x5 => self.instruction_skip2(instruction),
			0x6 => self.instruction_set(instruction),
			0x7 => self.instruction_add(instruction),
			0x8 => self.instruction_set_math(instruction),
			0x9 => self.instruction_skip_ne2(instruction),
			0xA => self.instruction_set_reg_i(instruction),
			0xB => self.instruction_jump_offset(instruction),
			0xC => self.instruction_rand(instruction),
			0xD => self.instruction_draw(instruction),
			0xE => self.instruction_key(instruction),
			0xF => self.instruction_f(instruction),
			_ => println!("Unimplemented isntruction: {:#x}", instruction),
		}

		#[cfg(feature = "print")]
		println!();
	}

	fn instruction_f(&mut self, instruction: u16) {
		let reg = (instruction & 0x0F00) >> 8;
		let mode = instruction & 0x00FF;

		match mode {
			0x07 => {
				//Read DT
				#[cfg(feature = "print")]
				println!("SET  V{} to DT", reg);
				self.registers[reg as usize] = self.reg_dt;
			}
			0x0A => todo!(),
			0x15 => {
				//Set DT
				#[cfg(feature = "print")]
				println!("SET  DT to V{}", reg);
				self.reg_dt = self.registers[reg as usize];
			}
			0x18 => {
				//Set ST
				#[cfg(feature = "print")]
				println!("SET  ST to V{}", reg);
				self.reg_st = self.registers[reg as usize];
			}
			0x1E => {
				//Set VI
				#[cfg(feature = "print")]
				println!("SET  VI to V{}", reg);
				self.reg_i += self.registers[reg as usize] as u16;
			}
			0x29 => {
				//Set VI to Digit
				#[cfg(feature = "print")]
				println!("SET  VI to Digit of V{}", reg);
				self.reg_i = self.registers[reg as usize] as u16 * 5;
			}
			0x33 => {
				//Set BCD
				#[cfg(feature = "print")]
				println!("SET  VI to Digit of V{}", reg);
				let vx = self.registers[reg as usize];
				self.ram[self.reg_i as usize] = vx / 100;
				self.ram[self.reg_i as usize + 1] = (vx / 10) - ((vx / 100) * 10);
				self.ram[self.reg_i as usize + 2] = vx - ((vx / 10) * 10);
			}
			0x55 => {
				//Store
				#[cfg(feature = "print")]
				println!("STORE  V0 thru V{} to ram", reg);
				let vx = reg as usize;
				for r in 0..vx {
					let i = self.reg_i as usize + r;
					self.ram[i] = self.registers[r];
				}
			}
			0x65 => {
				//Read
				#[cfg(feature = "print")]
				println!("READ  V0 thru V{} from ram", reg);
				let vx = reg as usize;
				for r in 0..vx {
					let i = self.reg_i as usize + r;
					self.registers[r] = self.ram[i];
				}
			}
			_ => (),
		}
	}

	fn instruction_key(&mut self, instruction: u16) {
		let reg = (instruction & 0x0F00) >> 8;
		let mode = instruction & 0x00FF;
		let k = self.registers[reg as usize];
		match mode {
			0x9E => {
				if self.keys[k as usize] {
					self.program_counter += 2;
				}
			}
			0xA1 => {
				if !self.keys[k as usize] {
					self.program_counter += 2;
				}
			}
			_ => (),
		};
	}

	fn instruction_draw(&mut self, instruction: u16) {
		let regx = (instruction & 0x0F00) >> 8;
		let regy = (instruction & 0x00F0) >> 4;
		let x = self.registers[regx as usize];
		let y = self.registers[regy as usize];
		let n = instruction & 0x000F;
		#[cfg(feature = "print")]
		println!("DRW  (V{}, V{})", regx, regy);
		self.need_draw = true;
		self.draw_sprite(x, y, n);
	}

	fn draw_sprite(&mut self, x: u8, y: u8, size: u16) {
		let slice = self.ram.iter().skip(self.reg_i as usize).take(size as usize);
		let mut s_y = y as usize;
		for row in slice {
			let data = self.translate_sprite_row(*row, x);
			let orig = self.display[s_y];
			self.display[s_y] = orig ^ data;

			if self.display[s_y] != orig | data {
				self.registers[0xF] = 1;
			}

			s_y += 1;
			if self.high_res {
				s_y = s_y % 64;
			} else {
				s_y = s_y % 32;
			}
		}
	}

	fn translate_sprite_row(&self, row: u8, x: u8) -> u128 {
		if self.high_res {
			let res = row as u128;
			return res.rotate_left(128 - ((x as u32 + 8) % 128));
		} else {
			let mut res = row as u64;
			res = res.rotate_left(64 - ((x as u32 + 8) % 64));
			return (res as u128) << 64;
		}
	}

	fn instruction_rand(&mut self, instruction: u16) {
		let reg = (instruction & 0x0F00) >> 8;
		let kk = (instruction & 0x00FF) as u8;
		let r: u8 = self.rng.random();
		#[cfg(feature = "print")]
		println!("RAND + {} to V{}", kk, reg);
		self.registers[reg as usize] = r & kk;
	}

	fn instruction_jump_offset(&mut self, instruction: u16) {
		let addr = instruction & 0x0FFF;
		#[cfg(feature = "print")]
		println!("JUMP {} + V0", addr);
		self.program_counter = (addr + self.registers[0] as u16) as usize;
	}

	fn instruction_set_reg_i(&mut self, instruction: u16) {
		let addr = instruction & 0x0FFF;
		#[cfg(feature = "print")]
		println!("SET VI to {}", addr);
		self.reg_i = addr;
	}

	fn instruction_skip_ne2(&mut self, instruction: u16) {
		let reg = (instruction & 0x0F00) >> 8;
		let reg2 = (instruction & 0x00F0) >> 4;

		#[cfg(feature = "print")]
		println!("SKIP V{} != V{}", reg, reg2);
		if self.registers[reg as usize] != self.registers[reg2 as usize] {
			self.program_counter += 2;
		}
	}

	fn instruction_add(&mut self, instruction: u16) {
		let reg = (instruction & 0x0F00) >> 8;
		let v = (instruction & 0x00FF) as u8;
		#[cfg(feature = "print")]
		println!("ADD V{} + {}", reg, v);
		let rv = self.registers[reg as usize];
		self.registers[reg as usize] = rv.wrapping_add(v);
	}

	fn instruction_set(&mut self, instruction: u16) {
		let reg = (instruction & 0x0F00) >> 8;
		let v = (instruction & 0x00FF) as u8;
		#[cfg(feature = "print")]
		println!("SET V{} to {}", reg, v);
		self.registers[reg as usize] = v;
	}

	fn instruction_set_math(&mut self, instruction: u16) {
		let reg = (instruction & 0x0F00) >> 8;
		let reg2 = (instruction & 0x00F0) >> 4;
		let m = instruction & 0x000F;
		match m {
			0x0 => {
				//Set
				#[cfg(feature = "print")]
				println!("SET V{} to V{}", reg, reg2);
				self.registers[reg as usize] = self.registers[reg2 as usize];
			}
			0x1 => {
				//OR
				#[cfg(feature = "print")]
				println!("OR V{} | V{}", reg, reg2);
				self.registers[reg as usize] |= self.registers[reg2 as usize];
			}
			0x2 => {
				//AND
				#[cfg(feature = "print")]
				println!("AND V{} & V{}", reg, reg2);
				self.registers[reg as usize] &= self.registers[reg2 as usize];
			}
			0x3 => {
				//XOR
				#[cfg(feature = "print")]
				println!("XOR V{} ^ V{}", reg, reg2);
				self.registers[reg as usize] ^= self.registers[reg2 as usize];
			}
			0x4 => {
				//Add + Carry
				#[cfg(feature = "print")]
				println!("ADD V{} + V{}", reg, reg2);
				let vx = self.registers[reg as usize];
				let vy = self.registers[reg2 as usize];
				let r = vx as u16 + vy as u16;
				self.registers[0xf] = if r > 255 { 1 } else { 0 };
				self.registers[reg as usize] = (r & 0x00FF) as u8;
			}
			0x5 => {
				//Sub + Borrow
				#[cfg(feature = "print")]
				println!("SUB V{} - V{}", reg, reg2);
				let vx = self.registers[reg as usize];
				let vy = self.registers[reg2 as usize];
				self.registers[0xf] = if vx > vy { 1 } else { 0 };
				self.registers[reg as usize] = vx.wrapping_sub(vy);
			}
			0x6 => {
				//SHR
				#[cfg(feature = "print")]
				println!("SHR V{} >> 1", reg);
				let vx = self.registers[reg as usize];
				self.registers[0xf] = vx & 0x1;
				self.registers[reg as usize] = vx >> 1;
			}
			0x7 => {
				//SubN + Borrow
				#[cfg(feature = "print")]
				println!("SUBN V{} - V{}", reg, reg2);
				let vx = self.registers[reg as usize];
				let vy = self.registers[reg2 as usize];
				self.registers[0xf] = if vx < vy { 1 } else { 0 };
				self.registers[reg as usize] = vx.wrapping_sub(vy);
			}
			0xE => {
				//SHL
				#[cfg(feature = "print")]
				println!("SHL V{} << 1", reg);
				let vx = self.registers[reg as usize];
				self.registers[0xf] = (vx & 0x80) >> 7;
				self.registers[reg as usize] = vx << 1;
			}
			_ => panic!("Invalid bitwise op"),
		}
	}

	fn instruction_skip_ne(&mut self, instruction: u16) {
		let reg = (instruction & 0x0F00) >> 8;
		let v = (instruction & 0x00FF) as u8;

		#[cfg(feature = "print")]
		println!("SKIP V{} != {}", reg, v);
		if self.registers[reg as usize] != v {
			self.program_counter += 2;
		}
	}

	fn instruction_skip(&mut self, instruction: u16) {
		let reg = (instruction & 0x0F00) >> 8;
		let v = (instruction & 0x00FF) as u8;

		#[cfg(feature = "print")]
		println!("SKIP V{} == {}", reg, v);
		if self.registers[reg as usize] == v {
			self.program_counter += 2;
		}
	}

	fn instruction_skip2(&mut self, instruction: u16) {
		let reg = (instruction & 0x0F00) >> 8;
		let reg2 = (instruction & 0x00F0) >> 4;

		#[cfg(feature = "print")]
		println!("SKIP V{} == V{}", reg, reg2);
		if self.registers[reg as usize] == self.registers[reg2 as usize] {
			self.program_counter += 2;
		}
	}

	fn instruction_call(&mut self, instruction: u16) {
		let addr = instruction & 0x0FFF;
		#[cfg(feature = "print")]
		println!("Call {:#x}", addr);
		self.stack_pointer += 1;
		self.stack[self.stack_pointer] = self.program_counter as u16;
		self.program_counter = addr as usize;
	}

	fn instruction_jump(&mut self, instruction: u16) {
		let addr = instruction & 0x0FFF;
		#[cfg(feature = "print")]
		println!("JMP to {:#x}", addr);
		self.program_counter = addr as usize;
	}

	fn instruction_zero(&mut self, instruction: u16) {
		match instruction {
			0x00E0 => self.instruction_clear(),
			0x00EE => self.instruction_ret(),
			_ => {
				self.is_halted = true;
			}
		}
	}

	fn instruction_clear(&mut self) {
		#[cfg(feature = "print")]
		println!("CLS");
		self.display = [0; 64];
	}

	fn instruction_ret(&mut self) {
		#[cfg(feature = "print")]
		println!("RET");
		self.program_counter = self.stack[self.stack_pointer] as usize;
		self.stack_pointer -= 1;
	}
}
