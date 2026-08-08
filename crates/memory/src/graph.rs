use crate::error::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(pub String);

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    User,
    Person,
    Device,
    Skill,
    Preference,
    Project,
    Task,
    Location,
    Application,
    Relationship,
    Organization,
    Custom(String),
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::User => write!(f, "User"),
            Self::Person => write!(f, "Person"),
            Self::Device => write!(f, "Device"),
            Self::Skill => write!(f, "Skill"),
            Self::Preference => write!(f, "Preference"),
            Self::Project => write!(f, "Project"),
            Self::Task => write!(f, "Task"),
            Self::Location => write!(f, "Location"),
            Self::Application => write!(f, "Application"),
            Self::Relationship => write!(f, "Relationship"),
            Self::Organization => write!(f, "Organization"),
            Self::Custom(s) => write!(f, "Custom({})", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeType {
    Uses,
    Owns,
    Likes,
    Dislikes,
    Created,
    WorksOn,
    ConnectedTo,
    DependsOn,
    AssignedTo,
    RelatedTo,
    Custom(String),
}

impl fmt::Display for EdgeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uses => write!(f, "Uses"),
            Self::Owns => write!(f, "Owns"),
            Self::Likes => write!(f, "Likes"),
            Self::Dislikes => write!(f, "Dislikes"),
            Self::Created => write!(f, "Created"),
            Self::WorksOn => write!(f, "WorksOn"),
            Self::ConnectedTo => write!(f, "ConnectedTo"),
            Self::DependsOn => write!(f, "DependsOn"),
            Self::AssignedTo => write!(f, "AssignedTo"),
            Self::RelatedTo => write!(f, "RelatedTo"),
            Self::Custom(s) => write!(f, "Custom({})", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: NodeId,
    pub node_type: NodeType,
    pub name: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub id: String,
    pub source: NodeId,
    pub target: NodeId,
    pub edge_type: EdgeType,
    pub weight: f64,
    pub properties: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct GraphQuery {
    pub source: Option<NodeId>,
    pub target: Option<NodeId>,
    pub edge_types: Option<Vec<EdgeType>>,
    pub node_types: Option<Vec<NodeType>>,
    pub max_depth: Option<usize>,
    pub max_results: usize,
}

#[async_trait::async_trait]
pub trait KnowledgeGraph: Send + Sync {
    async fn add_node(&self, node: GraphNode) -> Result<NodeId>;
    async fn get_node(&self, node_id: &NodeId) -> Result<GraphNode>;
    async fn update_node(&self, node: GraphNode) -> Result<()>;
    async fn delete_node(&self, node_id: &NodeId) -> Result<()>;
    async fn node_exists(&self, node_id: &NodeId) -> bool;
    async fn add_edge(&self, edge: GraphEdge) -> Result<String>;
    async fn get_edge(&self, edge_id: &str) -> Result<GraphEdge>;
    async fn delete_edge(&self, edge_id: &str) -> Result<()>;
    async fn query_graph(&self, query: &GraphQuery) -> Result<(Vec<GraphNode>, Vec<GraphEdge>)>;
    async fn find_path(
        &self,
        from: &NodeId,
        to: &NodeId,
        max_depth: usize,
    ) -> Result<Vec<Vec<GraphEdge>>>;
    async fn get_neighbors(&self, node_id: &NodeId) -> Result<Vec<(GraphNode, GraphEdge)>>;
    async fn node_count(&self) -> usize;
    async fn edge_count(&self) -> usize;
    async fn clear(&self) -> Result<()>;
}
