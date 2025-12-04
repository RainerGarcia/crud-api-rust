use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySqlPool, FromRow};
use std::env;
use log::{info, error};

// --- Modelos de Datos ---

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Item {
    id: i64,
    nombre: String,
    descripcion: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct NewItem {
    nombre: String,
    descripcion: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateItem {
    nombre: Option<String>,
    descripcion: Option<String>,
}

// --- Funciones de Utilidad ---

fn init_logger() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
}

async fn init_db() -> Result<MySqlPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL debe estar configurada en el archivo .env");

    info!("Conectando a la base de datos MySQL...");
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    info!("Verificando y creando tabla 'items' si no existe...");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS items (
            id INT AUTO_INCREMENT PRIMARY KEY,
            nombre VARCHAR(255) NOT NULL,
            descripcion TEXT
        )
        "#,
    )
    .execute(&pool)
    .await?;

    info!("Tabla 'items' verificada/creada exitosamente.");
    Ok(pool)
}

// --- Handlers de Rutas API ---

// POST /items - Crear un nuevo item
async fn create_item(pool: web::Data<MySqlPool>, item: web::Json<NewItem>) -> impl Responder {
    match sqlx::query(
        "INSERT INTO items (nombre, descripcion) VALUES (?, ?)"
    )
    .bind(&item.nombre)
    .bind(&item.descripcion)
    .execute(pool.get_ref())
    .await
    {
        Ok(result) => {
            let id = result.last_insert_id();
            let new_item = Item {
                id: id as i64, // MySQL's LAST_INSERT_ID() devuelve u64
                nombre: item.nombre.clone(),
                descripcion: item.descripcion.clone(),
            };
            info!("Item creado: {:?}", new_item);
            HttpResponse::Created().json(new_item)
        },
        Err(e) => {
            error!("Error al crear item: {:?}", e);
            HttpResponse::InternalServerError().body(format!("Error al crear item: {}", e))
        }
    }
}

// GET /items - Leer todos los items
async fn get_items(pool: web::Data<MySqlPool>) -> impl Responder {
    match sqlx::query_as::<_, Item>("SELECT id, nombre, descripcion FROM items")
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(items) => {
            info!("Items recuperados: {} elementos", items.len());
            HttpResponse::Ok().json(items)
        },
        Err(e) => {
            error!("Error al obtener items: {:?}", e);
            HttpResponse::InternalServerError().body(format!("Error al obtener items: {}", e))
        }
    }
}

// GET /items/{id} - Leer un item por ID
async fn get_item_by_id(pool: web::Data<MySqlPool>, path: web::Path<i64>) -> impl Responder {
    let item_id = path.into_inner();
    match sqlx::query_as::<_, Item>("SELECT id, nombre, descripcion FROM items WHERE id = ?")
        .bind(item_id)
        .fetch_optional(pool.get_ref())
        .await
    {
        Ok(Some(item)) => {
            info!("Item recuperado por ID {}: {:?}", item_id, item);
            HttpResponse::Ok().json(item)
        },
        Ok(None) => {
            info!("Item con ID {} no encontrado.", item_id);
            HttpResponse::NotFound().body(format!("Item con ID {} no encontrado", item_id))
        },
        Err(e) => {
            error!("Error al obtener item por ID {}: {:?}", item_id, e);
            HttpResponse::InternalServerError().body(format!("Error al obtener item: {}", e))
        }
    }
}

// PUT /items/{id} - Actualizar un item por ID
async fn update_item(
    pool: web::Data<MySqlPool>,
    path: web::Path<i64>,
    item_data: web::Json<UpdateItem>,
) -> impl Responder {
    let item_id = path.into_inner();

    // Primero, intenta obtener el item existente para ver si existe
    let existing_item = sqlx::query_as::<_, Item>("SELECT id, nombre, descripcion FROM items WHERE id = ?")
        .bind(item_id)
        .fetch_optional(pool.get_ref())
        .await;

    match existing_item {
        Ok(Some(mut item)) => {
            // Actualiza los campos si se proporcionan
            if let Some(nombre) = item_data.nombre.clone() {
                item.nombre = nombre;
            }
            if let Some(descripcion) = item_data.descripcion.clone() {
                item.descripcion = descripcion;
            }

            match sqlx::query(
                "UPDATE items SET nombre = ?, descripcion = ? WHERE id = ?"
            )
            .bind(&item.nombre)
            .bind(&item.descripcion)
            .bind(item_id)
            .execute(pool.get_ref())
            .await
            {
                Ok(result) if result.rows_affected() > 0 => {
                    info!("Item con ID {} actualizado.", item_id);
                    HttpResponse::Ok().json(item) // Devuelve el item actualizado
                },
                Ok(_) => {
                    // Esto no debería ocurrir si el item existía y no hubo error
                    error!("Item con ID {} no se pudo actualizar, pero no hubo error SQL.", item_id);
                    HttpResponse::InternalServerError().body("Error desconocido al actualizar item")
                },
                Err(e) => {
                    error!("Error al actualizar item con ID {}: {:?}", item_id, e);
                    HttpResponse::InternalServerError().body(format!("Error al actualizar item: {}", e))
                }
            }
        },
        Ok(None) => {
            info!("Item con ID {} no encontrado para actualizar.", item_id);
            HttpResponse::NotFound().body(format!("Item con ID {} no encontrado", item_id))
        },
        Err(e) => {
            error!("Error al buscar item para actualizar con ID {}: {:?}", item_id, e);
            HttpResponse::InternalServerError().body(format!("Error al buscar item: {}", e))
        }
    }
}

// DELETE /items/{id} - Eliminar un item por ID
async fn delete_item(pool: web::Data<MySqlPool>, path: web::Path<i64>) -> impl Responder {
    let item_id = path.into_inner();
    match sqlx::query("DELETE FROM items WHERE id = ?")
        .bind(item_id)
        .execute(pool.get_ref())
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                info!("Item con ID {} eliminado.", item_id);
                HttpResponse::NoContent().finish() // 204 No Content
            } else {
                info!("Item con ID {} no encontrado para eliminar.", item_id);
                HttpResponse::NotFound().body(format!("Item con ID {} no encontrado", item_id))
            }
        },
        Err(e) => {
            error!("Error al eliminar item con ID {}: {:?}", item_id, e);
            HttpResponse::InternalServerError().body(format!("Error al eliminar item: {}", e))
        }
    }
}

// --- Función Principal ---

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok(); // Cargar variables de entorno desde .env
    init_logger(); // Inicializar el logger

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8090".to_string());
    let addr = format!("{}:{}", host, port);

    let pool = match init_db().await {
        Ok(p) => p,
        Err(e) => {
            error!("Fallo al inicializar la base de datos: {:?}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Fallo al inicializar la base de datos"));
        }
    };

    info!("Servidor Actix Web ejecutándose en http://{} ...", &addr);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))

            // Rutas API: registrar aquí PRIMERO
            .service(
                web::resource("/items")
                    .route(web::get().to(get_items))
                    .route(web::post().to(create_item))
            )
            .service(
                web::resource("/items/{id}")
                    .route(web::get().to(get_item_by_id))
                    .route(web::put().to(update_item))
                    .route(web::delete().to(delete_item))
            )

            // Archivos estáticos AL FINAL para que no intercepten las rutas API
            .service(
                Files::new("/", "./src/static")
                    .index_file("index.html"),
            )
    })
    .bind(&addr)?
    .run()
    .await
}