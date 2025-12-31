//! Music Flow Server - Main Entry Point
//!
//! Subsonic API 兼容的音乐流媒体服务器

use crate::services::{song_service::CommState, BrowsingService};
use axum::{middleware as axum_middleware, Router};
use musicflow_server::utils::id_builder;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod database;
mod error;
mod extractors;
mod handlers;
mod middleware;
mod models;
mod response;
mod services;
mod utils;

use config::AppConfig;
use database::{get_db_pool, run_migrations, DbPool};
use services::{
    AuthService, LibraryService, PlaylistService, ScanService, SearchService, ServiceContext,
    UserService,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 加载配置
    let config = AppConfig::from_env()?;

    // 2. 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.rust_log))
        .with(
            tracing_subscriber::fmt::layer()
                .with_file(true) // 显示文件名
                .with_line_number(true) // 显示行号
                .with_target(true), // 显示模块路径(target)
        )
        .init();
    tracing::debug!("Loaded config: {:?}", config);

    tracing::info!("Starting {} v{}", config.app_name, config.app_version);

    // 3. 验证音乐库路径
    if let Err(e) = config.validate_music_library() {
        tracing::warn!("Music library validation warning: {}", e);
        tracing::error!("Please set MUSIC_LIBRARY_PATH to a valid directory");
        return Ok(());
    }

    // 4. 连接数据库
    tracing::info!("Connecting to database: {}", config.database_url);
    let pool = get_db_pool(&config.database_url).await?;
    tracing::info!("Database connected successfully");

    // 5. 运行数据库迁移
    tracing::info!("Running database migrations...");
    run_migrations(&pool).await?;
    tracing::info!("Migrations completed");

    // 6. 创建默认管理员用户（如果不存在）
    create_default_admin(&pool).await?;

    // 7. 构建应用路由
    let app = build_app(pool, config.clone());

    // 8. 启动服务器
    let addr = SocketAddr::from((config.host.parse::<std::net::IpAddr>()?, config.port));

    tracing::info!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 构建应用路由
fn build_app(pool: DbPool, config: AppConfig) -> Router {
    // 创建服务上下文
    let service_ctx = Arc::new(ServiceContext::new(pool.clone()));

    let auth_service = Arc::new(AuthService::new(pool.clone()));
    let scan_service = Arc::new(ScanService::new(
        pool.clone(),
        config.music_library_path.clone(),
    ));
    let library_service = Arc::new(LibraryService::new(service_ctx.clone()));
    let user_service = Arc::new(UserService::new(service_ctx.clone(), auth_service.clone()));
    let playlist_service = Arc::new(PlaylistService::new(service_ctx.clone()));
    let browsing_service = Arc::new(BrowsingService::new(service_ctx.clone()));
    let search_service = Arc::new(SearchService::new(service_ctx.clone()));

    // 创建共享状态
    let _auth_state = auth_service.clone();

    let pool = Arc::new(pool);
    let comm_state = CommState { pool: pool.clone() };

    // 构建各个模块的路由
    let system_routes = handlers::system::routes();
    let auth_routes = handlers::auth::routes().with_state(auth_service);
    let browsing_routes = handlers::browsing::routes(pool.clone(), browsing_service);
    let search_routes = handlers::search::routes().with_state(search_service.clone());
    let stream_routes = handlers::stream::routes().with_state(comm_state);
    let playlist_state = handlers::playlist::PlaylistState {
        playlist_service,
        pool: pool.clone(),
    };
    let playlist_routes = handlers::playlist::routes().with_state(playlist_state);
    let user_routes = handlers::user::routes().with_state(user_service);
    let library_routes = handlers::library::routes(pool.clone(), scan_service, library_service);
    let advanced_routes = handlers::advanced::routes().with_state(pool.clone());

    // 需要认证的 API 路由
    let protected_routes = Router::new()
        .merge(browsing_routes)
        .merge(search_routes)
        .merge(stream_routes)
        .merge(playlist_routes)
        .merge(user_routes)
        .merge(library_routes)
        .merge(advanced_routes)
        // 认证中间件（仅保护需要认证的端点）
        .layer(axum_middleware::from_fn(middleware::auth_middleware));

    // 合并所有路由
    Router::new()
        // 系统端点（公开访问）
        .merge(system_routes)
        // 认证端点（公开访问）
        .merge(auth_routes)
        // 需要认证的端点
        .merge(protected_routes)
        // 静态文件服务（web 管理面板，公开访问）
        .nest_service("/", ServeDir::new("web"))
        // CORS 配置
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        )
        // 日志中间件
        .layer(tower_http::trace::TraceLayer::new_for_http())
        // 将配置和数据库连接池添加到请求扩展中（供中间件使用）
        .layer(axum::Extension(config))
        .layer(axum::Extension(pool))
}

/// 创建默认管理员用户
async fn create_default_admin(pool: &DbPool) -> Result<(), anyhow::Error> {
    // 检查 users 表是否存在
    let table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='users')",
    )
    .fetch_one(pool)
    .await?;

    if !table_exists {
        tracing::warn!("Users table not found. Please run migrations first.");
        return Ok(());
    }

    // 检查是否存在管理员
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE is_admin = 1")
        .fetch_one(pool)
        .await?;

    if count == 0 {
        tracing::info!("No admin user found, creating default admin...");

        let id = id_builder::generate_id();

        sqlx::query(
            "INSERT INTO users (id, username, password, email, is_admin) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind("admin")
        .bind("admin")
        .bind("admin@localhost")
        .bind(true)
        .execute(pool)
        .await?;

        tracing::info!("Default admin created: username='admin', password='admin'");
        tracing::warn!("Please change the default password immediately!");
    }

    Ok(())
}
