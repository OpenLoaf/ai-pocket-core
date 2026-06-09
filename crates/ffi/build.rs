// UniFFI 构建脚本（proc-macro 模式，无 .udl）。
//
// 纯 proc-macro 方式下，scaffolding 由源码里的 `uniffi::setup_scaffolding!()`
// 与 `#[uniffi::export]` 在编译期注入，build.rs 不需要再生成 scaffolding。
// 这里保留 build.rs 仅用于声明重建依赖，并为将来切换到 .udl 留位。
fn main() {
    // 若以后改用 .udl 定义接口，可在此调用：
    //   uniffi::generate_scaffolding("src/ai_pocket_ffi.udl").unwrap();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=uniffi.toml");
}
