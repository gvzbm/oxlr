use anyhow::*;

fn main() -> Result<()> {
    let input_path  = std::env::args().nth(1).expect("missing path to text module");
    let output_path = std::env::args().nth(2).expect("missing output path");
    println!("{} -> {}", input_path, output_path);
    let module: ir::Module = ron::from_str(&std::fs::read_to_string(input_path)?)?;
    println!("module: {} v{}", module.path, module.version);
    let mut output = std::fs::File::create(output_path)?;
    rmp_serde::encode::write_named(&mut output, &module)?;
    Ok(())
}
