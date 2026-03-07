pub trait HydraAdapter {
    fn register_with_hydra(&self) -> Result<(), String> {
        Ok(())
    }

    fn report_to_hydra(&self, _event: &str, _payload: &str) -> Result<(), String> {
        Ok(())
    }
}

pub trait AegisGhostWriter {
    fn snapshot(&self) -> Result<Vec<u8>, String> {
        Ok(Vec::new())
    }

    fn restore(&mut self, _data: &[u8]) -> Result<(), String> {
        Ok(())
    }
}

impl HydraAdapter for super::NoOpBridges {}
impl AegisGhostWriter for super::NoOpBridges {}
