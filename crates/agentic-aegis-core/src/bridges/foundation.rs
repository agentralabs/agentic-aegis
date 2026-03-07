use super::traits::*;

pub struct FoundationBridges;

impl AegisBridge for FoundationBridges {}
impl TimeBridge for FoundationBridges {}
impl ContractBridge for FoundationBridges {}
impl IdentityBridge for FoundationBridges {}
impl MemoryBridge for FoundationBridges {}
impl CognitionBridge for FoundationBridges {}
impl CommBridge for FoundationBridges {}
impl CodebaseBridge for FoundationBridges {}
impl VisionBridge for FoundationBridges {}
impl PlanningBridge for FoundationBridges {}
impl RealityBridge for FoundationBridges {}
impl super::hydra::HydraAdapter for FoundationBridges {}
impl super::hydra::AegisGhostWriter for FoundationBridges {}
