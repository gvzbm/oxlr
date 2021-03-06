use anyhow::*;

fn main() -> Result<()> {
    let input_path  = std::env::args().nth(1).expect("missing path to text module");
    let output_path = std::env::args().nth(2).expect("missing output path");
    let module: ir::Module = ron::from_str(&std::fs::read_to_string(&input_path)?)?;
    let output_path = format!("{}/{}#{}.om", output_path, module.path, module.version);
    println!("{} -> {}", input_path, output_path);
    let mut output = std::fs::File::create(output_path)?;
    rmp_serde::encode::write_named(&mut output, &module)?;
    Ok(())
}
