load("@rules_rust//rust:defs.bzl", "rust_library", "rust_binary")
load("@crates//:defs.bzl", "all_crate_deps")

exports_files(glob(["src/**/*.rs"]))

rust_library(
    name = "harper_ui",
    crate_root = "src/lib.rs",
    srcs = glob(["src/**/*.rs"], exclude = ["src/main.rs", "src/bin/batch.rs"]),
    deps = [
        "//lib/harper-core:harper_core",
    ] + all_crate_deps(normal = True),
    proc_macro_deps = [
        "@crates//:async-trait",
    ],
    visibility = ["//visibility:public"],
)

rust_binary(
    name = "harper",
    crate_root = "src/main.rs",
    srcs = glob(["src/**/*.rs"]),
    deps = [
        ":harper_ui",
        "//lib/harper-core:harper_core",
        "@crates//:tokio",
        "@crates//:dotenvy",
        "@crates//:colored",
        "@crates//:turul-mcp-client",
        "@crates//:rusqlite",
    ] + all_crate_deps(normal = True),
    proc_macro_deps = [
        "@crates//:async-trait",
    ],
    crate_features = ["macros", "rt-multi-thread"],
    visibility = ["//visibility:public"],
)

rust_binary(
    name = "harper_batch",
    crate_root = "src/bin/batch.rs",
    srcs = ["src/bin/batch.rs"],
    deps = [
        ":harper_ui",
        "//lib/harper-core:harper_core",
    ] + all_crate_deps(normal = True),
    visibility = ["//visibility:public"],
)
