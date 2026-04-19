use core::ptr::NonNull;

use acpi::{AcpiHandler, PhysicalMapping};
use x86_64::PhysAddr;

use crate::memory::{MEMORY_MAPPER, to_virtual_address};

#[derive(Clone, Copy, Debug)]
pub struct AcpiHandlerImpl;

impl AcpiHandler for AcpiHandlerImpl {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let virtual_address = to_virtual_address(
            PhysAddr::new(physical_address as u64),
            MEMORY_MAPPER
                .try_get()
                .expect("Heap not initialized")
                .phys_offset()
                .as_u64(),
        );
        unsafe {
            PhysicalMapping::new(
                physical_address,
                NonNull::new(virtual_address.as_mut_ptr()).expect("address is null"),
                size,
                size,
                self.clone(),
            )
        }
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}
