use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn from_bytes(bytes: &[u8]) -> Self {
                let namespace = Uuid::NAMESPACE_OID;
                Self(Uuid::new_v5(&namespace, bytes))
            }

            pub fn from_string(s: &str) -> Self {
                Self::from_bytes(s.as_bytes())
            }

            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }

            pub fn to_hex(&self) -> String {
                self.0.to_string()
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_id!(AegisId);
define_id!(SessionId);
define_id!(ValidationId);
define_id!(SnapshotId);
define_id!(RollbackId);
