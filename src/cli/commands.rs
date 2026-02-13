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

    /// Manage peers
    Peer {
        #[command(subcommand)]
        command: PeerCommands,
    },

    /// Generate keypair for this node
    Keygen {
        /// Optional password to encrypt the config
        #[arg(long)]
        password: Option<String>,
    },
}

#[derive(Subcommand)]
enum PeerCommands {
    /// Add a new peer
    Add {
        /// Peer name
        name: String,

        /// Hostname or IP to connect to
        hostname: String,

        /// Auth info for the peer
        #[arg(long)]
        auth: Option<String>,
    },

    /// List all peers
    List,

    /// Remove a peer
    Remove {
        /// Peer name
        name: String,
    },

    /// Show pending peer requests
    Pending,

    /// Accept a pending peer request
    Accept {
        /// Peer name
        name: String,
    },

    /// Deny a pending peer request
    Deny {
        /// Peer name
        name: String,
    },

    /// Connect to a peer
    Connect {
        /// Peer name
        name: String,
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
            
            // Load config for peer server
            let config = crate::config::Config::load(None)?;
            
            // Create session manager
            let session_manager = Arc::new(crate::session::SessionManager::new(db));
            
            // Create MCP server
            let mcp_server = crate::mcp::McpServer::new(port, session_manager);
            
            // Create and start peer server (port + 1)
            let peer_port = port + 1;
            let config = Arc::new(tokio::sync::RwLock::new(config));
            let peer_server = crate::mcp::PeerServer::new(peer_port, config.clone());
            
            // Start both servers
            tokio::select! {
                result = mcp_server.run() => {
                    result?;
                }
                result = peer_server.start() => {
                    result?;
                }
            }
            
            Ok(())
        }

        Commands::Keygen { password } => {
            use crate::config::{keygen, Config};

            // Generate keypair
            let (private_key, public_key) = keygen::generate_keypair()?;

            // Load or create config
            let mut config = Config::load(None).unwrap_or_default();

            // Set name if not set
            if config.name.is_empty() {
                println!("Enter a name for this node:");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                config.name = input.trim().to_string();
            }

            config.private_key = private_key;
            config.public_key = public_key;

            // Save config
            config.save(None)?;

            println!("Generated keypair for node: {}", config.name);
            println!("Public key: {}", config.public_key);
            println!("Config saved to: ~/.supercode/config.yml");
            Ok(())
        }

        Commands::Peer { command } => {
            use crate::config::{Config, PeerConfig};

            match command {
                PeerCommands::Add { name, hostname, auth } => {
                    let mut config = Config::load(None)?;

                    let peer = PeerConfig {
                        auth: auth.unwrap_or_default(),
                        hostnames: vec![hostname],
                        public_key: String::new(),
                        verified: false,
                    };

                    config.add_peer(&name, peer);
                    config.save(None)?;

                    println!("Added peer: {}", name);
                    Ok(())
                }

                PeerCommands::List => {
                    let config = Config::load(None)?;

                    if config.peers.is_empty() {
                        println!("No peers configured");
                    } else {
                        for (name, peer) in config.peers {
                            println!("[{}] {} - {:?}", name, peer.hostnames.join(", "), if peer.verified { "verified" } else { "unverified" });
                        }
                    }
                    Ok(())
                }

                PeerCommands::Remove { name } => {
                    let mut config = Config::load(None)?;
                    config.remove_peer(&name);
                    config.save(None)?;
                    println!("Removed peer: {}", name);
                    Ok(())
                }

                PeerCommands::Pending => {
                    let config = Config::load(None)?;
                    let requests = config.get_pending_requests();

                    if requests.is_empty() {
                        println!("No pending peer requests");
                    } else {
                        for req in requests {
                            println!("[{}] from {} at {}", req.name, req.public_key, req.from_addr);
                        }
                    }
                    Ok(())
                }

                PeerCommands::Accept { name } => {
                    let mut config = Config::load(None)?;
                    let requests: Vec<_> = config.get_pending_requests();

                    let request = requests.iter()
                        .find(|r| r.name == name)
                        .ok_or_else(|| anyhow::anyhow!("No pending request from {}", name))?;

                    let peer = PeerConfig {
                        auth: String::new(),
                        hostnames: vec![request.from_addr.clone()],
                        public_key: request.public_key.clone(),
                        verified: true,
                    };

                    config.add_peer(&name, peer);
                    config.clear_pending_request(&name);
                    config.save(None)?;

                    println!("Accepted peer: {}", name);
                    Ok(())
                }

                PeerCommands::Deny { name } => {
                    let mut config = Config::load(None)?;
                    config.deny_peer(&name);
                    config.save(None)?;
                    println!("Denied peer: {}", name);
                    Ok(())
                }

                PeerCommands::Connect { name } => {
                    let config = Config::load(None)?;

                    if !config.can_peer() {
                        anyhow::bail!("Cannot connect: run 'supercode keygen' first");
                    }

                    let peer = config.get_peer(&name)
                        .ok_or_else(|| anyhow::anyhow!("Peer not found: {}", name))?;

                    println!("Connecting to peer {} at {:?}...", name, peer.hostnames);
                    // TODO: Implement actual connection
                    Ok(())
                }
            }
        }
        }
    })
}
