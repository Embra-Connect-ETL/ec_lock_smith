use clap::{Arg, Command};
use cli::models::{auth::Auth, session::Session};
use shared::models::{Secret, UserCredentials};

#[tokio::main]
async fn main() {
    let mut authed_user = Auth::new();
    let mut session = Session::new();
    let matches = Command::new("ec_lock_smith")
        .version("1.0")
        .about("Embra Connect Lock Smith CLI")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("login")
                .about("authenticates user to the embra connect secrets manager service")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("email")
                        .short('e')
                        .long("email")
                        .required(true)
                        .help("The user's email"),
                )
                .arg(
                    Arg::new("password")
                        .short('p')
                        .long("password")
                        .required(true)
                        .help("The user's password"),
                ),
        )
        .subcommand(
            Command::new("users")
                .about("allow users to execute user management capabilities of lock smith")
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("list")
                        .about("list user account in lock smith")
                        .arg(
                            Arg::new("id")
                                .short('i')
                                .long("id")
                                .required(false)
                                .help("user account id"),
                        ),
                )
                .subcommand(
                    Command::new("delete").about("delete user account").arg(
                        Arg::new("id")
                            .short('i')
                            .long("id")
                            .required(true)
                            .help("user account id"),
                    ),
                )
                .subcommand(
                    Command::new("create")
                        .about("create a new user account in lock smith")
                        .arg(
                            Arg::new("email")
                                .short('e')
                                .long("email")
                                .required(true)
                                .help("user email address"),
                        )
                        .arg(
                            Arg::new("password")
                                .short('p')
                                .long("password")
                                .required(true)
                                .help("user's password"),
                        ),
                ),
        )
        .subcommand(
            Command::new("secret")
                .about("allow users to execute secret management capablities of lock smith")
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("create")
                        .about("create a new secret in lock smith")
                        .arg(
                            Arg::new("key")
                                .short('k')
                                .long("key")
                                .required(true)
                                .help("Secret Key"),
                        )
                        .arg(
                            Arg::new("value")
                                .short('v')
                                .long("value")
                                .required(true)
                                .help("Seret Value"),
                        ),
                )
                .subcommand(
                    Command::new("list")
                        .about("list all secrets for your accounts in lock smith")
                        .arg(
                            Arg::new("id")
                                .short('i')
                                .long("id")
                                .required(false)
                                .help("Secret Id"),
                        ),
                )
                .subcommand(
                    Command::new("delete")
                        .about("delete secret in lock smith")
                        .arg(
                            Arg::new("id")
                                .short('i')
                                .long("id")
                                .required(true)
                                .help("Secrets Id"),
                        ),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("login", sub_matches)) => {
            let creds = UserCredentials {
                email: sub_matches.get_one::<String>("email").unwrap().to_string(),
                password: sub_matches
                    .get_one::<String>("password")
                    .unwrap()
                    .to_string(),
            };
            authed_user.login(creds).await.map_or_else(
                |error| println!("\x1b[0;31m Login failed: {error} \x1b[0m"),
                |_| println!("\x1b[0;32m Login successful \x1b[0m"),
            );
        }
        Some(("users", submatches)) => match submatches.subcommand() {
            Some(("list", submatches)) => {
                let id: Option<&str> = submatches.get_one::<String>("id").map(|id| id.as_str());
                session.get_users(id).await.map_or_else(
                    |error| println!("\x1b[0;31m Error fetching users: {error} \x1b[0m"),
                    |_| println!("\x1b[0;32m Fetch successful \x1b[0m"),
                );
            }
            Some(("delete", submatches)) => {
                let id: Option<&str> = submatches.get_one::<String>("id").map(|id| id.as_str());
                session.delete_user(id).await.map_or_else(
                    |error| println!("\x1b[0;31m Error deleting user: {error} \x1b[0m"),
                    |_| println!("\x1b[0;32m Deleted user successfully '\x1b[0m"),
                );
            }
            Some(("create", submatches)) => {
                let creds = UserCredentials {
                    email: submatches.get_one::<String>("email").unwrap().to_string(),
                    password: submatches
                        .get_one::<String>("password")
                        .unwrap()
                        .to_string(),
                };

                session.create_user(creds).await.map_or_else(
                    |error| println!("\x1b[0;31m Error creating user: {error} \x1b[0m"),
                    |_| println!("\x1b[0;32m User created successfully \x1b[0m"),
                );
            }
            _ => {}
        },

        Some(("secret", submatches)) => match submatches.subcommand() {
            Some(("create", submatches)) => {
                let secret = Secret {
                    key: submatches.get_one::<String>("key").unwrap().to_string(),
                    value: submatches.get_one::<String>("value").unwrap().to_string(),
                };
                session.create_secret(secret).await.map_or_else(
                    |error| println!("\x1b[0;31m Error creating secret: {error} \x1b[0m"),
                    |_| println!("\x1b[0;32m Secreted created successfully \x1b[0m"),
                );
            }

            Some(("list", submatches)) => {
                let id: Option<&str> = submatches.get_one::<String>("id").map(|id| id.as_str());
                session.list_secrets(id).await.map_or_else(
                    |error| println!("\x1b[0;31m Error fetching secrets: {error} \x1b[0m"),
                    |_| println!("\x1b[0;32m Fetched secrets successfully \x1b[0m"),
                );
            }

            Some(("delete", submatches)) => {
                let id: &str = submatches.get_one::<String>("id").unwrap().as_str();
                session.delete_secret(id).await.map_or_else(
                    |error| println!("\x1b[0;31m Error deleting secret: {error} \x1b[0m"),
                    |_| println!("\x1b[0;32m Deleted secret successfully \x1b[0m"),
                );
            }
            _ => {}
        },
        _ => {}
    }
}
