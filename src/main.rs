use cli::{Command, Profile};
use context::Context;
use log::*;
use nix::unistd::Uid;

mod cli;
mod config;
mod context;
mod modprobe;
mod runner;
mod smbios;
mod util;
mod virsh;

fn main() {
	let cli = cli::parse();
	init_logger(cli.debug);
	debug!("{:?}", cli);

	if !Uid::effective().is_root() {
		warn!("running as non-root, here be dragons");
	}

	match cli.command {
		Command::Run {
			profile,
			window,
			skip_attach,
		} => run(window, profile, skip_attach),
		Command::Detach { profile } => detach(profile),
		Command::Attach { profile } => attach(profile),
	}
}

fn detach(profile: Profile) {
	let context = get_context(false, &profile);

	runner::detach_devices(&context).ok();
}

fn attach(profile: Profile) {
	let context = get_context(false, &profile);

	runner::reattach_devices(&context).ok();
}

fn run(window: bool, profile: Profile, skip_attach: bool) {
	let context = get_context(window, &profile);

	if runner::run(context, skip_attach).is_ok() {
		info!("exit successful")
	}
}

fn get_context(window: bool, profile: &Profile) -> Context {
	let builder = config::get_builder(window, profile);
	debug!("{:?}", builder);

	let context = builder.build();
	debug!("{:?}", context);

	context
}

fn init_logger(debug: bool) {
	stderrlog::new()
		.timestamp(stderrlog::Timestamp::Off)
		.verbosity(if debug { 3 } else { 2 })
		.init()
		.expect("logger already initialized");
}
