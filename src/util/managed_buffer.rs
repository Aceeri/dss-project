use wgpu::util::DeviceExt;

use super::ReuseVec;

use std::{marker::PhantomData, ops::RangeBounds};

pub trait IdIndex {
    fn id(&self) -> usize;
    fn from_index(index: usize) -> Self;
}

pub struct ManagedBuffer<T, I: IdIndex> {
    label: Option<&'static str>,
    usage: wgpu::BufferUsages,
    contents: ReuseVec<T>,
    buffer: wgpu::Buffer,
    expand: bool,
    update: bool,
    phantom: PhantomData<I>,
}

impl<T, I> ManagedBuffer<T, I>
where
    T: bytemuck::Pod + bytemuck::Zeroable,
    I: IdIndex,
{
    pub fn new(
        device: &wgpu::Device,
        label: Option<&'static str>,
        usage: wgpu::BufferUsages,
    ) -> ManagedBuffer<T, I> {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: label,
            contents: &[],
            usage,
        });

        Self {
            label,
            usage,
            contents: ReuseVec::new(),
            buffer,
            expand: false,
            update: false,
            phantom: PhantomData,
        }
    }

    pub fn buffer_slice<S: RangeBounds<wgpu::BufferAddress>>(
        &self,
        range: S,
    ) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(range)
    }

    pub fn push(&mut self, element: T) -> I {
        self.expand = true;
        I::from_index(self.contents.push(element))
    }

    pub fn set(&mut self, index: I, element: T) {
        match self.contents.get_mut(index.id()) {
            Some(content) => {
                *content = element;
                self.update = true;
            }
            None => {}
        }
    }

    pub fn update_buffer(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        // Probably could implement some amortized growing here like a Vec.
        if self.expand {
            self.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: self.label,
                contents: bytemuck::cast_slice(self.contents.current().as_slice()),
                usage: self.usage,
            });

            self.expand = false;
            self.update = false;
        } else if self.update {
            queue.write_buffer(
                &self.buffer,
                0,
                bytemuck::cast_slice(self.contents.current().as_slice()),
            );

            self.update = false;
        }
    }
}
