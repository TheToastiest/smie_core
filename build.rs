fn main() {
    println!("cargo:rustc-link-search=native=C:/Users/brinn/Source Builds/faiss/build/faiss/Release");
    println!("cargo:rustc-link-search=native=C:/Users/brinn/Source Builds/faiss/build/c_api/Release");
    println!("cargo:rustc-link-search=native=C:/OpenBLAS/lib");

println!("cargo:rustc-link-lib=dylib=faiss");
println!("cargo:rustc-link-lib=dylib=faiss_c");
println!("cargo:rustc-link-lib=dylib=libopenblas"); // remove "lib" prefix

}
