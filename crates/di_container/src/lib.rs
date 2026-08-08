//! Lightweight dependency injection container.

pub mod error;
pub mod traits;

pub use error::{ContainerError, Result};
pub use traits::{Injectable, Service};

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Service scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// Singleton — one instance for the entire application.
    Singleton,
    /// Scoped — one instance per scope.
    Scoped,
    /// Transient — new instance every time.
    Transient,
}

/// A registered service entry.
struct ServiceEntry {
    factory: Box<dyn Fn() -> Arc<dyn Any + Send + Sync> + Send + Sync>,
    scope: Scope,
}

/// Dependency injection container.
pub struct Container {
    services: RwLock<HashMap<TypeId, ServiceEntry>>,
    singletons: RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl Container {
    /// Create a new container.
    pub fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
            singletons: RwLock::new(HashMap::new()),
        }
    }

    /// Register a singleton service.
    pub async fn register_singleton<T, F>(&self, factory: F)
    where
        T: Send + Sync + 'static,
        F: Fn() -> Arc<T> + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let entry = ServiceEntry {
            factory: Box::new(move || {
                let instance = factory();
                instance as Arc<dyn Any + Send + Sync>
            }),
            scope: Scope::Singleton,
        };
        self.services.write().await.insert(type_id, entry);
    }

    /// Register a transient service.
    pub async fn register_transient<T, F>(&self, factory: F)
    where
        T: Send + Sync + 'static,
        F: Fn() -> Arc<T> + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let entry = ServiceEntry {
            factory: Box::new(move || {
                let instance = factory();
                instance as Arc<dyn Any + Send + Sync>
            }),
            scope: Scope::Transient,
        };
        self.services.write().await.insert(type_id, entry);
    }

    /// Resolve a service.
    pub async fn resolve<T: Send + Sync + 'static>(&self) -> Result<Arc<T>> {
        let type_id = TypeId::of::<T>();

        let services = self.services.read().await;
        let entry = services
            .get(&type_id)
            .ok_or_else(|| ContainerError::NotRegistered(std::any::type_name::<T>().to_string()))?;

        match entry.scope {
            Scope::Singleton => {
                let mut singletons = self.singletons.write().await;
                if let Some(instance) = singletons.get(&type_id) {
                    instance
                        .clone()
                        .downcast::<T>()
                        .map_err(|_| ContainerError::ResolutionFailed("Type mismatch".to_string()))
                } else {
                    let instance = (entry.factory)();
                    singletons.insert(type_id, instance.clone());
                    instance
                        .downcast::<T>()
                        .map_err(|_| ContainerError::ResolutionFailed("Type mismatch".to_string()))
                }
            }
            Scope::Transient | Scope::Scoped => {
                let instance = (entry.factory)();
                instance
                    .downcast::<T>()
                    .map_err(|_| ContainerError::ResolutionFailed("Type mismatch".to_string()))
            }
        }
    }

    /// Check if a service is registered.
    pub async fn is_registered<T: 'static>(&self) -> bool {
        self.services.read().await.contains_key(&TypeId::of::<T>())
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestService {
        value: i32,
    }

    #[tokio::test]
    async fn container_creation() {
        let _container = Container::new();
    }

    #[tokio::test]
    async fn register_and_resolve() {
        let container = Container::new();
        container
            .register_singleton(|| Arc::new(TestService { value: 42 }))
            .await;

        let service = container.resolve::<TestService>().await.unwrap();
        assert_eq!(service.value, 42);
    }

    #[tokio::test]
    async fn resolve_unregistered() {
        let container = Container::new();
        let result = container.resolve::<TestService>().await;
        assert!(result.is_err());
    }
}
