
const API = "http://localhost:3000/api";

// --- State ---
let token = localStorage.getItem("token") || null;
let currentUser = null;

// --- Inicijalizacija ---
document.addEventListener("DOMContentLoaded", () => {
    if (token) {
        fetchCurrentUser();
    }
    loadEvents();
    showSection("events");
});

// AUTH

function switchTab(tab) {
    document.querySelectorAll(".tab").forEach(t => t.classList.remove("active"));
    if (tab === "login") {
        document.getElementById("form-login").classList.remove("hidden");
        document.getElementById("form-register").classList.add("hidden");
        document.querySelectorAll(".tab")[0].classList.add("active");
    } else {
        document.getElementById("form-login").classList.add("hidden");
        document.getElementById("form-register").classList.remove("hidden");
        document.querySelectorAll(".tab")[1].classList.add("active");
    }
    document.getElementById("auth-message").textContent = "";
}

async function handleLogin(e) {
    e.preventDefault();
    const email = document.getElementById("login-email").value;
    const password = document.getElementById("login-password").value;

    const res = await apiPost("/auth/login", { email, password });
    if (res.success) {
        token = res.data.token;
        localStorage.setItem("token", token);
        await fetchCurrentUser();
        showSection("events");
        toast("Uspesna prijava!", "success");
    } else {
        document.getElementById("auth-message").textContent = res.message;
        document.getElementById("auth-message").className = "message error";
    }
}

async function handleRegister(e) {
    e.preventDefault();
    const email = document.getElementById("reg-email").value;
    const password = document.getElementById("reg-password").value;

    const res = await apiPost("/auth/register", { email, password });
    if (res.success) {
        token = res.data.token;
        localStorage.setItem("token", token);
        await fetchCurrentUser();
        showSection("events");
        toast("Registracija uspesna!", "success");
    } else {
        document.getElementById("auth-message").textContent = res.message;
        document.getElementById("auth-message").className = "message error";
    }
}

async function fetchCurrentUser() {
    const res = await apiGet("/auth/me");
    if (res.success) {
        currentUser = res.data;
        updateNavbar();
    } else {
        logout();
    }
}

function logout() {
    token = null;
    currentUser = null;
    localStorage.removeItem("token");
    updateNavbar();
    showSection("events");
    toast("Odjavili ste se", "success");
}

function updateNavbar() {
    const isLoggedIn = !!currentUser;
    const isOrgOrAdmin = isLoggedIn && (currentUser.role === "Organizer" || currentUser.role === "Admin");
    const isAdmin = isLoggedIn && currentUser.role === "Admin";

    toggle("nav-login", !isLoggedIn);
    toggle("nav-logout", isLoggedIn);
    toggle("nav-my-reg", isLoggedIn);
    toggle("nav-analytics", isOrgOrAdmin);
    toggle("nav-user", isLoggedIn);
    toggle("create-event-box", isOrgOrAdmin);

    if (isLoggedIn) {
        document.getElementById("nav-user").textContent = currentUser.email + " (" + currentUser.role + ")";
    }
}

// EVENTS

async function loadEvents() {
    const search = document.getElementById("search-input")?.value || "";
    const category = document.getElementById("category-filter")?.value || "";

    let url = "/events?";
    if (search) url += "search=" + encodeURIComponent(search) + "&";
    if (category) url += "category=" + encodeURIComponent(category);

    const res = await apiGet(url);
    const container = document.getElementById("events-list");

    if (res.success && res.data.length > 0) {
        container.innerHTML = res.data.map(evt => `
            <div class="card">
                <span class="badge">${esc(evt.category)}</span>
                <h3>${esc(evt.title)}</h3>
                <p>${esc(evt.description)}</p>
                <p><strong>Lokacija:</strong> ${esc(evt.location)}</p>
                <p><strong>Datum:</strong> ${formatDate(evt.date_time)}</p>
                <div class="meta">
                    <span class="capacity">Kapacitet: ${evt.capacity}</span>
                    ${token ? `<button class="btn btn-primary btn-small" onclick="registerForEvent('${evt.id}')">Prijavi se</button>` : ""}
                </div>
            </div>
        `).join("");
    } else {
        container.innerHTML = "<p>Nema dogadjaja.</p>";
    }
}

function searchEvents() {
    loadEvents();
}

async function handleCreateEvent(e) {
    e.preventDefault();

    const data = {
        title: document.getElementById("evt-title").value,
        description: document.getElementById("evt-description").value,
        location: document.getElementById("evt-location").value,
        date_time: document.getElementById("evt-datetime").value + ":00",
        capacity: parseInt(document.getElementById("evt-capacity").value),
        category: document.getElementById("evt-category").value,
    };

    const res = await apiPost("/events", data);
    if (res.success) {
        toast("Dogadjaj kreiran!", "success");
        loadEvents();
        // Reset forme
        document.getElementById("evt-title").value = "";
        document.getElementById("evt-description").value = "";
        document.getElementById("evt-location").value = "";
        document.getElementById("evt-datetime").value = "";
    } else {
        toast(res.message, "error");
    }
}

// REGISTRATIONS

async function registerForEvent(eventId) {
    const res = await apiPost("/registrations", { event_id: eventId });
    if (res.success) {
        toast("Uspesno prijavljeni! Kod karte: " + res.data.ticket_code, "success");
    } else {
        toast(res.message, "error");
    }
}

async function loadMyRegistrations() {
    const res = await apiGet("/registrations/my");
    const container = document.getElementById("my-registrations-list");

    if (res.success && res.data.length > 0) {
        container.innerHTML = res.data.map(reg => `
            <div class="card">
                <span class="badge">${reg.status === "confirmed" ? "Potvrdjeno" : "Otkazano"}</span>
                <p><strong>Karta:</strong> <span class="ticket-code">${esc(reg.ticket_code)}</span></p>
                <p><strong>Event ID:</strong> ${reg.event_id}</p>
                <p><strong>Datum prijave:</strong> ${formatDate(reg.created_at)}</p>
                <div class="meta">
                    <button class="btn btn-secondary btn-small" onclick="downloadQR('${reg.id}')">QR Kod</button>
                    ${reg.status === "confirmed" ? `<button class="btn btn-danger btn-small" onclick="cancelRegistration('${reg.id}')">Otkazi</button>` : ""}
                </div>
            </div>
        `).join("");
    } else {
        container.innerHTML = "<p>Nemate prijava.</p>";
    }
}

async function downloadQR(regId) {
    try {
        const res = await fetch(API + "/registrations/" + regId + "/qr", {
            headers: token ? { "Authorization": "Bearer " + token } : {},
        });
        if (res.ok) {
            const blob = await res.blob();
            const url = URL.createObjectURL(blob);
            window.open(url, "_blank");
        } else {
            toast("Greska pri generisanju QR koda", "error");
        }
    } catch {
        toast("QR servis nije dostupan", "error");
    }
}

async function cancelRegistration(id) {
    const res = await apiDelete("/registrations/" + id);
    if (res.success) {
        toast("Prijava otkazana", "success");
        loadMyRegistrations();
    } else {
        toast(res.message, "error");
    }
}

// ANALYTICS

async function loadAnalytics() {
    const res = await apiGet("/analytics/overview");
    const container = document.getElementById("analytics-overview");

    if (res.success) {
        const s = res.data;
        container.innerHTML = `
            <div class="stat-card">
                <div class="number">${s.total_registrations}</div>
                <div class="label">Ukupno prijava</div>
            </div>
            <div class="stat-card">
                <div class="number">${s.total_confirmed}</div>
                <div class="label">Potvrdjeno</div>
            </div>
            <div class="stat-card">
                <div class="number">${s.total_cancelled}</div>
                <div class="label">Otkazano</div>
            </div>
            <div class="stat-card">
                <div class="number">${s.unique_events}</div>
                <div class="label">Dogadjaja</div>
            </div>
            <div class="stat-card">
                <div class="number">${s.unique_users}</div>
                <div class="label">Korisnika</div>
            </div>
        `;
    } else {
        container.innerHTML = "<p>" + res.message + "</p>";
    }
}

// NAVIGACIJA

function showSection(name) {
    document.querySelectorAll("main > section").forEach(s => s.classList.add("hidden"));
    document.getElementById("section-" + name)?.classList.remove("hidden");

    if (name === "events") loadEvents();
    if (name === "my-registrations") loadMyRegistrations();
    if (name === "analytics") loadAnalytics();
}

// HELPERS

async function apiGet(path) {
    try {
        const res = await fetch(API + path, {
            headers: token ? { "Authorization": "Bearer " + token } : {},
        });
        return await res.json();
    } catch {
        return { success: false, message: "Greska u komunikaciji sa serverom" };
    }
}

async function apiPost(path, body) {
    try {
        const res = await fetch(API + path, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                ...(token ? { "Authorization": "Bearer " + token } : {}),
            },
            body: JSON.stringify(body),
        });
        return await res.json();
    } catch {
        return { success: false, message: "Greska u komunikaciji sa serverom" };
    }
}

async function apiDelete(path) {
    try {
        const res = await fetch(API + path, {
            method: "DELETE",
            headers: token ? { "Authorization": "Bearer " + token } : {},
        });
        return await res.json();
    } catch {
        return { success: false, message: "Greska u komunikaciji sa serverom" };
    }
}

function toggle(id, show) {
    const el = document.getElementById(id);
    if (el) el.classList.toggle("hidden", !show);
}

function esc(str) {
    const div = document.createElement("div");
    div.textContent = str || "";
    return div.innerHTML;
}

function formatDate(str) {
    if (!str) return "";
    const d = new Date(str);
    return d.toLocaleDateString("sr-RS") + " " + d.toLocaleTimeString("sr-RS", { hour: "2-digit", minute: "2-digit" });
}

function toast(msg, type) {
    const el = document.getElementById("toast");
    el.textContent = msg;
    el.className = "toast " + type;
    setTimeout(() => el.classList.add("hidden"), 3000);
}
