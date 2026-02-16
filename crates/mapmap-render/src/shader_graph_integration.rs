//! Shader Graph to Effect Pipeline Integration
//!
//! This module bridges the ShaderGraph system with the EffectChainRenderer,
//! allowing node-based shaders to be used as custom effects in the render pipeline.

use mapmap_core::codegen::WGSLCodegen;
use mapmap_core::shader_graph::{GraphId, ShaderGraph};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, info, warn};

use crate::EffectChainRenderer;

/// Errors specific to shader graph integration
#[derive(Debug, Error)]
pub enum ShaderGraphIntegrationError {
    #[error("Shader graph validation failed: {0}")]
    ValidationFailed(String),

    #[error("Code generation failed: {0}")]
    CodegenFailed(String),

    #[error("Shader compilation failed: {0}")]
    CompilationFailed(String),

    #[error("Graph not found: {0}")]
    GraphNotFound(GraphId),
}

/// Compiled shader graph ready for rendering
pub struct CompiledShaderGraph {
    /// Original graph ID
    pub graph_id: GraphId,
    /// Graph name
    pub name: String,
    /// Generated WGSL source code
    pub wgsl_source: String,
    /// Compiled shader module (if compilation succeeded)
    shader_module: Option<wgpu::ShaderModule>,
    /// Render pipeline for this shader
    pipeline: Option<wgpu::RenderPipeline>,
}

impl CompiledShaderGraph {
    /// Check if the shader is ready for rendering
    pub fn is_ready(&self) -> bool {
        self.shader_module.is_some() && self.pipeline.is_some()
    }

    /// Get the render pipeline
    pub fn pipeline(&self) -> Option<&wgpu::RenderPipeline> {
        self.pipeline.as_ref()
    }
}

/// Manager for shader graph integration with the effect pipeline
pub struct ShaderGraphManager {
    /// Registered shader graphs
    graphs: HashMap<GraphId, ShaderGraph>,
    /// Compiled shaders
    compiled: HashMap<GraphId, CompiledShaderGraph>,
    /// Next graph ID
    next_id: GraphId,
}

impl Default for ShaderGraphManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ShaderGraphManager {
    /// Create a new shader graph manager
    pub fn new() -> Self {
        Self {
            graphs: HashMap::new(),
            compiled: HashMap::new(),
            next_id: 1,
        }
    }

    /// Register a new shader graph
    pub fn register(&mut self, graph: ShaderGraph) -> GraphId {
        let id = graph.id;
        self.graphs.insert(id, graph);
        self.next_id = self.next_id.max(id + 1);
        info!("Registered shader graph with ID: {}", id);
        id
    }

    /// Create a new empty shader graph
    pub fn create_graph(&mut self, name: String) -> GraphId {
        let id = self.next_id;
        self.next_id += 1;
        let graph = ShaderGraph::new(id, name);
        self.graphs.insert(id, graph);
        info!("Created new shader graph with ID: {}", id);
        id
    }

    /// Get a shader graph by ID
    pub fn get(&self, id: GraphId) -> Option<&ShaderGraph> {
        self.graphs.get(&id)
    }

    /// Get a mutable reference to a shader graph
    pub fn get_mut(&mut self, id: GraphId) -> Option<&mut ShaderGraph> {
        self.graphs.get_mut(&id)
    }

    /// Remove a shader graph
    pub fn remove(&mut self, id: GraphId) -> Option<ShaderGraph> {
        self.compiled.remove(&id);
        self.graphs.remove(&id)
    }

    /// Compile a shader graph to WGSL
    pub fn compile_to_wgsl(&self, id: GraphId) -> Result<String, ShaderGraphIntegrationError> {
        let graph = self
            .graphs
            .get(&id)
            .ok_or(ShaderGraphIntegrationError::GraphNotFound(id))?;

        // Validate the graph first
        if let Err(errors) = graph.validate() {
            return Err(ShaderGraphIntegrationError::ValidationFailed(
                errors.join("; "),
            ));
        }

        // Generate WGSL code
        let mut codegen = WGSLCodegen::new(graph.clone());
        let wgsl = codegen
            .generate()
            .map_err(|e| ShaderGraphIntegrationError::CodegenFailed(e.to_string()))?;

        debug!("Generated WGSL for graph {}: {} bytes", id, wgsl.len());

        Ok(wgsl)
    }

    /// Compile and create GPU resources for a shader graph
    pub fn compile_for_gpu(
        &mut self,
        id: GraphId,
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
        target_format: wgpu::TextureFormat,
    ) -> Result<(), ShaderGraphIntegrationError> {
        let graph = self
            .graphs
            .get(&id)
            .ok_or(ShaderGraphIntegrationError::GraphNotFound(id))?;

        let name = graph.name.clone();

        // Generate WGSL
        let wgsl = self.compile_to_wgsl(id)?;

        // Create shader module
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("ShaderGraph_{}", id)),
            source: wgpu::ShaderSource::Wgsl(wgsl.clone().into()),
        });

        // Create render pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("ShaderGraph_Pipeline_Layout_{}", id)),
            bind_group_layouts: &[bind_group_layout, uniform_bind_group_layout],
            push_constant_ranges: &[],
                    });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("ShaderGraph_Pipeline_{}", id)),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Store compiled shader
        let compiled = CompiledShaderGraph {
            graph_id: id,
            name,
            wgsl_source: wgsl,
            shader_module: Some(shader_module),
            pipeline: Some(pipeline),
        };

        self.compiled.insert(id, compiled);
        info!("Compiled shader graph {} for GPU", id);

        Ok(())
    }

    /// Get a compiled shader graph
    pub fn get_compiled(&self, id: GraphId) -> Option<&CompiledShaderGraph> {
        self.compiled.get(&id)
    }

    /// List all registered graphs
    pub fn list_graphs(&self) -> Vec<(GraphId, &str)> {
        self.graphs
            .iter()
            .map(|(id, g)| (*id, g.name.as_str()))
            .collect()
    }

    /// Check if a graph is compiled and ready
    pub fn is_compiled(&self, id: GraphId) -> bool {
        self.compiled
            .get(&id)
            .map(|c| c.is_ready())
            .unwrap_or(false)
    }
}

/// Extension trait for EffectChainRenderer to use shader graphs
pub trait ShaderGraphRendering {
    /// Apply a compiled shader graph as a post-process effect
    fn apply_shader_graph(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        compiled: &CompiledShaderGraph,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
        bind_group: &wgpu::BindGroup,
        uniform_bind_group: &wgpu::BindGroup,
    );
}

impl ShaderGraphRendering for EffectChainRenderer {
    fn apply_shader_graph(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        compiled: &CompiledShaderGraph,
        _input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
        bind_group: &wgpu::BindGroup,
        uniform_bind_group: &wgpu::BindGroup,
    ) {
        let pipeline = match compiled.pipeline() {
            Some(p) => p,
            None => {
                warn!("Shader graph {} not compiled", compiled.graph_id);
                return;
            }
        };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&format!("ShaderGraph_RenderPass_{}", compiled.graph_id)),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
                        occlusion_query_set: None,
        });

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.set_bind_group(1, uniform_bind_group, &[]);
        render_pass.draw(0..6, 0..1); // Fullscreen quad (2 triangles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mapmap_core::shader_graph::NodeType;

    #[test]
    fn test_shader_graph_manager() {
        let mut manager = ShaderGraphManager::new();

        // Create a graph
        let id = manager.create_graph("Test Graph".to_string());
        assert!(manager.get(id).is_some());

        // Add some nodes
        if let Some(graph) = manager.get_mut(id) {
            let _input = graph.add_node(NodeType::TextureInput);
            let _output = graph.add_node(NodeType::Output);
        }

        // Graph should be in list
        let graphs = manager.list_graphs();
        assert_eq!(graphs.len(), 1);
        assert_eq!(graphs[0].0, id);
    }
}
