const ROLE = "user";
const ui = window.VactionUI;

const ADMIN_DAY_WIDTH_DESKTOP = 28;
const ADMIN_DAY_WIDTH_TABLET = 26;
const ADMIN_DAY_WIDTH_COMPACT = 24;
const ADMIN_DAY_WIDTH_NARROW = 22;
const ADMIN_TIMELINE_DAYS_BEFORE = 21;
const ADMIN_TIMELINE_DAYS_AFTER = 56;
const ADMIN_TIMELINE_SCROLL_DAYS = 14;
const ADMIN_TIMELINE_DAY_COUNT = ADMIN_TIMELINE_DAYS_BEFORE + ADMIN_TIMELINE_DAYS_AFTER + 1;

function resolveAdminDayWidth() {
  const width = window.innerWidth || document.documentElement.clientWidth || ADMIN_DAY_WIDTH_DESKTOP;
  if (width <= 480) return ADMIN_DAY_WIDTH_NARROW;
  if (width <= 720) return ADMIN_DAY_WIDTH_COMPACT;
  if (width <= 960) return ADMIN_DAY_WIDTH_TABLET;
  return ADMIN_DAY_WIDTH_DESKTOP;
}

let ADMIN_DAY_WIDTH = resolveAdminDayWidth();
let adminResizeFrame = 0;
let adminTimelineWindowOffset = 0;
let adminSkipScrollToTodayOnce = false;
let adminScrollExtending = false;
let adminZoomLockUntil = 0;

const GEAR_ICON = `<svg class="icon-gear" viewBox="0 0 24 24" aria-hidden="true">
  <path fill="currentColor" d="M19.14 12.94c.04-.31.06-.63.06-.94s-.02-.63-.06-.94l2.03-1.58a.49.49 0 0 0 .12-.61l-1.92-3.32a.49.49 0 0 0-.59-.22l-2.39.96a7.03 7.03 0 0 0-1.62-.94l-.36-2.54A.48.48 0 0 0 14 2h-4a.48.48 0 0 0-.48.41l-.36 2.54c-.59.24-1.13.55-1.62.94l-2.39-.96a.49.49 0 0 0-.59.22L2.65 8.87a.49.49 0 0 0 .12.61l2.03 1.58c-.04.31-.06.63-.06.94s.02.63.06.94L2.77 14.5a.49.49 0 0 0-.12.61l1.92 3.32c.12.22.37.3.59.22l2.39-.96c.5.39 1.04.71 1.62.94l.36 2.54c.05.24.24.41.48.41h4c.24 0 .44-.17.48-.41l.36-2.54c.59-.24 1.13-.55 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32a.49.49 0 0 0-.12-.61l-2.03-1.58zM12 15.6A3.6 3.6 0 1 1 12 8.4a3.6 3.6 0 0 1 0 7.2z"/>
</svg>`;

const PLUS_ICON = `<svg class="icon-gear" viewBox="0 0 24 24" aria-hidden="true">
  <path fill="currentColor" d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
</svg>`;

const REMOVE_ICON = `<svg class="icon-gear" viewBox="0 0 24 24" aria-hidden="true">
  <path fill="currentColor" d="M19 13H5v-2h14v2z"/>
</svg>`;

const els = {
  loginPanel: document.getElementById("login-panel"),
  loginFailedPanel: document.getElementById("login-failed-panel"),
  loginFailedMessage: document.getElementById("login-failed-message"),
  appPanel: document.getElementById("app-panel"),
  sessionBadge: document.getElementById("session-badge"),
  logoutBtn: document.getElementById("logout-btn"),
  loginForm: document.getElementById("login-form"),
  loginError: document.getElementById("login-error"),
  tabs: document.querySelectorAll("#app-panel > .tabs .tab"),
  tabPanels: document.querySelectorAll("#app-panel > .tab-panel"),
  orgSummary: document.getElementById("org-summary"),
  userSummary: document.getElementById("user-summary"),
  orgDetailsForm: document.getElementById("org-details-form"),
  orgName: document.getElementById("org-name"),
  orgDetailsMeta: document.getElementById("org-details-meta"),
  orgGroupsSearch: document.getElementById("org-groups-search"),
  orgGroupsList: document.getElementById("org-groups-list"),
  toggleCreateGroup: document.getElementById("toggle-create-group"),
  createGroupCard: document.getElementById("create-group-card"),
  orgCreateGroupForm: document.getElementById("org-create-group-form"),
  orgEmployeesSearch: document.getElementById("org-employees-search"),
  orgEmployeesList: document.getElementById("org-employees-list"),
  employeeForm: document.getElementById("employee-form"),
  paymentForm: document.getElementById("payment-form"),
  toggleCreate: document.getElementById("toggle-create"),
  createCard: document.getElementById("create-card"),
  modal: document.getElementById("employee-modal"),
  modalTitle: document.getElementById("modal-title"),
  modalSubtitle: document.getElementById("modal-subtitle"),
  modalBody: document.getElementById("modal-body"),
  modalClose: document.getElementById("modal-close"),
  eventsList: document.getElementById("events-list"),
  eventsEmpty: document.getElementById("events-empty"),
  eventsFilter: document.getElementById("events-filter"),
  refreshEvents: document.getElementById("refresh-events"),
  vacationsEmpty: document.getElementById("admin-vacations-empty"),
  vacationsSearch: document.getElementById("vacations-search"),
  vacationsStatusFilter: document.getElementById("vacations-status-filter"),
  refreshVacations: document.getElementById("refresh-vacations"),
  timelineViewport: document.getElementById("admin-timeline-viewport"),
  timelineNames: document.getElementById("admin-timeline-names"),
  timelineNameRows: document.getElementById("admin-timeline-name-rows"),
  timelinePane: document.getElementById("admin-timeline-pane"),
  timelineDays: document.getElementById("admin-timeline-days"),
  timelineRows: document.getElementById("admin-timeline-rows"),
  timelineRangeLabel: document.getElementById("admin-timeline-range-label"),
  timelinePrev: document.getElementById("admin-timeline-prev"),
  timelineToday: document.getElementById("admin-timeline-today"),
  timelineNext: document.getElementById("admin-timeline-next"),
  vacationModal: document.getElementById("vacation-detail-modal"),
  vacationTitle: document.getElementById("vacation-detail-title"),
  vacationSubtitle: document.getElementById("vacation-detail-subtitle"),
  vacationClose: document.getElementById("vacation-detail-close"),
  vacationTabs: document.querySelectorAll("#vacation-detail-tabs .tab"),
  vacationPanels: document.querySelectorAll("#vacation-detail-modal .detail-tab-panel"),
  vacationOverview: document.getElementById("vacation-detail-overview"),
  vacationDetails: document.getElementById("vacation-detail-details"),
  vacationActivity: document.getElementById("vacation-detail-activity"),
};

let session = null;
let orgId = null;
let currentOrg = null;
let orgGroups = [];
let orgEmployees = [];
let employeeSummaries = {};
let employeesById = new Map();
let allEvents = [];
let allVacations = [];
let vacationsById = new Map();
let activeVacationId = null;
let activeEmployeeId = null;
let activeGroupId = null;
let loginFailed = false;
let adminTimelineStart = null;
let adminPendingScrollVacationId = null;
let adminAutoZoomKey = "";
let adminFocusedVacationId = null;
/** In-progress group editor: membership + permission edits applied only on Save. */
let groupEditorDraft = null;

function emptyGroupMemberPerms() {
  return ui.defaultPermissions(ui.GROUP_MEMBER_PERMISSION_LABELS);
}

function showLoginForm() {
  loginFailed = false;
  els.loginFailedPanel?.classList.add("hidden");
  els.loginPanel?.classList.remove("hidden");
}

function showLoginFailed(message) {
  loginFailed = true;
  if (els.loginFailedMessage) {
    els.loginFailedMessage.textContent = message || "Invalid email or password.";
  }
  els.loginPanel?.classList.add("hidden");
  els.loginFailedPanel?.classList.remove("hidden");
}

function fillDemoCredentials() {
  els.loginForm.email.value = "demo@test.com";
  els.loginForm.password.value = "demo";
}

function updateChrome() {
  const loggedIn = Boolean(session);
  if (loggedIn) {
    loginFailed = false;
    els.loginPanel?.classList.add("hidden");
    els.loginFailedPanel?.classList.add("hidden");
  } else if (loginFailed) {
    els.loginPanel?.classList.add("hidden");
    els.loginFailedPanel?.classList.remove("hidden");
  } else {
    els.loginPanel?.classList.remove("hidden");
    els.loginFailedPanel?.classList.add("hidden");
  }
  els.appPanel.classList.toggle("hidden", !loggedIn);
  els.logoutBtn.classList.toggle("hidden", !loggedIn);
  if (session) {
    els.sessionBadge.textContent = `Admin #${session.subject_id}`;
    els.sessionBadge.classList.remove("hidden");
  } else {
    els.sessionBadge.classList.add("hidden");
  }
}

function closeModal() {
  els.modal.classList.add("hidden");
  els.modal.setAttribute("aria-hidden", "true");
  activeEmployeeId = null;
  activeGroupId = null;
  groupEditorDraft = null;
}

function employeeLabel(employeeId) {
  const employee = employeesById.get(employeeId);
  return employee ? employee.username : `Employee #${employeeId}`;
}

function decisionMeta(vacation) {
  if (String(vacation.status).includes("CancellationRequested")) {
    const by = vacation.cancellation_requested_by
      ? employeeLabel(vacation.cancellation_requested_by)
      : null;
    return by ? `Cancellation requested by ${ui.escapeHtml(by)}` : "Cancellation requested";
  }
  if (String(vacation.status).includes("Cancelled")) {
    const by = vacation.cancelled_by ? employeeLabel(vacation.cancelled_by) : null;
    return by ? `Cancelled by ${ui.escapeHtml(by)}` : "Cancelled";
  }
  if (vacation.approved_by) {
    return `Approved by ${ui.escapeHtml(employeeLabel(vacation.approved_by))}`;
  }
  if (vacation.denied_by) {
    return `Denied by ${ui.escapeHtml(employeeLabel(vacation.denied_by))}`;
  }
  const votes = Array.isArray(vacation.group_approvals) ? vacation.group_approvals : [];
  if (votes.length && String(vacation.status).includes("Awaiting")) {
    const done = votes.filter((v) => v.status === "approved").length;
    return `Waiting for group approval (${done}/${votes.length})`;
  }
  if (String(vacation.status).includes("Awaiting")) return "Waiting for approval";
  return "";
}

function clearAdminVacationFilters({ resetWindow = true } = {}) {
  if (els.vacationsSearch) els.vacationsSearch.value = "";
  if (els.vacationsStatusFilter) els.vacationsStatusFilter.value = "";
  adminAutoZoomKey = "";
  adminFocusedVacationId = null;
  adminPendingScrollVacationId = null;
  if (resetWindow) {
    adminTimelineWindowOffset = 0;
    adminSkipScrollToTodayOnce = false;
  }
}

function adminVacationFiltersActive() {
  return (
    Boolean(String(els.vacationsSearch?.value || "").trim()) ||
    Boolean(String(els.vacationsStatusFilter?.value || ""))
  );
}

function filteredVacations() {
  const query = String(els.vacationsSearch?.value || "")
    .trim()
    .toLowerCase();
  const statusFilter = String(els.vacationsStatusFilter?.value || "");
  return allVacations.filter((vacation) => {
    if (statusFilter) {
      const status = String(vacation.status);
      if (statusFilter === "Approved") {
        if (!status.includes("Approved") || status.includes("Cancellation")) return false;
      } else if (!status.includes(statusFilter)) {
        return false;
      }
    }
    if (!query) return true;
    const name = employeeLabel(vacation.employee_id).toLowerCase();
    const details = String(vacation.details || "").toLowerCase();
    return (
      name.includes(query) ||
      details.includes(query) ||
      String(vacation.id).includes(query)
    );
  });
}

function sortedFilteredVacations() {
  return [...filteredVacations()].sort((a, b) => {
    const startCmp = String(a.start).localeCompare(String(b.start));
    if (startCmp !== 0) return startCmp;
    const endCmp = String(a.end).localeCompare(String(b.end));
    if (endCmp !== 0) return endCmp;
    return a.id - b.id;
  });
}

function latestFilteredVacation(list) {
  const items = list && list.length ? list : sortedFilteredVacations();
  if (!items.length) return null;
  // Prefer highest id among the filtered set as "latest request".
  return items.reduce((best, vacation) => (vacation.id > best.id ? vacation : best));
}

function scrollAdminTimelineToEmployee(employeeId) {
  if (employeeId == null) return;
  const nameRow = els.timelineNameRows?.querySelector(
    `[data-employee-id="${employeeId}"]`
  );
  const track = els.timelineRows?.querySelector(`[data-employee-track="${employeeId}"]`);
  if (!nameRow || !els.timelineNames || !els.timelinePane) return;
  const rowTop = nameRow.offsetTop;
  const rowHeight = nameRow.offsetHeight || 0;
  const viewHeight = els.timelineNames.clientHeight;
  const target = Math.max(0, rowTop - viewHeight / 2 + rowHeight / 2);
  els.timelineNames.scrollTop = target;
  els.timelinePane.scrollTop = target;
}

function setAdminFocusedVacation(vacationId) {
  adminFocusedVacationId = vacationId == null ? null : Number(vacationId);
  els.timelineRows?.querySelectorAll(".timeline-bar.is-focused").forEach((bar) => {
    bar.classList.remove("is-focused");
  });
  if (adminFocusedVacationId == null) return;
  const bar = els.timelineRows?.querySelector(
    `.timeline-bar[data-open-vacation="${adminFocusedVacationId}"]`
  );
  bar?.classList.add("is-focused");
}

function lockAdminTimelineZoom(ms = 600) {
  adminZoomLockUntil = Math.max(adminZoomLockUntil, Date.now() + ms);
  adminScrollExtending = true;
}

function unlockAdminTimelineZoomSoon(ms = 600) {
  window.setTimeout(() => {
    if (Date.now() >= adminZoomLockUntil) adminScrollExtending = false;
  }, ms);
}

function applyAdminVacationCenterScroll(vacation) {
  if (!vacation || !els.timelinePane || adminTimelineStart === null) return false;
  const pane = els.timelinePane;
  if (pane.clientWidth <= 0) return false;

  const bar = els.timelineRows?.querySelector(
    `.timeline-bar[data-open-vacation="${vacation.id}"]`
  );
  if (!bar) return false;

  lockAdminTimelineZoom(1200);
  pane.classList.add("is-programmatic-scroll");
  // Inline style wins over CSS `scroll-behavior: smooth`, which otherwise
  // drops or animates scrollLeft and leaves the start of the bar in view.
  const prevBehavior = pane.style.scrollBehavior;
  pane.style.scrollBehavior = "auto";

  const barLeft = parseFloat(bar.style.left);
  const barWidth = parseFloat(bar.style.width);
  const left = Number.isFinite(barLeft) ? barLeft : bar.offsetLeft;
  const width = Number.isFinite(barWidth) && barWidth > 0 ? barWidth : bar.offsetWidth;
  if (!(width > 0)) {
    pane.style.scrollBehavior = prevBehavior;
    pane.classList.remove("is-programmatic-scroll");
    return false;
  }

  const target = Math.max(0, left + width / 2 - pane.clientWidth / 2);
  pane.scrollLeft = target;

  // Correct any sub-pixel drift after layout.
  const paneRect = pane.getBoundingClientRect();
  const barRect = bar.getBoundingClientRect();
  const drift =
    barRect.left + barRect.width / 2 - (paneRect.left + paneRect.width / 2);
  if (Math.abs(drift) > 1) {
    pane.scrollLeft += drift;
  }

  scrollAdminTimelineToEmployee(vacation.employee_id);
  setAdminFocusedVacation(vacation.id);

  window.setTimeout(() => {
    pane.style.scrollBehavior = prevBehavior;
    pane.classList.remove("is-programmatic-scroll");
    unlockAdminTimelineZoomSoon(0);
  }, 100);
  return true;
}

function scrollAdminTimelineToVacation(vacation) {
  if (!vacation || !els.timelinePane) return false;
  setAdminFocusedVacation(vacation.id);
  lockAdminTimelineZoom(900);

  const today = adminStartOfToday();
  const start = ui.parseAppDate(vacation.start);
  const end = ui.parseAppDate(vacation.end);
  const midDate = new Date((start.getTime() + end.getTime()) / 2);
  const midNoon = new Date(
    midDate.getFullYear(),
    midDate.getMonth(),
    midDate.getDate(),
    12,
    0,
    0
  );
  const centerIndex = Math.floor(ADMIN_TIMELINE_DAY_COUNT / 2);
  const desiredOffset =
    ui.dayDiff(today, midNoon) - centerIndex + ADMIN_TIMELINE_DAYS_BEFORE;

  if (desiredOffset !== adminTimelineWindowOffset || adminTimelineStart === null) {
    adminTimelineWindowOffset = desiredOffset;
    adminSkipScrollToTodayOnce = true;
    adminPendingScrollVacationId = vacation.id;
    renderAdminVacationsList();
    return true;
  }

  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      if (!applyAdminVacationCenterScroll(vacation)) {
        window.setTimeout(() => applyAdminVacationCenterScroll(vacation), 50);
      }
    });
  });
  return true;
}

function maybeAutoZoomToFilteredVacation(list) {
  const query = String(els.vacationsSearch?.value || "").trim();
  const statusFilter = String(els.vacationsStatusFilter?.value || "");
  if (!query || !statusFilter) {
    adminAutoZoomKey = "";
    return false;
  }
  const target = latestFilteredVacation(list);
  if (!target) {
    adminAutoZoomKey = "";
    setAdminFocusedVacation(null);
    return false;
  }
  const key = `${query.toLowerCase()}|${statusFilter}|${target.id}`;
  adminAutoZoomKey = key;
  return scrollAdminTimelineToVacation(target);
}

function jumpAdminTimelineVacation(direction) {
  const items = sortedFilteredVacations();
  if (!items.length) {
    shiftAdminTimeline(direction * ADMIN_TIMELINE_SCROLL_DAYS);
    return;
  }
  let index = items.findIndex((vacation) => vacation.id === adminFocusedVacationId);
  if (index < 0) {
    index = direction > 0 ? -1 : items.length;
  }
  const nextIndex = Math.max(0, Math.min(items.length - 1, index + direction));
  const target = items[nextIndex];
  if (!target) return;
  adminAutoZoomKey = `${String(els.vacationsSearch?.value || "")
    .trim()
    .toLowerCase()}|${String(els.vacationsStatusFilter?.value || "")}|${target.id}`;
  scrollAdminTimelineToVacation(target, true);
}

function adminStartOfToday() {
  const now = new Date();
  return new Date(now.getFullYear(), now.getMonth(), now.getDate(), 12, 0, 0);
}

function adminTimelineOriginDate() {
  const today = adminStartOfToday();
  const start = new Date(today);
  start.setDate(start.getDate() - ADMIN_TIMELINE_DAYS_BEFORE + adminTimelineWindowOffset);
  return start;
}

function adminDateAtOffset(base, offset) {
  const date = new Date(base);
  date.setDate(date.getDate() + offset);
  return date;
}

function scrollAdminTimelineToToday(smooth = true) {
  if (!els.timelinePane || adminTimelineStart === null) return;
  const pane = els.timelinePane;
  const today = adminStartOfToday();
  const index = ui.dayDiff(adminTimelineStart, today);
  const target =
    index * ADMIN_DAY_WIDTH - pane.clientWidth / 2 + ADMIN_DAY_WIDTH / 2;
  if (smooth) {
    pane.scrollTo({ left: Math.max(0, target), behavior: "smooth" });
  } else {
    const prev = pane.style.scrollBehavior;
    pane.style.scrollBehavior = "auto";
    pane.scrollLeft = Math.max(0, target);
    pane.style.scrollBehavior = prev;
  }
}

function shiftAdminTimeline(days) {
  if (!days) return;
  adminTimelineWindowOffset += days;
  adminSkipScrollToTodayOnce = true;
  const pane = els.timelinePane;
  const keepScroll = pane?.scrollLeft ?? 0;
  if (orgEmployees.length) {
    renderAdminVacationsList();
    requestAnimationFrame(() => {
      if (!pane) return;
      pane.scrollLeft = Math.max(0, keepScroll - days * ADMIN_DAY_WIDTH);
    });
  }
}

function onAdminTimelinePaneScroll() {
  const pane = els.timelinePane;
  if (!pane || adminScrollExtending || Date.now() < adminZoomLockUntil) return;
  if (adminTimelineStart === null || adminPendingScrollVacationId != null) return;
  const edgePx = ADMIN_DAY_WIDTH * 4;
  const maxScroll = Math.max(0, pane.scrollWidth - pane.clientWidth);
  if (maxScroll <= edgePx * 2) return;
  let delta = 0;
  if (pane.scrollLeft <= edgePx) delta = -ADMIN_TIMELINE_SCROLL_DAYS;
  else if (pane.scrollLeft >= maxScroll - edgePx) delta = ADMIN_TIMELINE_SCROLL_DAYS;
  if (!delta) return;
  adminScrollExtending = true;
  shiftAdminTimeline(delta);
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      adminScrollExtending = false;
    });
  });
}

function renderAdminVacationsList() {
  if (!els.timelineDays || !els.timelineRows || !els.timelineNameRows) return;

  ADMIN_DAY_WIDTH = resolveAdminDayWidth();
  if (els.timelineViewport) {
    els.timelineViewport.style.setProperty("--day-width", `${ADMIN_DAY_WIDTH}px`);
  }

  const hasFilters = adminVacationFiltersActive();
  if (!hasFilters) {
    // Empty filters → full org schedule, not a leftover zoomed/filtered window.
    adminAutoZoomKey = "";
    adminFocusedVacationId = null;
    adminPendingScrollVacationId = null;
    if (adminTimelineWindowOffset !== 0 && !adminSkipScrollToTodayOnce) {
      adminTimelineWindowOffset = 0;
    }
  }

  const list = filteredVacations();

  const vacationsByEmployee = new Map();
  list.forEach((vacation) => {
    const rows = vacationsByEmployee.get(vacation.employee_id) || [];
    rows.push(vacation);
    vacationsByEmployee.set(vacation.employee_id, rows);
  });

  // With no filters, always show every employee. With filters, only matching rows.
  let employees = [...orgEmployees].sort((a, b) =>
    a.username.localeCompare(b.username, undefined, { sensitivity: "base" })
  );
  if (hasFilters) {
    employees = employees.filter((employee) => vacationsByEmployee.has(employee.id));
  }

  if (!employees.length) {
    els.timelineNameRows.innerHTML = "";
    els.timelineDays.innerHTML = "";
    els.timelineRows.innerHTML = "";
    if (els.timelineRangeLabel) els.timelineRangeLabel.textContent = "";
    els.vacationsEmpty?.classList.remove("hidden");
    els.vacationsEmpty.textContent = hasFilters
      ? "No vacation requests match this filter."
      : "No employees in this organization yet.";
    return;
  }

  els.vacationsEmpty?.classList.add("hidden");

  adminTimelineStart = adminTimelineOriginDate();
  const today = adminStartOfToday();
  const totalWidth = ADMIN_TIMELINE_DAY_COUNT * ADMIN_DAY_WIDTH;
  const todayOffset =
    ui.dayDiff(adminTimelineStart, today) * ADMIN_DAY_WIDTH + ADMIN_DAY_WIDTH / 2;

  els.timelineDays.style.width = `${totalWidth}px`;
  els.timelineDays.innerHTML = "";
  for (let i = 0; i < ADMIN_TIMELINE_DAY_COUNT; i += 1) {
    const date = adminDateAtOffset(adminTimelineStart, i);
    const parts = ui.formatTimelineDay(date);
    const isWeekend = date.getDay() === 0 || date.getDay() === 6;
    const isToday = ui.dayDiff(date, today) === 0;
    const showMonth = i === 0 || date.getDate() === 1;
    const cell = document.createElement("div");
    cell.className = [
      "timeline-day",
      isWeekend ? "is-weekend" : "",
      isToday ? "is-today" : "",
    ]
      .filter(Boolean)
      .join(" ");
    cell.innerHTML = `
      <span class="dow">${parts.dow}</span>
      <span class="dom">${parts.dom}</span>
      ${showMonth ? `<span class="muted" style="font-size:0.5rem">${parts.month}</span>` : ""}`;
    cell.title = date.toLocaleDateString("en-GB", {
      weekday: "long",
      day: "numeric",
      month: "long",
      year: "numeric",
    });
    els.timelineDays.appendChild(cell);
  }

  els.timelineNameRows.innerHTML = "";
  els.timelineRows.innerHTML = "";
  els.timelineRows.style.width = `${totalWidth}px`;

  const todayMarker = document.createElement("div");
  todayMarker.className = "timeline-today-marker";
  todayMarker.style.left = `${todayOffset}px`;
  els.timelineRows.appendChild(todayMarker);

  employees.forEach((employee) => {
    const nameRow = document.createElement("div");
    nameRow.className = "timeline-name-row";
    nameRow.dataset.employeeId = String(employee.id);
    nameRow.innerHTML = `<strong>${ui.escapeHtml(employee.username)}</strong>`;
    nameRow.title = employee.email || employee.username;
    els.timelineNameRows.appendChild(nameRow);

    const track = document.createElement("div");
    track.className = "timeline-track";
    track.dataset.employeeTrack = String(employee.id);
    track.style.width = `${totalWidth}px`;

    const employeeVacations = vacationsByEmployee.get(employee.id) || [];
    employeeVacations.forEach((vacation) => {
      const start = ui.parseAppDate(vacation.start);
      const end = ui.parseAppDate(vacation.end);
      const startIndex = ui.dayDiff(adminTimelineStart, start);
      const endIndex = ui.dayDiff(adminTimelineStart, end);
      const lastVisible = ADMIN_TIMELINE_DAY_COUNT - 1;
      if (endIndex < 0 || startIndex > lastVisible) return;

      const visibleStart = Math.max(0, startIndex);
      const visibleEnd = Math.min(lastVisible, endIndex);
      const spanDays = visibleEnd - visibleStart + 1;
      const left = visibleStart * ADMIN_DAY_WIDTH + 1;
      const width = spanDays * ADMIN_DAY_WIDTH - 2;

      const bar = document.createElement("div");
      bar.className = `timeline-bar ${ui.statusClass(vacation.status)}${
        adminFocusedVacationId === vacation.id ? " is-focused" : ""
      }`;
      bar.style.left = `${left}px`;
      bar.style.width = `${Math.max(width, 22)}px`;
      bar.dataset.openVacation = String(vacation.id);
      bar.setAttribute("role", "button");
      bar.tabIndex = 0;
      const statusLabel = ui.formatStatus(vacation.status);
      const label =
        width >= 56
          ? `${ui.formatAppDate(vacation.start)} – ${ui.formatAppDate(vacation.end)}`
          : width >= 36
            ? statusLabel
            : "";
      const avatar = document.createElement("span");
      avatar.className = `timeline-bar-avatar${employee.avatar_url ? " has-photo" : ""}`;
      avatar.setAttribute("aria-hidden", "true");
      if (employee.avatar_url) {
        const img = document.createElement("img");
        img.src = employee.avatar_url;
        img.alt = "";
        avatar.appendChild(img);
      } else {
        const parts = String(employee.username || "")
          .trim()
          .split(/\s+/)
          .filter(Boolean);
        avatar.textContent = parts.length
          ? parts.length === 1
            ? parts[0].slice(0, 2).toUpperCase()
            : `${parts[0][0]}${parts[1][0]}`.toUpperCase()
          : "?";
      }
      bar.appendChild(avatar);
      if (label) {
        const labelEl = document.createElement("span");
        labelEl.className = "timeline-bar-label";
        labelEl.textContent = label;
        bar.appendChild(labelEl);
      }
      bar.title = `${employee.username}: ${ui.formatAppDate(vacation.start)} → ${ui.formatAppDate(
        vacation.end
      )} (${statusLabel}) — click for details`;
      bar.addEventListener("click", (event) => {
        event.stopPropagation();
        openAdminVacationDetail(vacation.id);
      });
      bar.addEventListener("keydown", (event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          openAdminVacationDetail(vacation.id);
        }
      });
      track.appendChild(bar);
    });

    els.timelineRows.appendChild(track);
  });

  if (els.timelineRangeLabel) {
    const rangeStart = adminTimelineStart;
    const rangeEnd = adminDateAtOffset(adminTimelineStart, ADMIN_TIMELINE_DAY_COUNT - 1);
    els.timelineRangeLabel.textContent = `${rangeStart.toLocaleDateString("en-GB", {
      day: "numeric",
      month: "short",
    })} – ${rangeEnd.toLocaleDateString("en-GB", {
      day: "numeric",
      month: "short",
      year: "numeric",
    })}`;
  }

  requestAnimationFrame(() => {
    if (adminPendingScrollVacationId != null) {
      const vacationId = adminPendingScrollVacationId;
      const vacation = vacationsById.get(vacationId);
      adminPendingScrollVacationId = null;
      adminSkipScrollToTodayOnce = false;
      if (vacation) {
        requestAnimationFrame(() => {
          requestAnimationFrame(() => applyAdminVacationCenterScroll(vacation));
        });
        return;
      }
    }
    if (adminSkipScrollToTodayOnce) {
      adminSkipScrollToTodayOnce = false;
      return;
    }
    if (maybeAutoZoomToFilteredVacation(list)) return;
    scrollAdminTimelineToToday(false);
  });
}

async function refreshAdminVacations({ resetFilters = false } = {}) {
  if (resetFilters) clearAdminVacationFilters({ resetWindow: true });
  try {
    allVacations = await ui.api("/vacations?role=user");
    vacationsById = new Map(allVacations.map((v) => [v.id, v]));
    renderAdminVacationsList();
  } catch (err) {
    allVacations = [];
    vacationsById = new Map();
    if (els.timelineNameRows) els.timelineNameRows.innerHTML = "";
    if (els.timelineDays) els.timelineDays.innerHTML = "";
    if (els.timelineRows) els.timelineRows.innerHTML = "";
    if (els.vacationsEmpty) {
      els.vacationsEmpty.classList.remove("hidden");
      els.vacationsEmpty.textContent = err.message;
    }
  }
}

function showVacationDetailTab(name) {
  els.vacationTabs?.forEach((tab) => {
    tab.classList.toggle("active", tab.dataset.detailTab === name);
  });
  els.vacationPanels?.forEach((panel) => {
    panel.classList.toggle("hidden", panel.dataset.detailPanel !== name);
  });
}

function closeVacationModal() {
  if (!els.vacationModal) return;
  els.vacationModal.classList.add("hidden");
  els.vacationModal.setAttribute("aria-hidden", "true");
  activeVacationId = null;
}

function renderGroupApprovalList(vacation) {
  const votes = Array.isArray(vacation.group_approvals) ? vacation.group_approvals : [];
  if (!votes.length) return "";
  const rows = votes
    .map((vote) => {
      const statusLabel =
        vote.status === "approved"
          ? "Approved"
          : vote.status === "denied"
            ? "Denied"
            : "Pending";
      const by = vote.decided_by ? ` · ${ui.escapeHtml(employeeLabel(vote.decided_by))}` : "";
      return `<li><strong>${ui.escapeHtml(vote.group_name || `Group #${vote.group_id}`)}</strong>
        <span class="muted"> — ${statusLabel}${by}</span></li>`;
    })
    .join("");
  return `<div class="group-approval-list">
      <p><strong>Group votes</strong></p>
      <ul>${rows}</ul>
    </div>`;
}

function renderAdminVacationOverview(vacation) {
  if (!els.vacationOverview) return;
  const status = String(vacation.status || "");
  const canDecideAwaiting = status.includes("Awaiting");
  const canDecideCancellation = status.includes("CancellationRequested");
  const decideButtons = canDecideAwaiting
    ? `<button type="button" class="btn-primary btn-small" data-admin-approve="${vacation.id}">Approve</button>
       <button type="button" class="btn-danger btn-small" data-admin-deny="${vacation.id}">Deny</button>`
    : canDecideCancellation
      ? `<button type="button" class="btn-primary btn-small" data-admin-approve="${vacation.id}">Approve cancellation</button>
         <button type="button" class="btn-secondary btn-small" data-admin-deny="${vacation.id}">Deny cancellation</button>`
      : "";

  els.vacationOverview.innerHTML = `
    <div class="vacation-overview-head">
      <div>
        <p><strong>Status:</strong> <span class="status-pill ${ui.statusClass(vacation.status)}">${ui.escapeHtml(
          ui.formatStatus(vacation.status)
        )}</span></p>
        <p class="vacation-card-meta">${decisionMeta(vacation) || "—"}</p>
      </div>
      <div class="btn-row">
        ${decideButtons}
        <button type="button" class="btn-secondary btn-small" data-goto-edit>Edit</button>
        <button type="button" class="btn-danger btn-small" data-delete-vacation="${vacation.id}">Delete</button>
      </div>
    </div>
    <p><strong>Employee:</strong> ${ui.escapeHtml(employeeLabel(vacation.employee_id))}</p>
    <p><strong>Dates:</strong> ${ui.formatAppDate(vacation.start)} → ${ui.formatAppDate(vacation.end)}</p>
    <p><strong>Details:</strong> ${
      vacation.details ? ui.escapeHtml(vacation.details) : "<span class='muted'>None</span>"
    }</p>
    ${
      vacation.denial_reason
        ? `<p><strong>Denial reason:</strong> ${ui.escapeHtml(vacation.denial_reason)}</p>`
        : ""
    }
    ${renderGroupApprovalList(vacation)}`;

  els.vacationOverview.querySelector("[data-goto-edit]")?.addEventListener("click", () => {
    showVacationDetailTab("details");
  });
  els.vacationOverview.querySelector("[data-delete-vacation]")?.addEventListener("click", () => {
    deleteAdminVacation(vacation.id);
  });
  els.vacationOverview.querySelector("[data-admin-approve]")?.addEventListener("click", () => {
    decideAdminVacation(vacation.id, "approve");
  });
  els.vacationOverview.querySelector("[data-admin-deny]")?.addEventListener("click", () => {
    decideAdminVacation(vacation.id, "deny");
  });
}

function renderAdminVacationEditor(vacation) {
  if (!els.vacationDetails) return;
  const startIso = ui.toIsoDate(ui.parseAppDate(vacation.start));
  const endIso = ui.toIsoDate(ui.parseAppDate(vacation.end));
  els.vacationDetails.innerHTML = `
    <form id="admin-vacation-edit-form">
      <div class="field">
        <label for="admin-edit-vacation-start">Start date</label>
        <input id="admin-edit-vacation-start" name="start" type="date" required value="${startIso}">
      </div>
      <div class="field">
        <label for="admin-edit-vacation-end">End date</label>
        <input id="admin-edit-vacation-end" name="end" type="date" required value="${endIso}">
      </div>
      <p id="admin-edit-vacation-hint" class="field-error hidden"></p>
      <div class="field">
        <label for="admin-edit-vacation-details">Details</label>
        <textarea id="admin-edit-vacation-details" name="details" rows="3">${ui.escapeHtml(
          vacation.details || ""
        )}</textarea>
      </div>
      <div class="btn-row">
        <button type="submit" class="btn-primary">Save changes</button>
        <button type="button" class="btn-danger" data-delete-vacation="${vacation.id}">Delete</button>
      </div>
    </form>`;

  ui.bindVacationDateRange(
    "admin-edit-vacation-start",
    "admin-edit-vacation-end",
    "admin-edit-vacation-hint"
  );

  document.getElementById("admin-vacation-edit-form")?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    const startIsoValue = String(form.get("start") || "");
    const endIsoValue = String(form.get("end") || "");
    const hint = document.getElementById("admin-edit-vacation-hint");
    try {
      ui.assertVacationRange(startIsoValue, endIsoValue);
      if (hint) hint.classList.add("hidden");
    } catch (err) {
      if (hint) {
        hint.textContent = err.message;
        hint.classList.remove("hidden");
      }
      ui.toast(err.message, true);
      return;
    }
    const start = ui.dateToParts(startIsoValue);
    const end = ui.dateToParts(endIsoValue);
    try {
      const updated = await ui.api(`/vacations/${vacation.id}?role=user`, {
        method: "PUT",
        body: JSON.stringify({
          start_year: start.year,
          start_month: start.month,
          start_day: start.day,
          final_year: end.year,
          final_month: end.month,
          final_day: end.day,
          details: form.get("details") || "",
        }),
      });
      vacationsById.set(updated.id, updated);
      const idx = allVacations.findIndex((item) => item.id === updated.id);
      if (idx >= 0) allVacations[idx] = updated;
      else allVacations.push(updated);
      ui.toast("Vacation updated");
      renderAdminVacationsList();
      openAdminVacationDetail(updated.id);
      showVacationDetailTab("overview");
    } catch (err) {
      ui.toast(err.message, true);
    }
  });

  els.vacationDetails.querySelector("[data-delete-vacation]")?.addEventListener("click", () => {
    deleteAdminVacation(vacation.id);
  });
}

function eventTypeLabel(type) {
  switch (type) {
    case "vacation_request":
      return "Requested";
    case "vacation_approval":
      return "Approved";
    case "vacation_denial":
      return "Denied";
    case "vacation_group_vote":
      return "Group vote";
    case "vacation_modified":
      return "Edited";
    case "vacation_deleted":
      return "Deleted";
    case "vacation_cancellation_requested":
      return "Cancellation requested";
    case "vacation_cancellation_approved":
      return "Cancellation accepted";
    case "vacation_cancellation_denied":
      return "Cancellation denied";
    default:
      return type;
  }
}

function renderAdminVacationActivity(events) {
  if (!els.vacationActivity) return;
  if (!events.length) {
    els.vacationActivity.innerHTML = `<p class="muted">No activity yet.</p>`;
    return;
  }
  els.vacationActivity.innerHTML = `<div class="event-list">${events
    .map((event) => {
      const actor = event.actor_id
        ? employeeLabel(event.actor_id)
        : event.payload?.includes("admin_user_id=")
          ? `Admin #${event.payload.split("admin_user_id=")[1]}`
          : "System";
      return `<article class="event-item">
        <div class="vacation-card-top">
          <strong>${ui.escapeHtml(eventTypeLabel(event.event_type))}</strong>
          <span class="muted">${ui.escapeHtml(event.created_at || "")}</span>
        </div>
        <p class="vacation-card-meta">${ui.escapeHtml(actor)}${
          event.payload ? ` · ${ui.escapeHtml(event.payload)}` : ""
        }</p>
      </article>`;
    })
    .join("")}</div>`;
}

async function openAdminVacationDetail(vacationId) {
  const vacation = vacationsById.get(vacationId);
  if (!vacation || !els.vacationModal) return;
  activeVacationId = vacationId;
  setAdminFocusedVacation(vacationId);
  els.vacationTitle.textContent = `Request #${vacation.id}`;
  els.vacationSubtitle.textContent = `${employeeLabel(vacation.employee_id)} · ${ui.formatAppDate(
    vacation.start
  )} → ${ui.formatAppDate(vacation.end)}`;
  renderAdminVacationOverview(vacation);
  renderAdminVacationEditor(vacation);
  els.vacationActivity.innerHTML = `<p class="muted">Loading activity…</p>`;
  showVacationDetailTab("overview");
  els.vacationModal.classList.remove("hidden");
  els.vacationModal.setAttribute("aria-hidden", "false");
  try {
    const events = await ui.api(`/vacations/${vacationId}/events?role=user`);
    renderAdminVacationActivity(events);
  } catch (err) {
    els.vacationActivity.innerHTML = `<p class="field-error">${ui.escapeHtml(err.message)}</p>`;
  }
}

async function deleteAdminVacation(vacationId) {
  const vacation = vacationsById.get(vacationId);
  if (!vacation) return;
  if (
    !window.confirm(
      `Delete vacation #${vacation.id} for ${employeeLabel(vacation.employee_id)}?`
    )
  ) {
    return;
  }
  try {
    await ui.api(`/vacations/${vacationId}?role=user`, { method: "DELETE" });
    allVacations = allVacations.filter((item) => item.id !== vacationId);
    vacationsById.delete(vacationId);
    ui.toast("Vacation deleted");
    closeVacationModal();
    renderAdminVacationsList();
  } catch (err) {
    ui.toast(err.message, true);
  }
}

async function decideAdminVacation(vacationId, action) {
  const vacation = vacationsById.get(vacationId);
  if (!vacation) return;
  const isCancel = String(vacation.status || "").includes("CancellationRequested");
  const label = isCancel
    ? action === "approve"
      ? "approve this cancellation"
      : "deny this cancellation"
    : action === "approve"
      ? "approve this vacation"
      : "deny this vacation";
  if (!window.confirm(`${label[0].toUpperCase()}${label.slice(1)}?`)) return;
  try {
    let body = "{}";
    if (action === "deny" && !isCancel) {
      const reason = window.prompt("Reason for denial (optional):", "");
      if (reason === null) return;
      body = JSON.stringify({ reason: String(reason).trim() });
    }
    const path =
      action === "approve"
        ? `/vacations/${vacationId}/approve?role=user`
        : `/vacations/${vacationId}/deny?role=user`;
    const updated = await ui.api(path, { method: "POST", body });
    vacationsById.set(updated.id, updated);
    const idx = allVacations.findIndex((item) => item.id === updated.id);
    if (idx >= 0) allVacations[idx] = updated;
    ui.toast(
      isCancel
        ? action === "approve"
          ? "Cancellation approved"
          : "Cancellation denied"
        : action === "approve"
          ? "Vacation approved"
          : "Vacation denied"
    );
    renderAdminVacationOverview(updated);
    renderAdminVacationEditor(updated);
    renderAdminVacationsList();
    try {
      const events = await ui.api(`/vacations/${vacationId}/events?role=user`);
      renderAdminVacationActivity(events);
    } catch (_) {
      /* keep previous activity panel */
    }
  } catch (err) {
    ui.toast(err.message, true);
  }
}

async function loadEmployeeDetail(employeeId) {
  return ui.api(`/employees/${employeeId}/detail`);
}

function renderOrganizationPanel() {
  if (!els.orgDetailsForm) return;

  if (currentOrg) {
    els.orgName.value = currentOrg.name || "";
    const groupCount = orgGroups.length;
    const employeeCount = orgEmployees.length;
    els.orgDetailsMeta.textContent = `${employeeCount} employee${employeeCount === 1 ? "" : "s"} · ${groupCount} group${groupCount === 1 ? "" : "s"} · owner #${currentOrg.owner_id}`;
  } else {
    els.orgName.value = "";
    els.orgDetailsMeta.textContent = "No organization found.";
  }

  renderOrgGroupsList();
  renderOrgEmployeesList();
}

function renderOrgGroupsList() {
  if (!els.orgGroupsList) return;
  const query = (els.orgGroupsSearch?.value || "").trim().toLowerCase();
  const groups = orgGroups.filter((group) => {
    if (!query) return true;
    return `${group.name} #${group.id}`.toLowerCase().includes(query);
  });

  if (!groups.length) {
    els.orgGroupsList.innerHTML = `<p class="muted org-scroll-empty">${
      orgGroups.length ? "No groups match this search." : "No groups yet. Add one to get started."
    }</p>`;
    return;
  }

  els.orgGroupsList.innerHTML = groups
    .map((group) => {
      const memberCount = (group.member_ids || []).length;
      return `
        <div class="groups-editor-row" data-group-id="${group.id}">
          <div>
            <strong>${ui.escapeHtml(group.name)}</strong>
            <span class="muted">#${group.id} · ${memberCount} member${memberCount === 1 ? "" : "s"}</span>
          </div>
          <button type="button" class="icon-btn" data-org-group-settings="${group.id}" title="Group settings" aria-label="Group settings">
            ${GEAR_ICON}
          </button>
        </div>`;
    })
    .join("");

  els.orgGroupsList.querySelectorAll("[data-org-group-settings]").forEach((btn) => {
    btn.addEventListener("click", () => openGroupSettingsModal(Number(btn.dataset.orgGroupSettings)));
  });
}

function renderOrgEmployeesList() {
  if (!els.orgEmployeesList) return;
  const query = (els.orgEmployeesSearch?.value || "").trim().toLowerCase();
  const employees = orgEmployees.filter((employee) => {
    if (!query) return true;
    return `${employee.username} ${employee.email} #${employee.id}`.toLowerCase().includes(query);
  });

  if (!employees.length) {
    els.orgEmployeesList.innerHTML = `<p class="muted org-scroll-empty">${
      orgEmployees.length ? "No employees match this search." : "No employees yet."
    }</p>`;
    return;
  }

  els.orgEmployeesList.innerHTML = employees
    .map((employee) => {
      const detail = employeeSummaries[employee.id];
      const groups =
        detail?.profile.group_ids
          .map((id) => orgGroups.find((g) => g.id === id)?.name || `#${id}`)
          .join(", ") || "No groups";
      return `
        <div class="groups-editor-row" data-employee-id="${employee.id}">
          <div>
            <strong>${ui.escapeHtml(employee.username)}</strong>
            <span class="muted">${ui.escapeHtml(employee.email)} · #${employee.id} · ${ui.escapeHtml(groups)}</span>
          </div>
          <button type="button" class="icon-btn" data-org-employee-settings="${employee.id}" title="Employee settings" aria-label="Employee settings">
            ${GEAR_ICON}
          </button>
        </div>`;
    })
    .join("");

  els.orgEmployeesList.querySelectorAll("[data-org-employee-settings]").forEach((btn) => {
    btn.addEventListener("click", () => openEmployeeSettingsHub(Number(btn.dataset.orgEmployeeSettings)));
  });
}

async function openEmployeeSettingsHub(employeeId) {
  activeEmployeeId = employeeId;
  const detail = employeeSummaries[employeeId] || (await loadEmployeeDetail(employeeId));
  employeeSummaries[employeeId] = detail;
  const profile = detail.profile;

  els.modal.classList.remove("hidden");
  els.modal.setAttribute("aria-hidden", "false");
  els.modalTitle.textContent = profile.username;
  els.modalSubtitle.textContent = `${profile.email} · id ${profile.id}`;
  els.modalBody.innerHTML = `
    <p class="muted">Choose what to edit for this employee.</p>
    <div class="btn-row" style="flex-direction:column;align-items:stretch">
      <button type="button" class="btn-secondary" data-hub-action="account">Edit account</button>
      <button type="button" class="btn-secondary" data-hub-action="permissions">Edit permissions</button>
      <button type="button" class="btn-secondary" data-hub-action="groups">Assign groups</button>
      <button type="button" class="btn-secondary" data-hub-action="group-permissions">Group permissions</button>
      <button type="button" class="btn-secondary" data-hub-action="stats">View vacations</button>
      <button type="button" class="btn-danger" data-hub-action="delete">Delete employee</button>
    </div>`;

  els.modalBody.querySelectorAll("[data-hub-action]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      await openEmployeeModal(employeeId, btn.dataset.hubAction);
    });
  });
}

function closeCreateGroupForm() {
  els.createGroupCard?.classList.add("hidden");
  els.orgCreateGroupForm?.reset();
}

function employeeMatchesQuery(employee, query) {
  if (!query) return true;
  const hay = `${employee.username} ${employee.email} #${employee.id}`.toLowerCase();
  return hay.includes(query);
}

function captureGroupEditorFormState() {
  if (!groupEditorDraft) return;
  const nameInput = document.getElementById("org-group-name");
  if (nameInput) groupEditorDraft.name = nameInput.value;
  const searchInput = document.getElementById("group-member-search");
  if (searchInput) groupEditorDraft.memberSearch = searchInput.value;
}

function renderGroupMemberListRows(groupId) {
  if (!groupEditorDraft) return "<p class='muted org-scroll-empty'>No group selected.</p>";
  const query = String(groupEditorDraft.memberSearch || "")
    .trim()
    .toLowerCase();
  const selectedIds = groupEditorDraft.selectedIds;
  const addMode = Boolean(groupEditorDraft.addMode);

  const pool = addMode
    ? orgEmployees.filter((employee) => !selectedIds.has(employee.id))
    : orgEmployees.filter((employee) => selectedIds.has(employee.id));
  const rows = pool.filter((employee) => employeeMatchesQuery(employee, query));

  if (!rows.length) {
    if (addMode) {
      return `<p class="muted org-scroll-empty">${
        pool.length ? "No employees match this search." : "Everyone is already in this group."
      }</p>`;
    }
    return `<p class="muted org-scroll-empty">${
      selectedIds.size
        ? "No members match this search."
        : "No members yet. Use + to find people to add."
    }</p>`;
  }

  return rows
    .map((employee) => {
      if (addMode) {
        return `
          <div class="groups-editor-row" data-candidate-id="${employee.id}">
            <div>
              <strong>${ui.escapeHtml(employee.username)}</strong>
              <span class="muted">${ui.escapeHtml(employee.email)} · #${employee.id}</span>
            </div>
            <button type="button" class="btn-primary btn-small" data-add-member="${employee.id}">Add</button>
          </div>`;
      }
      return `
        <div class="groups-editor-row" data-member-id="${employee.id}">
          <div>
            <strong>${ui.escapeHtml(employee.username)}</strong>
            <span class="muted">${ui.escapeHtml(employee.email)} · #${employee.id}</span>
          </div>
          <div class="row-actions">
            <button type="button" class="icon-btn" data-member-permissions="${employee.id}" title="Group permissions" aria-label="Group permissions">
              ${GEAR_ICON}
            </button>
            <button type="button" class="icon-btn" data-remove-member="${employee.id}" title="Remove from group" aria-label="Remove from group">
              ${REMOVE_ICON}
            </button>
          </div>
        </div>`;
    })
    .join("");
}

function bindGroupMemberListControls(groupId) {
  const list = els.modalBody.querySelector("[data-member-list]");
  if (!list) return;

  list.querySelectorAll("[data-member-permissions]").forEach((btn) => {
    btn.addEventListener("click", () => {
      captureGroupEditorFormState();
      openAdminMemberPermissions(groupId, Number(btn.dataset.memberPermissions), {
        backTo: "group",
        staged: true,
      });
    });
  });

  list.querySelectorAll("[data-remove-member]").forEach((btn) => {
    btn.addEventListener("click", () => {
      if (!groupEditorDraft) return;
      const employeeId = Number(btn.dataset.removeMember);
      groupEditorDraft.selectedIds.delete(employeeId);
      refreshGroupMemberList(groupId);
    });
  });

  list.querySelectorAll("[data-add-member]").forEach((btn) => {
    btn.addEventListener("click", () => {
      if (!groupEditorDraft) return;
      const employeeId = Number(btn.dataset.addMember);
      groupEditorDraft.selectedIds.add(employeeId);
      refreshGroupMemberList(groupId);
    });
  });
}

function refreshGroupMemberList(groupId) {
  const list = els.modalBody.querySelector("[data-member-list]");
  const searchInput = document.getElementById("group-member-search");
  const addToggle = document.getElementById("group-member-add-toggle");
  if (!list || !groupEditorDraft) return;

  if (searchInput) groupEditorDraft.memberSearch = searchInput.value;
  list.innerHTML = renderGroupMemberListRows(groupId);
  bindGroupMemberListControls(groupId);

  if (searchInput) {
    searchInput.placeholder = groupEditorDraft.addMode
      ? "Search people to add…"
      : "Filter members…";
  }
  if (addToggle) {
    addToggle.classList.toggle("is-active", Boolean(groupEditorDraft.addMode));
    addToggle.setAttribute("aria-pressed", groupEditorDraft.addMode ? "true" : "false");
    addToggle.title = groupEditorDraft.addMode ? "Back to members" : "Add people";
  }
}

function openGroupSettingsModal(groupId) {
  const group = orgGroups.find((item) => item.id === groupId);
  if (!group) {
    ui.toast("Group not found", true);
    return;
  }

  activeGroupId = groupId;
  const keepDraft =
    groupEditorDraft &&
    groupEditorDraft.groupId === groupId &&
    groupEditorDraft.pendingPerms instanceof Map;

  if (!keepDraft) {
    groupEditorDraft = {
      groupId,
      selectedIds: new Set(group.member_ids || []),
      pendingPerms: new Map(),
      name: group.name,
      memberSearch: "",
      addMode: false,
    };
  } else {
    groupEditorDraft.memberSearch = groupEditorDraft.memberSearch || "";
    groupEditorDraft.addMode = Boolean(groupEditorDraft.addMode);
  }

  els.modal.classList.remove("hidden");
  els.modal.setAttribute("aria-hidden", "false");
  els.modalTitle.textContent = group.name;
  els.modalSubtitle.textContent = `Group #${group.id}`;
  els.modalBody.innerHTML = `
    <form id="org-group-form">
      <div class="field">
        <label for="org-group-name">Group name</label>
        <input id="org-group-name" name="name" required maxlength="120" value="${ui.escapeHtml(
          groupEditorDraft.name || group.name
        )}">
      </div>
      <div class="field">
        <label for="group-member-search">Members</label>
        <p class="muted" style="margin:0 0 0.5rem">Permission changes and membership edits are saved with the group.</p>
        <div class="member-search-bar">
          <input id="group-member-search" type="search" placeholder="Filter members…" autocomplete="off" value="${ui.escapeHtml(
            groupEditorDraft.memberSearch || ""
          )}">
          <button type="button" class="icon-btn${
            groupEditorDraft.addMode ? " is-active" : ""
          }" id="group-member-add-toggle" title="${
            groupEditorDraft.addMode ? "Back to members" : "Add people"
          }" aria-label="Toggle add people" aria-pressed="${
            groupEditorDraft.addMode ? "true" : "false"
          }">${PLUS_ICON}</button>
        </div>
        <div class="org-scroll-list" data-member-list>${renderGroupMemberListRows(groupId)}</div>
      </div>
      <div class="btn-row">
        <button type="submit" class="btn-primary">Save group</button>
        <button type="button" id="org-delete-group" class="btn-danger">Delete</button>
      </div>
    </form>`;

  bindGroupMemberListControls(groupId);

  document.getElementById("group-member-search")?.addEventListener("input", () => {
    refreshGroupMemberList(groupId);
  });

  document.getElementById("group-member-add-toggle")?.addEventListener("click", () => {
    if (!groupEditorDraft) return;
    groupEditorDraft.addMode = !groupEditorDraft.addMode;
    groupEditorDraft.memberSearch = "";
    const searchInput = document.getElementById("group-member-search");
    if (searchInput) searchInput.value = "";
    refreshGroupMemberList(groupId);
    searchInput?.focus();
  });

  document.getElementById("org-group-form").addEventListener("submit", async (event) => {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    const name = String(form.get("name") || "").trim();
    captureGroupEditorFormState();
    const selected = groupEditorDraft?.selectedIds || new Set();
    const pendingPerms = groupEditorDraft?.pendingPerms || new Map();
    try {
      await ui.api(`/groups/${groupId}?role=user`, {
        method: "PUT",
        body: JSON.stringify({ name }),
      });

      const currentMembersIds = new Set(group.member_ids || []);
      for (const employeeId of selected) {
        if (!currentMembersIds.has(employeeId)) {
          await ui.api(`/groups/${groupId}/members?role=user`, {
            method: "POST",
            body: JSON.stringify({ employee_id: employeeId }),
          });
        }
      }
      for (const employeeId of currentMembersIds) {
        if (!selected.has(employeeId)) {
          await ui.api(`/groups/${groupId}/members/${employeeId}?role=user`, {
            method: "DELETE",
          });
        }
      }

      for (const employeeId of selected) {
        if (!pendingPerms.has(employeeId)) continue;
        const perms = pendingPerms.get(employeeId);
        await ui.api(`/groups/${groupId}/members/${employeeId}/permissions?role=user`, {
          method: "PUT",
          body: JSON.stringify(perms),
        });
      }

      ui.toast("Group saved");
      groupEditorDraft = null;
      closeModal();
      await refreshAll();
    } catch (err) {
      ui.toast(err.message, true);
    }
  });

  document.getElementById("org-delete-group").addEventListener("click", async () => {
    if (!window.confirm(`Delete group "${group.name}"?`)) return;
    try {
      await ui.api(`/groups/${groupId}?role=user`, { method: "DELETE" });
      ui.toast("Group deleted");
      groupEditorDraft = null;
      closeModal();
      await refreshAll();
    } catch (err) {
      ui.toast(err.message, true);
    }
  });
}

function renderEmployeeGroupPermissionsModal(profile) {
  const groups = (profile.group_ids || [])
    .map((id) => orgGroups.find((group) => group.id === id))
    .filter(Boolean);

  const rows = groups.length
    ? groups
        .map(
          (group) => `
            <div class="groups-editor-row">
              <div>
                <strong>${ui.escapeHtml(group.name)}</strong>
                <span class="muted">Group #${group.id}</span>
              </div>
              <button type="button" class="icon-btn" data-emp-group-perms="${group.id}" title="Edit group permissions" aria-label="Edit group permissions">
                ${GEAR_ICON}
              </button>
            </div>`
        )
        .join("")
    : "<p class='muted'>This employee is not in any groups yet. Assign groups first.</p>";

  els.modalBody.innerHTML = `
    <p class="muted">Edit this employee's permissions inside each group they belong to.</p>
    <div class="org-scroll-list">${rows}</div>
    <div class="btn-row">
      <button type="button" class="btn-secondary" data-back-hub>Back</button>
    </div>`;

  els.modalBody.querySelectorAll("[data-emp-group-perms]").forEach((btn) => {
    btn.addEventListener("click", () => {
      openAdminMemberPermissions(Number(btn.dataset.empGroupPerms), profile.id, {
        backTo: "employee-groups",
        employeeId: profile.id,
      });
    });
  });

  els.modalBody.querySelector("[data-back-hub]")?.addEventListener("click", () => {
    openEmployeeSettingsHub(profile.id);
  });
}

async function openAdminMemberPermissions(groupId, employeeId, options = {}) {
  const employee =
    orgEmployees.find((item) => item.id === employeeId) || employeesById.get(employeeId);
  const group = orgGroups.find((item) => item.id === groupId);
  if (!employee || !group) {
    ui.toast("Member or group not found", true);
    return;
  }

  const staged = Boolean(options.staged && groupEditorDraft && groupEditorDraft.groupId === groupId);
  let perms = emptyGroupMemberPerms();

  if (staged && groupEditorDraft.pendingPerms.has(employeeId)) {
    perms = ui.mergePermissions(
      ui.GROUP_MEMBER_PERMISSION_LABELS,
      groupEditorDraft.pendingPerms.get(employeeId)
    );
  } else {
    const alreadyMember = (group.member_ids || []).includes(employeeId);
    if (alreadyMember) {
      try {
        perms = ui.mergePermissions(
          ui.GROUP_MEMBER_PERMISSION_LABELS,
          await ui.api(`/groups/${groupId}/members/${employeeId}/permissions?role=user`)
        );
      } catch (err) {
        ui.toast(err.message, true);
        return;
      }
    }
  }

  if (staged && groupEditorDraft) {
    captureGroupEditorFormState();
  }

  els.modal.classList.remove("hidden");
  els.modal.setAttribute("aria-hidden", "false");
  els.modalTitle.textContent = employee.username;
  els.modalSubtitle.textContent = `Group permissions · ${group.name}`;
  els.modalBody.innerHTML = `
    <p class="muted" style="margin:0 0 0.85rem">
      These apply only inside <strong>${ui.escapeHtml(group.name)}</strong>. ${
        staged
          ? "Changes stay on this screen until you save the group."
          : "Global rights are under Edit permissions."
      }
    </p>
    <form id="admin-member-perms-form">
      ${ui.renderPermissionCheckboxes(perms, ui.GROUP_MEMBER_PERMISSION_LABELS)}
      <div class="btn-row">
        <button class="btn-primary" type="submit">${staged ? "Done" : "Save group permissions"}</button>
        <button type="button" class="btn-secondary" data-back-member-perms>Back</button>
      </div>
    </form>`;

  document.getElementById("admin-member-perms-form").addEventListener("submit", async (event) => {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    const nextPerms = ui.readPermissionForm(form, ui.GROUP_MEMBER_PERMISSION_LABELS, perms);

    if (staged) {
      groupEditorDraft.pendingPerms.set(employeeId, nextPerms);
      openGroupSettingsModal(groupId);
      return;
    }

    try {
      await ui.api(`/groups/${groupId}/members/${employeeId}/permissions?role=user`, {
        method: "PUT",
        body: JSON.stringify(nextPerms),
      });
      ui.toast("Member permissions saved");
      if (options.backTo === "employee-groups") {
        const detail =
          employeeSummaries[options.employeeId] || (await loadEmployeeDetail(options.employeeId));
        renderEmployeeGroupPermissionsModal(detail.profile);
      } else {
        openGroupSettingsModal(groupId);
      }
    } catch (err) {
      ui.toast(err.message, true);
    }
  });

  els.modalBody.querySelector("[data-back-member-perms]")?.addEventListener("click", async () => {
    if (options.backTo === "employee-groups") {
      const detail =
        employeeSummaries[options.employeeId] || (await loadEmployeeDetail(options.employeeId));
      renderEmployeeGroupPermissionsModal(detail.profile);
    } else {
      openGroupSettingsModal(groupId);
    }
  });
}

async function refreshAll() {
  const user = await ui.api("/users/me");
  orgId = user.organization_id;
  els.userSummary.innerHTML = `
    <p><strong>${ui.escapeHtml(user.email)}</strong></p>
    <p>Admin ID: ${user.id}</p>
    <p>Organization ID: ${user.organization_id}</p>`;

  const orgs = await ui.api("/organizations");
  currentOrg = orgs.find((o) => o.id === user.organization_id) || null;
  els.orgSummary.innerHTML = currentOrg
    ? `<p><strong>${ui.escapeHtml(currentOrg.name)}</strong></p>
       <p>${currentOrg.employee_ids.length} employee${currentOrg.employee_ids.length === 1 ? "" : "s"}</p>`
    : "No organization found.";

  orgGroups = await ui.api(`/organizations/${orgId}/groups?role=user`);

  const employees = await ui.api("/employees");
  employeesById = new Map(employees.map((employee) => [employee.id, employee]));
  const orgEmployeeIds = currentOrg?.employee_ids || [];
  orgEmployees = employees.filter((e) => orgEmployeeIds.includes(e.id));
  const visibleEmployees = orgEmployees;

  employeeSummaries = {};
  await Promise.all(
    visibleEmployees.map(async (employee) => {
      try {
        employeeSummaries[employee.id] = await loadEmployeeDetail(employee.id);
      } catch {
        employeeSummaries[employee.id] = null;
      }
    })
  );

  renderOrganizationPanel();

  const payment = await ui.api("/users/me/payment");
  els.paymentForm.billing_name.value = payment.billing_name || "";
  els.paymentForm.card_last4.value = payment.card_last4 || "";
  els.paymentForm.plan.value = payment.plan || "Business";

  await refreshEvents();
}

function eventTypeLabel(type) {
  switch (type) {
    case "vacation_request":
      return "Requested";
    case "vacation_approval":
      return "Approved";
    case "vacation_denial":
      return "Denied";
    case "vacation_modified":
      return "Edited";
    case "vacation_deleted":
      return "Deleted";
    case "vacation_cancellation_requested":
      return "Cancellation requested";
    case "vacation_cancellation_approved":
      return "Cancellation accepted";
    case "vacation_cancellation_denied":
      return "Cancellation denied";
    case "employee_added":
      return "Employee created";
    case "employee_modified":
      return "Employee edited";
    case "employee_deleted":
      return "Employee deleted";
    default:
      return type;
  }
}

function actorLabel(event) {
  if (event.payload) {
    const adminMatch = event.payload.match(/admin_user_id=(\d+)/);
    if (adminMatch) return `Admin #${adminMatch[1]}`;
  }
  if (event.actor_id != null) return employeeLabel(event.actor_id);
  return "System";
}

function employeeLabel(id) {
  if (id == null) return "—";
  const employee = employeesById.get(id);
  return employee ? `${employee.username} (#${id})` : `Employee #${id}`;
}

function renderEventsList() {
  const filter = els.eventsFilter?.value || "";
  const events = filter ? allEvents.filter((event) => event.event_type === filter) : allEvents;

  if (!events.length) {
    els.eventsList.innerHTML = "";
    els.eventsEmpty?.classList.remove("hidden");
    els.eventsEmpty.textContent = filter ? "No events match this filter." : "No events recorded yet.";
    return;
  }

  els.eventsEmpty?.classList.add("hidden");
  els.eventsList.innerHTML = events
    .map((event) => {
      const actor = actorLabel(event);
      const subject =
        event.employee_id != null ? employeeLabel(event.employee_id) : "—";
      const vacation =
        event.vacation_id != null ? `Vacation #${event.vacation_id}` : "—";
      const lines = [`Actor: ${ui.escapeHtml(actor)}`];
      if (event.employee_id != null) {
        lines.push(`Employee: ${ui.escapeHtml(subject)}`);
      }
      if (event.vacation_id != null) {
        lines.push(ui.escapeHtml(vacation));
      }
      if (event.payload && !event.payload.startsWith("admin_user_id=")) {
        lines.push(ui.escapeHtml(event.payload));
      }
      return `<article class="event-item">
        <div class="vacation-card-top">
          <strong>${ui.escapeHtml(eventTypeLabel(event.event_type))}</strong>
          <span class="status-pill">${ui.escapeHtml(event.created_at)}</span>
        </div>
        <p class="vacation-card-meta">${lines.join("<br>")}</p>
      </article>`;
    })
    .join("");
}

async function refreshEvents() {
  if (!els.eventsList) return;
  try {
    allEvents = await ui.api("/events");
    renderEventsList();
  } catch (err) {
    els.eventsList.innerHTML = "";
    els.eventsEmpty?.classList.remove("hidden");
    els.eventsEmpty.textContent = err.message;
  }
}

async function openEmployeeModal(employeeId, action) {
  activeEmployeeId = employeeId;
  const detail = employeeSummaries[employeeId] || (await loadEmployeeDetail(employeeId));
  employeeSummaries[employeeId] = detail;
  const profile = detail.profile;

  els.modal.classList.remove("hidden");
  els.modal.setAttribute("aria-hidden", "false");
  els.modalTitle.textContent = profile.username;
  els.modalSubtitle.textContent = `${profile.email} · id ${profile.id}`;

  if (action === "stats") renderStatsModal(detail);
  else if (action === "account") renderAccountModal(profile);
  else if (action === "permissions") renderPermissionsModal(profile);
  else if (action === "groups") renderGroupsModal(profile);
  else if (action === "group-permissions") renderEmployeeGroupPermissionsModal(profile);
  else if (action === "delete") renderDeleteModal(profile);
}

function renderStatsModal(detail) {
  const v = detail.vacations;
  const rows = v.requests.length
    ? v.requests
        .map(
          (req) =>
            `<tr>
              <td>${req.id}</td>
              <td>${req.start} → ${req.end}</td>
              <td><span class="status-pill ${ui.statusClass(req.status)}">${ui.escapeHtml(ui.formatStatus(req.status))}</span></td>
            </tr>`
        )
        .join("")
    : `<tr><td colspan="3" class="muted">No vacation requests yet</td></tr>`;

  els.modalBody.innerHTML = `
    <div class="stat-grid">
      <div class="stat-box"><strong>${v.total}</strong><span>Total</span></div>
      <div class="stat-box"><strong>${v.awaiting}</strong><span>Pending</span></div>
      <div class="stat-box"><strong>${v.approved}</strong><span>Approved</span></div>
      <div class="stat-box"><strong>${v.denied}</strong><span>Denied</span></div>
    </div>
    <table>
      <thead><tr><th>ID</th><th>Dates</th><th>Status</th></tr></thead>
      <tbody>${rows}</tbody>
    </table>`;
}

function renderAccountModal(profile) {
  els.modalBody.innerHTML = `
    <form id="modal-account-form">
      <div class="field">
        <label>Username</label>
        <input name="username" required value="${ui.escapeHtml(profile.username)}">
      </div>
      <div class="field">
        <label>Email</label>
        <input name="email" type="email" required value="${ui.escapeHtml(profile.email)}">
      </div>
      <div class="field">
        <label>New password (optional)</label>
        <input name="password" type="text" placeholder="Leave blank to keep current">
      </div>
      <button type="submit" class="btn-primary">Save account details</button>
    </form>`;

  document.getElementById("modal-account-form").addEventListener("submit", async (event) => {
    event.preventDefault();
    const form = new FormData(event.target);
    try {
      await ui.api(
        `/employees/${activeEmployeeId}`,
        {
          method: "PUT",
          body: JSON.stringify({
            username: form.get("username"),
            email: form.get("email"),
            password: form.get("password") || null,
          }),
        },
      );
      ui.toast("Account updated");
      closeModal();
      await refreshAll();
    } catch (err) {
      ui.toast(err.message, true);
    }
  });
}

function renderPermissionsModal(profile) {
  const p = ui.mergePermissions(ui.EMPLOYEE_PERMISSION_LABELS, profile.permissions);
  els.modalBody.innerHTML = `
    <form id="modal-permissions-form">
      <p class="muted">Global permissions only. Add/edit/remove people and similar rights are assigned per group.</p>
      ${ui.renderPermissionCheckboxes(p, ui.EMPLOYEE_PERMISSION_LABELS)}
      <div class="btn-row"><button type="submit" class="btn-primary">Save permissions</button></div>
    </form>`;

  document.getElementById("modal-permissions-form").addEventListener("submit", async (event) => {
    event.preventDefault();
    const form = new FormData(event.target);
    try {
      await ui.api(
        `/employees/${activeEmployeeId}/permissions?role=user`,
        {
          method: "PUT",
          body: JSON.stringify(ui.readPermissionForm(form, ui.EMPLOYEE_PERMISSION_LABELS, p)),
        },
      );
      ui.toast("Permissions saved");
      closeModal();
      await refreshAll();
    } catch (err) {
      ui.toast(err.message, true);
    }
  });
}

function renderGroupsModal(profile) {
  const checks = orgGroups.length
    ? orgGroups
        .map(
          (group) =>
            `<label><input type="checkbox" name="group_ids" value="${group.id}" ${
              profile.group_ids.includes(group.id) ? "checked" : ""
            }> ${ui.escapeHtml(group.name)}</label>`
        )
        .join("")
    : "<p class='muted'>No groups in this organization yet.</p>";

  els.modalBody.innerHTML = `
    <p class="muted">Select which groups this employee belongs to.</p>
    <div class="group-checklist">${checks}</div>
    <button type="button" id="save-groups-btn" class="btn-primary">Save group assignment</button>`;

  document.getElementById("save-groups-btn").addEventListener("click", async () => {
    const selected = [...els.modalBody.querySelectorAll('input[name="group_ids"]:checked')].map(
      (input) => Number(input.value)
    );
    try {
      await ui.api(
        `/employees/${activeEmployeeId}/groups`,
        {
          method: "PUT",
          body: JSON.stringify({ group_ids: selected }),
        },
      );
      ui.toast("Groups updated");
      closeModal();
      await refreshAll();
    } catch (err) {
      ui.toast(err.message, true);
    }
  });
}

function renderDeleteModal(profile) {
  els.modalBody.innerHTML = `
    <div class="delete-panel">
      <p>Delete <strong>${ui.escapeHtml(profile.username)}</strong> and remove all their vacation records, sessions, and group memberships?</p>
      <div class="btn-row">
        <button type="button" id="confirm-delete-btn" class="btn-danger">Delete employee</button>
        <button type="button" class="btn-secondary" data-close-modal>Cancel</button>
      </div>
    </div>`;

  document.getElementById("confirm-delete-btn").addEventListener("click", async () => {
    try {
      await ui.api(`/employees/${activeEmployeeId}`, { method: "DELETE" });
      ui.toast("Employee deleted");
      closeModal();
      await refreshAll();
    } catch (err) {
      ui.toast(err.message, true);
    }
  });

  els.modalBody.querySelectorAll("[data-close-modal]").forEach((btn) => {
    btn.addEventListener("click", closeModal);
  });
}

els.tabs.forEach((tab) => {
  tab.addEventListener("click", () => {
    ui.showTab(els.tabs, els.tabPanels, tab.dataset.tab);
    if (tab.dataset.tab === "events") refreshEvents();
    if (tab.dataset.tab === "vacations") refreshAdminVacations();
  });
});

els.vacationsSearch?.addEventListener("input", () => {
  if (!adminVacationFiltersActive()) {
    clearAdminVacationFilters({ resetWindow: true });
  }
  renderAdminVacationsList();
});
els.vacationsStatusFilter?.addEventListener("change", () => {
  if (!adminVacationFiltersActive()) {
    clearAdminVacationFilters({ resetWindow: true });
  }
  renderAdminVacationsList();
});
els.refreshVacations?.addEventListener("click", () => {
  refreshAdminVacations({ resetFilters: true }).then(() => ui.toast("Vacations refreshed"));
});
els.timelineToday?.addEventListener("click", () => {
  adminTimelineWindowOffset = 0;
  adminSkipScrollToTodayOnce = false;
  if (orgEmployees.length) renderAdminVacationsList();
  else scrollAdminTimelineToToday(true);
});
els.timelinePrev?.addEventListener("click", () => jumpAdminTimelineVacation(-1));
els.timelineNext?.addEventListener("click", () => jumpAdminTimelineVacation(1));
els.timelinePane?.addEventListener("scroll", onAdminTimelinePaneScroll, { passive: true });

(function bindAdminTimelineVerticalSync() {
  const names = els.timelineNames;
  const pane = els.timelinePane;
  if (!names || !pane) return;
  let syncing = false;
  const sync = (from, to) => {
    if (syncing) return;
    syncing = true;
    to.scrollTop = from.scrollTop;
    syncing = false;
  };
  names.addEventListener("scroll", () => sync(names, pane));
  pane.addEventListener("scroll", () => sync(pane, names));
})();

els.vacationTabs?.forEach((tab) => {
  tab.addEventListener("click", () => showVacationDetailTab(tab.dataset.detailTab));
});
els.vacationClose?.addEventListener("click", closeVacationModal);
document.querySelectorAll("[data-close-vacation-modal]").forEach((el) => {
  el.addEventListener("click", closeVacationModal);
});

els.orgGroupsSearch?.addEventListener("input", renderOrgGroupsList);
els.orgEmployeesSearch?.addEventListener("input", renderOrgEmployeesList);
els.toggleCreateGroup?.addEventListener("click", () => {
  els.createGroupCard?.classList.toggle("hidden");
  if (!els.createGroupCard?.classList.contains("hidden")) {
    document.getElementById("org-new-group-name")?.focus();
  }
});

els.orgCreateGroupForm?.addEventListener("submit", async (event) => {
  event.preventDefault();
  const form = new FormData(event.currentTarget);
  try {
    const group = await ui.api(`/organizations/${orgId}/groups?role=user`, {
      method: "POST",
      body: JSON.stringify({ name: String(form.get("name") || "").trim() }),
    });
    ui.toast("Group created");
    closeCreateGroupForm();
    await refreshAll();
    openGroupSettingsModal(group.id);
  } catch (err) {
    ui.toast(err.message, true);
  }
});

els.orgDetailsForm?.addEventListener("submit", async (event) => {
  event.preventDefault();
  if (!orgId) return;
  const name = String(els.orgName.value || "").trim();
  try {
    currentOrg = await ui.api(`/organizations/${orgId}`, {
      method: "PUT",
      body: JSON.stringify({ name }),
    });
    ui.toast("Organization details saved");
    await refreshAll();
  } catch (err) {
    ui.toast(err.message, true);
  }
});

els.eventsFilter?.addEventListener("change", renderEventsList);
els.refreshEvents?.addEventListener("click", () => {
  refreshEvents().then(() => ui.toast("Events refreshed"));
});

els.loginForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  els.loginError?.classList.add("hidden");
  const form = new FormData(els.loginForm);
  try {
    session = await ui.api("/auth/login", {
      method: "POST",
      body: JSON.stringify({
        email: form.get("email"),
        password: form.get("password"),
        role: "user",
      }),
    });
    updateChrome();
    await refreshAll();
    ui.toast("Signed in as admin");
    ui.showTab(els.tabs, els.tabPanels, "overview");
  } catch (err) {
    showLoginFailed(err.message);
  }
});

document.getElementById("fill-demo")?.addEventListener("click", fillDemoCredentials);
document.getElementById("login-retry")?.addEventListener("click", showLoginForm);
document.getElementById("fill-demo-failed")?.addEventListener("click", () => {
  fillDemoCredentials();
  showLoginForm();
});

els.logoutBtn.addEventListener("click", async () => {
  try {
    await ui.logout(ROLE);
  } catch {
    /* ignore */
  }
  session = null;
  showLoginForm();
  updateChrome();
  ui.toast("Signed out");
});

els.toggleCreate?.addEventListener("click", () => {
  els.createCard?.classList.toggle("hidden");
});

els.employeeForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  const form = new FormData(els.employeeForm);
  try {
    await ui.api(
      "/employees",
      {
        method: "POST",
        body: JSON.stringify({
          username: form.get("username"),
          email: form.get("email"),
          password: form.get("password"),
          organization_id: orgId,
        }),
      },
      session
    );
    els.employeeForm.reset();
    els.employeeForm.password.value = "demo";
    els.createCard.classList.add("hidden");
    ui.toast("Employee created");
    await refreshAll();
  } catch (err) {
    ui.toast(err.message, true);
  }
});

els.paymentForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  const form = new FormData(els.paymentForm);
  try {
    await ui.api(
      "/users/me/payment",
      {
        method: "PUT",
        body: JSON.stringify({
          billing_name: form.get("billing_name"),
          card_last4: form.get("card_last4"),
          plan: form.get("plan"),
        }),
      },
      session
    );
    ui.toast("Billing details saved");
  } catch (err) {
    ui.toast(err.message, true);
  }
});

els.modalClose.addEventListener("click", closeModal);
els.modal.querySelector(".modal-backdrop").addEventListener("click", closeModal);
document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    closeModal();
  }
});

window.addEventListener("resize", () => {
  cancelAnimationFrame(adminResizeFrame);
  adminResizeFrame = requestAnimationFrame(() => {
    const next = resolveAdminDayWidth();
    if (next === ADMIN_DAY_WIDTH) return;
    const pane = els.timelinePane;
    const ratio = pane && pane.scrollWidth > 0 ? pane.scrollLeft / pane.scrollWidth : 0;
    renderAdminVacationsList();
    if (pane && pane.scrollWidth > 0) {
      pane.scrollLeft = ratio * pane.scrollWidth;
    }
  });
});

async function init() {
  session = await ui.getSession(ROLE);
  updateChrome();
  if (session) {
    refreshAll().catch(async () => {
      session = null;
      updateChrome();
    });
  }
}

init();
