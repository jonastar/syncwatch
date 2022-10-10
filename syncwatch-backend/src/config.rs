use clap::Parser;

/// Syncwatch backend
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// admin password
    #[arg(short, long, default_value = "password")]
    admin_pw: String,
}
