#[derive(Default)]
pub struct Chip8 {
	pub program_counter: usize,
	pub stack_pointer: usize,
	pub registers: [u8; 16],
	pub reg_i: u16,
	pub ram: Vec<u8>,
	pub stack: [u16; 16],

	pub reg_st: u8,
	pub reg_dt: u8,
	pub is_halted: bool,
}

const DIGITS: [u8; 16 * 5] = [
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
		let mut mem = Vec::with_capacity(4096);
		for i in 0..mem.capacity() {
			if i < DIGITS.len() {
				mem.push(DIGITS[i]);
			} else {
				mem.push(0);
			}
		}
		return Chip8 {
			ram: mem,
			..Default::default()
		};
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

	// fn zero_registers(&mut self) -> &mut Self {
	// 	self.registers = [0; 16];
	// 	self.reg_st = 0;
	// 	self.reg_dt = 0;
	// 	self.reg_i = 0;
	// 	return self;
	// }

	pub fn start(&mut self) {
		while self.is_halted {
			self.tick();
		}
	}

	pub fn run(&mut self, ticks: usize) {
		for _ in 0..ticks {
			self.tick();
		}
	}

	pub fn tick(&mut self) {
		self.process_chip8();
	}

	fn process_chip8(&mut self) {
		let b1 = &self.ram[self.program_counter];
		let b2 = &self.ram[self.program_counter + 1];
		let instruction = ((*b1 as u16) << 8) + (*b2 as u16);

		let g = b1 >> 4;

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
			0x8 => self.instruction_set_bitwise(instruction),
			_ => println!("Unimplemented isntruction: {:#x}", instruction),
		}

		println!();
	}

	fn instruction_add(&mut self, instruction: u16) {
		let reg = (instruction & 0x0F00) >> 8;
		let v = (instruction & 0x00FF) as u8;
		self.registers[reg as usize] += v;
		self.program_counter += 1;
	}

	fn instruction_set(&mut self, instruction: u16) {
		let reg = (instruction & 0x0F00) >> 8;
		let v = (instruction & 0x00FF) as u8;
		self.registers[reg as usize] = v;
		self.program_counter += 1;
	}

	fn instruction_set_bitwise(&mut self, instruction: u16) {
		let reg = (instruction & 0x0F00) >> 8;
		let reg2 = instruction & 0x00FF;
		let m = instruction & 0x000F;
		match m {
			0 => {
				//Set
				self.registers[reg as usize] = self.registers[reg2 as usize];
			}
			1 => {
				//OR
				self.registers[reg as usize] |= self.registers[reg2 as usize];
			}
			2 => {
				//AND
				self.registers[reg as usize] &= self.registers[reg2 as usize];
			}
			3 => {
				//XOR
				self.registers[reg as usize] ^= self.registers[reg2 as usize];
			}
			4 => {
				//Add + Carry
				let vx = self.registers[reg as usize];
				let vy = self.registers[reg2 as usize];
				let r = vx as u16 + vy as u16;
				self.registers[0xf] = if r > 255 { 1 } else { 0 };
				self.registers[reg as usize] = (r & 0x00FF) as u8;
			}
			5 => {
				//Sub + Borrow
				let vx = self.registers[reg as usize];
				let vy = self.registers[reg2 as usize];
				self.registers[0xf] = if vx > vy { 1 } else { 0 };
				self.registers[reg as usize] = vx - vy;
			}
			6 => {
				//SHR
				let vx = self.registers[reg as usize];
				let vy = self.registers[reg2 as usize];
				let r = vx as u16 + vy as u16;
				self.registers[0xf] = if r > 255 { 1 } else { 0 };
				self.registers[reg as usize] = (r & 0x00FF) as u8;
			}
			_ => panic!("Invalid bitwise op"),
		}

		self.program_counter += 1;
	}

	fn instruction_skip_ne(&mut self, instruction: u16) {
		let reg = (instruction & 0x0F00) >> 8;
		let v = (instruction & 0x00FF) as u8;

		println!("SKIP V{} != {}", reg, v);
		if self.registers[reg as usize] != v {
			self.program_counter += 2;
		} else {
			self.program_counter += 1;
		}
	}

	fn instruction_skip(&mut self, instruction: u16) {
		let reg = (instruction & 0x0F00) >> 8;
		let v = (instruction & 0x00FF) as u8;

		println!("SKIP V{} == {}", reg, v);
		if self.registers[reg as usize] == v {
			self.program_counter += 2;
		} else {
			self.program_counter += 1;
		}
	}

	fn instruction_skip2(&mut self, instruction: u16) {
		let reg = (instruction & 0x0F00) >> 8;
		let reg2 = (instruction & 0x00F0) >> 4;

		println!("SKIP V{} == V{}", reg, reg2);
		if self.registers[reg as usize] == self.registers[reg2 as usize] {
			self.program_counter += 2;
		} else {
			self.program_counter += 1;
		}
	}

	fn instruction_call(&mut self, instruction: u16) {
		let addr = instruction & 0x0FFF;
		println!("Call {:#x}", addr);
		self.stack_pointer += 1;
		self.stack[self.stack_pointer] = self.program_counter as u16;
		self.program_counter = addr as usize;
	}

	fn instruction_jump(&mut self, instruction: u16) {
		let addr = instruction & 0x0FFF;
		println!("JMP to {:#x}", addr);
		self.program_counter = addr as usize;
	}

	fn instruction_zero(&mut self, instruction: u16) {
		match instruction {
			0x00E0 => self.instruction_clear(),
			0x00EE => self.instruction_ret(),
			_ => panic!("Invalid instruction: {:#x}", instruction),
		}
	}

	fn instruction_clear(&mut self) {
		println!("CLS");
		self.program_counter += 1;
	}

	fn instruction_ret(&mut self) {
		println!("RET");
		self.program_counter = self.stack[self.stack_pointer] as usize;
		self.stack_pointer -= 1;
	}
}
