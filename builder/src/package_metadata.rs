//! Metadata to customize builds on docs.rs.

use crate::Result;
use cargo::core::Package;
use failure::err_msg;
use std::fs;
use std::path::Path;
use toml::Value;

/// PackageMetadata for custom builds
///
/// You can customize docs.rs builds by defining `[package.metadata.docs.rs]` table in your
/// crates' `Cargo.toml`.
///
/// An example metadata:
///
/// ```text
/// [package]
/// name = "test"
///
/// [package.metadata.docs.rs]
/// features = [ "feature1", "feature2" ]
/// all-features = true
/// no-default-features = true
/// default-target = "x86_64-unknown-linux-gnu"
/// rustc-args = [ "--example-rustc-arg" ]
/// rustdoc-args = [ "--example-rustdoc-arg" ]
/// dependencies = [ "example-system-dependency" ]
/// ```
///
/// You can define one or more fields in your `Cargo.toml`.
pub struct PackageMetadata {
    /// List of features docs.rs will build.
    ///
    /// By default, docs.rs will only build default features.
    pub features: Option<Vec<String>>,

    /// Set `all-features` to true if you want docs.rs to build all features for your crate
    pub all_features: bool,

    /// Docs.rs will always build default features.
    ///
    /// Set `no-default-fatures` to `false` if you want to build only certain features.
    pub no_default_features: bool,

    /// Docs.rs is running on `x86_64-unknown-linux-gnu` target system and default documentation
    /// is always built on this target. You can change default target by setting this.
    pub default_target: Option<String>,

    /// List of command line arguments for `rustc`.
    pub rustc_args: Option<Vec<String>>,

    /// List of command line arguments for `rustdoc`.
    pub rustdoc_args: Option<Vec<String>>,

    /// System dependencies.
    pub dependencies: Option<Vec<String>>,
}

impl PackageMetadata {
    pub fn from_package(pkg: &Package) -> Result<PackageMetadata> {
        let src_path = pkg
            .manifest_path()
            .parent()
            .ok_or_else(|| err_msg("Source path not available"))?;
        for c in ["Cargo.toml.orig", "Cargo.toml"].iter() {
            let manifest_path = Path::new(src_path).join(c);
            if manifest_path.exists() {
                return PackageMetadata::from_manifest(manifest_path);
            }
        }
        Err(err_msg("Manifest not found"))
    }

    pub fn from_manifest(path: impl AsRef<Path>) -> Result<PackageMetadata> {
        let ctx = fs::read_to_string(path)?;
        Ok(PackageMetadata::from_str(&ctx))
    }

    // This is similar to Default trait but it's private
    fn default() -> PackageMetadata {
        PackageMetadata {
            features: None,
            all_features: false,
            no_default_features: false,
            default_target: None,
            rustc_args: None,
            rustdoc_args: None,
            dependencies: None,
        }
    }

    fn from_str(manifest: &str) -> PackageMetadata {
        let mut metadata = PackageMetadata::default();

        let manifest = match manifest.parse::<Value>() {
            Ok(m) => m,
            Err(_) => return metadata,
        };

        if let Some(table) = manifest
            .get("package")
            .and_then(|p| p.as_table())
            .and_then(|p| p.get("metadata"))
            .and_then(|p| p.as_table())
            .and_then(|p| p.get("docs"))
            .and_then(|p| p.as_table())
            .and_then(|p| p.get("rs"))
            .and_then(|p| p.as_table())
        {
            metadata.features = table
                .get("features")
                .and_then(|f| f.as_array())
                .and_then(|f| f.iter().map(|v| v.as_str().map(|v| v.to_owned())).collect());
            metadata.no_default_features = table
                .get("no-default-features")
                .and_then(|v| v.as_bool())
                .unwrap_or(metadata.no_default_features);
            metadata.all_features = table
                .get("all-features")
                .and_then(|v| v.as_bool())
                .unwrap_or(metadata.all_features);
            metadata.default_target = table
                .get("default-target")
                .and_then(|v| v.as_str())
                .map(|v| v.to_owned());
            metadata.rustc_args = table
                .get("rustc-args")
                .and_then(|f| f.as_array())
                .and_then(|f| f.iter().map(|v| v.as_str().map(|v| v.to_owned())).collect());
            metadata.rustdoc_args = table
                .get("rustdoc-args")
                .and_then(|f| f.as_array())
                .and_then(|f| f.iter().map(|v| v.as_str().map(|v| v.to_owned())).collect());
            metadata.dependencies = table
                .get("dependencies")
                .and_then(|f| f.as_array())
                .and_then(|f| f.iter().map(|v| v.as_str().map(|v| v.to_owned())).collect());
        }

        metadata
    }
}

#[cfg(test)]
mod test {
    use super::PackageMetadata;

    #[test]
    fn test_metadata() {
        let manifest = r#"
            [package]
            name = "test"

            [package.metadata.docs.rs]
            features = [ "feature1", "feature2" ]
            all-features = true
            no-default-features = true
            default-target = "x86_64-unknown-linux-gnu"
            rustc-args = [ "--example-rustc-arg" ]
            rustdoc-args = [ "--example-rustdoc-arg" ]
            dependencies = [ "example-system-dependency" ]
        "#;

        let metadata = PackageMetadata::from_str(manifest);

        assert!(metadata.features.is_some());
        assert!(metadata.all_features == true);
        assert!(metadata.no_default_features == true);
        assert!(metadata.default_target.is_some());
        assert!(metadata.rustdoc_args.is_some());

        let features = metadata.features.unwrap();
        assert_eq!(features.len(), 2);
        assert_eq!(features[0], "feature1".to_owned());
        assert_eq!(features[1], "feature2".to_owned());

        assert_eq!(
            metadata.default_target.unwrap(),
            "x86_64-unknown-linux-gnu".to_owned()
        );

        let rustc_args = metadata.rustc_args.unwrap();
        assert_eq!(rustc_args.len(), 1);
        assert_eq!(rustc_args[0], "--example-rustc-arg".to_owned());

        let rustdoc_args = metadata.rustdoc_args.unwrap();
        assert_eq!(rustdoc_args.len(), 1);
        assert_eq!(rustdoc_args[0], "--example-rustdoc-arg".to_owned());

        let dependencies = metadata.dependencies.unwrap();
        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0], "example-system-dependency".to_owned());
    }
}
