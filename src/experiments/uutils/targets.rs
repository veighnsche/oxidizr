use crate::error::Result;
use crate::experiments::uutils::model::UutilsExperiment;
use crate::experiments::uutils::utils::resolve_target;
use crate::utils::worker::Worker;
use std::path::PathBuf;

impl UutilsExperiment {
    /// Lists target paths that would be affected by this experiment.
    pub fn list_targets<W: Worker>(&self, worker: &W) -> Result<Vec<PathBuf>> {
        let files = worker.list_files(&self.bin_directory)?;
        let mut out = Vec::new();
        for f in files {
            let filename = f
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            if filename.is_empty() {
                continue;
            }
            out.push(resolve_target(worker, &filename));
        }
        Ok(out)
    }
}
