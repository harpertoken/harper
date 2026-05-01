use harper_core::{
    compare_versions, current_target_key, download_release_artifact, evaluate_update,
    extract_release_executable, fetch_release_manifest, install_downloaded_executable,
    resolve_install_source, resolve_update_public_key, save_persisted_install_source,
    verify_artifact_checksum, verify_artifact_signature, InstallSource, VERSION,
};
use std::env;

const UPDATE_MANIFEST_ENV: &str = "HARPER_UPDATE_MANIFEST_URL";
const DEFAULT_UPDATE_MANIFEST_URL: &str =
    "https://github.com/harpertoken/harper/releases/latest/download/release-manifest.json";

pub async fn handle_update_command(args: &[String]) -> Option<i32> {
    if args.len() < 2 {
        return None;
    }

    match args[1].as_str() {
        "version" => {
            print_version();
            Some(0)
        }
        "self-update" => Some(handle_self_update(args).await),
        _ => None,
    }
}

pub async fn fetch_update_status() -> Option<String> {
    let manifest_url = manifest_url();
    let executable = env::current_exe().ok()?;
    let target = current_target_key();
    let _install_source = resolve_install_source(&executable).ok()?;
    let manifest = fetch_release_manifest(&manifest_url).await.ok()?;
    let result = evaluate_update(VERSION, &manifest, &target);

    if !result.artifact_available {
        return Some(format!("update: no {}", result.target));
    }

    match compare_versions(&result.current_version, &result.latest_version) {
        Some(std::cmp::Ordering::Less) => Some(format!("update: {}", result.latest_version)),
        Some(std::cmp::Ordering::Equal) => Some("update: latest".to_string()),
        Some(std::cmp::Ordering::Greater) => Some("update: local newer".to_string()),
        None => Some("update: unknown".to_string()),
    }
}

async fn handle_self_update(args: &[String]) -> i32 {
    let check_only = args.iter().skip(2).any(|arg| arg == "--check");
    if args
        .iter()
        .skip(2)
        .any(|arg| arg == "--help" || arg == "-h")
    {
        print_self_update_usage();
        return 0;
    }

    let executable = match env::current_exe() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("Failed to resolve current executable: {}", err);
            return 1;
        }
    };

    let install_source = match resolve_install_source(&executable) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("Failed to resolve install source: {}", err);
            return 1;
        }
    };
    let target = current_target_key();
    let manifest_url = manifest_url();

    println!("harper v{}", VERSION);
    println!("Install source: {}", install_source.display_name());
    println!("Target: {}", target);

    println!("Manifest: {}", manifest_url);

    let manifest = match fetch_release_manifest(&manifest_url).await {
        Ok(manifest) => manifest,
        Err(err) => {
            eprintln!("Update check failed: {}", err);
            if check_only {
                return 2;
            }
            return 1;
        }
    };

    if install_source != InstallSource::Unknown {
        if let Err(err) = save_persisted_install_source(install_source) {
            eprintln!("Warning: failed to persist install metadata: {}", err);
        }
    }

    let result = evaluate_update(VERSION, &manifest, &target);
    println!("Latest version: {}", result.latest_version);

    if !result.artifact_available {
        println!(
            "No release artifact is available for target {} in the current manifest.",
            result.target
        );
        return 2;
    }

    match compare_versions(&result.current_version, &result.latest_version) {
        Some(std::cmp::Ordering::Less) => {
            println!(
                "Update available: {} -> {}",
                result.current_version, result.latest_version
            );
        }
        Some(std::cmp::Ordering::Equal) => {
            println!("You are already on the latest version.");
        }
        Some(std::cmp::Ordering::Greater) => {
            println!("Current version is newer than the published manifest.");
        }
        None => {
            println!("Could not compare current and latest versions.");
            return 2;
        }
    }

    if check_only {
        return 0;
    }

    if !result.update_available {
        return 0;
    }

    match install_source {
        InstallSource::Homebrew | InstallSource::Cargo | InstallSource::Npm => {
            print_install_source_guidance(install_source)
        }
        InstallSource::Direct => {
            let Some(artifact) = manifest.artifact_for_target(&target) else {
                println!(
                    "No release artifact is available for target {} in the current manifest.",
                    target
                );
                return 2;
            };

            let bytes = match download_release_artifact(&artifact.url).await {
                Ok(bytes) => bytes,
                Err(err) => {
                    eprintln!("Update download failed: {}", err);
                    return 1;
                }
            };

            if let Err(err) = verify_artifact_checksum(&bytes, &artifact.sha256) {
                eprintln!("Update verification failed: {}", err);
                return 1;
            }

            let Some(signature) = artifact.signature.as_deref() else {
                eprintln!("Update verification failed: release artifact signature is missing.");
                return 1;
            };

            let public_key = match resolve_update_public_key() {
                Ok(key) => key,
                Err(err) => {
                    eprintln!("Update verification failed: {}", err);
                    return 1;
                }
            };

            if let Err(err) = verify_artifact_signature(&bytes, signature, &public_key) {
                eprintln!("Update verification failed: {}", err);
                return 1;
            }

            let executable_name = match executable.file_name().and_then(|name| name.to_str()) {
                Some(name) => name,
                None => {
                    eprintln!("Could not determine executable filename for update installation.");
                    return 1;
                }
            };

            let install_bytes =
                match extract_release_executable(&artifact.url, &bytes, executable_name) {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        eprintln!("Update extraction failed: {}", err);
                        return 1;
                    }
                };

            if let Err(err) = install_downloaded_executable(&executable, &install_bytes) {
                eprintln!("Update install failed: {}", err);
                return 1;
            }

            if let Err(err) = save_persisted_install_source(InstallSource::Direct) {
                eprintln!(
                    "Updated Harper, but failed to persist install metadata: {}",
                    err
                );
                return 1;
            }

            println!("Updated Harper to {}.", result.latest_version);
            0
        }
        InstallSource::Unknown => {
            println!("Install source is unknown.");
            println!("Refusing to mutate this install automatically.");
            println!("Use `harper self-update --check` and update the binary manually.");
            2
        }
    }
}

fn print_install_source_guidance(install_source: InstallSource) -> i32 {
    match install_source {
        InstallSource::Homebrew => {
            println!("This install is managed by Homebrew.");
            println!("Run: brew upgrade harpertoken/tap/harper-ai");
            0
        }
        InstallSource::Cargo => {
            println!("This install is managed by cargo.");
            println!("Run: cargo install harper-ui --force");
            0
        }
        InstallSource::Npm => {
            println!("This install is managed by npm.");
            println!("Run: npm install -g harper-ai@latest");
            0
        }
        InstallSource::Direct | InstallSource::Unknown => {
            println!("This install is not managed by a package manager.");
            println!("Use `harper self-update --check` to inspect the published release manifest.");
            2
        }
    }
}

fn manifest_url() -> String {
    env::var(UPDATE_MANIFEST_ENV).unwrap_or_else(|_| DEFAULT_UPDATE_MANIFEST_URL.to_string())
}

fn print_version() {
    println!("harper v{}", VERSION);
}

fn print_self_update_usage() {
    eprintln!("Usage:");
    eprintln!("  harper self-update --check");
    eprintln!("  harper self-update");
}

#[cfg(test)]
mod tests {
    use super::{
        fetch_update_status, handle_update_command, manifest_url, DEFAULT_UPDATE_MANIFEST_URL,
    };

    #[tokio::test]
    async fn version_command_is_handled() {
        let args = vec!["harper".to_string(), "version".to_string()];
        assert_eq!(handle_update_command(&args).await, Some(0));
    }

    #[tokio::test]
    async fn self_update_command_is_handled() {
        let args = vec![
            "harper".to_string(),
            "self-update".to_string(),
            "--check".to_string(),
        ];
        assert_eq!(handle_update_command(&args).await, Some(2));
    }

    #[tokio::test]
    async fn fetch_update_status_is_none_without_manifest_env() {
        let saved = std::env::var_os(super::UPDATE_MANIFEST_ENV);
        unsafe {
            std::env::remove_var(super::UPDATE_MANIFEST_ENV);
        }
        assert_eq!(manifest_url(), DEFAULT_UPDATE_MANIFEST_URL);
        assert!(fetch_update_status().await.is_none());
        if let Some(value) = saved {
            unsafe {
                std::env::set_var(super::UPDATE_MANIFEST_ENV, value);
            }
        }
    }
}
