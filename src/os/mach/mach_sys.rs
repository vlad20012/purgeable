#![allow(non_camel_case_types)]

use std::os::raw::{c_int, c_uint};

use libc::uintptr_t;

pub(crate) type kern_return_t = c_int;
pub(crate) type vm_offset_t = uintptr_t;
pub(crate) type vm_size_t = uintptr_t;
pub(crate) type mach_port_t = c_uint;
pub(crate) type vm_map_t = mach_port_t;
pub(crate) type vm_address_t = vm_offset_t;
pub(crate) type vm_purgable_t = c_int;

pub(crate) const KERN_SUCCESS: kern_return_t = 0;

// /// (really the absence of VM_FLAGS_ANYWHERE)
// ///  Allocate new VM region at the specified virtual address, if possible.
// pub(crate) const VM_FLAGS_FIXED: c_int = 0x0000;
/// Allocate new VM region anywhere it would fit in the address space
pub(crate) const VM_FLAGS_ANYWHERE: c_int = 0x0001;
/// Create a purgable VM object for that new VM region.
pub(crate) const VM_FLAGS_PURGABLE: c_int = 0x0002;
// /// The new VM region can replace existing VM regions if necessary
// ///  (to be used in combination with VM_FLAGS_FIXED).
// pub(crate) const VM_FLAGS_OVERWRITE: c_int = 0x4000;

/// set state of purgeable object
pub(crate) const VM_PURGABLE_SET_STATE: vm_purgable_t = 0;
/// get state of purgeable object
pub(crate) const VM_PURGABLE_GET_STATE: vm_purgable_t = 1;
// /// purge all volatile objects now
// pub(crate) const VM_PURGABLE_PURGE_ALL: vm_purgable_t = 2;
// /// set state from kernel
// pub(crate) const VM_PURGABLE_SET_STATE_FROM_KERNEL: vm_purgable_t = 3;

/*
 * Volatile memory ordering groups (group zero objects are purged before group 1, etc...
 * It is implementation dependent as to whether these groups are global or per-address space.
 * (for the moment, they are global).
 */
pub(crate) const VM_VOLATILE_GROUP_SHIFT: ::libc::c_int = 8;
// pub(crate) const VM_VOLATILE_GROUP_MASK: ::libc::c_int = 7 << VM_VOLATILE_GROUP_SHIFT;
pub(crate) const VM_VOLATILE_GROUP_DEFAULT: ::libc::c_int = VM_VOLATILE_GROUP_0;

pub(crate) const VM_VOLATILE_GROUP_0: ::libc::c_int = 0 << VM_VOLATILE_GROUP_SHIFT;
// pub(crate) const VM_VOLATILE_GROUP_1: ::libc::c_int = 1 << VM_VOLATILE_GROUP_SHIFT;
// pub(crate) const VM_VOLATILE_GROUP_2: ::libc::c_int = 2 << VM_VOLATILE_GROUP_SHIFT;
// pub(crate) const VM_VOLATILE_GROUP_3: ::libc::c_int = 3 << VM_VOLATILE_GROUP_SHIFT;
// pub(crate) const VM_VOLATILE_GROUP_4: ::libc::c_int = 4 << VM_VOLATILE_GROUP_SHIFT;
// pub(crate) const VM_VOLATILE_GROUP_5: ::libc::c_int = 5 << VM_VOLATILE_GROUP_SHIFT;
// pub(crate) const VM_VOLATILE_GROUP_6: ::libc::c_int = 6 << VM_VOLATILE_GROUP_SHIFT;
// pub(crate) const VM_VOLATILE_GROUP_7: ::libc::c_int = 7 << VM_VOLATILE_GROUP_SHIFT;

/*
 * Purgeable behavior
 * Within the same group, FIFO objects will be emptied before objects that are added later.
 * LIFO objects will be emptied after objects that are added later.
 * - Input only, not returned on state queries.
 */
// pub(crate) const VM_PURGABLE_BEHAVIOR_SHIFT: ::libc::c_int = 6;
// pub(crate) const VM_PURGABLE_BEHAVIOR_MASK: ::libc::c_int = 1 << VM_PURGABLE_BEHAVIOR_SHIFT;
// pub(crate) const VM_PURGABLE_BEHAVIOR_FIFO: ::libc::c_int = 0 << VM_PURGABLE_BEHAVIOR_SHIFT;
// pub(crate) const VM_PURGABLE_BEHAVIOR_LIFO: ::libc::c_int = 1 << VM_PURGABLE_BEHAVIOR_SHIFT;

/*
 * Obsolete object.
 * Disregard volatile group, and put object into obsolete queue instead, so it is the next object
 * to be purged.
 * - Input only, not returned on state queries.
 */
// pub(crate) const VM_PURGABLE_ORDERING_SHIFT: ::libc::c_int = 5;
// pub(crate) const VM_PURGABLE_ORDERING_MASK: ::libc::c_int = 1 << VM_PURGABLE_ORDERING_SHIFT;
// pub(crate) const VM_PURGABLE_ORDERING_OBSOLETE: ::libc::c_int = 1 << VM_PURGABLE_ORDERING_SHIFT;
// pub(crate) const VM_PURGABLE_ORDERING_NORMAL: ::libc::c_int = 0 << VM_PURGABLE_ORDERING_SHIFT;

/*
 * Obsolete parameter - do not use
 */
// pub(crate) const VM_VOLATILE_ORDER_SHIFT: ::libc::c_int = 4;
// pub(crate) const VM_VOLATILE_ORDER_MASK: ::libc::c_int = 1 << VM_VOLATILE_ORDER_SHIFT;
// pub(crate) const VM_VOLATILE_MAKE_FIRST_IN_GROUP: ::libc::c_int = 1 << VM_VOLATILE_ORDER_SHIFT;
// pub(crate) const VM_VOLATILE_MAKE_LAST_IN_GROUP: ::libc::c_int = 0 << VM_VOLATILE_ORDER_SHIFT;

// pub(crate) const VM_PURGABLE_STATE_MIN: ::libc::c_int = 0;
// pub(crate) const VM_PURGABLE_STATE_MAX: ::libc::c_int = 3;
// pub(crate) const VM_PURGABLE_STATE_MASK: ::libc::c_int = 3;
pub(crate) const VM_PURGABLE_NONVOLATILE: ::libc::c_int = 0;
pub(crate) const VM_PURGABLE_VOLATILE: ::libc::c_int = 1;
pub(crate) const VM_PURGABLE_EMPTY: ::libc::c_int = 2;
// pub(crate) const VM_PURGABLE_DENY: ::libc::c_int = 3;

extern "C" {
    pub(crate) fn mach_task_self() -> mach_port_t;

    pub(crate) fn vm_allocate(
        target_task: vm_map_t,
        address: *mut vm_address_t,
        size: vm_size_t,
        flags: c_int,
    ) -> kern_return_t;

    pub(crate) fn vm_deallocate(
        target_task: vm_map_t,
        address: vm_address_t,
        size: vm_size_t,
    ) -> kern_return_t;

    pub(crate) fn vm_purgable_control(
        target_task: vm_map_t,
        address: vm_address_t,
        control: vm_purgable_t,
        state: *mut c_int,
    ) -> kern_return_t;
}
