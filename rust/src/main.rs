use std::{env, fs, process};
use std::io::Write;
use std::path::Path;
use compiler::compile;

fn main() {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    // Check if input file was provided
    if args.len() < 2 {
        eprintln!("Usage: {} <input file>", args[0]);
        process::exit(1);
    }

    // Get the input file path
    let input_file = &args[1];
    let input_path = Path::new(input_file);

    // Check if the file exists
    if !input_path.exists() {
        eprintln!("File not found: {}", input_file);
        process::exit(1);
    }

    // Read the source code from the file
    let source = match fs::read_to_string(input_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file {}: {}", input_file, err);
            process::exit(1);
        }
    };

    // Determine the output file path (change extension to .asm)
    let output_path = {
        let mut path = input_path.to_path_buf();
        path.set_extension("asm");
        path
    };

    // Try to compile the source code
    match compile_and_write(&source, &output_path) {
        Ok(_) => {
            println!("Successfully compiled to: {}", output_path.display());
        }
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    }
}

/// Compile the source code and write the output to a file
fn compile_and_write(source: &str, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Compile the source code
    let output = compile(source.parse().unwrap())?;

    // Write the output to a file
    let mut file = fs::File::create(output_path)?;
    file.write_all(output.as_bytes())?;

    Ok(())
}
