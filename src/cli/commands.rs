//! CLI commands

use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::db::{
    repositories::project::ProjectRepository,
    repositories::session::{AgentType, SessionRepository, SessionStatus, SessionType},
    Database,
};

#[derive(Parser)]
#[command(name = "supercode")]
#[command(about = "Orchestration system for managing multiple coding agent sessions", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Database path (default: ~/.supercode/supercode.db)
    #[arg(long)]
    database: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// List all sessions
    Sessions {
        /// Filter by project ID
        #[arg(long)]
        project_id: Option<String>,

        /// Filter by status
        #[arg(long)]
        status: Option<String>,
    },

    /// Create a new session
    CreateSession {
        /// Agent type (manager, developer, reviewer)
        #[arg(long)]
        agent_type: String,

        /// Session type (opencode, claude)
        #[arg(long)]
        session_type: String,

        /// Project ID
        #[arg(long)]
        project_id: Option<String>,

        /// Working directory
        #[arg(long)]
        working_dir: Option<String>,
    },

    /// Kill a session
    KillSession {
        /// Session ID
        session_id: String,
    },

    /// List all projects
    Projects,

    /// Create a new project
    CreateProject {
        /// Project name
        name: String,

        /// Project description
        #[arg(long)]
        description: Option<String>,
    },

    /// Start MCP server
    Serve {
        /// Port number
        #[arg(long, default_value = "8080")]
        port: u16,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    // Use provided path or default to ~/.supercode/supercode.db
    let db_path = cli.database.unwrap_or_else(|| {
        dirs::home_dir()
            .map(|h| h.join(".supercode").join("supercode.db"))
            .unwrap_or_else(|| std::path::PathBuf::from("./supercode.db"))
            .to_string_lossy()
            .to_string()
    });

    // Ensure the directory exists
    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // Initialize database
    let db = Database::new(&db_path)?;
    let session_repo = SessionRepository::new(db.clone());
    let project_repo = ProjectRepository::new(db.clone());

    // Create a multi-threaded runtime for CLI operations
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    rt.block_on(async {
        match cli.command {
        Commands::Sessions { project_id, status } => {
            let status = status.map(|s| SessionStatus::from_str(&s)).transpose()?;

            let sessions = session_repo.list(project_id.as_deref(), status).await?;

            if sessions.is_empty() {
                println!("No sessions found");
            } else {
                for session in sessions {
                    println!(
                        "[{}] {} - {} ({}) - {}",
                        session.id.chars().take(8).collect::<String>(),
                        session.agent_type.as_str(),
                        session.session_type.as_str(),
                        session.status.as_str(),
                        session.working_dir.as_deref().unwrap_or("-")
                    );
                }
            }
            Ok(())
        }

        Commands::CreateSession {
            agent_type,
            session_type,
            project_id,
            working_dir,
        } => {
            let agent_type = AgentType::from_str(&agent_type)?;
            let session_type = SessionType::from_str(&session_type)?;

            let session = session_repo.create(
                agent_type,
                session_type,
                project_id,
                working_dir,
            ).await?;

            println!("Created session: {}", session.id);
            Ok(())
        }

        Commands::KillSession { session_id } => {
            session_repo.update_status(&session_id, SessionStatus::Terminated).await?;

            println!("Terminated session: {}", session_id);
            Ok(())
        }

        Commands::Projects => {
            let projects = project_repo.list().await?;

            if projects.is_empty() {
                println!("No projects found");
            } else {
                for project in projects {
                    println!(
                        "[{}] {} - {}",
                        project.id.chars().take(8).collect::<String>(),
                        project.name,
                        project.description.as_deref().unwrap_or("-")
                    );
                }
            }
            Ok(())
        }

        Commands::CreateProject { name, description } => {
            let project = project_repo.create(name, description).await?;

            println!("Created project: {} ({})", project.name, project.id);
            Ok(())
        }

        Commands::Serve { port } => {
            tracing::info!("Starting MCP server on port {}", port);
            
            // Create session manager
            let session_manager = Arc::new(crate::session::SessionManager::new(db));
            
            // Create and run MCP server
            let mcp_server = crate::mcp::McpServer::new(port, session_manager);
            mcp_server.run().await?;
            
            Ok(())
        }
        }
    })
}
