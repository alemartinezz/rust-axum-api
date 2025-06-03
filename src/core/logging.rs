// Logging configuration for the application
//
// Este sistema de logging está configurado para adaptarse a diferentes entornos:
//
// 1. ENVIRONMENT=development (por defecto):
//    - Logs de aplicación: INFO
//    - Framework logs: WARN (evita spam de axum/tower)
//    - SQLx: WARN (evita logs de cada query)
//    - Sin eventos de span (sin enter/exit)
//
// 2. ENVIRONMENT=production:
//    - Solo errores críticos y información importante
//    - SQLx: ERROR únicamente
//    - Sin eventos de span
//
// 3. ENVIRONMENT=debug:
//    - Más verboso para debugging
//    - Con eventos de span CLOSE (muestra duración)
//
// Para override manual, usar RUST_LOG:
// RUST_LOG=my_axum_project=debug,sqlx=info cargo run

use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::fmt::format::FmtSpan;

/// Initialize the tracing subscriber with environment-aware configuration
pub fn init_tracing() {
    let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
    
    let (env_filter, span_events) = match environment.as_str() {
        "production" => {
            let filter = EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "my_axum_project=info,sqlx=error,tower_http=error,axum=error".parse().unwrap());
            (filter, FmtSpan::NONE)
        }
        "debug" => {
            let filter = EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "my_axum_project=debug,sqlx=info,tower_http=debug,axum=debug".parse().unwrap());
            (filter, FmtSpan::CLOSE)
        }
        _ => { // development (default)
            let filter = EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "my_axum_project=info,sqlx=warn,tower_http=warn,axum=warn".parse().unwrap());
            (filter, FmtSpan::NONE)
        }
    };
    
    fmt()
        .with_env_filter(env_filter)
        .with_span_events(span_events)
        .init();
}

// End of file: /src/core/logging.rs