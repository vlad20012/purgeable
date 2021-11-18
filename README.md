Experimental Rust abstractions around purgeable memory.
- Macos - `vm_allocate`/`vm_purgable_control`
- Windows - `VirtualAlloc`(`MEM_RESET`/`MEM_RESET_UNDO`)
- Linux - `ashmem` `pin`/`unpin`