use std::collections::HashMap;
use std::sync::Arc;

use arkin_core::prelude::*;
use petgraph::graph::NodeIndex;
use petgraph::{
    algo::toposort,
    dot::{Config, Dot},
    graph::DiGraph,
};
use rayon::prelude::*;
use time::UtcDateTime;
use tracing::{debug, info};

use crate::Feature;
use crate::FeatureStore;

/// Computation Graph - represents the DAG of feature dependencies and execution order
#[derive(Debug)]
pub struct FeatureGraph {
    graph: DiGraph<Arc<dyn Feature>, ()>,
    order: Vec<NodeIndex>,
    layers: Vec<Vec<NodeIndex>>,
    parallel: bool,
}

impl FeatureGraph {
    pub fn new(features: Vec<Arc<dyn Feature>>, parallel: bool) -> Self {
        let mut graph = DiGraph::new();

        // Create a mapping from Node IDs to Node Indices
        let mut id_to_index = HashMap::new();
        let mut index_to_id = HashMap::new();

        // Add features as nodes and populate the mapping
        for feature in features {
            let output_ids = feature.outputs().to_vec();
            let node_index = graph.add_node(feature);
            for id in output_ids {
                id_to_index.insert(id.clone(), node_index);
                index_to_id.insert(node_index, id.clone());
            }
        }

        for (id, index) in &id_to_index {
            debug!("Node ID: {:?}, Node Index: {:?}", id, index);
        }

        // Add edges automatically
        let mut edges_to_add = Vec::new();
        for target_node in graph.node_indices() {
            for source_id in graph[target_node].inputs() {
                // Check if this is a raw input (no feature produces it)
                let source_node = match id_to_index.get(&source_id) {
                    Some(node) => *node,
                    None => {
                        // This is a raw input, skip adding edge
                        debug!("Raw input detected: {:?}", source_id);
                        continue;
                    }
                };
                edges_to_add.push((source_node, target_node));
            }
        }
        for (source, target) in &edges_to_add {
            debug!("Edge: {:?} -> {:?}", source, target);
        }

        // Add edges to the graph
        for (source, target) in edges_to_add {
            graph.add_edge(source, target, ());
        }

        // Save the topological order for parallel processing
        let order = toposort(&graph, None).expect("Cycle detected in graph");

        debug!("{:?}", Dot::with_config(&graph, &[Config::EdgeIndexLabel]));

        let mut layers: Vec<Vec<NodeIndex>> = Vec::new();
        let mut depth_map = HashMap::new();

        // Calculate depth for each node using topological order
        for &node in &order {
            let max_dep_depth = graph
                .neighbors_directed(node, petgraph::Incoming)
                .filter_map(|dep| depth_map.get(&dep))
                .max()
                .unwrap_or(&0);

            let depth = max_dep_depth + 1;
            depth_map.insert(node, depth);

            // Ensure we have enough layers
            while layers.len() < depth {
                layers.push(Vec::new());
            }

            layers[depth - 1].push(node);
        }

        FeatureGraph {
            graph: graph.into(),
            order,
            layers,
            parallel,
        }
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    pub fn print_dot(&self) {
        info!(target: "feature-graph", "{:?}", Dot::with_config(&self.graph, &[Config::EdgeIndexLabel]));
    }

    /// Export graph to DOT format for visualization with Graphviz
    pub fn to_dot_string(&self) -> String {
        format!("{:?}", Dot::with_config(&self.graph, &[Config::EdgeNoLabel]))
    }

    /// Export graph to DOT format with custom node labels (simplified output names only)
    pub fn to_dot_string_simple(&self) -> String {
        let mut dot = String::from("digraph {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box, style=rounded];\n\n");

        // Get raw inputs
        let raw_inputs = self.get_raw_inputs();

        // Add raw input nodes with special styling
        dot.push_str("  // Raw inputs\n");
        dot.push_str("  node [shape=ellipse, style=filled, fillcolor=lightblue];\n");
        for (i, input) in raw_inputs.iter().enumerate() {
            dot.push_str(&format!("  raw_{} [label=\"{}\"];\n", i, input));
        }
        dot.push('\n');

        // Add feature nodes
        dot.push_str("  // Features\n");
        dot.push_str("  node [shape=box, style=rounded, fillcolor=white];\n");
        for node in self.graph.node_indices() {
            let outputs = self.graph[node].outputs();
            let label = outputs.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
            dot.push_str(&format!("  {} [label=\"{}\"];\n", node.index(), label));
        }
        dot.push('\n');

        // Create a mapping from raw input names to their pseudo-node IDs
        let mut raw_input_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        for (i, input) in raw_inputs.iter().enumerate() {
            raw_input_map.insert(input.clone(), format!("raw_{}", i));
        }

        // Add edges from raw inputs to features
        dot.push_str("  // Edges from raw inputs\n");
        for node in self.graph.node_indices() {
            for input in self.graph[node].inputs() {
                if let Some(raw_id) = raw_input_map.get(input.as_str()) {
                    dot.push_str(&format!("  {} -> {};\n", raw_id, node.index()));
                }
            }
        }
        dot.push('\n');

        // Add edges between features
        dot.push_str("  // Edges between features\n");
        for edge in self.graph.edge_indices() {
            if let Some((source, target)) = self.graph.edge_endpoints(edge) {
                dot.push_str(&format!("  {} -> {};\n", source.index(), target.index()));
            }
        }

        dot.push_str("}\n");
        dot
    }

    /// Export pipeline graph to SVG file using graphviz
    ///
    /// # Arguments
    /// * `path` - Output path for the SVG file (e.g., "/tmp/pipeline_graph.svg")
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(String)` with error message if graphviz is not available or export fails
    ///
    pub fn export_svg(&self, path: impl AsRef<std::path::Path>) -> Result<(), String> {
        use graphviz_rust::{cmd::Format, exec, parse, printer::PrinterContext};

        let dot_string = self.to_dot_string_simple();

        // Parse DOT string
        let graph = parse(&dot_string).map_err(|e| format!("Failed to parse DOT: {:?}", e))?;

        // Execute graphviz to generate SVG
        let svg = exec(graph, &mut PrinterContext::default(), vec![Format::Svg.into()])
            .map_err(|e| format!("Failed to execute graphviz: {:?}", e))?;

        // Write to file
        std::fs::write(&path, svg).map_err(|e| format!("Failed to write SVG file: {}", e))?;

        Ok(())
    }

    /// Print a tree-like ASCII view of the pipeline showing data flow
    pub fn print_tree(&self) {
        info!(target: "feature-graph", "Pipeline Flow (Layer by Layer):");
        info!(target: "feature-graph", "================================");

        let raw_inputs = self.get_raw_inputs();
        info!(target: "feature-graph", "RAW INPUTS: {}", raw_inputs.join(", "));
        info!(target: "feature-graph", "");

        let layers = self.get_layers();
        for (i, layer) in layers.iter().enumerate() {
            info!(target: "feature-graph", "LAYER {} ({} features):", i + 1, layer.len());
            for feature in layer {
                let outputs = feature.outputs();
                let inputs = feature.inputs();
                info!(
                    "  {} <- [{}]",
                    outputs.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
                    inputs.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
                );
            }
            info!(target: "feature-graph", "");
        }
    }

    /// Get all raw input IDs (inputs that no feature produces)
    pub fn get_raw_inputs(&self) -> Vec<String> {
        let mut all_outputs: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut all_inputs: std::collections::HashSet<String> = std::collections::HashSet::new();

        for node in self.graph.node_indices() {
            for output in self.graph[node].outputs() {
                all_outputs.insert(output.to_string());
            }
            for input in self.graph[node].inputs() {
                all_inputs.insert(input.to_string());
            }
        }

        // Raw inputs are those that are used as inputs but never produced as outputs
        all_inputs.difference(&all_outputs).cloned().collect::<Vec<_>>()
    }

    /// Validate the DAG structure - checks for cycles and unreachable nodes
    pub fn validate(&self) -> Result<(), String> {
        // Check for cycles using petgraph's is_cyclic_directed
        if petgraph::algo::is_cyclic_directed(&self.graph) {
            return Err("Cycle detected in graph".to_string());
        }

        // Get raw inputs
        let raw_inputs = self.get_raw_inputs();

        // Check for nodes with no incoming edges from other features (sources)
        let sources: Vec<_> = self
            .graph
            .node_indices()
            .filter(|&node| self.graph.neighbors_directed(node, petgraph::Incoming).count() == 0)
            .collect();

        // Check for nodes with no outgoing edges (sinks/leaf nodes)
        let sinks: Vec<_> = self
            .graph
            .node_indices()
            .filter(|&node| self.graph.neighbors_directed(node, petgraph::Outgoing).count() == 0)
            .collect();

        info!(target: "feature-graph", "DAG Validation:");
        info!(target: "feature-graph", "  Raw inputs (not produced by any feature): {}", raw_inputs.len());
        for input in &raw_inputs {
            info!(target: "feature-graph", "    - {}", input);
        }
        info!(target: "feature-graph", "  Source features (depend only on raw inputs): {}", sources.len());
        info!(target: "feature-graph", "  Sink features (terminal outputs): {}", sinks.len());
        info!(target: "feature-graph", "  Total nodes: {}", self.graph.node_count());
        info!(target: "feature-graph", "  Total edges: {}", self.graph.edge_count());

        Ok(())
    }

    /// Get all source nodes (features that only depend on raw inputs)
    pub fn get_sources(&self) -> Vec<&Arc<dyn Feature>> {
        self.graph
            .node_indices()
            .filter(|&node| self.graph.neighbors_directed(node, petgraph::Incoming).count() == 0)
            .map(|node| &self.graph[node])
            .collect()
    }

    /// Get all sink nodes (terminal features with no dependents)
    pub fn get_sinks(&self) -> Vec<&Arc<dyn Feature>> {
        self.graph
            .node_indices()
            .filter(|&node| self.graph.neighbors_directed(node, petgraph::Outgoing).count() == 0)
            .map(|node| &self.graph[node])
            .collect()
    }

    /// Get dependencies for a specific output
    pub fn get_dependencies(&self, output: &str) -> Option<Vec<Arc<dyn Feature>>> {
        // Find the node that produces this output
        let node = self
            .graph
            .node_indices()
            .find(|&node| self.graph[node].outputs().iter().any(|o| o.as_str() == output))?;

        // Get all upstream dependencies using DFS
        let mut visited = std::collections::HashSet::new();
        let mut deps = Vec::new();
        self.collect_dependencies(node, &mut visited, &mut deps);

        Some(deps)
    }

    fn collect_dependencies(
        &self,
        node: NodeIndex,
        visited: &mut std::collections::HashSet<NodeIndex>,
        deps: &mut Vec<Arc<dyn Feature>>,
    ) {
        if visited.contains(&node) {
            return;
        }
        visited.insert(node);

        for neighbor in self.graph.neighbors_directed(node, petgraph::Incoming) {
            deps.push(Arc::clone(&self.graph[neighbor]));
            self.collect_dependencies(neighbor, visited, deps);
        }
    }

    /// Get the topological layers (features at each depth level)
    pub fn get_layers(&self) -> Vec<Vec<&Arc<dyn Feature>>> {
        let mut layers: Vec<Vec<&Arc<dyn Feature>>> = Vec::new();
        let mut depth_map = HashMap::new();

        // Calculate depth for each node using topological order
        for &node in &self.order {
            let max_dep_depth = self
                .graph
                .neighbors_directed(node, petgraph::Incoming)
                .filter_map(|dep| depth_map.get(&dep))
                .max()
                .unwrap_or(&0);

            let depth = max_dep_depth + 1;
            depth_map.insert(node, depth);

            // Ensure we have enough layers
            while layers.len() < depth {
                layers.push(Vec::new());
            }

            layers[depth - 1].push(&self.graph[node]);
        }

        layers
    }

    /// Print a summary of the pipeline structure
    pub fn print_summary(&self) {
        info!(target: "feature-graph", "Pipeline Summary:");
        info!(target: "feature-graph", "  Total Features: {}", self.graph.node_count());
        info!(target: "feature-graph", "  Total Dependencies: {}", self.graph.edge_count());

        let raw_inputs = self.get_raw_inputs();
        info!(target: "feature-graph", "  Raw Inputs: {}", raw_inputs.len());
        for input in &raw_inputs {
            info!(target: "feature-graph", "    - {}", input);
        }

        let sources = self.get_sources();
        info!(target: "feature-graph", "  Source Features (depend only on raw inputs): {}", sources.len());
        for source in &sources {
            info!(target: "feature-graph", "    - outputs: {:?}", source.outputs());
        }

        let sinks = self.get_sinks();
        info!(target: "feature-graph", "  Terminal Features: {}", sinks.len());
        for sink in &sinks {
            info!(target: "feature-graph", "    - outputs: {:?}", sink.outputs());
        }

        // Find bottleneck features (those with many dependents)
        let mut dependents_count: Vec<(NodeIndex, usize)> = self
            .graph
            .node_indices()
            .map(|node| {
                let count = self.graph.neighbors_directed(node, petgraph::Outgoing).count();
                (node, count)
            })
            .collect();
        dependents_count.sort_by(|a, b| b.1.cmp(&a.1));

        info!(target: "feature-graph", "  Bottleneck Features (top 5 by number of dependents):");
        for (node, count) in dependents_count.iter().take(5) {
            if *count > 0 {
                info!(target: "feature-graph", "    - {:?} ({} dependents)", self.graph[*node].outputs(), count);
            }
        }

        let layers = self.get_layers();
        info!(target: "feature-graph", "  Pipeline Depth: {} layers", layers.len());
        for (i, layer) in layers.iter().enumerate() {
            info!(target: "feature-graph", "    Layer {}: {} features", i + 1, layer.len());
        }
    }

    /// Get all real instruments used by features in the graph
    /// Collects unique real instruments from all feature scopes
    pub fn real_instruments(&self) -> Vec<Arc<Instrument>> {
        use std::collections::HashSet;
        let mut instruments = HashSet::new();

        for node in self.graph.node_indices() {
            for scope in self.graph[node].scopes() {
                // Collect real instruments from inputs
                for input in &scope.inputs {
                    if !input.synthetic {
                        instruments.insert(Arc::clone(input));
                    }
                }
                // Check if output is a real instrument
                if !scope.output.synthetic {
                    instruments.insert(Arc::clone(&scope.output));
                }
            }
        }

        instruments.into_iter().collect()
    }

    /// Get all synthetic instruments used by features in the graph
    /// Collects unique synthetic instruments from all feature scopes
    pub fn synthetic_instruments(&self) -> Vec<Arc<Instrument>> {
        use std::collections::HashSet;
        let mut instruments = HashSet::new();

        for node in self.graph.node_indices() {
            for scope in self.graph[node].scopes() {
                // Collect synthetic instruments from inputs
                for input in &scope.inputs {
                    if input.synthetic {
                        instruments.insert(Arc::clone(input));
                    }
                }
                // Check if output is a synthetic instrument
                if scope.output.synthetic {
                    instruments.insert(Arc::clone(&scope.output));
                }
            }
        }

        instruments.into_iter().collect()
    }

    /// Get all instruments (both real and synthetic) used by features in the graph
    pub fn all_instruments(&self) -> Vec<Arc<Instrument>> {
        use std::collections::HashSet;
        let mut instruments = HashSet::new();

        for node in self.graph.node_indices() {
            for scope in self.graph[node].scopes() {
                // Collect all inputs
                for input in &scope.inputs {
                    instruments.insert(Arc::clone(input));
                }
                // Collect output
                instruments.insert(Arc::clone(&scope.output));
            }
        }

        instruments.into_iter().collect()
    }

    /// Execute the computation graph in topological order with parallel execution where possible
    pub fn calculate(
        &self,
        state: &FeatureStore,
        pipeline: &Arc<Pipeline>,
        event_time: UtcDateTime,
    ) -> Vec<Arc<Insight>> {
        let mut all_insights = Vec::new();

        // Process layers sequentially to ensure dependencies are met
        for layer in &self.layers {
            debug!(target: "feature-graph", "Calculating layer with {} features...", layer.len());

            let layer_insights: Vec<Arc<Insight>> = if self.parallel {
                layer
                    .par_iter()
                    .flat_map(|node| {
                        debug!(target: "feature-graph", "Calculating feature with outputs: {:?}", self.graph[*node].outputs());
                        let feature = &self.graph[*node];
                        feature.calculate(state, pipeline, event_time).unwrap_or_default()
                    })
                    .collect()
            } else {
                layer
                    .iter()
                    .flat_map(|node| {
                        debug!(target: "feature-graph", "Calculating feature with outputs: {:?}", self.graph[*node].outputs());
                        let feature = &self.graph[*node];
                        feature.calculate(state, pipeline, event_time).unwrap_or_default()
                    })
                    .collect()
            };

            // Write this layer's insights to state so next layer can read them
            state.insert_batch(&layer_insights);

            // Collect for final return
            all_insights.extend(layer_insights);
        }

        all_insights
    }
}

// // Step 2: Enqueue nodes with zero in-degree
// let (queue_tx, queue_rx) = kanal::unbounded();
// for node in &self.order {
//     if in_degrees.read()[node.index()] == 0 {
//         info!("Ready node: {}", node.index());
//         queue_tx.send(Some(*node)).expect("Failed to send ready node");
//     }
// }

// // Step 3: Parallel processing
// let pipeline_result = Arc::new(Mutex::new(Vec::new()));
// self.pool.scope(|s| {
//     while let Some(node) = queue_rx.recv().expect("Failed to receive data") {
//         // let graph = Arc::clone(&self.graph);
//         let in_degrees = Arc::clone(&in_degrees);
//         let queue_tx = queue_tx.clone();
//         let pipeline_result = Arc::clone(&pipeline_result);
//         // let state_clone = Arc::clone(&state);

//         s.spawn(move |_| {
//             // Process the node
//             let feature = &self.graph[node];

//             // Calculate the feature
//             let insights = instruments
//                 .par_iter()
//                 .filter_map(|instrument| feature.calculate(state, pipeline, instrument, event_time))
//                 .flatten()
//                 .collect::<Vec<_>>();
//             pipeline_result.lock().extend(insights);

//             // Update in-degrees of neighbors and enqueue new zero in-degree nodes
//             for neighbor in self.graph.neighbors_directed(node, petgraph::Outgoing) {
//                 let mut in_degrees = in_degrees.write();
//                 in_degrees[neighbor.index()] -= 1;
//                 if in_degrees[neighbor.index()] == 0 {
//                     info!("Ready node: {}", neighbor.index());
//                     queue_tx.send(Some(neighbor)).expect("Failed to send ready node");
//                 }
//             }
//             info!("Dependency count: {:?}", in_degrees);
//             if in_degrees.read().iter().all(|&x| x == 0) {
//                 info!("All nodes processed");
//                 queue_tx.send(None).expect("Failed to send exit message");
//             }
//         });
//     }
// });
// info!("Finished graph calculation");
// let mut lock = pipeline_result.lock();

// std::mem::take(&mut *lock)
//     }
// }
