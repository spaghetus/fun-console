use std::{
	io::{Read, Write},
	ops::Range,
	path::PathBuf,
};

use emulator_6502::{Interface6502, MOS6502};
use memmap::MmapOptions;
use structopt::StructOpt;

type Bank = (memmap::MmapMut, bool);

pub struct Fun {
	pub mmap: [u8; 0xFFFF],
	pub banks: Vec<Bank>,
}

#[allow(dead_code)]
impl Fun {
	const FIRSTPAGE: Range<u16> = 0x0..0x0100;
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

	pub fn new(game: [u8; 0x1E00], banks: Vec<Bank>) -> Self {
		let mut fun = Fun {
			mmap: [0; 0xFFFF],
			banks,
		};
		fun.mmap[0x200..0x2000].copy_from_slice(&game);
		for (n, i) in include_bytes!("./bios.bin").iter().enumerate() {
			fun.mmap[0x0000 + n] = *i;
		}
		fun
	}
	pub fn title(&self) -> &str {
		std::str::from_utf8(&self.mmap[0x6..0x10]).unwrap_or("??????????")
	}
	pub fn game_code(&self) -> &[u8] {
		&self.mmap[0x100..0x2000]
	}
	pub fn rom_bank(&self, bank: u8) -> &[u8] {
		&self.banks[bank as usize].0
	}
	pub fn map_bank(&mut self, bank: u8) -> &mut Bank {
		if bank >= 8 {
			panic!("Invalid bank: {}", bank);
		}
		let index = self.mmap[0x4010 + bank as usize] as usize;
		&mut self.banks[index]
	}
}

impl Interface6502 for Fun {
	fn read(&mut self, address: u16) -> u8 {
		if address < 0x2000 || address > 0x5000 {
			self.mmap[address as usize]
		} else {
			match address {
				a if {
					Fun::ROM.contains(&a)
						|| Fun::RAM.contains(&a) || Fun::BANK_CTL.contains(&a)
						|| Fun::FIRSTPAGE.contains(&a)
				} =>
				{
					self.mmap[address as usize]
				}
				a if Fun::BANKS.contains(&a) => {
					let bank = ((a - Fun::BANKS.start) as f64 / 0x400 as f64).floor() as u16;
					let a = a - (Fun::BANKS.start) - (bank * 0x0400);
					self.map_bank(bank as u8).0[a as usize]
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
			self.mmap[address as usize] = data;
		} else {
			match address {
				a if { Fun::ROM.contains(&a) || Fun::RAM.contains(&a) } => return,
				a if Fun::STDIO == a => {
					let mut stdout = std::io::stdout().lock();
					stdout.write(&[data]).unwrap();
					stdout.flush().unwrap();
				}
				a if {
					Fun::RAM.contains(&a)
						|| Fun::BANK_CTL.contains(&a)
						|| Fun::FIRSTPAGE.contains(&a)
				} =>
				{
					self.mmap[address as usize] = data;
				}
				a if Fun::BANKS.contains(&a) => {
					let bank = ((a - Fun::BANKS.start) as f64 / 0x400 as f64).floor() as u16;
					if bank < 7 {
						return;
					}
					let a = a - (Fun::BANKS.start) - (bank * 0x400);
					let bank = self.map_bank(bank as u8);
					if bank.1 {
						bank.0[a as usize] = data;
						bank.0.flush().unwrap();
					}
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
	let banks: Vec<Bank> = (0..256)
		.into_iter()
		.map(|i| {
			let (game_path, writable) = {
				let game_path = args.game_path.join(format!("{}{}.bin", args.game_name, i));
				let game_write_path = args.game_path.join(format!("{}{}w.bin", args.game_name, i));
				if game_write_path.exists() {
					(Some(game_write_path), true)
				} else if game_path.exists() {
					(Some(game_path), false)
				} else {
					(None, false)
				}
			};
			let bank_game = if let Some(game_path) = game_path {
				let bank_game = std::fs::OpenOptions::new()
					.read(true)
					.write(true)
					.create(true)
					.open(&game_path)
					.unwrap();
				let bank_game =
					unsafe { MmapOptions::new().len(1024).map_mut(&bank_game).unwrap() };
				bank_game
			} else {
				MmapOptions::new().len(1024).map_anon().unwrap()
			};
			(bank_game, writable)
		})
		.collect::<Vec<_>>();
	let game: [u8; 0x1E00] = {
		let mut game_ = [0u8; 0x1E00];
		for (i, b) in game.iter().enumerate() {
			game_[i] = *b;
		}
		game_
	};
	let mut fun = Fun::new(game, banks);
	let mut cpu = MOS6502::new();
	cpu.set_program_counter(0x0300);
	loop {
		if args.debug {
			println!("{:#?}", cpu);
			std::io::stdin().read_line(&mut String::new()).unwrap();
		}
		cpu.cycle(&mut fun);
	}
}
