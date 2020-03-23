use crate::{analyze::Analysis, check::Extra};
//use proc_macro2::TokenStream as TokenStream2;
//use quote::quote;
use rtfm_syntax::ast::App;
// use std::{
//     env,
//     error::Error,
//     fmt, fs,
//     path::{Path, PathBuf},
//     process::{self, Command, Stdio},
//     time::{Instant, SystemTime},
// };
use std::{fs, path::Path};

// mod assertions;
// mod dispatchers;
// mod hardware_tasks;
// mod idle;
// mod init;
// mod locals;
// mod module;
// mod post_init;
// mod pre_init;
// mod resources;
// mod resources_struct;
// mod schedule;
// mod schedule_body;
// mod software_tasks;
// mod spawn;
// mod spawn_body;
// mod timer_queue;
// mod util;

pub fn app(_app: &App, _analysis: &Analysis, _extra: &Extra) {
    println!("// lane gen");
    if Path::new("memory_lane/src").exists() {
        fs::write("memory_lane/src/gen.rs", "hello").ok();
    } else {
        println!("// memory lost");
    }
}

// // forwarding of user arguments
// cargo.arg("rustc");
// if let Some(bin) = &opt.bin {
//     cargo.args(&["--bin", bin]);
// }

// if let Some(example) = &opt.example {
//     cargo.args(&["--example", &example]);
// }

// if opt.release {
//     cargo.arg("--release");
// }
// // klee specifics
// cargo
//     .args(&["--features", "klee-analysis"])
//     .arg("--")
//     // ignore linking
//     .args(&["-C", "linker=true"])
//     // force LTO, to get a single object file
//     .args(&["-C", "lto"])
//     // output the LLVM-IR (.ll file) for KLEE analysis
//     .arg("--emit=llvm-ir");
// // force panic=abort in all crates, override .cargo settings
// //.env("RUSTFLAGS", "-C panic=abort");

// println!("cargo {:?}", cargo);

// let status = cargo
//     .stdout(Stdio::inherit())
//     .stderr(Stdio::inherit())
//     .spawn()?
//     .wait()?;

// if !status.success() {
//     panic!("cargo trust command failed!");
// }

// // Try and get the cargo project information.
// let project = cargo_project::Project::query(".")
//     .map_err(|e| format_err!("failed to parse Cargo project information: {}", e))?;

// // Decide what artifact to use.
// let artifact = if let Some(bin) = &opt.bin {
//     cargo_project::Artifact::Bin(bin)
// // cargo trust --bin yyy
// } else if let Some(example) = &opt.example {
//     // cargo trust --example xxx
//     cargo_project::Artifact::Example(example)
// } else {
//     // cargo trust
//     cargo_project::Artifact::Bin(project.name())
// };

// let name = match artifact {
//     cargo_project::Artifact::Bin(bin) => bin,
//     cargo_project::Artifact::Example(example) => example,
//     _ => panic!("unimplemented artifact"),
// };

// println!("name {:?}", name);

// // Decide what profile to use.
// let profile = if opt.release {
//     cargo_project::Profile::Release
// } else {
//     cargo_project::Profile::Dev
// };

// // Try and get the artifact path.
// let path = project.path(
//     artifact,
//     profile,
//     opt.target.as_ref().map(|t| &**t),
//     "x86_64-unknown-linux-gnu",
// )?;

// let path_str = match path.to_str() {
//     Some(s) => s,
//     None => panic!(),
// };

// // get the directory of the binary;
// let dir = path.parent().expect("unreachable").to_path_buf();

// // lookup the latest .ll file
// // llvm-ir file
// let mut ll = None;
// // most recently modified
// let mut mrm = SystemTime::UNIX_EPOCH;
// // let prefix = format!("{}-", file.replace('-', "_"));

// println!("path {:?}", &dir);
// for e in fs::read_dir(&dir)? {
//     println!("e {:?}", e);
//     let e = e?;
//     let p = e.path();

//     if p.extension().map(|e| e == "ll").unwrap_or(false) {
//         if p.file_stem()
//             .expect("unreachable")
//             .to_str()
//             .expect("unreachable")
//             .starts_with(&name)
//         {
//             println!("found {}", name);
//             let modified = e.metadata()?.modified()?;
//             if ll.is_none() {
//                 ll = Some(p);
//                 mrm = modified;
//             } else {
//                 if modified > mrm {
//                     ll = Some(p);
//                     mrm = modified;
//                 }
//             }
//         }
//     }
// }

// println!("ll {:?}", ll);

// // klee analysis
// let mut klee = Command::new("klee");
// klee
//     // ll file to analyse
//     .arg(ll.unwrap());

// // execute the command and unwrap the result into status
// let status = klee.status()?;
// if !status.success() {
//     return Ok(status.code().unwrap_or(1));
// }

// // println!("    {} {}", "Flashing".green().bold(), path_str);

// return Ok(0);

// let matches = App::new("cargo-trust")
//     .version("0.1.0")
//     .author("Per Lindgren <per.lindgren@ltu.se>")
//     .about("The Verification Framework for Trusted Rust")
//     // as this is used as a Cargo subcommand the first argument will be the name of the binary
//     // we ignore this argument
//     .arg(Arg::with_name("binary-name").hidden(true))
//     // TODO: custom target support (for now only target host is supported)
//     // .arg(
//     //     Arg::with_name("target")
//     //         .long("target")
//     //         .takes_value(true)
//     //         .value_name("TRIPLE")
//     //         .help("Target triple for which the code is compiled"),
//     // )
//     .arg(
//         Arg::with_name("verbose")
//             .long("verbose")
//             .short("v")
//             .help("Use verbose output"),
//     )
//     .arg(
//         Arg::with_name("example")
//             .long("example")
//             .takes_value(true)
//             .value_name("NAME")
//             .required_unless("bin")
//             .conflicts_with("bin")
//             .help("Build only the specified example"),
//     )
//     .arg(
//         Arg::with_name("bin")
//             .long("bin")
//             .takes_value(true)
//             .value_name("NAME")
//             .required_unless("example")
//             .conflicts_with("example")
//             .help("Build only the specified binary"),
//     )
//     .arg(
//         Arg::with_name("release")
//             .long("release")
//             .help("Build artifacts in release mode, with optimizations"),
//     )
//     .arg(
//         Arg::with_name("features")
//             .long("features")
//             .takes_value(true)
//             .value_name("FEATURES")
//             .help("Space-separated list of features to activate"),
//     )
//     .arg(
//         Arg::with_name("all-features")
//             .long("all-features")
//             .takes_value(false)
//             .help("Activate all available features"),
//     )
//     // TODO, support additional parameters to KLEE
//     .arg(
//         Arg::with_name("klee")
//             .long("klee")
//             .short("k")
//             .help("Run KLEE test generatation [default enabled unless --replay]"),
//     )
//     .arg(
//         Arg::with_name("replay")
//             .long("replay")
//             .short("r")
//             .help("Generate replay binary in target directory"),
//     )
//     .arg(
//         Arg::with_name("gdb")
//             .long("gdb")
//             .short("g")
//             .help("Run the generated replay binary in `gdb`. The environment variable `GDB_CWD` determines the `gdb` working directory, if unset `gdb` will execute in the current working directory"),
//     )
//     .get_matches();

// let is_example = matches.is_present("example");
// let is_binary = matches.is_present("bin");
// let verbose = matches.is_present("verbose");
// let is_release = matches.is_present("release");
// let is_replay = matches.is_present("replay");
// let is_ktest = matches.is_present("klee");
// let is_gdb = matches.is_present("gdb");

// // let target_flag = matches.value_of("target"); // not currently supported

// // we rely on `clap` for either `example` or `bin`
// let file = if is_example {
//     matches.value_of("example").unwrap()
// } else {
//     matches.value_of("bin").unwrap()
// };

// // turn `cargo klee --example foo` into `cargo rustc --example foo -- (..)`
// let mut cargo = Command::new("cargo");
// cargo
//     // compile using rustc
//     .arg("rustc")
//     // verbose output for debugging purposes
//     .arg("-v");

// // set features, always inclidung `klee-analysis`
// if matches.is_present("all-features") {
//     cargo.arg("--all-features");
// } else {
//     if let Some(features) = matches.value_of("features") {
//         let mut vec: Vec<&str> = features.split(" ").collect::<Vec<&str>>();
//         vec.push("klee-analysis");
//         cargo.args(&["--features", &vec.join(" ")]);
//     } else {
//         cargo.args(&["--features", "klee-analysis"]);
//     }
// }

// // select (single) application to compile
// // derive basic settings from `cargo`
// if is_example {
//     cargo.args(&["--example", file]);
// } else {
//     cargo.args(&["--bin", file]);
// }

// // default is debug mode
// if is_release {
//     cargo.arg("--release");
// }

// cargo
//     // enable shell coloring of result
//     .arg("--color=always")
//     .arg("--")
//     // ignore linking
//     .args(&["-C", "linker=true"])
//     // force LTO, to get a single oject file
//     .args(&["-C", "lto"])
//     // output the LLVM-IR (.ll file) for KLEE analysis
//     .arg("--emit=llvm-ir")
//     // force panic=abort in all crates, override .cargo settings
//     .env("RUSTFLAGS", "-C panic=abort");
// // TODO, force `incremental=false`, `codegen-units=1`?

// if verbose {
//     eprintln!("\n{:?}\n", cargo);
// }

// // execute the command and unwrap the result into status
// let status = cargo.status()?;

// if !status.success() {
//     return Ok(status.code().unwrap_or(1));
// }

// let cwd = env::current_dir()?;

// let meta = rustc_version::version_meta()?;
// let host = meta.host;

// let project = Project::query(cwd)?;

// let profile = if is_release {
//     Profile::Release
// } else {
//     Profile::Dev
// };

// let mut path: PathBuf = if is_example {
//     project.path(Artifact::Example(file), profile, None, &host)?
// } else {
//     project.path(Artifact::Bin(file), profile, None, &host)?
// };

// // llvm-ir file
// let mut ll = None;
// // most recently modified
// let mut mrm = SystemTime::UNIX_EPOCH;
// let prefix = format!("{}-", file.replace('-', "_"));

// path = path.parent().expect("unreachable").to_path_buf();

// if is_binary {
//     path = path.join("deps"); // the .ll file is placed in ../deps
// }

// // lookup the latest .ll file
// for e in fs::read_dir(path)? {
//     let e = e?;
//     let p = e.path();

//     if p.extension().map(|e| e == "ll").unwrap_or(false) {
//         if p.file_stem()
//             .expect("unreachable")
//             .to_str()
//             .expect("unreachable")
//             .starts_with(&prefix)
//         {
//             let modified = e.metadata()?.modified()?;
//             if ll.is_none() {
//                 ll = Some(p);
//                 mrm = modified;
//             } else {
//                 if modified > mrm {
//                     ll = Some(p);
//                     mrm = modified;
//                 }
//             }
//         }
//     }
// }

// let mut obj = ll.clone().unwrap();
// let replay_name = obj.with_file_name(file).with_extension("replay");

// // replay compilation
// if is_replay {
//     // compile to object code for replay using `llc`
//     let mut llc = Command::new("llc");
//     llc.arg("-filetype=obj")
//         .arg("-relocation-model=pic")
//         .arg(ll.clone().unwrap());

//     if verbose {
//         eprintln!("\n{:?}\n", llc);
//     }

//     // TODO: better error handling, e.g., if `llc` is not installed/in path
//     let status = llc.status()?;
//     if !status.success() {
//         println!("llc failed: {:?}", status.code().unwrap_or(1));
//     } else {
//         // compile to executable for replay using `clang`
//         let mut clang = Command::new("clang");

//         obj = obj.with_extension("o");

//         clang
//             .arg(obj)
//             .arg("-lkleeRuntest")
//             .args(&["-o", replay_name.to_str().unwrap()]);

//         if verbose {
//             eprintln!("\n{:?}\n", clang);
//         }

//         // TODO: better error handling, e.g., if `clang` in not installed/in path
//         let status = clang.status()?;
//         if !status.success() {
//             println!("clang failed: {:?}", status.code().unwrap_or(1));
//         }
//     }
// }

// // klee analysis
// if is_ktest || !is_replay {
//     let mut klee = Command::new("klee");
//     klee
//         // ll file to analyse
//         .arg(ll.unwrap());

//     // execute the command and unwrap the result into status
//     let status = klee.status()?;
//     if !status.success() {
//         return Ok(status.code().unwrap_or(1));
//     }
// }

// // replay execution in `gdb`
// if is_gdb {
//     let mut gdb = Command::new("gdb");
//     if let Ok(cwd) = env::var("GDB_CWD") {
//         // set gdb current dir to `GDB_CWD`
//         gdb.current_dir(cwd);
//         // set replay name to be loaded by `gdb`
//         gdb.arg(replay_name);
//     } else {
//         // set gdb current dir to the target directory
//         gdb.current_dir(replay_name.parent().unwrap());
//         // set replay name to be loaded by gdb
//         gdb.arg(replay_name.file_name().unwrap());
//     };

//     if verbose {
//         eprintln!("\n{:?}\n", gdb);
//     }

//     // TODO: better error handling, e.g., if `gdb` is not installed/in path
//     let status = gdb.status()?;
//     if !status.success() {
//         println!("gdb failed: {:?}", status.code().unwrap_or(1));
//     }
// }
// // return to shell without error
// Ok(0)
//}
