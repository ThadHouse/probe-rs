use coresight::{
    access_ports::{
        AccessPortError,
    },
};

use probe::debug_probe::{
    MasterProbe,
    DebugProbe,
    DebugProbeError,
    DebugProbeType,
};

use memory::flash_writer::FlashError;

use std::error::Error; 
use std::fmt;

#[derive(Debug)]
pub enum CliError {
    DebugProbe(DebugProbeError),
    AccessPort(AccessPortError),
    FlashError(FlashError),
    StdIO(std::io::Error),
}

impl Error for CliError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use CliError::*;

        match self {
            DebugProbe(ref e) => Some(e),
            AccessPort(ref e) => Some(e),
            FlashError(ref e) => Some(e),
            StdIO(ref e) => Some(e),
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CliError::*;

        match self {
            DebugProbe(ref e) => e.fmt(f),
            AccessPort(ref e) => e.fmt(f),
            FlashError(ref e) => e.fmt(f),
            StdIO(ref e) => e.fmt(f),
        }
    }
}

impl From<AccessPortError> for CliError {
    fn from(error: AccessPortError) -> Self {
        CliError::AccessPort(error)
    }
}

impl From<DebugProbeError> for CliError {
    fn from(error: DebugProbeError) -> Self {
        CliError::DebugProbe(error)
    }
}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        CliError::StdIO(error)
    }
}

impl From<FlashError> for CliError {
    fn from(error: FlashError) -> Self {
        CliError::FlashError(error)
    }
}


/// Takes a closure that is handed an `DAPLink` instance and then executed.
/// After the closure is done, the USB device is always closed,
/// even in an error case inside the closure!
pub fn with_device<F>(n: usize, f: F) -> Result<(), CliError>
where
    F: FnOnce(&mut MasterProbe) -> Result<(), CliError>
{
    let device = {
        let mut list = daplink::tools::list_daplink_devices();
        list.extend(stlink::tools::list_stlink_devices());

        list.remove(n)
    };

    let mut probe = match device.probe_type {
        DebugProbeType::DAPLink => {
            let mut link = daplink::DAPLink::new_from_probe_info(device)?;

            link.attach(Some(probe::protocol::WireProtocol::Swd))?;
            
            MasterProbe::from_specific_probe(link, Some(probe::target::m0::CORTEX_M0))
        },
        DebugProbeType::STLink => {
            let mut link = stlink::STLink::new_from_probe_info(device)?;

            link.attach(Some(probe::protocol::WireProtocol::Swd))?;
            
            MasterProbe::from_specific_probe(link, Some(probe::target::m0::CORTEX_M0))
        },
    };
    
    f(&mut probe)
}