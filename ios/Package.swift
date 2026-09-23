// swift-tools-version:5.3
import PackageDescription

let package = Package(
  name: "tauri-plugin-webview-guard",
  platforms: [
    // Deliberately below the guard's own default floor (16.4): the guard has
    // to install and run on exactly the iOS versions it refuses, or it cannot
    // tell their users anything. 13 is what the sibling tauri-sign-keypair
    // declares; Tauri's own generated project defaults to 14.0.
    .iOS(.v13)
  ],
  products: [
    .library(
      name: "tauri-plugin-webview-guard",
      type: .static,
      targets: ["tauri-plugin-webview-guard"])
  ],
  dependencies: [
    .package(name: "Tauri", path: "../.tauri/tauri-api")
  ],
  targets: [
    .target(
      name: "tauri-plugin-webview-guard",
      dependencies: [
        .byName(name: "Tauri")
      ],
      path: "Sources")
  ]
)
