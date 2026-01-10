load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library")
load("@crates//:defs.bzl", "all_crate_deps")

rust_library(
    name = "harper",
    srcs = glob(["src/**/*.rs"]),
    deps = all_crate_deps(normal = True),
)

rust_binary(
    name = "harper_bin",
    srcs = ["src/main.rs"],
    deps = [":harper"] + all_crate_deps(normal = True),
)