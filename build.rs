
fn main() {
    // Add library path for MacOS openSSL
    // TODO Do nothing on linux
    println!("cargo:rustc-env=LIBRARY_PATH=/usr/local/opt/openssl/lib");
}
