mod hooks;
mod output;
mod session;
mod waybar;

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

#[derive(Parser)]
#[command(name = "claude-sessions", about = "Track Claude Code sessions")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, ValueEnum)]
enum Format {
    Human,
    Json,
}

#[derive(Subcommand)]
enum Command {
    /// Send a desktop notification via notify-send (reads a hook event from stdin)
    ProcessNotification,
    /// Output Waybar-compatible JSON
    Waybar,
    /// List active sessions
    Ps {
        /// Output format
        #[arg(long, value_enum, default_value_t = Format::Human)]
        format: Format,
    },
    /// Generate shell completions
    Completions {
        /// The shell to generate completions for
        shell: Shell,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::ProcessNotification => hooks::process_notification(),
        Command::Waybar => waybar::waybar(),
        Command::Ps { format } => match format {
            Format::Human => output::ps_human(),
            Format::Json => output::ps_json(),
        },
        Command::Completions { shell } => {
            clap_complete::generate(
                shell,
                &mut Cli::command(),
                "claude-sessions",
                &mut std::io::stdout(),
            );
            Ok(())
        }
    };
    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
