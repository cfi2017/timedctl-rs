use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{debug, error, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod auth;
mod config;
mod handlers;

use auth::AuthClient;
use config::{get_default_config_path, TimedConfig};
use handlers::{activity, config as config_handlers, data, get_overtime, report};
use libtimed::TimedClient;

#[derive(Parser)]
#[command(
    name = "timedctl",
    about = "CLI for Timed time tracking",
    version,
    author
)]
struct Cli {
    /// Path to config file
    #[arg(long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Don't attempt to renew the token
    #[arg(long)]
    no_renew_token: bool,

    /// Verbose output
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Force renew token
    #[command(name = "force-renew")]
    ForceRenew,

    /// Get different things
    #[command(name = "get", alias = "g", alias = "show", alias = "describe")]
    Get(GetCommand),

    /// Delete different things
    #[command(
        name = "delete",
        alias = "rm",
        alias = "d",
        alias = "remove",
        alias = "del"
    )]
    Delete(DeleteCommand),

    /// Add different things
    #[command(name = "add", alias = "a", alias = "create")]
    Add(AddCommand),

    /// Edit different things
    #[command(name = "edit", alias = "e", alias = "update")]
    Edit(EditCommand),

    /// Do stuff with activities
    #[command(name = "activity", alias = "ac")]
    Activity(ActivityCommand),

    /// Manage configuration
    #[command(name = "config", alias = "cfg", alias = "conf")]
    Config(ConfigCommand),
}

#[derive(Parser)]
struct ConfigCommand {
    #[command(subcommand)]
    command: ConfigCommands,
}

#[derive(Debug, Subcommand)]
enum ConfigCommands {
    /// View current configuration
    #[command(name = "view", alias = "show", alias = "get")]
    View,

    /// Set a configuration value
    #[command(name = "set")]
    Set {
        /// Configuration key to set
        key: String,

        /// Value to set
        value: String,
    },

    /// Reset configuration to defaults
    #[command(name = "reset")]
    Reset,

    /// Initialize new configuration
    #[command(name = "init")]
    Init,

    /// Show configuration file path
    #[command(name = "path")]
    Path,
}

#[derive(Parser)]
struct GetCommand {
    #[command(subcommand)]
    command: GetCommands,
}

#[derive(Debug, Subcommand)]
enum GetCommands {
    /// Get overtime of user
    #[command(name = "overtime", alias = "t", alias = "ot", alias = "undertime")]
    Overtime {
        /// Date to get overtime for
        #[arg(long)]
        date: Option<String>,
    },

    /// Get reports
    #[command(name = "reports", alias = "report", alias = "r")]
    Reports {
        /// Date to get reports for
        #[arg(long)]
        date: Option<String>,

        /// Include reports from all users, not just current user
        #[arg(short = 'A', long)]
        all_users: bool,

        /// From date for date range filtering (format: YYYY-MM-DD)
        #[arg(long)]
        from: Option<String>,

        /// To date for date range filtering (format: YYYY-MM-DD)
        #[arg(long)]
        to: Option<String>,

        /// Use interactive mode to prompt for date selection
        #[arg(short = 'i', long)]
        interactive: bool,
    },

    /// Get activities
    #[command(name = "activities", alias = "a", alias = "ac", alias = "activity")]
    Activities {
        /// Date to get activities for
        #[arg(long)]
        date: Option<String>,

        /// Start date for activities (format: YYYY-MM-DD)
        #[arg(long)]
        from: Option<String>,

        /// End date for activities (format: YYYY-MM-DD)
        #[arg(long)]
        to: Option<String>,

        /// Include activities from all users, not just current user
        #[arg(short = 'A', long)]
        all_users: bool,
    },

    /// Get raw data for building custom scripts
    #[command(name = "data")]
    Data(DataCommand),
}

#[derive(Debug, Parser)]
struct DataCommand {
    #[command(subcommand)]
    command: DataCommands,
}

#[derive(Debug, Subcommand)]
enum DataCommands {
    /// Get customers
    #[command(name = "customers")]
    Customers {
        /// Output format
        #[arg(long, default_value = "json")]
        format: String,
    },

    /// Get projects
    #[command(name = "projects")]
    Projects {
        /// Customer ID
        #[arg(long)]
        customer_id: Option<i32>,

        /// Customer name
        #[arg(long)]
        customer_name: Option<String>,

        /// Include archived projects
        #[arg(long)]
        archived: bool,

        /// Output format
        #[arg(long, default_value = "json")]
        format: String,
    },

    /// Get tasks
    #[command(name = "tasks")]
    Tasks {
        /// Customer ID
        #[arg(long)]
        customer_id: Option<i32>,

        /// Customer name
        #[arg(long)]
        customer_name: Option<String>,

        /// Project ID
        #[arg(long)]
        project_id: Option<i32>,

        /// Project name
        #[arg(long)]
        project_name: Option<String>,

        /// Include archived tasks
        #[arg(long)]
        archived: bool,

        /// Output format
        #[arg(long, default_value = "json")]
        format: String,
    },
}

#[derive(Parser)]
struct DeleteCommand {
    #[command(subcommand)]
    command: DeleteCommands,
}

#[derive(Debug, Subcommand)]
enum DeleteCommands {
    /// Delete report(s)
    #[command(name = "report", alias = "r")]
    Report {
        /// Date to delete reports for
        #[arg(long)]
        date: Option<String>,

        /// Include reports from all users, not just current user
        #[arg(short = 'A', long)]
        all_users: bool,

        /// Use non-interactive mode (for scripting)
        #[arg(short = 'n', long)]
        non_interactive: bool,
    },

    /// Delete an activity
    #[command(name = "activity", alias = "a")]
    Activity {
        /// Date to delete activity for
        #[arg(long)]
        date: Option<String>,
    },
}

#[derive(Parser)]
struct AddCommand {
    #[command(subcommand)]
    command: AddCommands,
}

#[derive(Debug, Subcommand)]
enum AddCommands {
    /// Add report(s)
    #[command(name = "report", alias = "r")]
    Report {
        /// Customer name
        #[arg(long)]
        customer: Option<String>,

        /// Project name
        #[arg(long)]
        project: Option<String>,

        /// Task name
        #[arg(long)]
        task: Option<String>,

        /// Report description
        #[arg(long)]
        description: Option<String>,

        /// Duration (format: HH:MM or decimal hours)
        #[arg(long)]
        duration: Option<String>,

        /// Date for the report (format: YYYY-MM-DD, defaults to today)
        #[arg(long)]
        date: Option<String>,

        /// Show archived customers/projects/tasks
        #[arg(long)]
        show_archived: bool,

        /// Mark report for review
        #[arg(long)]
        review: bool,

        /// Mark report as not billable
        #[arg(long)]
        not_billable: bool,

        /// Use non-interactive mode (for scripting)
        #[arg(short = 'n', long)]
        non_interactive: bool,
    },
}

#[derive(Parser)]
struct EditCommand {
    #[command(subcommand)]
    command: EditCommands,
}

#[derive(Debug, Subcommand)]
enum EditCommands {
    /// Edit report(s)
    #[command(name = "report", alias = "r")]
    Report {
        /// Date to edit reports for
        #[arg(long)]
        date: Option<String>,

        /// Use non-interactive mode (for scripting)
        #[arg(short = 'n', long)]
        non_interactive: bool,
    },
}

#[derive(Parser)]
struct ActivityCommand {
    #[command(subcommand)]
    command: ActivityCommands,
}

#[derive(Debug, Subcommand)]
enum ActivityCommands {
    /// Start recording activity
    #[command(name = "start", alias = "add", alias = "a")]
    Start {
        /// Comment for the activity (optional in interactive mode)
        #[arg(default_value = "")]
        comment: String,

        /// Customer name
        #[arg(long)]
        customer: Option<String>,

        /// Project name
        #[arg(long)]
        project: Option<String>,

        /// Task name
        #[arg(long)]
        task: Option<String>,

        /// Show archived projects and tasks
        #[arg(long)]
        show_archived: bool,

        /// Start time for the activity (format: HH:MM)
        #[arg(long)]
        start_time: Option<String>,

        /// Use non-interactive mode (for scripting)
        #[arg(short = 'n', long)]
        non_interactive: bool,
    },

    /// Stop current activity
    #[command(name = "stop", alias = "end", alias = "finish")]
    Stop,

    /// Show current activity
    #[command(name = "show", alias = "s", alias = "get", alias = "info")]
    Show {
        /// Short output
        #[arg(long)]
        short: bool,
    },

    /// Restart an activity
    #[command(name = "restart", alias = "r", alias = "continue", alias = "resume")]
    Restart {
        /// Date to restart activity from
        #[arg(long)]
        date: Option<String>,
    },

    /// Delete an activity
    #[command(name = "delete", alias = "d", alias = "rm", alias = "remove")]
    Delete {
        /// Date to delete activity for
        #[arg(long)]
        date: Option<String>,
    },

    /// Generate the timesheet of the current activities
    #[command(name = "generate-timesheet", alias = "gts", alias = "ts")]
    GenerateTimesheet,
}

/// Initialize logging with the appropriate verbosity level
fn init_logging(verbosity: u8) {
    let log_level = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("timedctl={log_level}")));

    let subscriber = FmtSubscriber::builder().with_env_filter(filter).finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set default subscriber");

    debug!("Logging initialized at level: {}", log_level);
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging based on verbosity
    init_logging(cli.verbose);

    info!("Starting timedctl-rs");

    // Get configuration path
    let config_path = cli
        .config
        .as_deref()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| {
            match get_default_config_path() {
                Ok(path) => path,
                Err(e) => {
                    error!("Failed to get default config path: {}", e);
                    PathBuf::new() // Return empty path as fallback
                }
            }
        });

    // Handle all config commands without loading config first
    if let Commands::Config(cmd) = &cli.command {
        match &cmd.command {
            ConfigCommands::Path => {
                if let Err(e) = config_handlers::config_path() {
                    error!("Failed to get configuration path: {}", e);
                }
                return Ok(());
            }
            ConfigCommands::Init => {
                if let Err(e) = config_handlers::init_config(&config_path) {
                    error!("Failed to initialize configuration: {}", e);
                }
                return Ok(());
            }
            ConfigCommands::Reset => {
                if let Err(e) = config_handlers::reset_config(&config_path) {
                    error!("Failed to reset configuration: {}", e);
                }
                return Ok(());
            }
            ConfigCommands::Set { key, value } => {
                if let Err(e) =
                    config_handlers::set_config_without_loading(&config_path, key, value)
                {
                    error!("Failed to set configuration: {}", e);
                }
                return Ok(());
            }
            ConfigCommands::View => {
                if let Err(e) = config_handlers::view_config_without_loading(&config_path) {
                    error!("Failed to view configuration: {}", e);
                }
                return Ok(());
            }
        }
    }

    // Only load configuration if we're not handling a config command (which was handled above)
    let config = match TimedConfig::load(Some(&config_path)) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return Err(anyhow::anyhow!("Configuration error: {}", e));
        }
    };

    // Create auth client
    let auth_client = AuthClient::new(config.clone());

    // Ensure we have a valid token unless we're doing a force-renew
    let token_result = if !matches!(cli.command, Commands::ForceRenew) && !cli.no_renew_token {
        match auth_client.ensure_valid_token().await {
            Ok(token) => Some(token),
            Err(e) => {
                error!("Authentication error: {}", e);
                return Err(anyhow::anyhow!("Authentication error: {}", e));
            }
        }
    } else {
        None
    };

    // Create API client if we have a token
    let _api_client = token_result.as_ref().map(|t| {
        info!("Using existing token");
        TimedClient::new(&config.timed_url, "api/v1", Some(t.clone()))
    });

    // Create API client if we don't already have it for force-renew command
    let api_client = if token_result.is_none() && matches!(cli.command, Commands::ForceRenew) {
        info!("Force renewing token...");
        match auth_client.force_renew_token().await {
            Ok(token) => {
                info!("Token renewed successfully");
                Some(TimedClient::new(&config.timed_url, "api/v1", Some(token)))
            }
            Err(e) => {
                error!("Failed to renew token: {}", e);
                None
            }
        }
    } else {
        _api_client
    };

    // All config commands were already handled above

    // Ensure we have a client for commands that need it
    let client = match (api_client, &cli.command) {
        (Some(client), _) => client,
        (None, Commands::ForceRenew) => {
            // ForceRenew was already handled above
            return Ok(());
        }
        _ => {
            error!("No authentication token available. Run with --force-renew first.");
            return Err(anyhow::anyhow!("Authentication required"));
        }
    };

    // Handle commands
    match cli.command {
        Commands::ForceRenew => {
            // Already handled above
        }
        Commands::Get(cmd) => match cmd.command {
            GetCommands::Overtime { date } => match get_overtime(&client, date.as_deref()).await {
                Ok(balance) => println!("Overtime: {balance}"),
                Err(e) => error!("Failed to get overtime: {}", e),
            },
            GetCommands::Reports {
                date,
                from,
                to,
                all_users,
                interactive,
            } => {
                if let Err(e) = report::get_reports(
                    &client,
                    date.as_deref(),
                    from.as_deref(),
                    to.as_deref(),
                    all_users,
                    interactive,
                )
                .await
                {
                    error!("Failed to get reports: {}", e);
                }
            }
            GetCommands::Activities {
                date,
                from,
                to,
                all_users,
            } => {
                if let Err(e) = activity::show_activity(
                    &client,
                    false,
                    date.as_deref(),
                    from.as_deref(),
                    to.as_deref(),
                    all_users,
                )
                .await
                {
                    error!("Failed to get activities: {}", e);
                }
            }
            GetCommands::Data(data_cmd) => match data_cmd.command {
                DataCommands::Customers { format } => {
                    if let Err(e) = data::get_customers(&client, &format).await {
                        error!("Failed to get customers: {}", e);
                    }
                }
                DataCommands::Projects {
                    customer_id,
                    customer_name,
                    archived,
                    format,
                } => {
                    if let Err(e) = data::get_projects(
                        &client,
                        customer_id,
                        customer_name.as_deref(),
                        archived,
                        &format,
                    )
                    .await
                    {
                        error!("Failed to get projects: {}", e);
                    }
                }
                DataCommands::Tasks {
                    customer_id,
                    customer_name,
                    project_id,
                    project_name,
                    archived,
                    format,
                } => {
                    if let Err(e) = data::get_tasks(
                        &client,
                        customer_id,
                        customer_name.as_deref(),
                        project_id,
                        project_name.as_deref(),
                        archived,
                        &format,
                    )
                    .await
                    {
                        error!("Failed to get tasks: {}", e);
                    }
                }
            },
        },
        Commands::Delete(cmd) => match cmd.command {
            DeleteCommands::Report {
                date,
                all_users,
                non_interactive,
            } => {
                if let Err(e) =
                    report::delete_report(&client, date.as_deref(), all_users, !non_interactive)
                        .await
                {
                    error!("Failed to delete report: {}", e);
                }
            }
            DeleteCommands::Activity { date } => {
                if let Err(e) = activity::delete_activity(&client, date.as_deref()).await {
                    error!("Failed to delete activity: {}", e);
                }
            }
        },
        Commands::Add(cmd) => match cmd.command {
            AddCommands::Report {
                customer,
                project,
                task,
                description,
                duration,
                date,
                show_archived,
                review,
                not_billable,
                non_interactive,
            } => {
                if let Err(e) = report::add_report(
                    &client,
                    customer.as_deref(),
                    project.as_deref(),
                    task.as_deref(),
                    description.as_deref(),
                    duration.as_deref(),
                    date.as_deref(),
                    show_archived,
                    review,
                    not_billable,
                    !non_interactive,
                )
                .await
                {
                    error!("Failed to add report: {}", e);
                }
            }
        },
        Commands::Edit(cmd) => match cmd.command {
            EditCommands::Report {
                date,
                non_interactive,
            } => {
                if let Err(e) =
                    report::edit_report(&client, date.as_deref(), !non_interactive).await
                {
                    error!("Failed to edit report: {}", e);
                }
            }
        },
        Commands::Activity(cmd) => match cmd.command {
            ActivityCommands::Start {
                ref comment,
                customer,
                project,
                task,
                show_archived,
                start_time,
                non_interactive,
            } => {
                if let Err(e) = activity::start_activity(
                    &client,
                    comment,
                    customer.as_deref(),
                    project.as_deref(),
                    task.as_deref(),
                    show_archived,
                    start_time.as_deref(),
                    !non_interactive,
                )
                .await
                {
                    error!("Failed to start activity: {}", e);
                }
            }
            ActivityCommands::Stop => {
                if let Err(e) = activity::stop_activity(&client).await {
                    error!("Failed to stop activity: {}", e);
                }
            }
            ActivityCommands::Show { short } => {
                if let Err(e) = activity::get_active_activity(&client, short).await {
                    error!("Failed to show activity: {}", e);
                }
            }
            ActivityCommands::Restart { date } => {
                if let Err(e) = activity::restart_activity(&client, date.as_deref()).await {
                    error!("Failed to restart activity: {}", e);
                }
            }
            ActivityCommands::Delete { date } => {
                if let Err(e) = activity::delete_activity(&client, date.as_deref()).await {
                    error!("Failed to delete activity: {}", e);
                }
            }
            ActivityCommands::GenerateTimesheet => {
                if let Err(e) = activity::generate_timesheet(&client).await {
                    error!("Failed to generate timesheet: {}", e);
                }
            }
        },
        Commands::Config(_) => {
            // Already handled above
        }
    }

    Ok(())
}
