load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")
load("@crates//:defs.bzl", "all_crate_deps")

rust_library(
    name = "harper_sandbox",
    srcs = glob(["src/**/*.rs"]),
    deps = all_crate_deps(normal = True),
    visibility = ["//visibility:public"],
)

rust_test(
    name = "harper_sandbox_test",
    crate = ":harper_sandbox",
    deps = all_crate_deps(normal_dev = True),
)
