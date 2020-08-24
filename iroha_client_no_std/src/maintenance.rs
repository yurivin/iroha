//! `Maintenance` module provides structures and implementation blocks related to `Iroha`
//! maintenance functions like Healthcheck, Monitoring, etc.

// use iroha_derive::Io;
use parity_scale_codec::{Decode, Encode};
/// `Health` enumerates different variants of Iroha `Peer` states.
/// Each variant can provide additional information if needed.
#[derive(Clone, Debug, Encode, Decode)]
pub enum Health {
    /// `Healthy` variant means that `Peer` has finished initial setup.
    Healthy,
    /// `Ready` variant means that `Peer` bootstrapping completed.
    Ready,
}

/// Metrics struct compose all Iroha metrics and provides an ability to export them in monitoring
/// systems.
#[derive(Clone, Debug, Default, Encode, Decode)]
pub struct Metrics {
    cpu: cpu::Cpu,
    disk: disk::Disk,
    memory: memory::Memory,
}

mod disk {
    // use iroha_derive::Io;
    use alloc::string::String;
    use parity_scale_codec::{Decode, Encode};
    #[derive(Clone, Debug, Default, Encode, Decode)]
    // #[derive(Clone, Debug, Default, Encode, Decode)]
    pub struct Disk {
        block_storage_size: u64,
        block_storage_path: String,
    }
}

mod cpu {
    // use iroha_derive::Io;
    use alloc::string::String;
    use parity_scale_codec::{Decode, Encode};
    #[derive(Clone, Debug, Default, Encode, Decode)]
    pub struct Cpu {
        load: Load,
    }

    #[derive(Clone, Debug, Default, Encode, Decode)]
    pub struct Load {
        frequency: String,
        stats: String,
        time: String,
    }
}

mod memory {
    // use iroha_derive::Io;
    use alloc::string::String;
    use parity_scale_codec::{Decode, Encode};
    #[derive(Clone, Debug, Default, Encode, Decode)]
    pub struct Memory {
        memory: String,
        swap: String,
    }
}
