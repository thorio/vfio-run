use clap::{Parser, Subcommand, ValueEnum};

pub fn parse() -> Args {
	Args::parse()
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
	#[command(subcommand)]
	pub command: Command,

	/// enable debug loglevel
	#[arg(long)]
	pub debug: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
	/// Run the VM, detaching and attaching as required.
	Run {
		#[arg(value_enum)]
		configuration: Configuration,

		/// open qemu GUI
		#[arg(long, short)]
		window: bool,

		/// skip re-attaching PCI devices and such
		#[arg(long, short)]
		skip_attach: bool,
	},

	/// Unload drivers and detach devices
	Detach {
		#[arg(value_enum)]
		configuration: Configuration,
	},

	/// Reload drivers and reattach devices
	Attach {
		#[arg(value_enum)]
		configuration: Configuration,
	},
}

#[derive(Clone, ValueEnum, Debug)]
pub enum Configuration {
	/// start with no GPU
	Foil,

	/// start with iGPU
	Thin,

	/// start with dGPU
	Fat,
}
