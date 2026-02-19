use clap::{Parser, Subcommand};
use colored::*;
use mongo_odbc_core::{odbc_uri::ODBCUri, MongoConnection, TypeMode};
use std::time::Instant;

#[derive(Parser)]
#[command(name = "MongoDB ODBC Connectivity Tester")]
#[command(about = "Test MongoDB ODBC connectivity with various authentication mechanisms", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Test connection using ODBC connection string
    Odbc {
        /// ODBC connection string (e.g., "DRIVER={MongoDB Atlas SQL ODBC Driver};USER=myuser;PWD=mypass;SERVER=localhost:27017")
        #[arg(short, long)]
        connection_string: String,

        /// Database to connect to (optional, overrides connection string)
        #[arg(short, long)]
        database: Option<String>,

        /// Login timeout in seconds (default: 30)
        #[arg(short = 't', long, default_value = "30")]
        login_timeout: u32,

        /// Connection timeout in seconds (optional)
        #[arg(short = 'c', long)]
        connection_timeout: Option<u32>,

        /// Use simple type mode (default: false, uses standard types)
        #[arg(short, long)]
        simple_types: bool,

        /// Enable max string length of 4000 characters
        #[arg(short = 'm', long)]
        max_string_length: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Test connection using MongoDB URI
    Uri {
        /// MongoDB URI (e.g., "mongodb://user:pass@localhost:27017/?authSource=admin")
        #[arg(short, long)]
        uri: String,

        /// Database to connect to (optional)
        #[arg(short, long)]
        database: Option<String>,

        /// Login timeout in seconds (default: 30)
        #[arg(short = 't', long, default_value = "30")]
        login_timeout: u32,

        /// Connection timeout in seconds (optional)
        #[arg(short = 'c', long)]
        connection_timeout: Option<u32>,

        /// Use simple type mode (default: false, uses standard types)
        #[arg(short, long)]
        simple_types: bool,

        /// Enable max string length of 4000 characters
        #[arg(short = 'm', long)]
        max_string_length: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show example connection strings for different authentication mechanisms
    Examples,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Odbc {
            connection_string,
            database,
            login_timeout,
            connection_timeout,
            simple_types,
            max_string_length,
            verbose,
        } => {
            test_connection(
                connection_string,
                database,
                login_timeout,
                connection_timeout,
                simple_types,
                max_string_length,
                verbose,
            );
        }
        Commands::Uri {
            uri,
            database,
            login_timeout,
            connection_timeout,
            simple_types,
            max_string_length,
            verbose,
        } => {
            // Note: We add dummy USER and PWD here because ODBCUri requires them
            // for validation, but they will be cleared for X.509 and other mechanisms
            // that don't use username/password (see core/src/odbc_uri.rs lines 314-316)
            let connection_string = format!("URI={};USER=dummy;PWD=dummy", uri);
            test_connection(
                connection_string,
                database,
                login_timeout,
                connection_timeout,
                simple_types,
                max_string_length,
                verbose,
            );
        }
        Commands::Examples => {
            show_examples();
        }
    }
}

fn test_connection(
    connection_string: String,
    database: Option<String>,
    login_timeout: u32,
    connection_timeout: Option<u32>,
    simple_types: bool,
    max_string_length: bool,
    verbose: bool,
) {
    println!("{}", "=".repeat(80).bright_blue());
    println!("{}", "MongoDB ODBC Connectivity Test".bright_blue().bold());
    println!("{}", "=".repeat(80).bright_blue());
    println!();

    if verbose {
        println!("{}", "Configuration:".bright_yellow());
        println!("  Connection String: {}", connection_string.dimmed());
        println!("  Database: {}", database.as_deref().unwrap_or("(from connection string)"));
        println!("  Login Timeout: {}s", login_timeout);
        println!("  Connection Timeout: {}", connection_timeout.map_or("(none)".to_string(), |t| format!("{}s", t)));
        println!("  Type Mode: {}", if simple_types { "Simple" } else { "Standard" });
        println!("  Max String Length: {}", if max_string_length { "4000 chars" } else { "Unlimited" });
        println!();
    }

    let start = Instant::now();

    println!("{}", "Step 1: Parsing connection string...".bright_cyan());

    let mut odbc_uri = match ODBCUri::new(connection_string.clone()) {
        Ok(uri) => {
            println!("  {} Connection string parsed successfully", "✓".green());
            uri
        }
        Err(e) => {
            println!("  {} Failed to parse connection string", "✗".red());
            println!("  Error: {}", e.to_string().red());
            std::process::exit(1);
        }
    };

    println!();
    println!("{}", "Step 2: Creating tokio runtime...".bright_cyan());

    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => {
            println!("  {} Runtime created successfully", "✓".green());
            rt
        }
        Err(e) => {
            println!("  {} Failed to create runtime", "✗".red());
            println!("  Error: {}", e.to_string().red());
            std::process::exit(1);
        }
    };

    println!();
    println!("{}", "Step 3: Parsing client options...".bright_cyan());

    let client_options = runtime.block_on(async {
        odbc_uri.try_into_client_options().await
    });

    let user_options = match client_options {
        Ok(opts) => {
            println!("  {} Client options parsed successfully", "✓".green());
            if verbose {
                if let Some(cred) = &opts.client_options.credential {
                    println!("    Username: {}", cred.username.as_deref().unwrap_or("(none)").dimmed());
                    println!("    Password: {}", if cred.password.is_some() { "***" } else { "(none)" }.dimmed());
                    println!("    Auth Mechanism: {:?}", cred.mechanism.as_ref().map_or("(default)".to_string(), |m| format!("{:?}", m)).dimmed());
                    println!("    Auth Source: {}", cred.source.as_deref().unwrap_or("(default)").dimmed());
                }
                println!("    Hosts: {:?}", opts.client_options.hosts.iter().map(|h| h.to_string()).collect::<Vec<_>>().join(", ").dimmed());
                if opts.client_options.tls.is_some() {
                    println!("    TLS: Configured");
                }
            }
            opts
        }
        Err(e) => {
            println!("  {} Failed to parse client options", "✗".red());
            println!("  Error: {}", e.to_string().red());
            println!();
            print_troubleshooting_tips(&e.to_string());
            std::process::exit(1);
        }
    };

    println!();
    println!("{}", "Step 4: Establishing connection...".bright_cyan());

    let type_mode = if simple_types {
        TypeMode::Simple
    } else {
        TypeMode::Standard
    };

    let max_str_len = if max_string_length {
        Some(constants::DEFAULT_MAX_STRING_LENGTH)
    } else {
        None
    };

    let connection_result = MongoConnection::connect(
        user_options,
        database,
        connection_timeout,
        Some(login_timeout),
        type_mode,
        Some(runtime),
        max_str_len,
    );

    let elapsed = start.elapsed();

    match connection_result {
        Ok(conn) => {
            println!("  {} Connection established successfully!", "✓".green().bold());
            println!();
            println!("{}", "Connection Details:".bright_green());
            println!("  Time taken: {:.2}s", elapsed.as_secs_f64());
            println!("  Cluster type: {:?}", conn.cluster_type);
            if let Some(uuid_repr) = conn.uuid_repr {
                println!("  UUID representation: {:?}", uuid_repr);
            }
            println!();
            println!("{}", "✓ SUCCESS: Connection test passed!".green().bold());

            // Cleanup
            let _ = conn.shutdown();
        }
        Err(e) => {
            println!("  {} Connection failed", "✗".red().bold());
            println!();
            println!("{}", "Error Details:".bright_red());
            println!("  {}", e.to_string().red());
            println!("  Time taken: {:.2}s", elapsed.as_secs_f64());
            println!();
            print_troubleshooting_tips(&e.to_string());
            std::process::exit(1);
        }
    }

    println!("{}", "=".repeat(80).bright_blue());
}


fn print_troubleshooting_tips(error_msg: &str) {
    println!("{}", "Troubleshooting Tips:".bright_yellow().bold());

    let error_lower = error_msg.to_lowercase();

    if error_lower.contains("authentication") || error_lower.contains("auth") {
        println!("  {} Authentication issues detected:", "!".yellow());
        println!("    - Verify username and password are correct");
        println!("    - Check authSource parameter (usually 'admin' or '$external')");
        println!("    - Ensure the correct authMechanism is specified:");
        println!("      • SCRAM-SHA-1 or SCRAM-SHA-256 (default for most deployments)");
        println!("      • MONGODB-X509 (for certificate-based auth)");
        println!("      • PLAIN (for LDAP)");
        println!("      • MONGODB-AWS (for AWS IAM)");
        println!("      • MONGODB-OIDC (for OpenID Connect)");
        println!("      • GSSAPI (for Kerberos)");
    }

    if error_lower.contains("timeout") || error_lower.contains("timed out") {
        println!("  {} Timeout issues detected:", "!".yellow());
        println!("    - Check network connectivity to the server");
        println!("    - Verify firewall rules allow connections");
        println!("    - Increase login timeout with --login-timeout flag");
        println!("    - Check if the server address and port are correct");
    }

    if error_lower.contains("dns") || error_lower.contains("resolve") {
        println!("  {} DNS resolution issues detected:", "!".yellow());
        println!("    - Verify the hostname is correct and resolvable");
        println!("    - On Windows, Cloudflare DNS is used by default");
        println!("    - Try using an IP address instead of hostname");
        println!("    - Check your DNS server configuration");
    }

    if error_lower.contains("tls") || error_lower.contains("ssl") {
        println!("  {} TLS/SSL issues detected:", "!".yellow());
        println!("    - Ensure TLS is enabled in the URI: ?tls=true");
        println!("    - For self-signed certificates, use: ?tlsAllowInvalidCertificates=true");
        println!("    - Verify certificate paths if using tlsCertificateKeyFile");
        println!("    - Check if tlsCAFile is needed for custom CA");
    }

    if error_lower.contains("database") || error_lower.contains("no database") {
        println!("  {} Database issues detected:", "!".yellow());
        println!("    - Specify a database with --database flag or DATABASE= in connection string");
        println!("    - Ensure the user has access to the specified database");
    }

    if error_lower.contains("connection refused") || error_lower.contains("could not connect") {
        println!("  {} Connection refused:", "!".yellow());
        println!("    - Verify the server is running");
        println!("    - Check the server address and port (default: 27017)");
        println!("    - Ensure no firewall is blocking the connection");
        println!("    - For Atlas, ensure your IP is whitelisted");
    }

    println!();
    println!("{}", "Common Connection String Formats:".bright_yellow());
    println!("  ODBC format:");
    println!("    DRIVER={{MongoDB Atlas SQL ODBC Driver}};USER=myuser;PWD=mypass;SERVER=localhost:27017;DATABASE=mydb");
    println!("  MongoDB URI format:");
    println!("    mongodb://user:pass@localhost:27017/?authSource=admin");
    println!("    mongodb+srv://user:pass@cluster.mongodb.net/?authSource=admin");
}

fn show_examples() {
    println!("{}", "=".repeat(80).bright_blue());
    println!("{}", "MongoDB ODBC Connection Examples".bright_blue().bold());
    println!("{}", "=".repeat(80).bright_blue());
    println!();

    println!("{}", "1. Basic SCRAM-SHA-256 Authentication (Default)".bright_green().bold());
    println!("   ODBC Connection String:");
    println!("     {}", "DRIVER={{MongoDB Atlas SQL ODBC Driver}};USER=myuser;PWD=mypass;SERVER=localhost:27017;DATABASE=mydb".dimmed());
    println!("   MongoDB URI:");
    println!("     {}", "mongodb://myuser:mypass@localhost:27017/?authSource=admin".dimmed());
    println!("   Test command:");
    println!("     {}", "connectivity_tester odbc -c \"DRIVER={{MongoDB Atlas SQL ODBC Driver}};USER=myuser;PWD=mypass;SERVER=localhost:27017\" -d mydb".cyan());
    println!();

    println!("{}", "2. MongoDB Atlas (SRV)".bright_green().bold());
    println!("   MongoDB URI:");
    println!("     {}", "mongodb+srv://myuser:mypass@cluster.mongodb.net/?authSource=admin".dimmed());
    println!("   Test command:");
    println!("     {}", "connectivity_tester uri -u \"mongodb+srv://myuser:mypass@cluster.mongodb.net/\" -d mydb".cyan());
    println!();

    println!("{}", "3. X.509 Certificate Authentication".bright_green().bold());
    println!("   MongoDB URI:");
    println!("     {}", "mongodb://localhost:27017/?authMechanism=MONGODB-X509&authSource=$external&tls=true&tlsCertificateKeyFile=/path/to/cert.pem".dimmed());
    println!("   Note: Username and password are not required for X.509");
    println!();

    println!("{}", "4. LDAP Authentication (PLAIN)".bright_green().bold());
    println!("   MongoDB URI:");
    println!("     {}", "mongodb://ldapuser:ldappass@localhost:27017/?authMechanism=PLAIN&authSource=$external".dimmed());
    println!();

    println!("{}", "5. AWS IAM Authentication".bright_green().bold());
    println!("   MongoDB URI:");
    println!("     {}", "mongodb://localhost:27017/?authMechanism=MONGODB-AWS&authSource=$external".dimmed());
    println!("   Note: AWS credentials are typically from environment variables or IAM role");
    println!();

    println!("{}", "6. Kerberos (GSSAPI) Authentication".bright_green().bold());
    println!("   MongoDB URI:");
    println!("     {}", "mongodb://user@REALM@localhost:27017/?authMechanism=GSSAPI&authSource=$external".dimmed());
    println!();

    println!("{}", "7. OpenID Connect (OIDC) Authentication".bright_green().bold());
    println!("   MongoDB URI:");
    println!("     {}", "mongodb://localhost:27017/?authMechanism=MONGODB-OIDC&authSource=$external".dimmed());
    println!("   Note: Will open browser for authentication flow");
    println!();

    println!("{}", "8. TLS/SSL Connection".bright_green().bold());
    println!("   MongoDB URI:");
    println!("     {}", "mongodb://myuser:mypass@localhost:27017/?tls=true&tlsCAFile=/path/to/ca.pem".dimmed());
    println!();

    println!("{}", "Additional Options:".bright_yellow().bold());
    println!("  --simple-types          Use simple type mode (for compatibility)");
    println!("  --max-string-length     Enable 4000 character limit (required by some BI tools)");
    println!("  --login-timeout <secs>  Set login timeout (default: 30s)");
    println!("  --connection-timeout    Set connection timeout for operations");
    println!("  --verbose               Show detailed connection information");
    println!();

    println!("{}", "Environment Variables for URI Parameters:".bright_yellow().bold());
    println!("  You can also specify these in the MongoDB URI:");
    println!("    ?authSource=admin           - Authentication database");
    println!("    &authMechanism=SCRAM-SHA-256 - Authentication mechanism");
    println!("    &tls=true                   - Enable TLS/SSL");
    println!("    &tlsAllowInvalidCertificates=true - Allow self-signed certs");
    println!("    &retryWrites=true           - Enable retry writes");
    println!("    &w=majority                 - Write concern");
    println!();

    println!("{}", "=".repeat(80).bright_blue());
}

