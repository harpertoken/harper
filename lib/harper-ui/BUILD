load("@rules_rust//rust:defs.bzl", "rust_library", "rust_binary")
load("@crates//:defs.bzl", "aliases")

exports_files(glob(["src/**/*.rs"]))

rust_library(
    name = "harper_ui",
    crate_root = "src/lib.rs",
    srcs = glob(["src/**/*.rs"], exclude = ["src/main.rs", "src/bin/batch.rs"]),
    aliases = aliases(
        normal = True,
        package_name = "lib/harper-ui",
    ),
    deps = [
        "//lib/harper-core:harper_core",
        "@crates//:arboard",
        "@crates//:colored",
        "@crates//:crossterm",
        "@crates//:dotenvy",
        "@crates//:image",
        "@crates//:keyring",
        "@crates//:ratatui",
        "@crates//:reqwest",
        "@crates//:rusqlite",
        "@crates//:rustyline",
        "@crates//:serde",
        "@crates//:serde_json",
        "@crates//:syntect",
        "@crates//:tokio",
        "@crates//:turul-mcp-client",
        "@crates//:uuid",
    ],
    proc_macro_deps = [
        "@crates//:async-trait",
    ],
    visibility = ["//visibility:public"],
)

rust_binary(
    name = "harper",
    crate_root = "src/main.rs",
    srcs = glob(["src/**/*.rs"]),
    aliases = aliases(
        normal = True,
        package_name = "lib/harper-ui",
    ),
    deps = [
        ":harper_ui",
        "//lib/harper-core:harper_core",
        "@crates//:colored",
        "@crates//:dotenvy",
        "@crates//:keyring",
        "@crates//:rusqlite",
        "@crates//:tokio",
        "@crates//:turul-mcp-client",
    ],
    crate_features = ["macros", "rt-multi-thread"],
    visibility = ["//visibility:public"],
)

rust_binary(
    name = "harper_batch",
    crate_root = "src/bin/batch.rs",
    srcs = ["src/bin/batch.rs"],
    aliases = aliases(
        normal = True,
        package_name = "lib/harper-ui",
    ),
    deps = [
        ":harper_ui",
        "@crates//:colored",
    ],
    visibility = ["//visibility:public"],
)
