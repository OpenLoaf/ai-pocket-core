// uniffi-bindgen CLI 入口。
//
// 该可执行文件供 scripts/gen-bindings.sh 调用，用来从已编译的 cdylib 生成
// Swift / Kotlin 绑定。它只是把命令行参数透传给 UniFFI 的标准入口。
fn main() {
    uniffi::uniffi_bindgen_main();
}
