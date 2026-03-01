use std::sync::Arc;

/// Allocation from a uniform buffer allocator
pub struct Allocation<'a> {
    /// The buffer containing the allocation
    pub buffer: &'a wgpu::Buffer,
    /// The offset within the buffer
    pub offset: u64,
    /// The index of the page (for caching)
    pub page_index: usize,
}

/// Manages uniform buffers to avoid frequent allocations.
///
/// Uses a ring buffer strategy where a large buffer is allocated and sliced.
/// When the buffer is full, a new one is allocated.
/// At the beginning of the frame, the allocator is reset to reuse buffers.
pub struct UniformBufferAllocator {
    device: Arc<wgpu::Device>,
    label: String,
    uniform_alignment: u64,

    // Page size for new buffers
    page_size: u64,

    // Allocated pages
    pages: Vec<wgpu::Buffer>,

    // Current page index
    current_page: usize,

    // Offset in the current page
    current_offset: u64,
}

impl UniformBufferAllocator {
    /// Create a new allocator with default page size (64KB).
    pub fn new(device: Arc<wgpu::Device>, label: &str) -> Self {
        let uniform_alignment = device.limits().min_uniform_buffer_offset_alignment as u64;
        Self {
            device,
            label: label.to_string(),
            uniform_alignment,
            page_size: 65536, // 64KB default page
            pages: Vec::new(),
            current_page: 0,
            current_offset: 0,
        }
    }

    /// Reset the allocator for a new frame.
    /// This allows reusing previously allocated buffers.
    pub fn reset(&mut self) {
        self.current_page = 0;
        self.current_offset = 0;
    }

    /// Allocate a uniform buffer with the given content.
    /// Returns the buffer, the offset within it, and the page index.
    /// Note: The buffer might be shared, so use dynamic offsets or the returned offset.
    pub fn allocate(&mut self, queue: &wgpu::Queue, content: &[u8]) -> Allocation<'_> {
        let size = content.len() as u64;
        let padded_size = (size + self.uniform_alignment - 1) & !(self.uniform_alignment - 1);

        loop {
            if self.current_page < self.pages.len()
                && self.current_offset + padded_size <= self.pages[self.current_page].size()
            {
                break;
            }

            // If we have pages but didn't fit, move to next.
            if self.current_page + 1 < self.pages.len() {
                self.current_page += 1;
                self.current_offset = 0;
                continue;
            }

            // Create new page
            let new_page_size = self.page_size.max(padded_size * 2); // Ensure it fits and has room
            let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("{} Page {}", self.label, self.pages.len())),
                size: new_page_size,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            self.pages.push(buffer);
            self.current_page = self.pages.len() - 1;
            self.current_offset = 0;
        }

        {
            let page = &self.pages[self.current_page];
            queue.write_buffer(page, self.current_offset, content);
            let offset = self.current_offset;
            let page_index = self.current_page;
            self.current_offset += padded_size;
            Allocation {
                buffer: page,
                offset,
                page_index,
            }
        }
    }
}
