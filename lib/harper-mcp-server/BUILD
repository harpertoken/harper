load("@rules_rust//rust:defs.bzl", "rust_binary")
load("@crates//:defs.bzl", "all_crate_deps")

exports_files(glob(["src/**/*.rs"]))

rust_binary(
    name = "harper_mcp_server",
    srcs = glob(["src/**/*.rs"]),
    deps = all_crate_deps(normal = True) + [
        "@crates//:lazy_static",
    ],
    visibility = ["//visibility:public"],
)
