use log::error;

use crate::context::Context;
use std::process::Command;

const QEMU_CMD: &str = "qemu-system-x86_64";

pub fn run(context: Context) {
	// TODO un-/rebind PCI

	let result = get_command(&context)
		.args(context.args)
		.envs(context.env)
		.spawn()
		.map(|mut handle| handle.wait());

	if let Err(e) = result {
		error!("error running qemu: {}", e)
	}
}

fn get_command(context: &Context) -> Command {
	match &context.cpu_affinity {
		None => Command::new(QEMU_CMD),
		Some(affinity) => {
			let mut cmd = Command::new("taskset");
			cmd.arg("--cpu-list").arg(affinity);
			cmd.arg(QEMU_CMD);

			cmd
		}
	}
}
