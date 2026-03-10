const API = '/api/servers';
let servers = [];

// --- API calls ---

async function fetchServers() {
    const res = await fetch(API);
    if (res.ok) {
        servers = await res.json();
        render();
    }
}

async function createServer(data) {
    const res = await fetch(API, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
    });
    if (!res.ok) {
        const text = await res.text();
        alert('Error: ' + text);
        return;
    }
    closeModal();
    fetchServers();
}

async function updateServer(id, data) {
    const res = await fetch(`${API}/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
    });
    if (!res.ok) {
        const text = await res.text();
        alert('Error: ' + text);
        return;
    }
    closeModal();
    fetchServers();
}

async function deleteServer(id) {
    if (!confirm('Delete this server?')) return;
    await fetch(`${API}/${id}`, { method: 'DELETE' });
    fetchServers();
}

async function setMode(id, mode) {
    await fetch(`${API}/${id}/mode`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ mode }),
    });
    fetchServers();
}

// --- Rendering ---

const MODE_LABELS = {
    up: 'Up',
    down_connection_refused: 'Refused',
    down_503: '503',
    down_timeout: 'Timeout',
};

const MODE_DISPLAY = {
    up: 'UP',
    down_connection_refused: 'CONN REFUSED',
    down_503: '503',
    down_timeout: 'TIMEOUT',
};

function render() {
    const list = document.getElementById('server-list');

    if (servers.length === 0) {
        list.innerHTML = '<p class="empty-state">No servers yet. Click "+ Add Server" to create one.</p>';
        return;
    }

    list.innerHTML = servers
        .sort((a, b) => a.port - b.port)
        .map(s => {
            const modeClass = `mode-${s.status}`;
            return `
            <div class="server-card ${modeClass}">
                <div class="card-top">
                    <div class="card-info">
                        <h3>${esc(s.name)}</h3>
                        <div class="card-meta">
                            <span>Port: <strong>${s.port}</strong></span>
                            <span>HTTP: <strong>${s.http_status_code}</strong></span>
                            <span>Title: <strong>${esc(s.html_title)}</strong></span>
                            ${s.response_delay_ms > 0 ? `<span>Delay: <strong>${s.response_delay_ms}ms</strong></span>` : ''}
                        </div>
                    </div>
                    <div class="card-actions">
                        <span class="status-badge ${s.status}">${MODE_DISPLAY[s.status]}</span>
                        <button class="btn" onclick="openEdit('${s.id}')">Edit</button>
                        <button class="btn btn-danger" onclick="deleteServer('${s.id}')">Delete</button>
                    </div>
                </div>
                <div class="mode-buttons">
                    ${Object.entries(MODE_LABELS).map(([mode, label]) => `
                        <button class="mode-btn ${s.status === mode ? 'active-' + mode : ''}"
                                onclick="setMode('${s.id}', '${mode}')">${label}</button>
                    `).join('')}
                </div>
            </div>`;
        })
        .join('');
}

function esc(str) {
    const div = document.createElement('div');
    div.textContent = str || '';
    return div.innerHTML;
}

// --- Modal ---

function openAdd() {
    document.getElementById('modal-title').textContent = 'Add Server';
    document.getElementById('form-id').value = '';
    document.getElementById('form-name').value = '';
    document.getElementById('form-port').value = '';
    document.getElementById('form-port').disabled = false;
    document.getElementById('form-http-status').value = '200';
    document.getElementById('form-delay').value = '0';
    document.getElementById('form-title').value = '';
    document.getElementById('form-body').value = '';
    document.getElementById('form-headers').value = '';
    document.getElementById('modal-overlay').classList.remove('hidden');
}

function openEdit(id) {
    const s = servers.find(x => x.id === id);
    if (!s) return;
    document.getElementById('modal-title').textContent = 'Edit Server';
    document.getElementById('form-id').value = s.id;
    document.getElementById('form-name').value = s.name;
    document.getElementById('form-port').value = s.port;
    document.getElementById('form-port').disabled = true;
    document.getElementById('form-http-status').value = s.http_status_code;
    document.getElementById('form-delay').value = s.response_delay_ms;
    document.getElementById('form-title').value = s.html_title;
    document.getElementById('form-body').value = s.response_body || '';
    document.getElementById('form-headers').value =
        Object.keys(s.custom_headers).length > 0 ? JSON.stringify(s.custom_headers) : '';
    document.getElementById('modal-overlay').classList.remove('hidden');
}

function closeModal() {
    document.getElementById('modal-overlay').classList.add('hidden');
}

// --- Event Listeners ---

document.getElementById('btn-add').addEventListener('click', openAdd);
document.getElementById('modal-close').addEventListener('click', closeModal);
document.getElementById('btn-cancel').addEventListener('click', closeModal);

document.getElementById('modal-overlay').addEventListener('click', (e) => {
    if (e.target === e.currentTarget) closeModal();
});

document.getElementById('server-form').addEventListener('submit', (e) => {
    e.preventDefault();
    const id = document.getElementById('form-id').value;
    const name = document.getElementById('form-name').value.trim();
    const port = parseInt(document.getElementById('form-port').value, 10);
    const httpStatus = parseInt(document.getElementById('form-http-status').value, 10);
    const delay = parseInt(document.getElementById('form-delay').value, 10) || 0;
    const title = document.getElementById('form-title').value.trim();
    const body = document.getElementById('form-body').value.trim();
    const headersStr = document.getElementById('form-headers').value.trim();

    let customHeaders = {};
    if (headersStr) {
        try {
            customHeaders = JSON.parse(headersStr);
        } catch {
            alert('Custom headers must be valid JSON object');
            return;
        }
    }

    if (id) {
        // Update
        updateServer(id, {
            name,
            http_status_code: httpStatus,
            html_title: title || undefined,
            response_body: body || null,
            custom_headers: customHeaders,
            response_delay_ms: delay,
        });
    } else {
        // Create
        createServer({
            name,
            port,
            http_status_code: httpStatus,
            html_title: title || undefined,
            response_body: body || undefined,
            custom_headers: Object.keys(customHeaders).length > 0 ? customHeaders : undefined,
            response_delay_ms: delay > 0 ? delay : undefined,
        });
    }
});

// --- Init ---
fetchServers();
setInterval(fetchServers, 2000);
