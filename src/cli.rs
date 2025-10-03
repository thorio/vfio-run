use clap::{Args, Parser, Subcommand, ValueEnum};

pub fn parse() -> CliArgs {
	CliArgs::parse()
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct CliArgs {
	#[command(subcommand)]
	pub command: Command,

	/// enable debug loglevel
	#[arg(long, global = true)]
	pub debug: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
	/// Run the VM, detaching and attaching as required.
	Run {
		#[command(flatten)]
		config: Options,

		/// skip re-attaching PCI devices and such
		#[arg(long, short)]
		skip_attach: bool,
	},

	/// Unload drivers and detach devices
	Detach {
		#[command(flatten)]
		config: Options,
	},

	/// Reload drivers and reattach devices
	Attach {
		#[command(flatten)]
		config: Options,
	},
}

#[derive(Args, Debug)]
pub struct Options {
	#[arg(value_enum)]
	pub profile: Profile,

	#[arg(long, short, value_enum, default_value_t)]
	pub cpu: Cpu,

	#[arg(long, short, value_enum, default_value_t)]
	pub graphics: Graphics,

	/// open qemu GUI
	#[arg(long, short)]
	pub window: bool,
}

#[derive(Clone, Copy, ValueEnum, Debug)]
pub enum Profile {
	Game,
	Work,
}

#[derive(Clone, Copy, ValueEnum, Debug, Default)]
pub enum Cpu {
	#[default]
	Slim,
	Full,
}

#[derive(Clone, Copy, ValueEnum, Debug, Default)]
pub enum Graphics {
	#[default]
	Virtual,
	Passthrough,
}
