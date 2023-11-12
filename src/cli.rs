use clap::{Parser, ValueEnum};

pub fn parse() -> Args {
	Args::parse()
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
	#[arg(value_enum)]
	pub configuration: Configurations,

	/// open qemu GUI
	#[arg(long, short)]
	pub window: bool,

	/// skip re-attaching PCI devices and such
	#[arg(long, short)]
	pub skip_attach: bool,

	/// enable debug loglevel
	#[arg(long)]
	pub debug: bool,
}

#[derive(Clone, ValueEnum, Debug)]
pub enum Configurations {
	/// start with no GPU
	Foil,

	/// start with iGPU
	Thin,

	/// start with dGPU
	Fat,
}
