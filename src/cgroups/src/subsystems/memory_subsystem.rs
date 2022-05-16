use crate::subsystems::subsystem::*;
pub struct MemorySubsystem {

}

impl Subsystem for MemorySubsystem {
    fn name(&self) -> &str {
        "memory"
    }

    fn set(&self, path: &str, res: &ResourceConfig) -> Result<(), String> {
        unimplemented!()
    }

    fn apply(&self, path: &str, pid: i32) -> Result<(), String> {
        unimplemented!()
    }

    fn remove(&self, path: &str) -> Result<(), String> {
        unimplemented!()
    }
}

impl MemorySubsystem {
    pub fn new() -> Self {
        MemorySubsystem {
        }
    }
}
    

    
