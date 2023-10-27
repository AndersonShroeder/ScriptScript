use std::alloc::Layout;
use std::ptr::NonNull;

pub type BlockPtr = NonNull<u8>;
pub type BlockSize = usize;

#[derive(Debug, PartialEq)] // ??
pub enum BlockError {
    BadRequest, // Alignment is incorrect
    OOM, // Insufficient memory
}

/*
Struct that represents a memory block allocated by the VM, contains an
address to the block in memory as well as a size.
 */
pub struct Block {
    ptr: BlockPtr,
    size: BlockSize,
}

// Implements block struct
impl Block {
    pub fn new(size: BlockSize) -> Result<Block, BlockError> {
        if !size.is_power_of_two() {
            return Err(BlockError::BadRequest);
        }

        Ok(Block {
            ptr: internal::alloc_block(size)?, // unpacks Result and propagates error
            size,
        })
    }

    pub fn alloc_block(size: BlockSize) -> Result<BlockPtr, BlockError> {
        unsafe {
            let layout = Layout::from_size_align_unchekced(size, size);
            let ptr = alloc(layout);

            if ptr.is_null() {
                Err(BlockError::OOM);
            } else {
                Ok(NonNull::new_unchecked(ptr))
            }
        }
    }

    pub fn dealloc_block(ptr: BlockPtr, size: BlockSize) {
        unsafe {
            let layout = Layout::from_size_align_unchecked(size, size);

            dealloc(ptr.as_ptr(), layout);
        }
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr();
    }
}

trait AllocRaw {
    fn alloc<T>(&self, object: T) -> *const T;
}

pub const BLOCK_SIZE_BITS: usize = 15;
pub const BLOCK_SIZE: usize = 1 << BLOCK_SIZE_BITS;

/*
Contains the information for allocating blocks/garbage collection
 */
pub struct BumpBlock {
    cursor: *const u8, // Bump pointer -> index of block where last object was written
    limit: *const u8,
    block: Block, // The block where objects are being written
    meta: BlockMeta,
}

impl BumpBlock {

    // Returns a pointer to the next available location for memory with specified size. Does not write.
    pub fn inner_alloc(&mut self, alloc_size: usize) -> Option<*const u8> {
        // Retrieves raw ptr and converts it to integer
        let block_start_ptr = self.block.as_ptr() as usize;
        let cursor_ptr = self.cursor as usize;

        let align_mask = usize = !(size_of::<usize>() - 1);

        // Shift insertion location and apply mask
        let next_ptr = cursor_ptr.checked_sub(alloc_size)? & align_mask;

        // Allocation is invalid because the pointer points to before the start of the block
        if next_ptr < block_start_ptr {
            None
        } else {
            self.cursor = next_ptr as *const u8;
            Some(next_ptr)
        }
    }
}