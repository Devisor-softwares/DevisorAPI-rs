use anyhow::Result;
use chrono::Utc;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::{
    CreateServerRequest, CreateUserRequest, LoginRequest, Server, ServerCommand, ServerCommandType,
    ServerMetrics, ServerStatus, UpdateServerRequest, User, UserRole,
};
use crate::auth::{hash_password, verify_password, generate_token, verify_token};

pub struct UserService {
    pool: PgPool,
    pub jwt_secret: String,
}

impl UserService {
    pub fn new(pool: PgPool, jwt_secret: String) -> Self {
        Self { pool, jwt_secret }
    }

    pub async fn create_user(&self, request: CreateUserRequest) -> Result<User> {
        let password_hash = hash_password(&request.password)?;
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO users (id, username, email, password_hash, role, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, username, email, password_hash, role, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(&request.username)
        .bind(&request.email)
        .bind(&password_hash)
        .bind(request.role as UserRole)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(User {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            role: row.get("role"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn login(&self, request: LoginRequest) -> Result<(User, String)> {
        let row = sqlx::query("SELECT * FROM users WHERE email = $1")
            .bind(&request.email)
            .fetch_one(&self.pool)
            .await?;

        let user = User {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            role: row.get("role"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        let is_valid = verify_password(&request.password, &user.password_hash)?;
        if !is_valid {
            return Err(anyhow::anyhow!("Invalid credentials"));
        }

        let token = generate_token(&user, &self.jwt_secret)?;
        Ok((user, token))
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>> {
        let row = sqlx::query("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => Ok(Some(User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                role: row.get("role"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })),
            None => Ok(None),
        }
    }

    pub async fn verify_token(&self, token: &str) -> Result<User> {
        let claims = verify_token(token, &self.jwt_secret)?;
        let user_id = Uuid::parse_str(&claims.sub)?;
        
        let user = self.get_user_by_id(user_id).await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;
        
        Ok(user)
    }
}

pub struct ServerService {
    pool: PgPool,
}

impl ServerService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_server(&self, request: CreateServerRequest, owner_id: Uuid) -> Result<Server> {
        let server_id = Uuid::new_v4();
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO servers (id, name, description, game_type, ip_address, port, status, owner_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, name, description, game_type, ip_address, port, status, owner_id, created_at, updated_at
            "#,
        )
        .bind(server_id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(request.game_type as crate::models::GameType)
        .bind(&request.ip_address)
        .bind(request.port)
        .bind(ServerStatus::Offline as ServerStatus)
        .bind(owner_id)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(Server {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            game_type: row.get("game_type"),
            ip_address: row.get("ip_address"),
            port: row.get("port"),
            status: row.get("status"),
            owner_id: row.get("owner_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn get_servers_by_user(&self, user_id: Uuid) -> Result<Vec<Server>> {
        let rows = sqlx::query("SELECT * FROM servers WHERE owner_id = $1 ORDER BY created_at DESC")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;

        let servers = rows.into_iter().map(|row| Server {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            game_type: row.get("game_type"),
            ip_address: row.get("ip_address"),
            port: row.get("port"),
            status: row.get("status"),
            owner_id: row.get("owner_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }).collect();

        Ok(servers)
    }

    pub async fn get_server_by_id(&self, server_id: Uuid) -> Result<Option<Server>> {
        let row = sqlx::query("SELECT * FROM servers WHERE id = $1")
            .bind(server_id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => Ok(Some(Server {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                game_type: row.get("game_type"),
                ip_address: row.get("ip_address"),
                port: row.get("port"),
                status: row.get("status"),
                owner_id: row.get("owner_id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })),
            None => Ok(None),
        }
    }

    pub async fn update_server(&self, server_id: Uuid, request: UpdateServerRequest) -> Result<Server> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            UPDATE servers 
            SET name = COALESCE($1, name),
                description = COALESCE($2, description),
                port = COALESCE($3, port),
                updated_at = $4
            WHERE id = $5
            RETURNING id, name, description, game_type, ip_address, port, status, owner_id, created_at, updated_at
            "#,
        )
        .bind(&request.name)
        .bind(&request.description)
        .bind(request.port)
        .bind(now)
        .bind(server_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Server {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            game_type: row.get("game_type"),
            ip_address: row.get("ip_address"),
            port: row.get("port"),
            status: row.get("status"),
            owner_id: row.get("owner_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn delete_server(&self, server_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM servers WHERE id = $1")
            .bind(server_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_server_status(&self, server_id: Uuid, status: ServerStatus) -> Result<()> {
        let now = Utc::now();

        sqlx::query("UPDATE servers SET status = $1, updated_at = $2 WHERE id = $3")
            .bind(status as ServerStatus)
            .bind(now)
            .bind(server_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn execute_server_command(&self, command: ServerCommand) -> Result<()> {
        match command.command {
            ServerCommandType::Start => {
                self.update_server_status(command.server_id, ServerStatus::Starting).await?;
                // TODO: Implement actual server start logic
                self.update_server_status(command.server_id, ServerStatus::Online).await?;
            }
            ServerCommandType::Stop => {
                self.update_server_status(command.server_id, ServerStatus::Stopping).await?;
                // TODO: Implement actual server stop logic
                self.update_server_status(command.server_id, ServerStatus::Offline).await?;
            }
            ServerCommandType::Restart => {
                self.update_server_status(command.server_id, ServerStatus::Starting).await?;
                // TODO: Implement actual server restart logic
                self.update_server_status(command.server_id, ServerStatus::Online).await?;
            }
            ServerCommandType::Update => {
                // TODO: Implement server update logic
            }
            ServerCommandType::Backup => {
                // TODO: Implement server backup logic
            }
            ServerCommandType::ExecuteCommand => {
                // TODO: Implement custom command execution
            }
        }

        Ok(())
    }

    pub async fn record_metrics(&self, metrics: ServerMetrics) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO server_metrics (id, server_id, cpu_usage, memory_usage, disk_usage, network_in, network_out, player_count, max_players, recorded_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(metrics.id)
        .bind(metrics.server_id)
        .bind(metrics.cpu_usage)
        .bind(metrics.memory_usage)
        .bind(metrics.disk_usage)
        .bind(metrics.network_in)
        .bind(metrics.network_out)
        .bind(metrics.player_count)
        .bind(metrics.max_players)
        .bind(metrics.recorded_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_server_metrics(&self, server_id: Uuid, limit: i64) -> Result<Vec<ServerMetrics>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM server_metrics 
            WHERE server_id = $1 
            ORDER BY recorded_at DESC 
            LIMIT $2
            "#,
        )
        .bind(server_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let metrics = rows.into_iter().map(|row| ServerMetrics {
            id: row.get("id"),
            server_id: row.get("server_id"),
            cpu_usage: row.get("cpu_usage"),
            memory_usage: row.get("memory_usage"),
            disk_usage: row.get("disk_usage"),
            network_in: row.get("network_in"),
            network_out: row.get("network_out"),
            player_count: row.get("player_count"),
            max_players: row.get("max_players"),
            recorded_at: row.get("recorded_at"),
        }).collect();

        Ok(metrics)
    }
}
