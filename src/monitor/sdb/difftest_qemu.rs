use cfg_if::cfg_if;
use log::info;
use crate::utils::cfg_if_feat;

cfg_if! {
    if #[cfg(feature="difftest")] {
        use std::process::{Child, Command, Stdio};
        use std::{thread, time};
        use log::error;
        use crate::monitor::sdb::gdb_interface::GdbContext;
        pub struct DifftestContext {
            qemu_proc: Child,
            pub gdb_ctx: GdbContext,
        }
    } else {
        pub struct DifftestContext {}
    }
}

impl DifftestContext {
    pub fn exit(&mut self) {
        cfg_if_feat!("difftest", {
            if let Err(err) = self.qemu_proc.kill() {
                error!("{}", err);
            }
        });
    }
}

#[allow(dead_code)]
pub struct DifftestInfo {
    pub(crate) qemu_bin: String,
    pub(crate) reset_vec: u64,
}

impl DifftestContext {
    pub fn init(_info: DifftestInfo, _binary: Option<String>) -> Self {
        cfg_if! {
            if #[cfg(feature="difftest")] {
                let qemu_proc = Command::new(_info.qemu_bin)
                    .args(["-M", "virt", "-m", "256M", "-nographic", "-s", "-S",
                        "-bios", _binary.unwrap().as_str()])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .spawn().unwrap();
                info!("Difftest enabled. qemu spawned with pid {}.", qemu_proc.id());
                thread::sleep(time::Duration::from_millis(200));
                let mut gdb_ctx = GdbContext::new();
                gdb_ctx.continue_to_addr(_info.reset_vec);
                Self {
                    qemu_proc,
                    gdb_ctx,
                }
            } else {
                Self {}
            }
        }
    }
}

