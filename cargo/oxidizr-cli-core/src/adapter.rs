use std::path::Path;

use crate::packages::PackageKind;

pub trait DistroAdapter {
    /// Enumerate the set of command names provided by the distro package for this PackageKind
    /// under /usr/bin (and legacy /bin) for the given root. If enumeration isn't possible
    /// (e.g., non-live root), return an empty Vec.
    fn enumerate_package_commands(&self, root: &Path, pkg: PackageKind) -> Vec<String>;
}
