//! Mesh Buffer Cache - caches GPU buffers for meshes
//!
//! Prevents re-allocating vertex and index buffers every frame for static geometry.

use crate::mesh_renderer::GpuVertex;
use mapmap_core::{mapping::MappingId, Mesh, MeshType};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

/// Cached GPU buffers for a mesh
#[derive(Debug)]

pub struct CachedMeshBuffers {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub mesh_revision: u64,
    pub mesh_type: MeshType,
    pub vertex_count: usize,
}

/// Manages GPU buffers for meshes to avoid per-frame allocation
pub struct MeshBufferCache {
    cache: HashMap<MappingId, CachedMeshBuffers>,
}

impl MeshBufferCache {
    /// Create a new mesh buffer cache
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Get buffers for a mapping, creating or updating them if necessary
    pub fn get_buffers(
        &mut self,
        device: &wgpu::Device,
        mapping_id: MappingId,
        mesh: &Mesh,
    ) -> (&wgpu::Buffer, &wgpu::Buffer, u32) {
        // Check if we have a valid cached version
        // We must check type and vertex count to handle "Revision 0" collisions when replacing meshes
        let is_valid = if let Some(cached) = self.cache.get(&mapping_id) {
            cached.mesh_revision == mesh.revision
                && cached.mesh_type == mesh.mesh_type
                && cached.vertex_count == mesh.vertices.len()
        } else {
            false
        };

        if is_valid {
            let cached = self.cache.get(&mapping_id).unwrap();
            return (
                &cached.vertex_buffer,
                &cached.index_buffer,
                cached.index_count,
            );
        }

        // Cache miss or stale - create new buffers
        let vertices: Vec<GpuVertex> = mesh
            .vertices
            .iter()
            .map(GpuVertex::from_mesh_vertex)
            .collect();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Mesh Vertex Buffer {}", mapping_id)),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Mesh Index Buffer {}", mapping_id)),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let index_count = mesh.indices.len() as u32;

        let cached = CachedMeshBuffers {
            vertex_buffer,
            index_buffer,
            index_count,
            mesh_revision: mesh.revision,
            mesh_type: mesh.mesh_type,
            vertex_count: mesh.vertices.len(),
        };

        self.cache.insert(mapping_id, cached);

        let cached = self.cache.get(&mapping_id).unwrap();
        (
            &cached.vertex_buffer,
            &cached.index_buffer,
            cached.index_count,
        )
    }

    /// Remove a mapping from the cache
    pub fn remove(&mut self, mapping_id: MappingId) {
        self.cache.remove(&mapping_id);
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

impl Default for MeshBufferCache {
    fn default() -> Self {
        Self::new()
    }
}
