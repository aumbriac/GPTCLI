# GPTCLI - Rust CLI for GPT API

**This project is not affiliated with OpenAI.**

This command-line tool provides a simple interface for interacting with the following OpenAI models:

- GPT-3.5 Turbo
- GPT-4
- GPT-4 Vision
- DALL-E 3

## Prerequisites

- **Rust**: You can install Rust by following the instructions on the official website: [https://www.rust-lang.org/learn/get-started](https://www.rust-lang.org/learn/get-started)

### Obtain an OpenAI API Key

1. Get an OpenAI API key from [https://openai.com/api/](https://openai.com/api/).
2. Add the API key to your shell environment by appending this line to `~/.bashrc` (or `~/.bash_profile` on Mac): `export OPENAI_API_KEY=your-api-key`
3. Apply the changes by restarting your shell or sourcing your profile: `source ~/.bashrc  # or source ~/.bash_profile on Mac`

## Installation Steps

1. Clone the GPTCLI repository: `git clone https://github.com/aumbriac/GPTCLI`
2. Go to the directory: `cd GPTCLI`
3. Test the CLI using Cargo: `cargo run What is the meaning of life?` or `cargo run <image_path> [optional_prompt]`
4. Build the CLI: `cargo build --release`
5. Add the built executable to your PATH:
   - **Windows**: Copy it to a directory in your `Path`, like `C:\Windows\System32`
   - **Mac/Linux**: Copy it to `/usr/local/bin/` using: `sudo cp target/release/gpt /usr/local/bin/`

### Using GPTCLI

With GPTCLI installed, use it directly in your shell.

- To chat with GPT 3.5 Turbo: `gpt What is the capital of California?`
- To chat with GPT 4: `gpt 4 What is the meaning of life?`
- For image analysis: `gpt v rust_astronaut.png What colors are in this image?`
- To generate an image: `gpt d an astronaut in a rusty spacesuit on mars holding a crab`

## Licensing

GPT4CLI is under the MIT License. See [LICENSE.md](LICENSE.md) for details.
