use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySqlPool, FromRow};
use std::env;
use log::{info, error};

// Estructura para la respuesta JSON que incluye el nombre de la prioridad
#[derive(Debug, Serialize, Deserialize, FromRow)]
struct ItemResponse {
    id: i64,
    nombre: String,
    descripcion: String,
    id_prioridad: i32,
    tipo_prioridad: String,
}

// Estructura para crear un nuevo item
#[derive(Debug, Serialize, Deserialize)]
struct NewItem {
    nombre: String,
    descripcion: String,
    id_prioridad: i32,
}

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

    info!("Verificando y creando tabla 'prioridad' si no existe...");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS prioridad (
            id_prioridad INT AUTO_INCREMENT PRIMARY KEY,
            tipo_prioridad VARCHAR(50) NOT NULL UNIQUE
        )
        "#,
    )
    .execute(&pool)
    .await?;

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM prioridad").fetch_one(&pool).await?;
    if count.0 == 0 {
        info!("Poblando la tabla 'prioridad' con valores por defecto...");
        let prioridades = vec!["Urgente", "Medio", "Bajo"];
        for p in prioridades {
            sqlx::query("INSERT INTO prioridad (tipo_prioridad) VALUES (?)")
                .bind(p)
                .execute(&pool)
                .await?;
        }
    }

    info!("Verificando y creando tabla 'items' si no existe...");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS items (
            id BIGINT AUTO_INCREMENT PRIMARY KEY,
            nombre VARCHAR(255) NOT NULL,
            descripcion TEXT,
            id_prioridad INT,
            FOREIGN KEY (id_prioridad) REFERENCES prioridad(id_prioridad)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    

    info!("Tabla 'items' verificada/creada exitosamente.");
    Ok(pool)
}

// --- Rutas API ---

// POST /items - Crear un nuevo item
async fn create_item(pool: web::Data<MySqlPool>, item: web::Json<NewItem>) -> impl Responder {
    let new_item_data = item.into_inner();
    let result = sqlx::query(
        "INSERT INTO items (nombre, descripcion, id_prioridad) VALUES (?, ?, ?)"
    )
    .bind(&new_item_data.nombre)
    .bind(&new_item_data.descripcion)
    .bind(new_item_data.id_prioridad)
    .execute(pool.get_ref()).await;

    match result {
        Ok(result) => {
            let id = result.last_insert_id();
            // Devolvemos el item completo como fue creado
            let new_item = sqlx::query_as::<_, ItemResponse>("SELECT i.id, i.nombre, i.descripcion, i.id_prioridad, p.tipo_prioridad FROM items i JOIN prioridad p ON i.id_prioridad = p.id_prioridad WHERE i.id = ?")
                .bind(id)
                .fetch_one(pool.get_ref()).await.unwrap(); // Usamos unwrap aquí por simplicidad, ya que acabamos de insertarlo.
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
    let query = r#"
        SELECT i.id, i.nombre, i.descripcion, i.id_prioridad, p.tipo_prioridad
        FROM items as i
        INNER JOIN prioridad as p ON i.id_prioridad = p.id_prioridad
        ORDER BY i.id
    "#;

    match sqlx::query_as::<_, ItemResponse>(query)
        .fetch_all(pool.get_ref()).await {

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
/*
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
*/

// PUT /items/{id} - Actualizar un item por ID
async fn update_item(
    pool: web::Data<MySqlPool>,
    path: web::Path<i64>,
    item_data: web::Json<NewItem>,
) -> impl Responder {
    let item_id = path.into_inner();
    let item = item_data.into_inner();

    let result = sqlx::query(
        "UPDATE items SET nombre = ?, descripcion = ?, id_prioridad = ? WHERE id = ?"
    )
    .bind(&item.nombre)
    .bind(&item.descripcion)
    .bind(item.id_prioridad)
    .bind(item_id)
    .execute(pool.get_ref()).await;

    match result {
        Ok(res) => {
            if res.rows_affected() > 0 {
                match sqlx::query_as::<_, ItemResponse>("SELECT i.id, i.nombre, i.descripcion, i.id_prioridad, p.tipo_prioridad FROM items i JOIN prioridad p ON i.id_prioridad = p.id_prioridad WHERE i.id = ?")
                    .bind(item_id)
                    .fetch_one(pool.get_ref()).await {
                        Ok(updated_item) => {
                            info!("Item con ID {} actualizado.", item_id);
                            HttpResponse::Ok().json(updated_item)
                        },
                        Err(e) => {
                            error!("Item actualizado, pero no se pudo recuperar: {:?}", e);
                            HttpResponse::InternalServerError().body("Item actualizado, pero no se pudo recuperar.")
                        }
                    }
            } else {
                info!("Item con ID {} no encontrado para actualizar.", item_id);
                HttpResponse::NotFound().body(format!("Item con ID {} no encontrado", item_id))
            }
        }
        Err(e) => {
            error!("Error al actualizar item con ID {}: {:?}", item_id, e);
            HttpResponse::InternalServerError().body(format!("Error al actualizar item: {}", e))
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
                HttpResponse::Ok().finish() // Cambiado a Ok() para que el frontend lo maneje mejor
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
    dotenvy::dotenv().ok(); 
    init_logger();

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
            // Rutas API
            .service(
                web::resource("/items")
                    .route(web::get().to(get_items))
                    .route(web::post().to(create_item))
            )
            .service(
                web::resource("/items/{id}")
                    //.route(web::get().to(get_item_by_id))
                    .route(web::put().to(update_item))
                    .route(web::delete().to(delete_item))
            )

            //ruta raiz
            .service(
                Files::new("/", "./src/static")
                    .index_file("index.html"),
            )
    })
    .bind(&addr)?
    .run()
    .await
}