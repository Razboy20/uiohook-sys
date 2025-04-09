extern crate bindgen;

use std::env;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
  let target = env::var("TARGET").expect("TARGET was not set");

  println!("cargo:rerun-if-changed=build.rs");

  if cfg!(feature = "with-bindgen") {
    let bindings = bindgen::Builder::default()
      .header("vendor/include/uiohook.h")
      .allowlist_var("_event_type_EVENT.*")
      .allowlist_var("_log_level_LOG_LEVEL.*")
      .allowlist_function("logger_proc")
      .allowlist_function("hook_set_logger_proc")
      .allowlist_function("hook_post_event")
      .allowlist_function("hook_set_dispatch_proc")
      .allowlist_function("hook_run")
      .allowlist_function("hook_stop")
      .allowlist_function("hook_create_screen_info")
      .allowlist_function("hook_get_auto_repeat_rate")
      .allowlist_function("hook_get_auto_repeat_delay")
      .allowlist_function("hook_get_pointer_acceleration_multiplier")
      .allowlist_function("hook_get_pointer_acceleration_threshold")
      .allowlist_function("hook_get_pointer_sensitivity")
      .allowlist_function("hook_get_multi_click_time")
      .trust_clang_mangling(false)
      .rustfmt_bindings(true)
      .derive_debug(false)
      .generate()
      .expect("Unable to generate bindings");

    let out_path = PathBuf::from("src").join("bindings.rs");

    let data = bindings
      .to_string()
      .replace("_event_type_EVENT", "EVENT")
      .replace("_log_level_LOG_LEVEL", "LOG_LEVEL");

    let mut file = File::create(out_path).expect("couldn't open file!");
    file
      .write_all(data.as_bytes())
      .expect("couldn't write bindings.rs!");
  }

  if cfg!(feature = "static") {
    if !Path::new("vendor/.git").exists() {
      let _ = Command::new("git")
        .args(&["submodule", "update", "--init"])
        .status();
    }

    let dst = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR should be set."));
    let include = dst.join("include");
    let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set.");

    fs::create_dir_all(&include).expect("Directory should have been created.");
    fs::copy("vendor/include/uiohook.h", include.join("uiohook.h"))
      .expect("vendor/include/uiohook.h should exist.");

    let libdir = dst.join("vendor/build");
    if !libdir.exists() {
      let mut args = vec![
        "-S",
        "..",
        "-DBUILD_DEMO=OFF",
        "-DUSE_CARBON_LEGACY=OFF",
        "-DBUILD_SHARED_LIBS=ON",
        "-DCMAKE_INSTALL_PREFIX=../dist",
      ];

      if target.contains("musl") {
        env::set_var("CC", "musl-gcc");
      }

      if target.contains("windows-gnu") {
        if target.contains("x86_64") {
          args.push("-DCMAKE_SYSTEM_NAME=x86_64-w64-mingw32");
        } else {
          args.push("-DCMAKE_SYSTEM_NAME=i686-w64-mingw32");
        }
      }

      // Create build directory
      let vendor_build_dir = PathBuf::from("vendor/build");
      fs::create_dir_all(&vendor_build_dir).expect("Failed to create vendor/build directory");

      // Run cmake configure
      Command::new("cmake")
        .current_dir(&vendor_build_dir)
        .args(&args)
        .status()
        .expect("Failed to run cmake configure");

      // Run cmake build and install
      Command::new("cmake")
        .current_dir(&vendor_build_dir)
        .args(&["--build", ".", "--parallel", "2"])
        .status()
        .expect("Failed to run cmake build");
    }

    println!("cargo:rustc-link-search={}", libdir.display());
    println!("cargo:rustc-link-lib=uiohook");

    if target.contains("darwin") {
      println!("cargo:rustc-link-lib=framework=IOKit");
      println!("cargo:rustc-link-lib=framework=Carbon");
    }
  // println!("cargo:root={}", env::var("OUT_DIR").unwrap());
  } else {
    println!("cargo:rustc-link-lib=uiohook");
  }
}
