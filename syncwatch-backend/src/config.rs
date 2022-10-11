use clap::Parser;

/// Syncwatch backend
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// admin password
    #[arg(short, long, default_value = "password")]
    pub admin_pw: String,

    #[arg(short, long, default_value = "0.0.0.0:8080")]
    pub listen_addr: String,
}
