use core::cell::OnceCell;

use acpi::{AcpiTables, mcfg::Mcfg, rsdp::Rsdp};
use spin::Mutex;
use virtio_drivers::{
    device::blk::VirtIOBlk,
    transport::pci::{
        PciTransport,
        bus::{Cam, ConfigurationAccess, DeviceFunction, PciRoot},
    },
};
use x86_64::PhysAddr;

use crate::{
    fs::device::BlockDevice,
    io::{
        acpi::AcpiHandlerImpl, configuration_access::ConfigurationAccessImpl, disk, hal::HalImpl,
    },
    memory::{MEMORY_MAPPER, to_virtual_address},
};

pub static BLOCK_DEVICE: Mutex<OnceCell<Device>> = Mutex::new(OnceCell::new());

pub struct Device {
    blk: VirtIOBlk<HalImpl, PciTransport>,
}

impl BlockDevice for OnceCell<Device> {
    fn read(&mut self, block_id: usize, buf: &mut [u8]) {
        self.get_mut()
            .expect("Block device not initialized")
            .blk
            .read_blocks(block_id, buf)
            .expect("Read failed");
    }

    fn write(&mut self, block_id: usize, block: &[u8]) {
        self.get_mut()
            .expect("Block device not initialized")
            .blk
            .write_blocks(block_id, block)
            .expect("Write failed");
    }
}

fn find_disk_function<C: ConfigurationAccess>(pci: &PciRoot<C>) -> Option<DeviceFunction> {
    for bus in 0..=255 {
        for (device, info) in pci.enumerate_bus(bus) {
            if info.vendor_id == 0x1AF4 {
                return Some(device);
            }
        }
    }
    None
}

pub fn init() {
    let rsdp = unsafe { Rsdp::search_for_on_bios(AcpiHandlerImpl) };
    let acpi = unsafe {
        AcpiTables::from_rsdp(AcpiHandlerImpl, rsdp.unwrap().physical_start())
            .expect("ACPI tables invalid")
    };

    let config = acpi.find_table::<Mcfg>().unwrap().entries()[0];
    let address = to_virtual_address(
        PhysAddr::new(config.base_address),
        MEMORY_MAPPER
            .try_get()
            .expect("Heap not initialized")
            .phys_offset()
            .as_u64(),
    );

    let mut pci = unsafe {
        PciRoot::new(ConfigurationAccessImpl::new(
            address.as_mut_ptr(),
            Cam::Ecam,
        ))
    };

    let disk_function = find_disk_function(&pci).expect("No disk found");
    let transport =
        PciTransport::new::<HalImpl, _>(&mut pci, disk_function).expect("Cannot create transport");
    let blk: VirtIOBlk<HalImpl, _> = VirtIOBlk::new(transport).expect("Cannot create driver");

    BLOCK_DEVICE
        .lock()
        .set(Device { blk })
        .map_err(|_| ()) // Because `Device` does not implement `Debug`
        .expect("Disk already initialized");

    disk::driver::init();
}
