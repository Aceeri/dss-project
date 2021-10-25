

use wgpu::util::DeviceExt;

use super::ReuseVec;

use stsd::{
    ops::{RangeBounds},
}

pub struct ManagedBuffer<T: bytemuck::Pod + bytemuck::Zeroable> {
    label: Option<String>,
    usage: wgpu::BufferUsages,
    contents: ReuseVec<T>,
    buffer: wgpu::Buffer,
    expand: bool,
    update: bool,
}

impl<T> ManagedBuffer<T>
where
    T: bytemuck::Pod + bytemuck::Zeroable
{
    pub fn new(device: &wgpu::Device, label: Option<String>, usage: wgpu::BufferUsages) -> ManagedBuffer<T> {
        let buffer = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: label.map(|s| s.as_str()),
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
        }
    }

    pub fn buffer_slice<S: RangeBounds<wgpu::BufferAddress>>(&self, range: S) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(range)
    }

    pub fn push(&mut self, element: T) -> usize {
        self.expand = true;
        self.contents.push(element)
    }

    pub fn set(&mut self, index: usize, element: T) {
        match self.contents.get_mut(index) {
            Some(mut content) => {
                *content = element;
                self.update = true;
            },
            None => {}
        }
    }

    pub fn update_buffer(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        // Probably could implement some amortized growing here like a Vec.
        if self.expand {
            let buffer = device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: self.label.map(|s| s.as_str()),
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