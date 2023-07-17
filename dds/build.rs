fn main() {
    #[cfg(target_os = "linux")]
    println!(
        "cargo:rustc-link-arg=-Wl,/home/alvaro/Programming/workspace-rust/rusty-dealer/libdds.so"
    );

    #[cfg(target_os = "windows")]
    println!(
        "cargo:rustc-link-arg=C:\\Users\\gttlv\\RustProjects\\bridge_bro\\rusty_dealer\\dds\\dds.lib"
    );
}