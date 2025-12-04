document.addEventListener('DOMContentLoaded', () => {
            const form = document.getElementById('crud-form');
            const tableBody = document.getElementById('table-body');
            const itemIdInput = document.getElementById('item-id');
            const itemNameInput = document.getElementById('item-name');
            const itemDescriptionInput = document.getElementById('item-description');
            const itemPrioridadSelect = document.getElementById('item-prioridad');
            const submitButton = document.getElementById('submit-button');

            const API_URL = '/items';

            const fetchAndRenderItems = async () => {
                try {
                    const response = await fetch(API_URL);
                    if (!response.ok) {
                        throw new Error(`Error HTTP: ${response.status}`);
                    }
                    const items = await response.json();
                    
                    tableBody.innerHTML = ''; 

                    if (items.length === 0) {
                        tableBody.innerHTML = '<tr><td colspan="5">No hay elementos para mostrar.</td></tr>';
                        return;
                    }

                    items.forEach(item => {
                        const row = document.createElement('tr');
                        row.setAttribute('data-item-id', item.id);
                        row.setAttribute('data-prioridad-id', item.id_prioridad);

                        row.innerHTML = `
                            <td>${item.id}</td>
                            <td>${item.nombre}</td>
                            <td>${item.descripcion}</td>
                            <td>${item.tipo_prioridad}</td>
                            <td>
                                <button class="action-btn edit-btn">Editar</button>
                                <button class="action-btn delete-btn">Eliminar</button>
                            </td>
                        `;
                        tableBody.appendChild(row);
                    });
                } catch (error) {
                    console.error('Error al cargar los elementos:', error);
                    tableBody.innerHTML = `<tr><td colspan="5">Error al cargar datos. Revisa la consola.</td></tr>`;
                }
            };

            form.addEventListener('submit', async (event) => {
                event.preventDefault(); 

                const id = itemIdInput.value;
                const nombre = itemNameInput.value.trim();
                const descripcion = itemDescriptionInput.value.trim();
                const id_prioridad = parseInt(itemPrioridadSelect.value, 10); 

                if (!nombre || !descripcion || !id_prioridad) {
                    alert('Por favor, completa todos los campos.');
                    return;
                }

                const itemData = { nombre, descripcion, id_prioridad };
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

                    form.reset();
                    itemIdInput.value = '';
                    itemPrioridadSelect.value = '3';
                    submitButton.textContent = 'Guardar Elemento';
                    await fetchAndRenderItems();

                } catch (error) {
                    console.error('Error al guardar el elemento:', error);
                    alert('No se pudo guardar el elemento.');
                }
            });

            tableBody.addEventListener('click', async (event) => {
                const target = event.target;
                const row = target.closest('tr');
                if (!row) return;

                const id = row.getAttribute('data-item-id');
                const prioridadId = row.getAttribute('data-prioridad-id');

                if (target.classList.contains('edit-btn')) {
                    const nombre = row.cells[1].textContent;     
                    const descripcion = row.cells[2].textContent; 

                    itemIdInput.value = id;
                    itemNameInput.value = nombre;
                    itemDescriptionInput.value = descripcion;
                    itemPrioridadSelect.value = prioridadId;
                    submitButton.textContent = 'Actualizar Elemento';
                    window.scrollTo(0, 0); 
                }

                if (target.classList.contains('delete-btn')) {
                    if (confirm(`¿Estás seguro de que quieres eliminar el elemento con ID ${id}?`)) {
                        await fetch(`${API_URL}/${id}`, { method: 'DELETE' });
                        await fetchAndRenderItems(); 
                    }
                }
            });

            // --- CARGA INICIAL DE DATOS ---
            fetchAndRenderItems();
        });