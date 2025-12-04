document.addEventListener('DOMContentLoaded', () => {
            const form = document.getElementById('crud-form');
            const tableBody = document.getElementById('table-body');
            const itemIdInput = document.getElementById('item-id');
            const itemNameInput = document.getElementById('item-name');
            const itemDescriptionInput = document.getElementById('item-description');
            const submitButton = document.getElementById('submit-button');

            const API_URL = '/items';

            // --- 1. FUNCIÓN PARA CARGAR Y RENDERIZAR LOS ELEMENTOS ---
            const fetchAndRenderItems = async () => {
                try {
                    const response = await fetch(API_URL);
                    if (!response.ok) {
                        throw new Error(`Error HTTP: ${response.status}`);
                    }
                    const items = await response.json();
                    
                    tableBody.innerHTML = ''; // Limpiar la tabla antes de renderizar

                    if (items.length === 0) {
                        tableBody.innerHTML = '<tr><td colspan="4">No hay elementos para mostrar.</td></tr>';
                        return;
                    }

                    // Usamos un ciclo para generar cada fila de la tabla
                    items.forEach(item => {
                        const row = document.createElement('tr');
                        row.setAttribute('data-item-id', item.id);

                        row.innerHTML = `
                            <td>${item.id}</td>
                            <td>${item.nombre}</td>
                            <td>${item.descripcion}</td>
                            <td>
                                <button class="action-btn edit-btn">Editar</button>
                                <button class="action-btn delete-btn">Eliminar</button>
                            </td>
                        `;
                        tableBody.appendChild(row);
                    });
                } catch (error) {
                    console.error('Error al cargar los elementos:', error);
                    tableBody.innerHTML = `<tr><td colspan="4">Error al cargar datos. Revisa la consola.</td></tr>`;
                }
            };

            // --- 2. MANEJO DEL FORMULARIO (CREAR Y ACTUALIZAR) ---
            form.addEventListener('submit', async (event) => {
                event.preventDefault(); // Evitar que la página se recargue

                const id = itemIdInput.value;
                const nombre = itemNameInput.value.trim();
                const descripcion = itemDescriptionInput.value.trim();

                if (!nombre || !descripcion) {
                    alert('Por favor, completa todos los campos.');
                    return;
                }

                const itemData = { nombre, descripcion };
                const isUpdating = id !== '';

                const url = isUpdating ? `${API_URL}/${id}` : API_URL;
                const method = isUpdating ? 'PUT' : 'POST';

                try {
                    const response = await fetch(url, {
                        method: method,
                        headers: {
                            'Content-Type': 'application/json',
                        },
                        body: JSON.stringify(itemData),
                    });

                    if (!response.ok) {
                        throw new Error(`Error al guardar: ${response.statusText}`);
                    }

                    // Limpiar formulario y recargar la tabla
                    form.reset();
                    itemIdInput.value = '';
                    submitButton.textContent = 'Guardar Elemento';
                    await fetchAndRenderItems();

                } catch (error) {
                    console.error('Error al guardar el elemento:', error);
                    alert('No se pudo guardar el elemento.');
                }
            });

            // --- 3. MANEJO DE BOTONES DE ACCIÓN (EDITAR Y ELIMINAR) ---
            tableBody.addEventListener('click', async (event) => {
                const target = event.target;
                const row = target.closest('tr');
                if (!row) return;

                const id = row.getAttribute('data-item-id');

                // Botón de Editar
                if (target.classList.contains('edit-btn')) {
                    // Ajustamos los índices para que coincidan con la estructura de la tabla en el HTML:
                    // cells[0] es el ID/Tarea
                    // cells[1] es el Nombre
                    // cells[2] es la Descripción
                    const nombre = row.cells[1].textContent;      // Columna "Nombre"
                    const descripcion = row.cells[2].textContent; // Columna "Descripción"

                    itemIdInput.value = id;
                    itemNameInput.value = nombre;
                    itemDescriptionInput.value = descripcion;
                    submitButton.textContent = 'Actualizar Elemento';
                    window.scrollTo(0, 0); // Subir al inicio de la página para ver el formulario
                }

                // Botón de Eliminar
                if (target.classList.contains('delete-btn')) {
                    if (confirm(`¿Estás seguro de que quieres eliminar el elemento con ID ${id}?`)) {
                        await fetch(`${API_URL}/${id}`, { method: 'DELETE' });
                        await fetchAndRenderItems(); // Recargar la tabla
                    }
                }
            });

            // --- CARGA INICIAL DE DATOS ---
            fetchAndRenderItems();
        });