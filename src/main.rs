use std::{
	io::{Read, Write},
	ops::Range,
	path::PathBuf,
	str::FromStr,
	sync::{Arc, RwLock},
};

use emulator_6502::{Interface6502, MOS6502};
use structopt::StructOpt;

pub struct Fun {
	pub MMAP: [u8; 0xFFFF],
	pub BANKS: [[u8; 0x1000]; 256],
}

impl Fun {
	const ROM: Range<u16> = 0x0200..0x2000;
	const TITLE: Range<u16> = 0x206..0x210;
	const GAME: Range<u16> = 0x100..0x2000;
	const BANKS: Range<u16> = 0x2000..0x4000;
	const CONTROL: Range<u16> = 0x4000..0x4200;
	const CONTROLLERS: Range<u16> = 0x4000..0x4004;
	const BG_TILEMAP: [u16; 2] = [0x4006, 0x4007];
	const FG_TILEMAP: [u16; 2] = [0x4008, 0x4009];
	const SCROLL: u16 = 0x400A;
	const BANK_CTL: Range<u16> = 0x4010..0x4018;
	const STDIO: u16 = 0x4019;
	const RAM: Range<u16> = 0x5000..0xFFFF;

	pub fn new(game: [u8; 0x2000], banks: [[u8; 0x1000]; 256]) -> Self {
		let mut fun = Fun {
			MMAP: [0; 0xFFFF],
			BANKS: banks,
		};
		fun.MMAP[0x200..0x2200].copy_from_slice(&game);
		for (n, i) in include_bytes!("./bios.bin").iter().enumerate() {
			fun.MMAP[0x0000 + n] = *i;
		}
		fun
	}
	pub fn title(&self) -> &str {
		std::str::from_utf8(&self.MMAP[0x6..0x10]).unwrap_or("??????????")
	}
	pub fn game_code(&self) -> &[u8] {
		&self.MMAP[0x100..0x2000]
	}
	pub fn rom_bank(&self, bank: u8) -> &[u8] {
		&self.BANKS[bank as usize]
	}
	pub fn map_bank(&self, bank: u8) -> &[u8] {
		if bank >= 8 {
			panic!("Invalid bank: {}", bank);
		}
		let index = self.MMAP[0x4010 + bank as usize] as usize;
		&self.BANKS[index]
	}
}

impl Interface6502 for Fun {
	fn read(&mut self, address: u16) -> u8 {
		if address < 0x2000 || address > 0x4200 {
			// print!("{:04X} ", address);
			self.MMAP[address as usize]
		} else {
			match address {
				a if {
					Fun::ROM.contains(&a) || Fun::RAM.contains(&a) || Fun::BANK_CTL.contains(&a)
				} =>
				{
					self.MMAP[address as usize]
				}
				a if Fun::BANKS.contains(&a) => {
					let bank = ((a - Fun::BANKS.start) as f64 / 0x1000 as f64).floor() as u16;
					let a = a - (Fun::BANKS.start) - (bank * 0x1000);
					self.map_bank(bank as u8)[a as usize]
				}
				a if Fun::STDIO == a => {
					let mut stdin = std::io::stdin().lock();
					let mut b = [0u8];
					stdin.read_exact(&mut b).unwrap();
					b[0]
				}
				_ => 0,
			}
		}
	}

	fn write(&mut self, address: u16, data: u8) {
		if address < 0x2000 || address > 0x4200 {
			self.MMAP[address as usize] = data;
		} else {
			match address {
				a if {
					Fun::ROM.contains(&a) || Fun::RAM.contains(&a) || Fun::BANKS.contains(&a)
				} =>
				{
					return
				}
				a if Fun::STDIO == a => {
					let mut stdout = std::io::stdout().lock();
					stdout.write(&[data]).unwrap();
					stdout.flush().unwrap();
				}
				a if { Fun::RAM.contains(&a) || Fun::BANK_CTL.contains(&a) } => {
					self.MMAP[address as usize] = data;
				}
				_ => (),
			}
		}
	}
}

#[derive(structopt::StructOpt)]
struct Args {
	#[structopt(parse(from_os_str))]
	pub game_path: PathBuf,
	pub game_name: String,
	#[structopt(short = "d", long = "debug")]
	pub debug: bool,
}

fn main() {
	let args = Args::from_args();
	let rom_path = args.game_path.join(format!("{}.bin", args.game_name));
	let game = std::fs::read(&rom_path).unwrap();
	let banks = (0..256)
		.into_iter()
		.map(|i| {
			let mut bank = [0u8; 0x1000];
			let game_path = args.game_path.join(format!("{}{}.bin", args.game_name, i));
			let bank_game = std::fs::read(&game_path).unwrap_or([0u8; 0x1000].to_vec());
			assert!(bank_game.len() <= 0x1000);
			bank_game
		})
		.collect::<Vec<_>>();
	let game: [u8; 0x2000] = {
		let mut game_ = [0u8; 0x2000];
		for (i, b) in game.iter().enumerate() {
			game_[i] = *b;
		}
		game_
	};
	let banks: Vec<[u8; 0x1000]> = banks
		.into_iter()
		.map(|b| {
			let mut bank = [0u8; 0x1000];
			for (i, b) in b.iter().enumerate() {
				bank[i] = *b;
			}
			bank
		})
		.collect();
	let banks: [[u8; 0x1000]; 256] = {
		let mut banks_ = [[0u8; 0x1000]; 256];
		for (i, b) in banks.iter().enumerate() {
			banks_[i] = *b;
		}
		banks_
	};
	let mut fun = Fun::new(game, banks);
	let mut cpu = MOS6502::new();
	cpu.set_program_counter(0x00);
	loop {
		if args.debug {
			println!("{:#?}", cpu);
			std::io::stdin().read_line(&mut String::new()).unwrap();
		}
		cpu.execute_instruction(&mut fun);
	}
}
