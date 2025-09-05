use crate::error::Result;
use std::process::Command;

/// Trait defining the interface for core utility commands
pub trait CoreUtil: Send + Sync {
    /// Execute the core utility with the given arguments
    fn execute(&self, args: &[String]) -> Result<()>;
    
    /// Get the name of the utility
    fn name(&self) -> &'static str;
}

/// Implementation for GNU coreutils
pub struct GnuCoreUtil {
    pub command: String,
}

impl CoreUtil for GnuCoreUtil {
    fn execute(&self, args: &[String]) -> Result<()> {
        let output = Command::new(&self.command)
            .args(args)
            .spawn()?
            .wait()?;
        
        if output.success() {
            Ok(())
        } else {
            Err(crate::error::CoreutilsError::ExecutionFailed(
                format!("Command '{}' failed with status: {}", self.command, output)
            ))
        }
    }

    fn name(&self) -> &'static str {
        "GNU Coreutils"
    }
}

/// Implementation for uutils coreutils
pub struct UutilsCoreUtil {
    pub command: String,
}

impl CoreUtil for UutilsCoreUtil {
    fn execute(&self, args: &[String]) -> Result<()> {
        // uutils uses a single binary with subcommands
        let mut full_args = vec![self.command.clone()];
        full_args.extend_from_slice(args);
        
        let output = Command::new("uutils")
            .args(&full_args)
            .spawn()?
            .wait()?;
        
        if output.success() {
            Ok(())
        } else {
            Err(crate::error::CoreutilsError::ExecutionFailed(
                format!("Command 'uutils {}' failed with status: {}", self.command, output)
            ))
        }
    }

    fn name(&self) -> &'static str {
        "uutils Coreutils"
    }
}

/// Factory to create the appropriate core utility implementation
pub enum CoreUtilsImpl {
    Gnu,
    Uutils,
}

/// Factory function to create the appropriate core utility
pub fn create_core_util(name: &str, implementation: CoreUtilsImpl) -> Box<dyn CoreUtil> {
    match implementation {
        CoreUtilsImpl::Gnu => Box::new(GnuCoreUtil {
            command: name.to_string(),
        }),
        CoreUtilsImpl::Uutils => Box::new(UutilsCoreUtil {
            command: name.to_string(),
        }),
    }
}
