load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")
load("@crates//:defs.bzl", "all_crate_deps")

rust_library(
    name = "harper_core",
    srcs = glob(["src/**/*.rs"]),
    deps = all_crate_deps(normal = True),
    proc_macro_deps = ["@crates//:async-trait"],
    rustc_env = {
        "HARPER_VERSION": "0.7.1",
    },
    visibility = ["//visibility:public"],
)

rust_test(
    name = "harper_core_test",
    crate = ":harper_core",
    deps = all_crate_deps(normal_dev = True),
    proc_macro_deps = ["@crates//:async-trait"],
    rustc_env = {
        "HARPER_VERSION": "0.7.1",
    },
)
