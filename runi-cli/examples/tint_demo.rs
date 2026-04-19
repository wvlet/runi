use runi_cli::Tint;

fn main() {
    println!("{}", Tint::red().paint("Error: something went wrong"));
    println!("{}", Tint::yellow().bold().paint("Warning: check this"));
    println!("{}", Tint::green().paint("Success: all tests passed"));
    println!("{}", Tint::cyan().paint("Info: processing..."));
    println!("{}", Tint::blue().dimmed().paint("Debug: internal state"));

    println!();

    // Chained styles
    println!("{}", Tint::white().bold().bg_red().paint(" FAIL "));
    println!("{}", Tint::black().bg_green().paint(" PASS "));
    println!("{}", Tint::white().bg_blue().paint(" INFO "));

    println!();

    // Style reuse
    let header = Tint::cyan().bold().underline();
    let dim = Tint::blue().dimmed();
    println!("{}", header.paint("=== Runi CLI Demo ==="));
    println!(
        "  {} {}",
        dim.paint("version:"),
        Tint::white().paint("0.1.0")
    );
    println!("  {} {}", dim.paint("crates:"), Tint::white().paint("4"));

    println!();

    // 256 and RGB colors
    println!("{}", Tint::color(208).paint("ANSI 256: orange"));
    println!("{}", Tint::rgb(255, 105, 180).paint("RGB: hot pink"));
}
