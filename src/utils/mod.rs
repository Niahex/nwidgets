pub mod icon;
pub mod runtime;

pub use icon::Icon;
// IconName est deprecated, utilisez Icon::new("nom-fichier") directement
#[allow(deprecated)]
pub use icon::IconName;
