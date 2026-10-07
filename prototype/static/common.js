window.VactionUI = (() => {
  const toastEl = document.getElementById("toast");

  function toast(message, isError = false) {
    if (!toastEl) return;
    toastEl.textContent = message;
    toastEl.classList.toggle("error", isError);
    toastEl.classList.add("show");
    clearTimeout(toast._timer);
    toast._timer = setTimeout(() => toastEl.classList.remove("show"), 3200);
  }

  async function api(path, options = {}) {
    const res = await fetch(path, {
      credentials: "same-origin",
      headers: { "Content-Type": "application/json", ...(options.headers || {}) },
      ...options,
    });
    const text = await res.text();
    let data = null;
    if (text) {
      try {
        data = JSON.parse(text);
      } catch {
        data = text;
      }
    }
    if (!res.ok) {
      const msg = typeof data === "string" ? data : text || res.statusText;
      throw new Error(msg);
    }
    return data;
  }

  async function getSession(role) {
    try {
      return await api(`/auth/me?role=${encodeURIComponent(role)}`);
    } catch {
      return null;
    }
  }

  async function getSessions() {
    try {
      return await api("/auth/sessions");
    } catch {
      return { admin: null, employee: null };
    }
  }

  async function logout(role) {
    await api(`/auth/logout?role=${encodeURIComponent(role)}`, { method: "POST" });
  }

  function escapeHtml(str) {
    return String(str)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function dateToParts(isoDate) {
    const [y, m, d] = isoDate.split("-").map(Number);
    return { year: y % 100, month: m, day: d };
  }

  function addDays(isoDate, days) {
    const date = new Date(`${isoDate}T12:00:00`);
    date.setDate(date.getDate() + days);
    return date.toISOString().slice(0, 10);
  }

  /** Parse API vacation dates formatted as YY-MM-DD. */
  function parseAppDate(str) {
    const [yy, mo, d] = String(str).split("-").map(Number);
    const year = yy >= 70 ? 1900 + yy : 2000 + yy;
    return new Date(year, mo - 1, d, 12, 0, 0);
  }

  function toIsoDate(date) {
    const y = date.getFullYear();
    const m = String(date.getMonth() + 1).padStart(2, "0");
    const d = String(date.getDate()).padStart(2, "0");
    return `${y}-${m}-${d}`;
  }

  function startOfDay(date) {
    return new Date(date.getFullYear(), date.getMonth(), date.getDate(), 12, 0, 0);
  }

  function dayDiff(from, to) {
    const ms = 86400000;
    return Math.round((startOfDay(to) - startOfDay(from)) / ms);
  }

  function formatAppDate(str) {
    const date = parseAppDate(str);
    return date.toLocaleDateString("en-GB", { day: "numeric", month: "short", year: "numeric" });
  }

  const DOW = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

  function formatTimelineDay(date) {
    return {
      dow: DOW[date.getDay()],
      dom: date.getDate(),
      month: date.toLocaleDateString("en-GB", { month: "short" }),
    };
  }

  function statusClass(status) {
    const label = formatStatus(status).toLowerCase();
    if (label.includes("pending") || String(status).includes("Awaiting")) return "status-awaiting";
    if (String(status).includes("CancellationRequested")) return "status-cancel-pending";
    if (label.includes("cancelled") || String(status).includes("Cancelled")) return "status-cancelled";
    if (label.includes("approved") || String(status).includes("Approved")) return "status-approved";
    if (label.includes("denied") || String(status).includes("Denied")) return "status-denied";
    return "";
  }

  function formatStatus(status) {
    const raw = String(status || "");
    if (raw.includes("Awaiting")) return "Pending";
    if (raw.includes("CancellationRequested")) return "Cancel pending";
    if (raw.includes("Cancelled")) return "Cancelled";
    if (raw.includes("Approved")) return "Approved";
    if (raw.includes("Denied")) return "Denied";
    return raw || "Unknown";
  }

  /** Global employee permission keys → labels (UI source of truth). */
  const EMPLOYEE_PERMISSION_LABELS = {
    create_groups: "Create groups",
    approve_vacations: "Globally approve others' vacations",
    approve_own_vacations: "Globally approve own vacations",
    request_vacation: "Request vacation",
    edit_own_account: "Edit own account",
    edit_vacations: "Globally edit vacations",
    delete_vacations: "Globally delete vacations",
  };

  /** Group-scoped member permission keys → labels (UI source of truth). */
  const GROUP_MEMBER_PERMISSION_LABELS = {
    can_approve_vacations: "Approve vacations for this group",
    can_approve_own_vacations: "Approve own vacations for this group",
    can_edit_group: "Change this group's information",
    can_add_members: "Add people to this group",
    can_remove_members: "Remove people from this group",
    can_delete_group: "Delete this group",
  };

  function permissionKeys(labels) {
    return Object.keys(labels || {});
  }

  function defaultPermissions(labels, overrides = {}) {
    const out = {};
    for (const key of permissionKeys(labels)) {
      out[key] = Boolean(overrides[key]);
    }
    return out;
  }

  function mergePermissions(labels, perms) {
    return defaultPermissions(labels, perms || {});
  }

  function humanizePermissionKey(key) {
    return String(key || "")
      .replace(/^can_/, "")
      .replace(/_/g, " ")
      .replace(/\b\w/g, (ch) => ch.toUpperCase());
  }

  /**
   * Render checkboxes for every key in `labels`, using values from `perms`.
   * Extra boolean keys present on `perms` but missing from `labels` are also shown.
   */
  function renderPermissionCheckboxes(perms, labels) {
    const catalog = labels || {};
    const values = perms && typeof perms === "object" ? perms : {};
    const keys = [
      ...permissionKeys(catalog),
      ...Object.keys(values).filter(
        (key) => typeof values[key] === "boolean" && !Object.prototype.hasOwnProperty.call(catalog, key)
      ),
    ];
    return keys
      .map((key) => {
        const checked = Boolean(values[key]);
        const label = catalog[key] || humanizePermissionKey(key);
        return `<label class="check-row"><input type="checkbox" name="${escapeHtml(key)}"${
          checked ? " checked" : ""
        }> ${escapeHtml(label)}</label>`;
      })
      .join("");
  }

  /** Read checkbox values for every catalog key (and any extra boolean keys in `base`). */
  function readPermissionForm(formData, labels, base = {}) {
    const catalog = labels || {};
    const keys = [
      ...permissionKeys(catalog),
      ...Object.keys(base || {}).filter(
        (key) => typeof base[key] === "boolean" && !Object.prototype.hasOwnProperty.call(catalog, key)
      ),
    ];
    const out = {};
    for (const key of keys) {
      out[key] = formData.get(key) === "on";
    }
    return out;
  }

  function formatPermissionFlags(perms, labels = EMPLOYEE_PERMISSION_LABELS) {
    const catalog = labels || EMPLOYEE_PERMISSION_LABELS;
    const values = perms && typeof perms === "object" ? perms : {};
    const active = [
      ...permissionKeys(catalog).filter((key) => values[key]),
      ...Object.keys(values).filter(
        (key) =>
          values[key] &&
          typeof values[key] === "boolean" &&
          !Object.prototype.hasOwnProperty.call(catalog, key)
      ),
    ].map((key) => catalog[key] || humanizePermissionKey(key));
    return active.length ? active.join(" · ") : "No permissions";
  }

  function showTab(tabs, panels, name) {
    tabs.forEach((tab) => tab.classList.toggle("active", tab.dataset.tab === name));
    panels.forEach((panel) => panel.classList.toggle("hidden", panel.dataset.panel !== name));
  }

  /** End date must be strictly after start date. */
  function vacationRangeValid(startIso, endIso) {
    if (!startIso || !endIso) return false;
    return endIso > startIso;
  }

  function syncVacationEndMin(startInput, endInput, hintEl) {
    if (!startInput?.value) return;
    const minEnd = addDays(startInput.value, 1);
    endInput.min = minEnd;
    if (!endInput.value || endInput.value <= startInput.value) {
      endInput.value = minEnd;
    }
    if (hintEl) {
      hintEl.textContent = vacationRangeValid(startInput.value, endInput.value)
        ? ""
        : "End date must be after the start date.";
      hintEl.classList.toggle("hidden", vacationRangeValid(startInput.value, endInput.value));
    }
  }

  function bindVacationDateRange(startId, endId, hintId) {
    const start = document.getElementById(startId);
    const end = document.getElementById(endId);
    const hint = hintId ? document.getElementById(hintId) : null;
    if (!start || !end) return;

    const today = new Date().toISOString().slice(0, 10);
    if (!start.value) start.value = today;
    syncVacationEndMin(start, end, hint);

    start.addEventListener("change", () => syncVacationEndMin(start, end, hint));
    end.addEventListener("change", () => {
      if (hint) {
        const ok = vacationRangeValid(start.value, end.value);
        hint.textContent = ok ? "" : "End date must be after the start date.";
        hint.classList.toggle("hidden", ok);
      }
      if (start.value) end.min = addDays(start.value, 1);
    });
  }

  function assertVacationRange(startIso, endIso) {
    if (!vacationRangeValid(startIso, endIso)) {
      throw new Error("End date must be after the start date.");
    }
  }

  return {
    getSession,
    getSessions,
    logout,
    toast,
    api,
    escapeHtml,
    dateToParts,
    parseAppDate,
    toIsoDate,
    dayDiff,
    formatAppDate,
    formatTimelineDay,
    statusClass,
    formatStatus,
    EMPLOYEE_PERMISSION_LABELS,
    GROUP_MEMBER_PERMISSION_LABELS,
    permissionKeys,
    defaultPermissions,
    mergePermissions,
    renderPermissionCheckboxes,
    readPermissionForm,
    formatPermissionFlags,
    showTab,
    vacationRangeValid,
    bindVacationDateRange,
    assertVacationRange,
  };
})();
