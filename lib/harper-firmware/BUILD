load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test", "rust_binary")
load("@crates//:defs.bzl", "all_crate_deps")

rust_library(
    name = "harper_firmware",
    srcs = glob(["src/**/*.rs"]),
    deps = all_crate_deps(normal = True),
    proc_macro_deps = ["@crates//:async-trait"],
    rustc_env = {
        "HARPER_VERSION": "0.1.0",
    },
    visibility = ["//visibility:public"],
)

rust_test(
    name = "harper_firmware_test",
    crate = ":harper_firmware",
    deps = all_crate_deps(normal_dev = True),
    proc_macro_deps = ["@crates//:async-trait"],
    rustc_env = {
        "HARPER_VERSION": "0.1.0",
    },
)

rust_binary(
    name = "basic",
    srcs = ["examples/basic.rs"],
    deps = [
        ":harper_firmware",
    ] + all_crate_deps(normal = True),
)
