## 🦀 CRUD HTTP en Rust

Este proyecto implementa las operaciones básicas **CRUD** (**C**rear, **R**eer, **U**pdate, **D**elete) a través de un servidor **HTTP** desarrollado en **Rust**. Utiliza **MySQL** como base de datos, gestionada a través del entorno local de **XAMPP**.

---

### 📝 Requisitos

Para poder ejecutar y desarrollar este proyecto, es necesario tener instalado:

1.  **Rust:** El lenguaje de programación y su gestor de paquetes **Cargo**.
2.  **XAMPP:** Necesario para levantar el servidor **Apache** y la base de datos **MySQL**.

---

### ⚙️ Configuración de la Base de Datos (MySQL con XAMPP)

Es **obligatorio** configurar el entorno de base de datos usando **XAMPP** y crear la base de datos específica antes de ejecutar la aplicación Rust.

#### 1. Iniciar Servicios de XAMPP

* Abre el **Panel de Control de XAMPP**.
* Inicia los módulos **Apache** y **MySQL**.

#### 2. Crear la Base de Datos

* Accede a **phpMyAdmin** (generalmente en `http://localhost/phpmyadmin`).
* **Crear base de datos llamado: `crud-rust`**

> ❗ **IMPORTANTE:** El nombre de la base de datos debe ser **`crud-rust`** exactamente para que la conexión predeterminada del proyecto funcione.

#### 3. Estructura de la Tabla (opcional)

Una vez creada la base de datos `crud-rust`, se recomienda crear una tabla llamada `items` para probar las operaciones CRUD:

```sql
CREATE TABLE items (
    id INT AUTO_INCREMENT PRIMARY KEY,
    nombre VARCHAR(255) NOT NULL,
    descripcion TEXT
);
