use core::ptr::NonNull;

use crate::memory::memory_mapper::MemoryMapper;
use acpi::{AcpiHandler, PhysicalMapping};
use x86_64::PhysAddr;

#[derive(Clone, Copy, Debug)]
pub struct AcpiHandlerImpl;

impl AcpiHandler for AcpiHandlerImpl {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let virtual_address = MemoryMapper::to_virt(PhysAddr::new(physical_address as u64));
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
