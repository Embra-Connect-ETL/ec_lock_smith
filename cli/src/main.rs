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
                .about("User management commands")
                .subcommand(
                    Command::new("list").about("List a user by email").arg(
                        Arg::new("email")
                            .short('e')
                            .long("email")
                            .required(true)
                            .help("User email"),
                    ),
                )
                .subcommand(
                    Command::new("delete").about("Delete user").arg(
                        Arg::new("id")
                            .short('i')
                            .long("id")
                            .required(true)
                            .help("User ID"),
                    ),
                )
                .subcommand(
                    Command::new("create")
                        .about("Create a new user")
                        .arg(
                            Arg::new("email")
                                .short('e')
                                .long("email")
                                .required(true)
                                .help("Email"),
                        )
                        .arg(
                            Arg::new("password")
                                .short('p')
                                .long("password")
                                .required(true)
                                .help("Password"),
                        ),
                ),
        )
        .subcommand(
            Command::new("secret")
                .about("Secret management commands")
                .subcommand(
                    Command::new("create")
                        .about("Create a secret")
                        .arg(
                            Arg::new("key")
                                .short('k')
                                .long("key")
                                .required(true)
                                .help("Secret key"),
                        )
                        .arg(
                            Arg::new("value")
                                .short('v')
                                .long("value")
                                .required(true)
                                .help("Secret value"),
                        ),
                )
                .subcommand(Command::new("list").about("List secrets"))
                .subcommand(
                    Command::new("get").about("Get secret value by ID").arg(
                        Arg::new("id")
                            .short('i')
                            .long("id")
                            .required(true)
                            .help("Secret ID"),
                    ),
                )
                .subcommand(
                    Command::new("delete").about("Delete secret").arg(
                        Arg::new("id")
                            .short('i')
                            .long("id")
                            .required(true)
                            .help("Secret ID"),
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

            match authed_user.login(creds).await {
                Ok(_) => println!("\x1b[0;32mLogin successful\x1b[0m"),
                Err(e) => eprintln!("\x1b[0;31mLogin failed: {e}\x1b[0m"),
            }
        }

        Some(("users", submatches)) => match submatches.subcommand() {
            Some(("list", submatches)) => {
                let email = submatches
                    .get_one::<String>("email")
                    .map(String::as_str)
                    .unwrap();

                session.get_user(email).await.map_or_else(
                    |e| eprintln!("\x1b[0;31mError fetching users: {e}\x1b[0m"),
                    |_| println!("\x1b[0;32mUsers fetched successfully\x1b[0m"),
                );
            }

            Some(("delete", submatches)) => {
                let id = submatches
                    .get_one::<String>("id")
                    .map(String::as_str)
                    .unwrap();

                session.delete_user(id).await.map_or_else(
                    |e| eprintln!("\x1b[0;31mError deleting user: {e}\x1b[0m"),
                    |_| println!("\x1b[0;32mUser deleted successfully\x1b[0m"),
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
                    |e| eprintln!("\x1b[0;31mError creating user: {e}\x1b[0m"),
                    |_| println!("\x1b[0;32mUser created successfully\x1b[0m"),
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
                    |e| eprintln!("\x1b[0;31mError creating secret: {e}\x1b[0m"),
                    |_| println!("\x1b[0;32mSecret created successfully\x1b[0m"),
                );
            }

            Some(("list", _)) => {
                session.list_secrets().await.map_or_else(
                    |e| eprintln!("\x1b[0;31mError listing secrets: {e}\x1b[0m"),
                    |_| println!("\x1b[0;32mSecrets listed successfully\x1b[0m"),
                );
            }

            Some(("get", submatches)) => {
                let id = submatches.get_one::<String>("id").unwrap();
                match session.get_secret_value(id).await {
                    Ok(value) => println!("\x1b[0;32mSecret value: {}\x1b[0m", value),
                    Err(e) => eprintln!("\x1b[0;31mError getting secret: {e}\x1b[0m"),
                }
            }

            Some(("delete", submatches)) => {
                let id = submatches.get_one::<String>("id").unwrap();
                match session.delete_secret(id).await {
                    Ok(msg) => println!("\x1b[0;32m{}\x1b[0m", msg), // green
                    Err(msg) => eprintln!("\x1b[0;31m{}\x1b[0m", msg), // red
                }
            }

            _ => {}
        },

        _ => {}
    }
}
