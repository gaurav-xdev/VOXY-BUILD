use std::fmt;

#[derive(Debug, Clone)]
pub enum MemoryEvent {
    ItemStored {
        memory_id: String,
        memory_type: String,
        importance: f64,
    },
    ItemRetrieved {
        memory_id: String,
        memory_type: String,
    },
    ItemConsolidated {
        memory_id: String,
        from_type: String,
        to_type: String,
    },
    ItemCompressed {
        memory_id: String,
    },
    ItemArchived {
        memory_id: String,
    },
    ItemDeleted {
        memory_id: String,
        reason: String,
    },
    ItemForgotten {
        memory_id: String,
        new_state: String,
    },
    GraphNodeAdded {
        node_id: String,
        node_type: String,
    },
    GraphEdgeAdded {
        edge_id: String,
        source: String,
        target: String,
    },
    HermesClassification {
        memory_id: String,
        decision: String,
        confidence: f64,
    },
    ConsolidationRun {
        items_processed: usize,
        duration_ms: u64,
    },
    ForgettingRun {
        items_affected: usize,
        duration_ms: u64,
    },
    ReflectionCompleted {
        task_id: String,
        insights: usize,
    },
}

impl fmt::Display for MemoryEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ItemStored {
                memory_id,
                memory_type,
                importance,
            } => {
                write!(
                    f,
                    "ItemStored: id={}, type={}, importance={}",
                    memory_id, memory_type, importance
                )
            }
            Self::ItemRetrieved {
                memory_id,
                memory_type,
            } => {
                write!(f, "ItemRetrieved: id={}, type={}", memory_id, memory_type)
            }
            Self::ItemConsolidated {
                memory_id,
                from_type,
                to_type,
            } => {
                write!(
                    f,
                    "ItemConsolidated: id={}, from={}, to={}",
                    memory_id, from_type, to_type
                )
            }
            Self::ItemCompressed { memory_id } => {
                write!(f, "ItemCompressed: id={}", memory_id)
            }
            Self::ItemArchived { memory_id } => {
                write!(f, "ItemArchived: id={}", memory_id)
            }
            Self::ItemDeleted { memory_id, reason } => {
                write!(f, "ItemDeleted: id={}, reason={}", memory_id, reason)
            }
            Self::ItemForgotten {
                memory_id,
                new_state,
            } => {
                write!(
                    f,
                    "ItemForgotten: id={}, new_state={}",
                    memory_id, new_state
                )
            }
            Self::GraphNodeAdded { node_id, node_type } => {
                write!(f, "GraphNodeAdded: id={}, type={}", node_id, node_type)
            }
            Self::GraphEdgeAdded {
                edge_id,
                source,
                target,
            } => {
                write!(
                    f,
                    "GraphEdgeAdded: id={}, source={}, target={}",
                    edge_id, source, target
                )
            }
            Self::HermesClassification {
                memory_id,
                decision,
                confidence,
            } => {
                write!(
                    f,
                    "HermesClassification: id={}, decision={}, confidence={}",
                    memory_id, decision, confidence
                )
            }
            Self::ConsolidationRun {
                items_processed,
                duration_ms,
            } => {
                write!(
                    f,
                    "ConsolidationRun: items={}, duration={}ms",
                    items_processed, duration_ms
                )
            }
            Self::ForgettingRun {
                items_affected,
                duration_ms,
            } => {
                write!(
                    f,
                    "ForgettingRun: items={}, duration={}ms",
                    items_affected, duration_ms
                )
            }
            Self::ReflectionCompleted { task_id, insights } => {
                write!(
                    f,
                    "ReflectionCompleted: task={}, insights={}",
                    task_id, insights
                )
            }
        }
    }
}
