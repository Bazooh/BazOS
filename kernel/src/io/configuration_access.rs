use virtio_drivers::transport::pci::bus::{Cam, ConfigurationAccess, DeviceFunction};

#[derive(Debug)]
pub struct ConfigurationAccessImpl {
    mmio_base: *mut u32,
    cam: Cam,
}

impl ConfigurationAccessImpl {
    /// Wraps the PCI root complex with the given MMIO base address.
    ///
    /// Panics if the base address is not aligned to a 4-byte boundary.
    ///
    /// # Safety
    ///
    /// `mmio_base` must be a valid pointer to an appropriately-mapped MMIO region of at least
    /// 16 MiB (if `cam == Cam::MmioCam`) or 256 MiB (if `cam == Cam::Ecam`). The pointer must be
    /// valid for the entire lifetime of the program (i.e. `'static`), which implies that no Rust
    /// references may be used to access any of the memory region at any point.
    pub unsafe fn new(mmio_base: *mut u8, cam: Cam) -> Self {
        assert!(mmio_base as usize & 0x3 == 0);
        Self {
            mmio_base: mmio_base as *mut u32,
            cam,
        }
    }

    fn cam_offset(&self, device_function: DeviceFunction, register_offset: u8) -> u32 {
        assert!(device_function.valid());

        let bdf = (device_function.bus as u32) << 8
            | (device_function.device as u32) << 3
            | device_function.function as u32;
        let address =
            bdf << match self.cam {
                Cam::MmioCam => 8,
                Cam::Ecam => 12,
            } | register_offset as u32;
        // Ensure that address is within range.
        assert!(address < self.cam.size());
        // Ensure that address is word-aligned.
        assert!(address & 0x3 == 0);
        address
    }
}

impl ConfigurationAccess for ConfigurationAccessImpl {
    /// Makes a clone of the `PciRoot`, pointing at the same MMIO region.
    ///
    /// # Safety
    ///
    /// This function allows concurrent mutable access to the PCI CAM. To avoid this causing
    /// problems, the returned `PciRoot` instance must only be used to read read-only fields.
    unsafe fn unsafe_clone(&self) -> Self {
        Self {
            mmio_base: self.mmio_base,
            cam: self.cam,
        }
    }

    /// Reads 4 bytes from configuration space using the appropriate CAM.
    fn read_word(&self, device_function: DeviceFunction, register_offset: u8) -> u32 {
        let address = self.cam_offset(device_function, register_offset);
        // Safe because both the `mmio_base` and the address offset are properly aligned, and the
        // resulting pointer is within the MMIO range of the CAM.
        unsafe {
            // Right shift to convert from byte offset to word offset.
            (self.mmio_base.add((address >> 2) as usize)).read_volatile()
        }
    }

    /// Writes 4 bytes to configuration space using the appropriate CAM.
    fn write_word(&mut self, device_function: DeviceFunction, register_offset: u8, data: u32) {
        let address = self.cam_offset(device_function, register_offset);
        // Safe because both the `mmio_base` and the address offset are properly aligned, and the
        // resulting pointer is within the MMIO range of the CAM.
        unsafe {
            // Right shift to convert from byte offset to word offset.
            (self.mmio_base.add((address >> 2) as usize)).write_volatile(data)
        }
    }
}
