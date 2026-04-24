fn main() {
    let mut build = cc::Build::new();
    build
        .file("../opengl_wrapper_lib/opengl_wrapper_lib.c")
        .include("../opengl_wrapper_lib");

    if cfg!(target_os = "windows") {
        build.include("C:/msys64/mingw64/include");
    } else if cfg!(target_os = "macos") {
        let brew_prefix = std::process::Command::new("brew")
            .arg("--prefix")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();
        let brew_prefix = brew_prefix.trim();
        if !brew_prefix.is_empty() {
            build.include(format!("{}/include", brew_prefix));
        }
        build.define("GL_SILENCE_DEPRECATION", None);
    }

    build.compile("openglwrapper");

    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-search=native=C:/msys64/mingw64/lib");
        println!("cargo:rustc-link-lib=glfw3");
        println!("cargo:rustc-link-lib=opengl32");
        println!("cargo:rustc-link-lib=gdi32");
    } else if cfg!(target_os = "macos") {
        let brew_prefix = std::process::Command::new("brew")
            .arg("--prefix")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();
        let brew_prefix = brew_prefix.trim();
        if !brew_prefix.is_empty() {
            println!("cargo:rustc-link-search=native={}/lib", brew_prefix);
        }
        println!("cargo:rustc-link-lib=glfw");
        println!("cargo:rustc-link-lib=framework=OpenGL");
    } else {
        println!("cargo:rustc-link-lib=glfw");
        println!("cargo:rustc-link-lib=GL");
    }

    println!("cargo:rerun-if-changed=../opengl_wrapper_lib/opengl_wrapper_lib.c");
    println!("cargo:rerun-if-changed=../opengl_wrapper_lib/opengl_wrapper_lib.h");
}
