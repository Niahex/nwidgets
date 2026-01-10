fn main() {
    println!("cargo:rustc-link-search=native=/nix/store/kij99q5dsm360pvcknzp3k5q8pkj666r-system-path/lib");
    println!("cargo:rustc-link-lib=gbm");
}
