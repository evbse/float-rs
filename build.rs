extern crate cc;

fn main() {
    std::env::set_var("CC", "clang");

    println!("cargo:rerun-if-changed=cpp");

    cc::Build::new()
        .file("cpp/link.cc")
        .flag("-std=c++17")
        .opt_level(3)
        .compile("float");
}
