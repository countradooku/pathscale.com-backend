use std::fmt::Debug;

/// Routing target for messages in a [`SubscriptionRouter`].
///
/// [`SubscriptionRouter`]: super::SubscriptionRouter
#[derive(Debug, Clone)]
pub enum Receiver<K> {
    /// Broadcast to all subscribers
    All,
    /// Send to a specific key
    Concrete(K),
    /// Send to multiple specific keys
    ConcreteMulti(Vec<K>),
}
