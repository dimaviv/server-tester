use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "server-tester",
    about = "Simulate multiple backend servers for testing monitoring tools"
)]
pub struct Args {
    /// Port for the management API and Web UI
    #[arg(short = 'p', long, default_value_t = 3000)]
    pub management_port: u16,

    /// Path to the JSON file for persisting server configs
    #[arg(short = 'd', long, default_value = "servers.json")]
    pub data_file: String,
}
