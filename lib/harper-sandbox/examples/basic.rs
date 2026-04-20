use harper_sandbox::{Sandbox, SandboxConfig};

fn main() {
    println!("Harper Sandbox Test\n");

    let config = SandboxConfig::default();
    let sandbox = Sandbox::new(config);

    println!("Backend: {}", sandbox.backend_name());
    println!("Available: {}", sandbox.is_available());
    println!("\n✓ harper-sandbox crate is working!");

    println!("\nTo enable sandbox, set in config:");
    println!("  [exec_policy.sandbox]");
    println!("  enabled = true");
}
