use clap::{Parser, Subcommand};
use ngram::client::Client;
use ngram::server::Server;

// TODO:
// Fill out the `Args` struct to parse the command line arguments. You may find clap "subcommands"
// helpful.
/// An archive service allowing publishing and searching of books
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Client {
        server_address: String,
        server_port: u16,
        #[command(subcommand)]
        action: ClientAction,
    },
    Server {
        listen_port: u16,
    },
}

#[derive(Subcommand, Debug)]
enum ClientAction {
    Publish { path: String },
    Search { word: String },
    Retrieve { id: usize },
}

// TODO:
// Inspect the contents of the `args` struct that has been created from the command line arguments
// the user passed. Depending on the arguments, either start a server or make a client and send the
// appropriate request. You may find it helpful to print the request response.
fn main() {
    let args = Args::parse();
    match args.command {
        Command::Client {
            server_address,
            server_port,
            action,
        } => {
            let client = Client::new(&server_address, server_port);
            match action {
                ClientAction::Publish { path } => {
                    let response = client.publish_from_path(&path).unwrap();
                    println!("{:?}", response);
                }
                ClientAction::Retrieve { id } => {
                    let response = client.retrieve(id).unwrap();
                    println!("{:?}", response);
                }
                ClientAction::Search { word } => {
                    let response = client.search(&word).unwrap();
                    println!("{:?}", response);
                }
            }
        }
        Command::Server { listen_port } => {
            let server = Server::new();
            server.run(listen_port);
        }
    }
}
