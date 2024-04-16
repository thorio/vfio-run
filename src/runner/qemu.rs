use crate::context::Context;
use std::io;
use std::process::{Command, ExitStatus};

const QEMU_CMD: &str = "qemu-system-x86_64";

pub fn run_qemu(context: &Context) -> Result<ExitStatus, io::Error> {
	get_command(context)
		.args(&context.args)
		.envs(&context.env)
		.spawn()
		.and_then(|mut handle| handle.wait())
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
