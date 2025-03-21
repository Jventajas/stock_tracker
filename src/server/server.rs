use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use std::error::Error;
use std::convert::TryFrom;
use std::sync::Arc;
use tracing::info;

use crate::server::request::Request;
use crate::server::router::Router;
use crate::server::route::Route;
use crate::routes::root::Root;
use crate::routes::detail::Detail;
use crate::routes::static_files::StaticFiles;
use crate::services::database::Database;

pub struct HttpServer {
    router: Router,
}

impl HttpServer {
    pub fn new(database: Arc<Database>) -> Self {
        let mut routes: Vec<Arc<dyn Route>> = Vec::new();

        let root = Arc::new(Root::new(Arc::clone(&database)));
        routes.push(root);

        let detail = Arc::new(Detail::new(Arc::clone(&database)));
        routes.push(detail);

        let static_files = Arc::new(StaticFiles::new("static".to_string()));
        routes.push(static_files);

        let router = Router::new(routes);

        Self {
            router,
        }
    }

    pub async fn handle_connection(&self, mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
        let mut buffer = [0; 4096];

        let n = stream.read(&mut buffer).await?;
        if n == 0 {
            return Ok(());
        }

        let request_string = String::from_utf8_lossy(&buffer[0..n]).to_string();
        let request = Request::try_from(request_string.as_str())?;

        info!("Parsed request: \n\n{:?}", request);

        let response = self.router.route(request).await?;
        let response_str: String = response.into();

        info!("Response: \n\n{:?}", response_str);

        stream.write_all(response_str.as_bytes()).await?;
        stream.flush().await?;

        Ok(())

    }
}