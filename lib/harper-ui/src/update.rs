use harper_core::{
    compare_versions, current_target_key, download_release_artifact, evaluate_update,
    extract_release_executable, fetch_release_manifest, install_downloaded_executable,
    resolve_install_source, resolve_update_public_key, save_persisted_install_source,
    verify_artifact_checksum, verify_artifact_signature, InstallSource,
};
use serde::Deserialize;
use std::env;

const UPDATE_MANIFEST_ENV: &str = "HARPER_UPDATE_MANIFEST_URL";
const RELEASES_API_URL: &str = "https://api.github.com/repos/harpertoken/harper/releases";
const MANIFEST_ASSET_NAME: &str = "release-manifest.json";

#[derive(Debug, Deserialize)]
struct GitHubReleaseAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    draft: bool,
    prerelease: bool,
    assets: Vec<GitHubReleaseAsset>,
}

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

pub async fn fetch_update_status() -> Result<Option<String>, String> {
    let manifest_url = resolve_manifest_url().await?;
    let executable = env::current_exe()
        .map_err(|err| format!("failed to resolve current executable: {}", err))?;
    let target = current_target_key();
    let _install_source = resolve_install_source(&executable)
        .map_err(|err| format!("failed to resolve install source: {}", err))?;
    let manifest = fetch_release_manifest(&manifest_url)
        .await
        .map_err(|err| err.to_string())?;
    let result = evaluate_update(crate::CLI_VERSION, &manifest, &target);

    if !result.artifact_available {
        return Ok(Some(format!("update: no {}", result.target)));
    }

    match compare_versions(&result.current_version, &result.latest_version) {
        Some(std::cmp::Ordering::Less) => Ok(Some(format!("update: {}", result.latest_version))),
        Some(std::cmp::Ordering::Equal) => Ok(Some("update: latest".to_string())),
        Some(std::cmp::Ordering::Greater) => Ok(Some("update: local newer".to_string())),
        None => Ok(Some("update: unknown".to_string())),
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
    let manifest_url = match resolve_manifest_url().await {
        Ok(url) => url,
        Err(err) => {
            eprintln!("Update check failed: {}", err);
            if check_only {
                return 2;
            }
            return 1;
        }
    };

    println!("harper v{}", crate::CLI_VERSION);
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

    let result = evaluate_update(crate::CLI_VERSION, &manifest, &target);
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

fn configured_manifest_url() -> Option<String> {
    env::var(UPDATE_MANIFEST_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

async fn resolve_manifest_url() -> Result<String, String> {
    if let Some(url) = configured_manifest_url() {
        return Ok(url);
    }

    let client = reqwest::Client::builder()
        .user_agent(format!("harper-ui/{}", crate::CLI_VERSION))
        .build()
        .map_err(|err| format!("failed to build update client: {}", err))?;

    let response = client
        .get(RELEASES_API_URL)
        .send()
        .await
        .map_err(|err| format!("API error: failed to fetch releases: {}", err))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|err| format!("API error: failed to read releases response: {}", err))?;

    if !status.is_success() {
        return Err(format!(
            "API error: release list request failed with status {}",
            status
        ));
    }

    let releases: Vec<GitHubRelease> = serde_json::from_str(&body)
        .map_err(|err| format!("API error: invalid releases response: {}", err))?;

    find_manifest_url(&releases).ok_or_else(|| {
        format!(
            "API error: no stable release asset named {} was found",
            MANIFEST_ASSET_NAME
        )
    })
}

fn find_manifest_url(releases: &[GitHubRelease]) -> Option<String> {
    releases
        .iter()
        .filter(|release| !release.draft && !release.prerelease)
        .flat_map(|release| release.assets.iter())
        .find(|asset| asset.name == MANIFEST_ASSET_NAME)
        .map(|asset| asset.browser_download_url.clone())
}

fn print_version() {
    println!("harper v{}", crate::CLI_VERSION);
}

fn print_self_update_usage() {
    eprintln!("Usage:");
    eprintln!("  harper self-update --check");
    eprintln!("  harper self-update");
}

#[cfg(test)]
mod tests {
    use super::{
        configured_manifest_url, fetch_update_status, find_manifest_url, handle_update_command,
        GitHubRelease, GitHubReleaseAsset,
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
    async fn fetch_update_status_is_err_with_invalid_manifest_env() {
        let saved = std::env::var_os(super::UPDATE_MANIFEST_ENV);
        unsafe {
            std::env::set_var(super::UPDATE_MANIFEST_ENV, "://invalid-manifest-url");
        }
        assert_eq!(
            configured_manifest_url().as_deref(),
            Some("://invalid-manifest-url")
        );
        assert!(fetch_update_status().await.is_err());
        if let Some(value) = saved {
            unsafe {
                std::env::set_var(super::UPDATE_MANIFEST_ENV, value);
            }
        } else {
            unsafe {
                std::env::remove_var(super::UPDATE_MANIFEST_ENV);
            }
        }
    }

    #[test]
    fn find_manifest_url_prefers_first_stable_release_with_manifest() {
        let releases = vec![
            GitHubRelease {
                draft: false,
                prerelease: false,
                assets: vec![],
            },
            GitHubRelease {
                draft: false,
                prerelease: true,
                assets: vec![GitHubReleaseAsset {
                    name: "release-manifest.json".to_string(),
                    browser_download_url: "https://example.com/prerelease.json".to_string(),
                }],
            },
            GitHubRelease {
                draft: false,
                prerelease: false,
                assets: vec![
                    GitHubReleaseAsset {
                        name: "something-else.txt".to_string(),
                        browser_download_url: "https://example.com/other.txt".to_string(),
                    },
                    GitHubReleaseAsset {
                        name: "release-manifest.json".to_string(),
                        browser_download_url: "https://example.com/release-manifest.json"
                            .to_string(),
                    },
                ],
            },
        ];

        assert_eq!(
            find_manifest_url(&releases).as_deref(),
            Some("https://example.com/release-manifest.json")
        );
    }
}
