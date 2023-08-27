extern crate bindgen;
extern crate cc;
extern crate num_cpus;

use bindgen::callbacks::{MacroParsingBehavior, ParseCallbacks};
use bindgen::Builder;
use std::collections::HashSet;
use std::env;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, RwLock};

const PHP_VERSION: &'static str = "php-8.2.9";

/// println_stderr and run_command_or_fail are copied from rdkafka-sys
macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

fn run_command_or_fail(dir: String, cmd: &str, args: &[&str]) {
    println_stderr!(
        "Running command: \"{} {}\" in dir: {}",
        cmd,
        args.join(" "),
        dir
    );
    let ret = Command::new(cmd).current_dir(dir).args(args).status();
    match ret.map(|status| (status.success(), status.code())) {
        Ok((true, _)) => return,
        Ok((false, Some(c))) => panic!("Command failed with error code {}", c),
        Ok((false, None)) => panic!("Command got killed"),
        Err(e) => panic!("Command failed with error: {}", e),
    }
}

fn target(path: &str) -> String {
    let osdir = env::var("PWD").unwrap();
    let pfx = match env::var("CARGO_TARGET_DIR") {
        Ok(d) => d,
        Err(_) => String::from("target"),
    };
    let profile = env::var("PROFILE").unwrap();
    format!("{}/{}/{}/native/{}", osdir, pfx, profile, path)
}

fn exists(path: &str) -> bool {
    Path::new(target(path).as_str()).exists()
}

/// This is needed to prevent bindgen to create multiple definitions of the same macro and fail
#[derive(Debug)]
struct MacroCallback {
    macros: Arc<RwLock<HashSet<String>>>,
}

impl ParseCallbacks for MacroCallback {
    fn will_parse_macro(&self, name: &str) -> MacroParsingBehavior {
        self.macros.write().unwrap().insert(name.into());

        match name {
            "FP_NAN" | "FP_INFINITE" | "FP_ZERO" | "FP_SUBNORMAL" | "FP_NORMAL" => {
                MacroParsingBehavior::Ignore
            }
            _ => MacroParsingBehavior::Default,
        }
    }
}

fn main() {
    let cpus = format!("{}", num_cpus::get());
    #[cfg(all(target_os = "linux"))]
    let default_link_static = false;
    #[cfg(all(target_os = "macos"))]
    let default_link_static = true;
    let php_version = option_env!("PHP_VERSION").unwrap_or(PHP_VERSION);
    let macros = Arc::new(RwLock::new(HashSet::new()));
    // Create the target directory if it doesn't exist
    std::fs::create_dir_all(target("")).unwrap();

    println!("cargo:rerun-if-env-changed=PHP_VERSION");
    println!("cargo:rerun-if-env-changed=PHP_LINK_STATIC");

    let link_dynamic = env::var_os("PHP_LINK_DYNAMIC")
        .map(|_| true)
        .unwrap_or(false);
    let link_static = env::var_os("PHP_LINK_STATIC")
        .map(|_| true)
        .unwrap_or(default_link_static && !link_dynamic);

    if !exists("php-src/LICENSE") {
        println_stderr!("Setting up PHP {}", php_version);
        run_command_or_fail(
            target(""),
            "git",
            &[
                "clone",
                "https://github.com/php/php-src",
                format!("--branch={}", php_version).as_str(),
                "--depth=1",
            ],
        );
        run_command_or_fail(
            target("php-src"),
            "sed",
            &[
                "-e",
                "s/void zend_signal_startup/ZEND_API void zend_signal_startup/g",
                "-ibk",
                "Zend/zend_signal.c",
                "Zend/zend_signal.h",
            ],
        );
        run_command_or_fail(target("php-src"), "./scripts/dev/genfiles", &[]);
        run_command_or_fail(target("php-src"), "./buildconf", &["--force"]);

        let embed_type = if link_static { "static" } else { "shared" };

        #[cfg(all(target_os = "linux"))]
        let config = &[
            //"--enable-debug",
            &format!("--enable-embed={}", embed_type),
            "--disable-cli",
            "--disable-cgi",
            "--enable-zts",
            // "--without-iconv",
            //"--disable-libxml",
            //"--disable-dom",
            //"--disable-xml",
            //"--disable-simplexml",
            //"--disable-xmlwriter",
            //"--disable-xmlreader",
            // "--without-pear",
            // "--with-libdir=lib64",
            // "--with-pic",
        ];
        #[cfg(all(target_os = "macos"))]
        let config = &[
            //"--enable-debug",
            &format!("--enable-embed={}", embed_type),
            "--disable-cli",
            "--disable-cgi",
            "--enable-zts",
            //"--without-iconv",
            //"--disable-libxml",
            //"--disable-dom",
            //"--disable-xml",
            //"--disable-simplexml",
            //"--disable-xmlwriter",
           // "--disable-xmlreader",
            // "--without-pear",
            // "--with-libdir=lib64",
            // "--with-pic",
        ];
        run_command_or_fail(target("php-src"), "./configure", config);
        run_command_or_fail(target("php-src"), "make", &["-j", cpus.as_str()]);
    }

    let include_dir = target("php-src");
    let lib_dir = target("php-src/libs");

    let link_type = if link_static { "=static" } else { "" };

    println!("cargo:rustc-link-lib{}=php", link_type);
    println!("cargo:rustc-link-search={}", lib_dir);

    let includes = ["/", "/TSRM", "/Zend", "/main"]
        .iter()
        .map(|d| format!("-I{}{}", include_dir, d))
        .collect::<Vec<String>>();
    dbg!(includes.clone().join(" "));
    let bindings = Builder::default()
        .clang_args(includes)
        .allowlist_function("_zend_file_handle__bindgen_ty_1")
        .allowlist_function("php_execute_script")
        .allowlist_function("php_module_startup")
        .allowlist_function("php_request_shutdown")
        .allowlist_function("php_request_startup")
        .allowlist_function("phprpm_fopen")
        .allowlist_function("sapi_send_headers")
        .allowlist_function("sapi_startup")
        .allowlist_function("sg_request_info")
        .allowlist_function("sg_sapi_headers")
        .allowlist_function("sg_server_context")
        .allowlist_function("sg_server_context")
        .allowlist_function("sg_set_server_context")
        .allowlist_function("sg_set_server_context")
        .allowlist_function("ts_resource_ex")
        .allowlist_function("tsrm_startup")
        .allowlist_function("zend_error")
        .allowlist_function("zend_signal_startup")
        .allowlist_function("zend_tsrmls_cache_update")
        .allowlist_function("_zend_string_init")
        .allowlist_function("_zend_string_release")
        .allowlist_function("_zend_string_release_ex")
        .allowlist_function("_zend_string_realloc")
        .allowlist_function("_zend_string_extend")
        .allowlist_function("_zend_string_truncate")
        .allowlist_function("_zend_string_dup")
        .allowlist_function("_zend_string_copy")
        .allowlist_var("SAPI_HEADER_SENT_SUCCESSFULLY")
        .allowlist_type("sapi_headers_struc")
        .allowlist_type("sapi_module_struc")
        .allowlist_type("sapi_request_info")
        .allowlist_type("ZEND_RESULT_CODE")
        .allowlist_type("zend_stream_type")
        .allowlist_type("zval")
        .parse_callbacks(Box::new(MacroCallback {
            macros: macros.clone(),
        })).derive_default(true)
        .header("wrapper.h")
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
    cc::Build::new()
        .file("src/shim.c")
        .include(&include_dir)
        .flag("-fPIC")
        .flag("-m64")
        .include(&format!("{}/TSRM", include_dir))
        .include(&format!("{}/Zend", include_dir))
        .include(&format!("{}/main", include_dir))
        .compile("foo");
}
