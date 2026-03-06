use std::fmt::Debug;

use super::receiver::Receiver;

#[derive(Debug, Clone)]
pub struct RoutingMessage<K, M> {
    pub receiver: Receiver<K>,
    pub payload: M,
}

impl<K, M> RoutingMessage<K, M> {
    pub fn for_concrete(key: K, payload: M) -> Self {
        Self {
            receiver: Receiver::Concrete(key),
            payload,
        }
    }

    #[allow(dead_code)]
    pub fn for_all(payload: M) -> Self {
        Self {
            receiver: Receiver::All,
            payload,
        }
    }

    #[allow(dead_code)]
    pub fn for_multi(keys: Vec<K>, payload: M) -> Self {
        Self {
            receiver: Receiver::ConcreteMulti(keys),
            payload,
        }
    }
}
