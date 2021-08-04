const SHADER_BIN_DIR: &str ="shader_bin";

fn main() {
    create_dir_if_does_not_exist(std::path::Path::new(SHADER_BIN_DIR));

    compile_shader(std::path::PathBuf::from("frag.frag"));
    compile_shader(std::path::PathBuf::from("vert.vert"));
}

fn compile_shader(input: std::path::PathBuf) {
    let mut output = std::path::PathBuf::from(SHADER_BIN_DIR);
    output.push(input.clone());
    output.set_extension("spv");
    
    let result = std::process::Command::new("glslc.exe")
        .arg("./shaders/".to_owned() + input.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output();

    check_compilation_result(result);
}

fn create_dir_if_does_not_exist(dir: &std::path::Path) {
    if !dir.exists() {
        let create_dir_result = std::fs::create_dir(dir);
        match create_dir_result {
            Ok(_) => {}
            Err(error) => {
                panic!("Failed to create directory {}: {}", dir.display(), error);
            }
        }
    }
}

fn check_compilation_result(result: std::io::Result<std::process::Output>) {
    match result {
        Ok(output) => {
            if output.status.success() {
                println!("Shader compilation succedeed.");
                print!(
                    "stdout: {}",
                    String::from_utf8(output.stdout).unwrap_or("Failed to print program stdout".to_string())
                );
            } else {
                eprintln!("\nShader compilation failed ({})", output.status);
                eprint!(
                    "stdout:\n{}\n",
                    String::from_utf8(output.stdout).unwrap_or("Failed to print program stdout".to_string())
                );
                eprint!(
                    "stderr:\n{}\n",
                    String::from_utf8(output.stderr).unwrap_or("Failed to print program stderr".to_string())
                );
                panic!();
            }
        }
        Err(error) => {
            panic!("Failed to compile shader. Cause: {}", error);
        }
    }
}
