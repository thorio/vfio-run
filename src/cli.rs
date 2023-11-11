use clap::{Parser, Subcommand};

pub fn parse() -> Args {
	Args::parse()
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
	#[command(subcommand)]
	pub configuration: Configurations,

	/// open qemu GUI
	#[arg(long)]
	pub window: bool,
}

#[derive(Subcommand)]
pub enum Configurations {
	/// start with no GPU
	Foil,

	/// start with iGPU
	Thin,

	/// start with dGPU
	Fat,
}
