use colored::Colorize;

pub fn print_help() {
    println!("{:━^60}", " GPTCLI ".yellow());
    println!("Usage:");
    println!("  {} [option] <argument>", "gpt".bold().green());
    println!("\nOptions:");
    println!("  {}   GPT-3.5-Turbo (default for text prompts).", " ");
    println!("  {}   GPT-4 model for text prompts.", "4".bold().cyan());
    println!(
        "  {}   GPT-4 Vision model for image analysis.",
        "v".bold().magenta()
    );
    println!(
        "  {}   DALL-E 3 model for image generation.",
        "d".bold().red()
    );
    println!(
        "  {}     Display this help message.",
        "-h, -help".bold().blue()
    );
    println!("\nArguments:");
    println!(
        "  {}  A text prompt for GPT-3.5-Turbo.",
        "<prompt>".bold().green()
    );
    println!("  {}  A text prompt for GPT-4.", "4 <prompt>".bold().cyan());
    println!(
        "  {}  A path to an image file and optional description for GPT-4 Vision.",
        "v <image_path> [description]".bold().magenta()
    );
    println!(
        "  {}  A text prompt for DALL-E 3.",
        "d <prompt>".bold().red()
    );
    println!("\nExamples:");
    println!(
        "  {} What is the capital of California?",
        "gpt".bold().green()
    );
    println!("  {} What is the meaning of life?", "gpt 4".bold().cyan());
    println!(
        "  {} rust_astronaut.jpg What colors are in this image?",
        "gpt v".bold().magenta()
    );
    println!(
        "  {} An astronaut on Mars in a rusty spacesuit holding a crab",
        "gpt d".bold().red()
    );
    println!("{:━^60}", "".yellow());
}
