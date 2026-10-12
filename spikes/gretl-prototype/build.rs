#![forbid(unsafe_code)]

fn main() {
  if std::env::var_os("CARGO_FEATURE_COUNT_PLUGIN").is_some() {
    // Link-args from a dependency's build script do not propagate to these
    // executable/test targets. The native core exports its build-owned directory.
    let directory = std::env::var("DEP_GRETL_SHARED_LIB_DIR")
      .expect("count-plugin shared-core directory metadata");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{directory}");
  }
}
