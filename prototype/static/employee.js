const ROLE = "employee";
const ui = window.VactionUI;

const GEAR_ICON = `<svg class="icon-gear" viewBox="0 0 24 24" aria-hidden="true">
  <path fill="currentColor" d="M19.14 12.94c.04-.31.06-.63.06-.94s-.02-.63-.06-.94l2.03-1.58a.49.49 0 0 0 .12-.61l-1.92-3.32a.49.49 0 0 0-.59-.22l-2.39.96a7.03 7.03 0 0 0-1.62-.94l-.36-2.54A.48.48 0 0 0 14 2h-4a.48.48 0 0 0-.48.41l-.36 2.54c-.59.24-1.13.55-1.62.94l-2.39-.96a.49.49 0 0 0-.59.22L2.65 8.87a.49.49 0 0 0 .12.61l2.03 1.58c-.04.31-.06.63-.06.94s.02.63.06.94L2.77 14.5a.49.49 0 0 0-.12.61l1.92 3.32c.12.22.37.3.59.22l2.39-.96c.5.39 1.04.71 1.62.94l.36 2.54c.05.24.24.41.48.41h4c.24 0 .44-.17.48-.41l.36-2.54c.59-.24 1.13-.55 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32a.49.49 0 0 0-.12-.61l-2.03-1.58zM12 15.6A3.6 3.6 0 1 1 12 8.4a3.6 3.6 0 0 1 0 7.2z"/>
</svg>`;

const DAY_WIDTH_DESKTOP = 44;
const DAY_WIDTH_TABLET = 40;
const DAY_WIDTH_COMPACT = 36;
const DAY_WIDTH_NARROW = 32;
const TIMELINE_DAYS_BEFORE = 21;
const TIMELINE_DAYS_AFTER = 56;
const TIMELINE_SCROLL_DAYS = 14;

function resolveDayWidth() {
  const width = window.innerWidth || document.documentElement.clientWidth || DAY_WIDTH_DESKTOP;
  if (width <= 480) return DAY_WIDTH_NARROW;
  if (width <= 720) return DAY_WIDTH_COMPACT;
  if (width <= 960) return DAY_WIDTH_TABLET;
  return DAY_WIDTH_DESKTOP;
}

let DAY_WIDTH = resolveDayWidth();
let resizeFrame = 0;

const els = {
  loginPanel: document.getElementById("login-panel"),
  loginFailedPanel: document.getElementById("login-failed-panel"),
  loginFailedMessage: document.getElementById("login-failed-message"),
  appPanel: document.getElementById("app-panel"),
  headerActions: document.getElementById("header-actions"),
  logoutBtn: document.getElementById("logout-btn"),
  loginForm: document.getElementById("login-form"),
  loginError: document.getElementById("login-error"),
  profileSummary: document.getElementById("profile-summary"),
  permissionsSummary: document.getElementById("permissions-summary"),
  accountAvatar: document.getElementById("account-avatar"),
  accountAvatarBtn: document.getElementById("account-avatar-btn"),
  accountAvatarInput: document.getElementById("account-avatar-input"),
  accountRemoveAvatar: document.getElementById("account-remove-avatar"),
  accountPhotoActions: document.getElementById("account-photo-actions"),
  accountName: document.getElementById("account-name"),
  accountEmail: document.getElementById("account-email"),
  accountForm: document.getElementById("account-form"),
  accountUsername: document.getElementById("account-username"),
  accountEmailInput: document.getElementById("account-email-input"),
  accountPassword: document.getElementById("account-password"),
  accountPasswordEmail: document.getElementById("account-password-email"),
  togglePasswordChange: document.getElementById("toggle-password-change"),
  passwordChangeFields: document.getElementById("password-change-fields"),
  accountSaveBar: document.getElementById("account-save-bar"),
  accountSaveBtn: document.getElementById("account-save-btn"),
  accountReadonlyNote: document.getElementById("account-readonly-note"),
  myVacationsGrid: document.getElementById("my-vacations-grid"),
  approvalsGrid: document.getElementById("approvals-grid"),
  approvalsEmpty: document.getElementById("approvals-empty"),
  approvalsCount: document.getElementById("approvals-count"),
  openVacationRequest: document.getElementById("open-vacation-request"),
  openVacationsPanel: document.getElementById("open-vacations-panel"),
  openApprovalsPanel: document.getElementById("open-approvals-panel"),
  openSettingsPanel: document.getElementById("open-settings-panel"),
  vacationsPanel: document.getElementById("vacations-panel"),
  approvalsPanel: document.getElementById("approvals-panel"),
  settingsPanel: document.getElementById("settings-panel"),
  settingsTitle: document.getElementById("settings-title"),
  settingsBack: document.getElementById("settings-back"),
  settingsViews: document.querySelectorAll("[data-settings-view]"),
  settingsRootAvatar: document.getElementById("settings-root-avatar"),
  settingsRootName: document.getElementById("settings-root-name"),
  settingsRootEmail: document.getElementById("settings-root-email"),
  settingsOpenGroups: document.getElementById("settings-open-groups"),
  groupsOrgSummary: document.getElementById("groups-org-summary"),
  empGroupsSearch: document.getElementById("emp-groups-search"),
  empGroupsList: document.getElementById("emp-groups-list"),
  empToggleCreateGroup: document.getElementById("emp-toggle-create-group"),
  empCreateGroupCard: document.getElementById("emp-create-group-card"),
  empCreateGroupForm: document.getElementById("emp-create-group-form"),
  empEmployeesSection: document.getElementById("emp-employees-section"),
  empEmployeesSearch: document.getElementById("emp-employees-search"),
  empEmployeesList: document.getElementById("emp-employees-list"),
  myGroupDetail: document.getElementById("my-group-detail"),
  employeePermsModal: document.getElementById("employee-perms-modal"),
  employeePermsTitle: document.getElementById("employee-perms-title"),
  employeePermsSubtitle: document.getElementById("employee-perms-subtitle"),
  groupEditModal: document.getElementById("group-edit-modal"),
  groupEditTitle: document.getElementById("group-edit-title"),
  groupEditSubtitle: document.getElementById("group-edit-subtitle"),
  groupEditBody: document.getElementById("group-edit-body"),
  memberPermsModal: document.getElementById("member-perms-modal"),
  memberPermsTitle: document.getElementById("member-perms-title"),
  memberPermsSubtitle: document.getElementById("member-perms-subtitle"),
  memberPermsBody: document.getElementById("member-perms-body"),
  vacationModal: document.getElementById("vacation-request-modal"),
  vacationModalClose: document.getElementById("vacation-modal-close"),
  vacationForm: document.getElementById("vacation-form"),
  vacationStart: document.getElementById("vacation-start"),
  vacationEnd: document.getElementById("vacation-end"),
  vacationDetails: document.getElementById("vacation-details"),
  vacationDateHint: document.getElementById("vacation-date-hint"),
  vacationDetailModal: document.getElementById("vacation-detail-modal"),
  vacationDetailAvatar: document.getElementById("vacation-detail-avatar"),
  vacationDetailTitle: document.getElementById("vacation-detail-title"),
  vacationDetailSubtitle: document.getElementById("vacation-detail-subtitle"),
  vacationDetailOverview: document.getElementById("vacation-detail-overview"),
  vacationDetailDetails: document.getElementById("vacation-detail-details"),
  vacationDetailActivity: document.getElementById("vacation-detail-activity"),
  vacationDetailOptions: document.getElementById("vacation-detail-options"),
  vacationDetailTabs: document.querySelectorAll("[data-detail-tab]"),
  vacationDetailPanels: document.querySelectorAll("[data-detail-panel]"),
  permEmployeeId: document.getElementById("perm-employee-id"),
  permissionsForm: document.getElementById("permissions-form"),
  timelinePane: document.getElementById("timeline-pane"),
  timelineDays: document.getElementById("timeline-days"),
  timelineRows: document.getElementById("timeline-rows"),
  timelineNameRows: document.getElementById("timeline-name-rows"),
  timelineRangeLabel: document.getElementById("timeline-range-label"),
  timelineLegend: document.getElementById("timeline-legend"),
  timelineToday: document.getElementById("timeline-today"),
  timelinePrev: document.getElementById("timeline-prev"),
  timelineNext: document.getElementById("timeline-next"),
  colorMatchGroups: document.getElementById("color-match-groups"),
};

const COLOR_MATCH_KEY = "vaction.colorMatchGroups";
const GROUP_COLOR_PALETTE = [
  { bg: "#b7d4f0", border: "#6a9fd4" },
  { bg: "#c5e6c8", border: "#6fb87a" },
  { bg: "#f0d4b0", border: "#d49a5c" },
  { bg: "#e0c8f0", border: "#a87cc8" },
  { bg: "#f0c8d4", border: "#d47a96" },
  { bg: "#c8e8e4", border: "#5fb8ae" },
  { bg: "#e8e0b8", border: "#c4b86a" },
  { bg: "#d4dce8", border: "#8a9ab0" },
];
const UNGROUPED_COLOR = { bg: "#e8ecf0", border: "#aeb6c0" };

let session = null;
let profile = null;
let orgId = null;
let employeesById = new Map();
let timelineStart = null;
let timelineDayCount = TIMELINE_DAYS_BEFORE + TIMELINE_DAYS_AFTER + 1;
let timelineWindowOffset = 0;
let pendingTimelineScrollIndex = null;
let skipScrollToTodayOnce = false;
let timelineScrollExtending = false;
let accountBaseline = null;
let passwordChangeOpen = false;
let canRequestVacation = false;
let canApproveVacations = false;
let canEditVacations = false;
let requestModeActive = false;
let vacationsById = new Map();
let activeVacationId = null;
let cachedGroups = [];
let myGroupPermsById = new Map();
let participantsFilter = "";
let colorMatchGroups = localStorage.getItem(COLOR_MATCH_KEY) === "1";
let lastTimelineEmployees = [];
let lastTimelineVacations = [];
let cachedEmployees = [];
let activeGroupId = null;
let membersPage = 0;
let selectedEmployeePerms = null;
let selectionLayer = null;
let selectionRangeEl = null;
let selectionStartHandle = null;
let selectionEndHandle = null;
let selectionDrag = null;
let syncingSelectionFromInputs = false;
let loginFailed = false;

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
  els.loginForm.email.value = "bob@test.com";
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
  els.headerActions?.classList.toggle("hidden", !loggedIn);
  if (!loggedIn) {
    closeAllPanels();
    closeGroupEditModal();
    closeMemberPermsModal();
    closeEmployeePermsModal();
  }
}

function setModalOpen(modal, open) {
  if (!modal) return;
  modal.classList.toggle("hidden", !open);
  modal.setAttribute("aria-hidden", open ? "false" : "true");
}

function closeGroupEditModal() {
  setModalOpen(els.groupEditModal, false);
}

function closeCreateGroupForm() {
  els.empCreateGroupCard?.classList.add("hidden");
  els.empCreateGroupForm?.reset();
}

function closeMemberPermsModal() {
  setModalOpen(els.memberPermsModal, false);
}

function closeEmployeePermsModal() {
  setModalOpen(els.employeePermsModal, false);
  if (els.permEmployeeId) els.permEmployeeId.value = "";
  selectedEmployeePerms = null;
}

function myPerms() {
  return profile?.permissions || {};
}

function canCreateGroups() {
  return Boolean(myPerms().create_groups);
}

function canAdministerGroups() {
  return canCreateGroups();
}

function setPanelOpen(panel, open) {
  if (!panel) return;
  panel.classList.toggle("hidden", !open);
  panel.setAttribute("aria-hidden", open ? "false" : "true");
}

function closeAllPanels() {
  setPanelOpen(els.vacationsPanel, false);
  setPanelOpen(els.approvalsPanel, false);
  setPanelOpen(els.settingsPanel, false);
  closeVacationModal();
  closeVacationDetailModal();
}

function openVacationsPanel() {
  setPanelOpen(els.approvalsPanel, false);
  setPanelOpen(els.settingsPanel, false);
  setPanelOpen(els.vacationsPanel, true);
}

function openApprovalsPanel() {
  setPanelOpen(els.vacationsPanel, false);
  setPanelOpen(els.settingsPanel, false);
  setPanelOpen(els.approvalsPanel, true);
}

function openSettingsPanel(view = "root") {
  setPanelOpen(els.vacationsPanel, false);
  setPanelOpen(els.approvalsPanel, false);
  setPanelOpen(els.settingsPanel, true);
  showSettingsView(view);
}

const SETTINGS_TITLES = {
  root: "Settings",
  account: "Account Settings",
  groups: "My Groups",
  "group-detail": "Group",
};

let settingsView = "root";

function showSettingsView(name) {
  const view = SETTINGS_TITLES[name] ? name : "root";
  settingsView = view;
  els.settingsViews.forEach((panel) => {
    panel.classList.toggle("hidden", panel.dataset.settingsView !== view);
  });
  if (els.settingsTitle) {
    if (view === "group-detail" && activeGroupId != null) {
      const group = cachedGroups.find((item) => item.id === activeGroupId);
      els.settingsTitle.textContent = group?.name || "Group";
    } else {
      els.settingsTitle.textContent = SETTINGS_TITLES[view];
    }
  }
  els.settingsBack?.classList.toggle("hidden", view === "root");
  if (els.settingsBack) {
    const label = view === "group-detail" ? "My Groups" : "Settings";
    els.settingsBack.setAttribute("aria-label", `Back to ${label}`);
    els.settingsBack.innerHTML = `<span class="settings-back-chevron" aria-hidden="true">‹</span>${label}`;
  }
  updateAccountSaveBar();
}

function accountFormSnapshot() {
  return {
    username: String(els.accountUsername?.value || "").trim(),
    email: String(els.accountEmailInput?.value || "").trim(),
    password: String(els.accountPassword?.value || ""),
    passwordEmail: String(els.accountPasswordEmail?.value || "").trim(),
    avatar: Boolean(els.accountAvatarInput?.files?.length),
  };
}

function isAccountDirty() {
  if (!accountBaseline || !profile?.permissions?.edit_own_account) return false;
  const now = accountFormSnapshot();
  if (now.username !== accountBaseline.username) return true;
  if (now.email !== accountBaseline.email) return true;
  if (now.avatar) return true;
  if (passwordChangeOpen && now.password) return true;
  if (
    passwordChangeOpen &&
    now.passwordEmail &&
    now.passwordEmail !== accountBaseline.passwordEmail
  ) {
    return true;
  }
  return false;
}

function updateAccountSaveBar() {
  const show = settingsView === "account" && isAccountDirty();
  els.accountSaveBar?.classList.toggle("hidden", !show);
}

function setPasswordChangeOpen(open) {
  passwordChangeOpen = Boolean(open);
  els.passwordChangeFields?.classList.toggle("hidden", !passwordChangeOpen);
  if (els.togglePasswordChange) {
    els.togglePasswordChange.textContent = passwordChangeOpen ? "Cancel password change" : "Change password";
  }
  if (!passwordChangeOpen) {
    if (els.accountPassword) els.accountPassword.value = "";
    if (els.accountPasswordEmail && profile) {
      els.accountPasswordEmail.value = profile.email || "";
    }
  } else if (els.accountPasswordEmail && !els.accountPasswordEmail.value && profile) {
    els.accountPasswordEmail.value = profile.email || "";
  }
  updateAccountSaveBar();
}

function settingsGoBack() {
  if (settingsView === "group-detail") {
    closeMemberPermsModal();
    activeGroupId = null;
    participantsFilter = "";
    showSettingsView("groups");
    renderGroupsSettingsPanel();
    return;
  }
  if (settingsView === "root") {
    setPanelOpen(els.settingsPanel, false);
    return;
  }
  showSettingsView("root");
}

function initialsFromName(name) {
  const parts = String(name || "")
    .trim()
    .split(/\s+/)
    .filter(Boolean);
  if (!parts.length) return "?";
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
  return `${parts[0][0]}${parts[1][0]}`.toUpperCase();
}

function setAvatarElement(el, username, avatarUrl) {
  if (!el) return;
  el.replaceChildren();
  if (avatarUrl) {
    const img = document.createElement("img");
    img.src = avatarUrl;
    img.alt = "";
    el.appendChild(img);
    el.classList.add("has-photo");
  } else {
    el.textContent = initialsFromName(username);
    el.classList.remove("has-photo");
  }
}

function avatarMarkup(username, avatarUrl, extraClass = "") {
  const cls = `timeline-bar-avatar${extraClass ? ` ${extraClass}` : ""}${avatarUrl ? " has-photo" : ""}`;
  if (avatarUrl) {
    return `<span class="${cls}" aria-hidden="true"><img src="${ui.escapeHtml(avatarUrl)}" alt=""></span>`;
  }
  return `<span class="${cls}" aria-hidden="true">${ui.escapeHtml(initialsFromName(username))}</span>`;
}

function fileToAvatarDataUrl(file) {
  return new Promise((resolve, reject) => {
    if (!file || !String(file.type || "").startsWith("image/")) {
      reject(new Error("Choose an image file"));
      return;
    }
    const reader = new FileReader();
    reader.onerror = () => reject(new Error("Could not read image"));
    reader.onload = () => {
      const img = new Image();
      img.onerror = () => reject(new Error("Invalid image"));
      img.onload = () => {
        const size = 128;
        const canvas = document.createElement("canvas");
        canvas.width = size;
        canvas.height = size;
        const ctx = canvas.getContext("2d");
        const scale = Math.max(size / img.width, size / img.height);
        const w = img.width * scale;
        const h = img.height * scale;
        ctx.drawImage(img, (size - w) / 2, (size - h) / 2, w, h);
        resolve(canvas.toDataURL("image/jpeg", 0.85));
      };
      img.src = reader.result;
    };
    reader.readAsDataURL(file);
  });
}

function openAvatarPicker() {
  const input = els.accountAvatarInput;
  if (!input || input.disabled) return;
  // Modern Chrome/Firefox: native picker (gallery / camera sheet on phones).
  if (typeof input.showPicker === "function") {
    try {
      input.showPicker();
      return;
    } catch (_) {
      // Not allowed outside a user gesture, or unsupported — fall through.
    }
  }
  input.click();
}

function groupColorForId(groupId) {
  if (groupId == null) return UNGROUPED_COLOR;
  return GROUP_COLOR_PALETTE[groupId % GROUP_COLOR_PALETTE.length];
}

function primaryGroupForEmployee(employeeId) {
  const matches = cachedGroups
    .filter((group) => (group.member_ids || []).includes(employeeId))
    .sort((a, b) => a.id - b.id);
  return matches[0] || null;
}

function myMemberGroups() {
  if (!profile) return [];
  return cachedGroups.filter((group) => (group.member_ids || []).includes(profile.id));
}

function emptyGroupPerms() {
  return ui.defaultPermissions(ui.GROUP_MEMBER_PERMISSION_LABELS);
}

function isGroupCreator(perms) {
  return Boolean(perms?.can_delete_group || (perms?.can_edit_group && perms?.can_add_members));
}

function groupStatusLabel(groupId) {
  return isGroupCreator(myGroupPermsById.get(groupId)) ? "Creator" : "Part of";
}

async function loadMyGroupPermissions() {
  myGroupPermsById = new Map();
  if (!profile) return;
  const groups = myMemberGroups();
  await Promise.all(
    groups.map(async (group) => {
      try {
        const perms = await ui.api(`/groups/${group.id}/members/${profile.id}/permissions?role=employee`);
        myGroupPermsById.set(group.id, perms);
      } catch {
        myGroupPermsById.set(group.id, emptyGroupPerms());
      }
    })
  );
  canApproveVacations = Boolean(
    profile.permissions.approve_vacations ||
      profile.permissions.approve_own_vacations ||
      [...myGroupPermsById.values()].some(
        (perms) => perms.can_approve_vacations || perms.can_approve_own_vacations
      )
  );
  els.openApprovalsPanel?.classList.toggle("hidden", !canApproveVacations);
  if (!canApproveVacations) setPanelOpen(els.approvalsPanel, false);
}

function setColorMatchGroups(enabled) {
  colorMatchGroups = Boolean(enabled);
  localStorage.setItem(COLOR_MATCH_KEY, colorMatchGroups ? "1" : "0");
  if (els.colorMatchGroups) els.colorMatchGroups.checked = colorMatchGroups;
  if (lastTimelineEmployees.length) {
    renderTimeline(lastTimelineEmployees, lastTimelineVacations);
  }
}

function renderTimelineLegend(visibleGroups) {
  if (!els.timelineLegend) return;
  const range = els.timelineRangeLabel;
  const rangeHtml = range
    ? `<span class="timeline-range-label muted" id="timeline-range-label">${ui.escapeHtml(range.textContent || "")}</span>`
    : "";

  if (!colorMatchGroups) {
    els.timelineLegend.innerHTML = `
      <span data-status-legend><i class="legend-swatch approved"></i> Approved</span>
      <span data-status-legend><i class="legend-swatch pending"></i> Pending</span>
      <span data-status-legend><i class="legend-swatch denied"></i> Denied</span>
      ${rangeHtml}`;
  } else {
    const groupItems = visibleGroups.length
      ? visibleGroups
          .map((group) => {
            const color = groupColorForId(group.id);
            return `<span><i class="legend-swatch group-swatch" style="background:${color.bg};border-color:${color.border}"></i> ${ui.escapeHtml(group.name)}</span>`;
          })
          .join("")
      : `<span class="muted">No group colors yet</span>`;
    const ungrouped = `<span><i class="legend-swatch group-swatch" style="background:${UNGROUPED_COLOR.bg};border-color:${UNGROUPED_COLOR.border}"></i> Ungrouped</span>`;
    els.timelineLegend.innerHTML = `${groupItems}${ungrouped}${rangeHtml}`;
  }

  els.timelineRangeLabel = document.getElementById("timeline-range-label");
}

function employeeLabel(employeeId) {
  const employee = employeesById.get(employeeId);
  return employee ? employee.username : `Employee #${employeeId}`;
}

function canEditVacationRecord(vacation) {
  if (!profile || !vacation) return false;
  // Only global editors may change details before/after group approval votes.
  return Boolean(profile.permissions.edit_vacations);
}

function canDeleteVacationRecord(vacation) {
  if (!profile || !vacation) return false;
  if (profile.permissions.delete_vacations) return true;
  const isOwner = vacation.employee_id === profile.id;
  const awaiting = String(vacation.status).includes("Awaiting");
  return isOwner && awaiting;
}

function sharedGroupsWithEmployee(employeeId) {
  return cachedGroups.filter((group) => (group.member_ids || []).includes(employeeId));
}

function canRequestVacationCancellation(vacation) {
  if (!profile || !vacation) return false;
  if (!String(vacation.status).includes("Approved")) return false;
  if (String(vacation.status).includes("Cancellation")) return false;
  // Owner can always ask to cancel their own approved vacation.
  if (vacation.employee_id === profile.id) return true;
  // Group creators may also request cancellation for members they admin.
  return sharedGroupsWithEmployee(vacation.employee_id).some((group) =>
    isGroupCreator(myGroupPermsById.get(group.id))
  );
}

function canDecideVacationCancellation(vacation) {
  if (!profile || !vacation) return false;
  if (!String(vacation.status).includes("CancellationRequested")) return false;

  const votes = groupApprovals(vacation);
  if (votes.length) {
    return votablePendingGroups(vacation).length > 0;
  }

  // Legacy path: no group votes required (same rules as vacation requests).
  const isOwn = vacation.employee_id === profile.id;
  if (isOwn) return Boolean(profile.permissions.approve_own_vacations);
  return Boolean(profile.permissions.approve_vacations);
}

function groupApprovals(vacation) {
  return Array.isArray(vacation?.group_approvals) ? vacation.group_approvals : [];
}

function pendingGroupApprovals(vacation) {
  return groupApprovals(vacation).filter((vote) => vote.status === "pending");
}

function canVoteForGroup(groupId, vacationOwnerId) {
  if (!profile) return false;
  const isOwn = vacationOwnerId === profile.id;
  if (isOwn) {
    if (profile.permissions.approve_own_vacations) return true;
    const perms = myGroupPermsById.get(groupId);
    return Boolean(perms?.can_approve_own_vacations);
  }
  if (profile.permissions.approve_vacations) return true;
  const perms = myGroupPermsById.get(groupId);
  return Boolean(perms?.can_approve_vacations);
}

function votablePendingGroups(vacation) {
  return pendingGroupApprovals(vacation).filter((vote) =>
    canVoteForGroup(vote.group_id, vacation.employee_id)
  );
}

function decisionMeta(vacation) {
  if (String(vacation.status).includes("CancellationRequested")) {
    const by = vacation.cancellation_requested_by
      ? employeeLabel(vacation.cancellation_requested_by)
      : null;
    const votes = groupApprovals(vacation);
    if (votes.length) {
      const done = votes.filter((v) => v.status === "approved").length;
      const base = by
        ? `Cancellation requested by ${ui.escapeHtml(by)}`
        : "Cancellation requested";
      return `${base} · waiting for group approval (${done}/${votes.length})`;
    }
    return by
      ? `Cancellation requested by ${ui.escapeHtml(by)} · waiting for approval`
      : "Cancellation requested · waiting for approval";
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
  const votes = groupApprovals(vacation);
  if (votes.length && String(vacation.status).includes("Awaiting")) {
    const done = votes.filter((v) => v.status === "approved").length;
    return `Waiting for group approval (${done}/${votes.length})`;
  }
  if (String(vacation.status).includes("Awaiting")) return "Waiting for approval";
  return "";
}

function renderGroupApprovalList(vacation) {
  const votes = groupApprovals(vacation);
  if (!votes.length) return "";
  const rows = votes
    .map((vote) => {
      const statusLabel =
        vote.status === "approved"
          ? "Approved"
          : vote.status === "denied"
            ? "Denied"
            : "Pending";
      const by = vote.decided_by
        ? ` · ${ui.escapeHtml(employeeLabel(vote.decided_by))}`
        : "";
      return `<li><strong>${ui.escapeHtml(vote.group_name || `Group #${vote.group_id}`)}</strong>
        <span class="muted"> — ${statusLabel}${by}</span></li>`;
    })
    .join("");
  return `<div class="group-approval-list">
      <p><strong>Group votes</strong></p>
      <ul>${rows}</ul>
    </div>`;
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

function showDetailTab(name) {
  if (name === "details") {
    const vacation = activeVacationId != null ? vacationsById.get(activeVacationId) : null;
    if (!canEditVacationRecord(vacation)) name = "overview";
  }
  els.vacationDetailTabs.forEach((tab) => {
    tab.classList.toggle("active", tab.dataset.detailTab === name);
  });
  els.vacationDetailPanels.forEach((panel) => {
    panel.classList.toggle("hidden", panel.dataset.detailPanel !== name);
  });
}

function closeVacationDetailModal() {
  if (!els.vacationDetailModal) return;
  els.vacationDetailModal.classList.add("hidden");
  els.vacationDetailModal.setAttribute("aria-hidden", "true");
  activeVacationId = null;
}

async function openVacationDetail(vacationId) {
  const vacation = vacationsById.get(vacationId);
  if (!vacation || !els.vacationDetailModal) return;
  activeVacationId = vacationId;
  const employee = employeesById.get(vacation.employee_id);
  const name = employeeLabel(vacation.employee_id);
  const isOwn = vacation.employee_id === profile?.id;
  els.vacationDetailTitle.textContent = isOwn ? `${name} (you)` : name;
  els.vacationDetailSubtitle.textContent = `${ui.formatAppDate(vacation.start)} → ${ui.formatAppDate(
    vacation.end
  )} · ${ui.formatStatus(vacation.status)}`;
  setAvatarElement(
    els.vacationDetailAvatar,
    employee?.username || name,
    employee?.avatar_url || null
  );
  renderVacationOverview(vacation);
  renderVacationOptions(vacation);

  const canModify = canEditVacationRecord(vacation);
  const detailsTab = document.querySelector('[data-detail-tab="details"]');
  const detailsPanel = document.querySelector('[data-detail-panel="details"]');
  detailsTab?.classList.toggle("hidden", !canModify);
  if (!canModify) {
    detailsPanel?.classList.add("hidden");
    if (els.vacationDetailDetails) els.vacationDetailDetails.innerHTML = "";
  } else {
    renderVacationDetailsEditor(vacation);
  }

  els.vacationDetailActivity.innerHTML = `<p class="muted">Loading activity…</p>`;
  showDetailTab("overview");
  els.vacationDetailModal.classList.remove("hidden");
  els.vacationDetailModal.setAttribute("aria-hidden", "false");
  try {
    const events = await ui.api(`/vacations/${vacationId}/events?role=employee`);
    renderVacationActivity(events);
  } catch (err) {
    els.vacationDetailActivity.innerHTML = `<p class="field-error">${ui.escapeHtml(err.message)}</p>`;
  }
}

function renderVacationOverview(vacation) {
  const decideActions = [];
  if (canDecideVacation(vacation) && String(vacation.status).includes("Awaiting")) {
    decideActions.push(
      `<button type="button" class="btn-primary btn-small" data-approve="${vacation.id}">Approve</button>`,
      `<button type="button" class="btn-danger btn-small" data-deny="${vacation.id}">Deny</button>`
    );
  }
  if (canDecideVacationCancellation(vacation)) {
    decideActions.push(
      `<button type="button" class="btn-primary btn-small" data-approve-cancellation="${vacation.id}">Approve cancellation</button>`,
      `<button type="button" class="btn-secondary btn-small" data-deny-cancellation="${vacation.id}">Deny cancellation</button>`
    );
  }

  els.vacationDetailOverview.innerHTML = `
    <div class="vacation-overview-head">
      <div>
        <p><strong>Status:</strong> <span class="status-pill ${ui.statusClass(vacation.status)}">${ui.escapeHtml(
          ui.formatStatus(vacation.status)
        )}</span></p>
        <p class="vacation-card-meta">${decisionMeta(vacation) || "—"}</p>
      </div>
    </div>
    <p><strong>Dates:</strong> ${ui.formatAppDate(vacation.start)} → ${ui.formatAppDate(vacation.end)}</p>
    <p><strong>Details:</strong> ${vacationDetailsHtml(vacation)}</p>
    ${
      vacation.denial_reason
        ? `<p><strong>Denial reason:</strong> ${ui.escapeHtml(vacation.denial_reason)}</p>`
        : ""
    }
    ${renderGroupApprovalList(vacation)}
    ${decideActions.length ? `<div class="vacation-card-actions">${decideActions.join("")}</div>` : ""}`;

  bindVacationCardActions(els.vacationDetailOverview);
  els.vacationDetailOverview.querySelectorAll("[data-approve-cancellation]").forEach((btn) => {
    btn.addEventListener("click", () => decideVacationCancellation(Number(btn.dataset.approveCancellation), "approve"));
  });
  els.vacationDetailOverview.querySelectorAll("[data-deny-cancellation]").forEach((btn) => {
    btn.addEventListener("click", () => decideVacationCancellation(Number(btn.dataset.denyCancellation), "deny"));
  });
}

function renderVacationOptions(vacation) {
  if (!els.vacationDetailOptions) return;
  const status = String(vacation.status || "");

  if (status.includes("Cancelled")) {
    els.vacationDetailOptions.innerHTML = `
      <div class="vacation-options-section">
        <h3 class="vacation-options-title">Cancel request</h3>
        <p class="muted">This vacation was cancelled${
          vacation.cancelled_by
            ? ` by ${ui.escapeHtml(employeeLabel(vacation.cancelled_by))}`
            : ""
        }.</p>
      </div>`;
    return;
  }

  if (status.includes("CancellationRequested")) {
    const canDecide = canDecideVacationCancellation(vacation);
    els.vacationDetailOptions.innerHTML = `
      <div class="vacation-options-section">
        <h3 class="vacation-options-title">Cancel request</h3>
        <p class="muted">Cancellation is waiting for approval${
          vacation.cancellation_requested_by
            ? ` (requested by ${ui.escapeHtml(employeeLabel(vacation.cancellation_requested_by))})`
            : ""
        } — same rules as vacation requests.</p>
        ${renderGroupApprovalList(vacation)}
        ${
          canDecide
            ? `<div class="btn-row">
                 <button type="button" class="btn-primary" data-approve-cancellation="${vacation.id}">Approve cancellation</button>
                 <button type="button" class="btn-secondary" data-deny-cancellation="${vacation.id}">Deny cancellation</button>
               </div>`
            : ""
        }
      </div>`;
    els.vacationDetailOptions.querySelector("[data-approve-cancellation]")?.addEventListener("click", () => {
      decideVacationCancellation(vacation.id, "approve");
    });
    els.vacationDetailOptions.querySelector("[data-deny-cancellation]")?.addEventListener("click", () => {
      decideVacationCancellation(vacation.id, "deny");
    });
    return;
  }

  if (canRequestVacationCancellation(vacation)) {
    const isOwn = vacation.employee_id === profile?.id;
    els.vacationDetailOptions.innerHTML = `
      <div class="vacation-options-section">
        <h3 class="vacation-options-title">Cancel request</h3>
        <p class="muted">${
          isOwn
            ? "Request to cancel your approved vacation. It must be approved the same way as a vacation request (group votes or global approvers)."
            : "Ask to cancel this approved vacation. It must be approved the same way as a vacation request (group votes or global approvers)."
        }</p>
        <div class="btn-row">
          <button type="button" class="btn-danger" data-request-cancellation="${vacation.id}">Request cancellation</button>
        </div>
      </div>`;
    els.vacationDetailOptions.querySelector("[data-request-cancellation]")?.addEventListener("click", () => {
      requestVacationCancellation(vacation.id);
    });
    return;
  }

  if (canDeleteVacationRecord(vacation) && status.includes("Awaiting")) {
    els.vacationDetailOptions.innerHTML = `
      <div class="vacation-options-section">
        <h3 class="vacation-options-title">Withdraw request</h3>
        <p class="muted">This request is still pending. You can withdraw it without approval.</p>
        <div class="btn-row">
          <button type="button" class="btn-danger" data-withdraw-vacation="${vacation.id}">Withdraw request</button>
        </div>
      </div>`;
    els.vacationDetailOptions.querySelector("[data-withdraw-vacation]")?.addEventListener("click", () => {
      withdrawVacation(vacation.id);
    });
    return;
  }

  els.vacationDetailOptions.innerHTML = `
    <div class="vacation-options-section">
      <h3 class="vacation-options-title">Cancel request</h3>
      <p class="muted">No cancellation actions available for this request.</p>
    </div>`;
}

async function requestVacationCancellation(vacationId) {
  if (!confirm("Request cancellation of this vacation? It must be approved before it takes effect.")) return;
  try {
    const updated = await ui.api(`/vacations/${vacationId}/request-cancellation`, {
      method: "POST",
      body: "{}",
    });
    const autoCancelled = updated && String(updated.status || "").includes("Cancelled");
    ui.toast(
      autoCancelled
        ? "Vacation cancelled (no approver configured)"
        : "Cancellation requested — waiting for approval"
    );
    vacationsById.set(vacationId, updated);
    if (activeVacationId === vacationId) {
      renderVacationOverview(updated);
      renderVacationOptions(updated);
    }
    await refreshVacations();
  } catch (err) {
    ui.toast(err.message, true);
  }
}

async function decideVacationCancellation(vacationId, action) {
  const path =
    action === "approve"
      ? `/vacations/${vacationId}/approve-cancellation`
      : `/vacations/${vacationId}/deny-cancellation`;
  try {
    const updated = await ui.api(path, { method: "POST", body: "{}" });
    if (
      action === "approve" &&
      updated &&
      String(updated.status).includes("CancellationRequested")
    ) {
      const pending = pendingGroupApprovals(updated).length;
      ui.toast(
        pending
          ? `Group vote recorded. Still waiting on ${pending} group${pending === 1 ? "" : "s"}.`
          : "Group vote recorded."
      );
    } else {
      ui.toast(
        action === "approve" ? "Cancellation approved" : "Cancellation denied — vacation kept"
      );
    }
    vacationsById.set(vacationId, updated);
    if (activeVacationId === vacationId) {
      renderVacationOverview(updated);
      renderVacationOptions(updated);
    }
    await refreshVacations();
  } catch (err) {
    ui.toast(err.message, true);
  }
}

async function withdrawVacation(vacationId) {
  if (!confirm("Withdraw this pending vacation request? This cannot be undone.")) return;
  try {
    await ui.api(`/vacations/${vacationId}?role=employee`, { method: "DELETE" });
    ui.toast("Request withdrawn");
    closeVacationDetailModal();
    await refreshVacations();
  } catch (err) {
    ui.toast(err.message, true);
  }
}

function renderVacationDetailsEditor(vacation) {
  const editable = canEditVacationRecord(vacation);
  const startIso = ui.toIsoDate(ui.parseAppDate(vacation.start));
  const endIso = ui.toIsoDate(ui.parseAppDate(vacation.end));
  if (!editable) {
    els.vacationDetailDetails.innerHTML = `
      <p class="muted">${
        profile?.permissions.edit_vacations
          ? ""
          : "Only someone with Globally edit vacations permission can change details before approval."
      }</p>
      <p>${vacationDetailsHtml(vacation, "No details")}</p>
      <p class="vacation-card-meta">${ui.formatAppDate(vacation.start)} → ${ui.formatAppDate(vacation.end)}</p>`;
    return;
  }
  els.vacationDetailDetails.innerHTML = `
    <form id="vacation-edit-form">
      <div class="field">
        <label for="edit-vacation-start">Start date</label>
        <input id="edit-vacation-start" name="start" type="date" required value="${startIso}">
      </div>
      <div class="field">
        <label for="edit-vacation-end">End date</label>
        <input id="edit-vacation-end" name="end" type="date" required value="${endIso}">
      </div>
      <div class="field">
        <label for="edit-vacation-details">Details</label>
        <textarea id="edit-vacation-details" name="details" rows="4">${ui.escapeHtml(
          vacation.details || ""
        )}</textarea>
      </div>
      <p id="edit-vacation-hint" class="field-error hidden"></p>
      <div class="btn-row">
        <button type="submit" class="btn-primary">Save changes</button>
      </div>
    </form>`;
  ui.bindVacationDateRange("edit-vacation-start", "edit-vacation-end", "edit-vacation-hint");
  document.getElementById("vacation-edit-form").addEventListener("submit", async (event) => {
    event.preventDefault();
    const form = new FormData(event.target);
    const startIsoValue = form.get("start");
    const endIsoValue = form.get("end");
    const hint = document.getElementById("edit-vacation-hint");
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
      await ui.api(`/vacations/${vacation.id}?role=employee`, {
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
      ui.toast("Vacation updated");
      closeVacationDetailModal();
      await refreshVacations();
    } catch (err) {
      ui.toast(err.message, true);
    }
  });
}

function renderVacationActivity(events) {
  if (!events.length) {
    els.vacationDetailActivity.innerHTML = `<p class="muted">No activity yet.</p>`;
    return;
  }
  els.vacationDetailActivity.innerHTML = `<div class="event-list">${events
    .map((event) => {
      const actor = event.actor_id ? employeeLabel(event.actor_id) : "System";
      return `<article class="event-item">
        <strong>${ui.escapeHtml(eventTypeLabel(event.event_type))}</strong>
        <span class="vacation-card-meta">by ${ui.escapeHtml(actor)} · ${ui.escapeHtml(
        event.created_at
      )}</span>
      </article>`;
    })
    .join("")}</div>`;
}

function canSeeVacationDetails(vacation) {
  if (!profile || !vacation) return false;
  if (vacation.employee_id === profile.id) return true;
  if (profile.permissions.approve_vacations || profile.permissions.edit_vacations) return true;
  return groupApprovals(vacation).some((vote) =>
    canVoteForGroup(vote.group_id, vacation.employee_id)
  );
}

function vacationDetailsHtml(vacation, emptyLabel = "None") {
  if (!canSeeVacationDetails(vacation)) {
    return `<span class="muted">Private</span>`;
  }
  return vacation.details
    ? ui.escapeHtml(vacation.details)
    : `<span class="muted">${ui.escapeHtml(emptyLabel)}</span>`;
}

function canDecideVacation(vacation) {
  if (!profile || !vacation) return false;
  if (!String(vacation.status).includes("Awaiting")) return false;

  const votes = groupApprovals(vacation);
  if (votes.length) {
    return votablePendingGroups(vacation).length > 0;
  }

  // Legacy path: no group votes required.
  const isOwn = vacation.employee_id === profile.id;
  if (isOwn) return Boolean(profile.permissions.approve_own_vacations);
  return Boolean(profile.permissions.approve_vacations);
}

function canSeeWaitingVacation(vacation) {
  if (!profile) return false;
  if (!String(vacation.status).includes("Awaiting")) return true;
  // Own waiting requests are always visible to the requester.
  if (vacation.employee_id === profile.id) return true;
  if (
    profile.permissions.approve_vacations ||
    profile.permissions.edit_vacations ||
    profile.permissions.delete_vacations
  ) {
    return true;
  }
  // Group approvers for any required group on this vacation.
  return groupApprovals(vacation).some((vote) =>
    canVoteForGroup(vote.group_id, vacation.employee_id)
  );
}

function vacationCardTone(status) {
  if (String(status).includes("Awaiting") || String(status).includes("CancellationRequested")) {
    return "is-waiting";
  }
  if (String(status).includes("Cancelled")) return "is-denied";
  if (String(status).includes("Approved")) return "is-approved";
  if (String(status).includes("Denied")) return "is-denied";
  return "";
}

function bindVacationCardActions(root) {
  root.querySelectorAll("[data-approve]").forEach((btn) => {
    btn.addEventListener("click", (event) => {
      event.stopPropagation();
      actOnVacation(Number(btn.dataset.approve), "approve");
    });
  });
  root.querySelectorAll("[data-deny]").forEach((btn) => {
    btn.addEventListener("click", (event) => {
      event.stopPropagation();
      actOnVacation(Number(btn.dataset.deny), "deny");
    });
  });
  root.querySelectorAll("[data-open-vacation]").forEach((card) => {
    card.addEventListener("click", () => openVacationDetail(Number(card.dataset.openVacation)));
  });
}

function renderMyVacationCards(mine) {
  const target = els.myVacationsGrid;
  if (!target) return;

  if (!mine.length) {
    target.innerHTML = `<p class="vacation-card-empty">No vacation requests yet.</p>`;
    return;
  }

  target.innerHTML = mine
    .map((v) => {
      const pending = String(v.status).includes("Awaiting");
      const actions =
        pending && canDecideVacation(v)
          ? `<div class="vacation-card-actions">
               <button type="button" class="btn-primary btn-small" data-approve="${v.id}">Approve</button>
               <button type="button" class="btn-danger btn-small" data-deny="${v.id}">Deny</button>
             </div>`
          : "";
      const meta = decisionMeta(v);
      return `<article class="vacation-card ${vacationCardTone(v.status)}" data-open-vacation="${v.id}">
        <div class="vacation-card-top">
          <h3 class="vacation-card-title">Request #${v.id}</h3>
          <span class="status-pill ${ui.statusClass(v.status)}">${ui.escapeHtml(ui.formatStatus(v.status))}</span>
        </div>
        <p class="vacation-card-meta">${ui.formatAppDate(v.start)} → ${ui.formatAppDate(v.end)}</p>
        ${meta ? `<p class="vacation-card-meta">${meta}</p>` : ""}
        ${
          canSeeVacationDetails(v) && v.details
            ? `<p class="vacation-card-meta">${ui.escapeHtml(v.details)}</p>`
            : ""
        }
        ${
          v.denial_reason
            ? `<p class="vacation-card-meta">Denied: ${ui.escapeHtml(v.denial_reason)}</p>`
            : ""
        }
        ${actions}
      </article>`;
    })
    .join("");

  bindVacationCardActions(target);
}

function renderApprovalCards(pending) {
  const target = els.approvalsGrid;
  if (!target) return;

  const actionable = pending.filter(
    (v) => canDecideVacation(v) || canDecideVacationCancellation(v)
  );
  const count = actionable.length;

  if (els.approvalsCount) {
    els.approvalsCount.textContent = String(count);
    els.approvalsCount.classList.toggle("hidden", count === 0);
  }
  els.openApprovalsPanel?.classList.toggle("has-pending", count > 0);

  if (!actionable.length) {
    target.innerHTML = "";
    els.approvalsEmpty?.classList.remove("hidden");
    return;
  }

  els.approvalsEmpty?.classList.add("hidden");
  target.innerHTML = actionable
    .map((v) => {
      const name = employeeLabel(v.employee_id);
      const employee = employeesById.get(v.employee_id);
      const email = employee ? employee.email : "";
      const isOwn = v.employee_id === profile?.id;
      const meta = decisionMeta(v);
      const isCancel = String(v.status).includes("CancellationRequested");
      const actions = isCancel
        ? `<button type="button" class="btn-primary btn-small" data-approve-cancellation="${v.id}">Approve cancellation</button>
           <button type="button" class="btn-secondary btn-small" data-deny-cancellation="${v.id}">Deny cancellation</button>`
        : `<button type="button" class="btn-primary btn-small" data-approve="${v.id}">Approve</button>
           <button type="button" class="btn-danger btn-small" data-deny="${v.id}">Deny</button>`;
      return `<article class="vacation-card is-waiting" data-open-vacation="${v.id}">
        <div class="vacation-card-top">
          <div class="vacation-card-person">
            ${avatarMarkup(employee?.username || name, employee?.avatar_url || null, "vacation-card-avatar")}
            <div class="vacation-card-person-text">
              <h3 class="vacation-card-title">${ui.escapeHtml(name)}${isOwn ? " (you)" : ""}</h3>
              ${email ? `<p class="vacation-card-meta">${ui.escapeHtml(email)}</p>` : ""}
            </div>
          </div>
          <span class="status-pill status-awaiting">${isCancel ? "Cancel pending" : "Waiting"}</span>
        </div>
        <p class="vacation-card-meta">
          ${ui.formatAppDate(v.start)} → ${ui.formatAppDate(v.end)}${isCancel ? " · cancellation" : ""}
        </p>
        ${meta ? `<p class="vacation-card-meta">${meta}</p>` : ""}
        ${
          canSeeVacationDetails(v) && v.details
            ? `<p class="vacation-card-meta">${ui.escapeHtml(v.details)}</p>`
            : ""
        }
        <div class="vacation-card-actions">
          ${actions}
        </div>
      </article>`;
    })
    .join("");

  bindVacationCardActions(target);
  target.querySelectorAll("[data-approve-cancellation]").forEach((btn) => {
    btn.addEventListener("click", (event) => {
      event.stopPropagation();
      decideVacationCancellation(Number(btn.dataset.approveCancellation), "approve");
    });
  });
  target.querySelectorAll("[data-deny-cancellation]").forEach((btn) => {
    btn.addEventListener("click", (event) => {
      event.stopPropagation();
      decideVacationCancellation(Number(btn.dataset.denyCancellation), "deny");
    });
  });
}

function permFlags(perms) {
  return ui.formatPermissionFlags(perms);
}

function timelineOriginDate() {
  const today = startOfToday();
  const start = new Date(today);
  start.setDate(start.getDate() - TIMELINE_DAYS_BEFORE + timelineWindowOffset);
  return start;
}

function startOfToday() {
  const now = new Date();
  return new Date(now.getFullYear(), now.getMonth(), now.getDate(), 12, 0, 0);
}

function dateAtOffset(base, offset) {
  const date = new Date(base);
  date.setDate(date.getDate() + offset);
  return date;
}

function setTimelineCssVars() {
  DAY_WIDTH = resolveDayWidth();
  document.documentElement.style.setProperty("--day-width", `${DAY_WIDTH}px`);
}

function handleTimelineResize() {
  const next = resolveDayWidth();
  if (next === DAY_WIDTH) return;
  const pane = els.timelinePane;
  const ratio = pane && pane.scrollWidth > 0 ? pane.scrollLeft / pane.scrollWidth : 0;
  setTimelineCssVars();
  if (lastTimelineEmployees.length) {
    renderTimeline(lastTimelineEmployees, lastTimelineVacations || []);
    if (pane && pane.scrollWidth > 0) {
      pane.scrollLeft = ratio * pane.scrollWidth;
    }
  }
  if (requestModeActive) {
    syncSelectionFromInputs();
  }
}

function scrollTimelineToToday(smooth = true) {
  if (!els.timelinePane || timelineStart === null) return;
  const today = startOfToday();
  const index = ui.dayDiff(timelineStart, today);
  scrollTimelineToIndex(index, smooth);
}

function scrollTimelineToIndex(index, smooth = true) {
  if (!els.timelinePane || timelineStart === null) return;
  const target = index * DAY_WIDTH - els.timelinePane.clientWidth / 2 + DAY_WIDTH / 2;
  els.timelinePane.scrollTo({ left: Math.max(0, target), behavior: smooth ? "smooth" : "auto" });
}

function scrollTimelineToSelection(smooth = true) {
  const indexes = currentSelectionIndexes();
  if (!indexes || !els.timelinePane || timelineStart === null) return;
  const mid =
    (Math.max(0, Math.min(timelineDayCount - 1, indexes.start)) +
      Math.max(0, Math.min(timelineDayCount - 1, indexes.end))) /
    2;
  scrollTimelineToIndex(mid, smooth);
}

function ensureDatesInTimelineWindow(startIndex, endIndex, reRender = true) {
  let shift = 0;
  if (startIndex < 0) shift = startIndex;
  else if (endIndex >= timelineDayCount) shift = endIndex - (timelineDayCount - 1);
  if (!shift) return false;
  timelineWindowOffset += shift;
  if (selectionDrag) {
    if (selectionDrag.anchor != null) selectionDrag.anchor -= shift;
    if (selectionDrag.current != null) selectionDrag.current -= shift;
    if (selectionDrag.fixedStart != null) selectionDrag.fixedStart -= shift;
    if (selectionDrag.fixedEnd != null) selectionDrag.fixedEnd -= shift;
  }
  pendingTimelineScrollIndex = null;
  skipScrollToTodayOnce = true;
  if (reRender && lastTimelineEmployees.length) {
    const pane = els.timelinePane;
    const keepScroll = pane?.scrollLeft ?? 0;
    renderTimeline(lastTimelineEmployees, lastTimelineVacations || []);
    requestAnimationFrame(() => {
      if (pane) pane.scrollLeft = Math.max(0, keepScroll - shift * DAY_WIDTH);
    });
  }
  return true;
}

function shiftTimeline(days) {
  if (!days) return;
  timelineWindowOffset += days;
  skipScrollToTodayOnce = true;
  const pane = els.timelinePane;
  const keepScroll = pane?.scrollLeft ?? 0;
  if (lastTimelineEmployees.length) {
    renderTimeline(lastTimelineEmployees, lastTimelineVacations || []);
    requestAnimationFrame(() => {
      if (!pane) return;
      // Origin moved by `days`, so keep the same dates under the viewport.
      pane.scrollLeft = Math.max(0, keepScroll - days * DAY_WIDTH);
    });
  }
}

function onTimelinePaneScroll() {
  const pane = els.timelinePane;
  if (!pane || timelineScrollExtending || timelineStart === null) return;
  const edgePx = DAY_WIDTH * 4;
  const maxScroll = Math.max(0, pane.scrollWidth - pane.clientWidth);
  if (maxScroll <= edgePx * 2) return;
  let delta = 0;
  if (pane.scrollLeft <= edgePx) delta = -TIMELINE_SCROLL_DAYS;
  else if (pane.scrollLeft >= maxScroll - edgePx) delta = TIMELINE_SCROLL_DAYS;
  if (!delta) return;
  timelineScrollExtending = true;
  shiftTimeline(delta);
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      timelineScrollExtending = false;
    });
  });
}

function toIsoFromDate(date) {
  return ui.toIsoDate(date);
}

function dayIndexFromClientX(clientX) {
  if (!els.timelineRows || timelineStart === null) return null;
  const rect = els.timelineRows.getBoundingClientRect();
  const x = clientX - rect.left;
  // Allow indexes outside the current window so drag/select can extend the range.
  return Math.floor(x / DAY_WIDTH);
}

function ensureSelectionLayer() {
  if (!els.timelineRows) return null;
  if (selectionLayer && selectionLayer.isConnected) return selectionLayer;

  selectionLayer = document.createElement("div");
  selectionLayer.className = "timeline-selection-layer";
  selectionLayer.setAttribute("aria-hidden", "true");

  selectionRangeEl = document.createElement("div");
  selectionRangeEl.className = "timeline-selection-range";

  selectionStartHandle = document.createElement("div");
  selectionStartHandle.className = "timeline-selection-handle";
  selectionStartHandle.title = "Drag start date";
  selectionEndHandle = document.createElement("div");
  selectionEndHandle.className = "timeline-selection-handle is-end";
  selectionEndHandle.title = "Drag end date";

  selectionRangeEl.appendChild(selectionStartHandle);
  selectionRangeEl.appendChild(selectionEndHandle);
  selectionLayer.appendChild(selectionRangeEl);
  els.timelineRows.appendChild(selectionLayer);

  selectionLayer.addEventListener("pointerdown", onSelectionPointerDown);
  selectionStartHandle.addEventListener("pointerdown", (event) => {
    onSelectionHandlePointerDown(event, "start");
  });
  selectionEndHandle.addEventListener("pointerdown", (event) => {
    onSelectionHandlePointerDown(event, "end");
  });
  return selectionLayer;
}

function setSelectionEnabled(enabled) {
  const layer = ensureSelectionLayer();
  if (!layer) return;
  layer.classList.toggle("is-disabled", !enabled);
}

function currentSelectionIndexes() {
  if (timelineStart === null) return null;
  const startIso = els.vacationStart?.value;
  const endIso = els.vacationEnd?.value;
  if (!startIso || !endIso) return null;
  const [ys, ms, ds] = startIso.split("-").map(Number);
  const [ye, me, de] = endIso.split("-").map(Number);
  const startDate = new Date(ys, ms - 1, ds, 12, 0, 0);
  const endDate = new Date(ye, me - 1, de, 12, 0, 0);
  return {
    start: ui.dayDiff(timelineStart, startDate),
    end: ui.dayDiff(timelineStart, endDate),
  };
}

function paintSelectionRange(startIndex, endIndex) {
  const layer = ensureSelectionLayer();
  if (!layer || !selectionRangeEl) return;

  const from = Math.min(startIndex, endIndex);
  const to = Math.max(startIndex, endIndex);
  const left = from * DAY_WIDTH;
  const width = (to - from + 1) * DAY_WIDTH;

  selectionRangeEl.style.left = `${left}px`;
  selectionRangeEl.style.width = `${width}px`;
  selectionRangeEl.classList.add("is-visible");
  selectionStartHandle.style.left = "0px";
  selectionEndHandle.style.left = `${width}px`;
}

function clearSelectionRange() {
  if (selectionRangeEl) selectionRangeEl.classList.remove("is-visible");
}

function applySelectionToInputs(startIndex, endIndex) {
  if (!els.vacationStart || !els.vacationEnd || timelineStart === null) return;
  let from = Math.min(startIndex, endIndex);
  let to = Math.max(startIndex, endIndex);
  // End must be strictly after start for the API; if same day, bump end by 1.
  if (to === from) {
    to = from + 1;
  }

  const startDate = dateAtOffset(timelineStart, from);
  const endDate = dateAtOffset(timelineStart, to);

  syncingSelectionFromInputs = true;
  els.vacationStart.value = toIsoFromDate(startDate);
  els.vacationEnd.value = toIsoFromDate(endDate);
  els.vacationEnd.min = toIsoFromDate(dateAtOffset(startDate, 1));
  if (els.vacationDateHint) {
    const ok = ui.vacationRangeValid(els.vacationStart.value, els.vacationEnd.value);
    els.vacationDateHint.textContent = ok ? "" : "End date must be after the start date.";
    els.vacationDateHint.classList.toggle("hidden", ok);
  }
  syncingSelectionFromInputs = false;

  if (from < 0 || to >= timelineDayCount) {
    ensureDatesInTimelineWindow(from, to, true);
    return;
  }
  paintSelectionRange(from, to);
}

function syncSelectionFromInputs() {
  if (syncingSelectionFromInputs || timelineStart === null) return;
  const indexes = currentSelectionIndexes();
  if (!indexes) {
    clearSelectionRange();
    return;
  }
  if (indexes.start < 0 || indexes.end >= timelineDayCount || indexes.end < 0) {
    ensureDatesInTimelineWindow(indexes.start, indexes.end, true);
    return;
  }
  const visibleStart = Math.max(0, indexes.start);
  const visibleEnd = Math.min(timelineDayCount - 1, indexes.end);
  paintSelectionRange(visibleStart, visibleEnd);
  scrollTimelineToSelection(true);
}

function beginSelectionDrag(event, dragState) {
  event.preventDefault();
  event.stopPropagation();
  selectionDrag = dragState;
  selectionLayer.setPointerCapture?.(event.pointerId);

  const activeHandle =
    dragState.mode === "start"
      ? selectionStartHandle
      : dragState.mode === "end"
        ? selectionEndHandle
        : null;
  activeHandle?.classList.add("is-dragging");

  const onMove = (moveEvent) => {
    if (!selectionDrag) return;
    const next = dayIndexFromClientX(moveEvent.clientX);
    if (next === null) return;

    if (selectionDrag.mode === "create") {
      selectionDrag.current = next;
      applySelectionToInputs(selectionDrag.anchor, selectionDrag.current);
      return;
    }

    if (selectionDrag.mode === "start") {
      // Move start only; keep end fixed (leave at least one day gap).
      const start = Math.min(next, selectionDrag.fixedEnd - 1);
      applySelectionToInputs(start, selectionDrag.fixedEnd);
      return;
    }

    if (selectionDrag.mode === "end") {
      // Move end only; keep start fixed (leave at least one day gap).
      const end = Math.max(next, selectionDrag.fixedStart + 1);
      applySelectionToInputs(selectionDrag.fixedStart, end);
    }
  };

  const onUp = () => {
    activeHandle?.classList.remove("is-dragging");
    selectionDrag = null;
    window.removeEventListener("pointermove", onMove);
    window.removeEventListener("pointerup", onUp);
    window.removeEventListener("pointercancel", onUp);
  };

  window.addEventListener("pointermove", onMove);
  window.addEventListener("pointerup", onUp);
  window.addEventListener("pointercancel", onUp);
}

function onSelectionHandlePointerDown(event, which) {
  if (!requestModeActive || !canRequestVacation || timelineStart === null) return;
  if (event.button !== undefined && event.button !== 0) return;

  const indexes = currentSelectionIndexes();
  if (!indexes) return;

  beginSelectionDrag(event, {
    mode: which,
    fixedStart: indexes.start,
    fixedEnd: indexes.end,
  });
}

function onSelectionPointerDown(event) {
  if (!requestModeActive || !canRequestVacation || timelineStart === null) return;
  if (event.button !== undefined && event.button !== 0) return;
  if (event.target?.closest?.(".timeline-selection-handle")) return;

  const index = dayIndexFromClientX(event.clientX);
  if (index === null) return;

  applySelectionToInputs(index, index);
  beginSelectionDrag(event, { mode: "create", anchor: index, current: index });
}

function openVacationModal() {
  if (!els.vacationModal || !canRequestVacation) return;
  if (els.vacationStart && !els.vacationStart.value) {
    els.vacationStart.value = ui.toIsoDate(startOfToday());
  }
  if (els.vacationStart?.value && els.vacationEnd) {
    const minEnd = (() => {
      const [y, m, d] = els.vacationStart.value.split("-").map(Number);
      const next = new Date(y, m - 1, d + 1, 12, 0, 0);
      return ui.toIsoDate(next);
    })();
    els.vacationEnd.min = minEnd;
    if (!els.vacationEnd.value || els.vacationEnd.value <= els.vacationStart.value) {
      els.vacationEnd.value = minEnd;
    }
  }
  requestModeActive = true;
  document.body.classList.add("request-mode-open");
  setSelectionEnabled(true);
  els.vacationModal.classList.remove("hidden");
  els.vacationModal.setAttribute("aria-hidden", "false");
  syncSelectionFromInputs();
}

function closeVacationModal() {
  if (!els.vacationModal) return;
  requestModeActive = false;
  document.body.classList.remove("request-mode-open");
  setSelectionEnabled(false);
  clearSelectionRange();
  els.vacationModal.classList.add("hidden");
  els.vacationModal.setAttribute("aria-hidden", "true");
}

function renderTimeline(employees, vacations) {
  if (!els.timelineDays || !els.timelineRows || !els.timelineNameRows) return;

  lastTimelineEmployees = employees;
  lastTimelineVacations = vacations;

  setTimelineCssVars();
  timelineStart = timelineOriginDate();
  const today = startOfToday();

  const sortedEmployees = [...employees].sort((a, b) =>
    a.username.localeCompare(b.username, undefined, { sensitivity: "base" })
  );

  const vacationsByEmployee = new Map();
  vacations.forEach((vacation) => {
    const list = vacationsByEmployee.get(vacation.employee_id) || [];
    list.push(vacation);
    vacationsByEmployee.set(vacation.employee_id, list);
  });

  const totalWidth = timelineDayCount * DAY_WIDTH;
  const todayOffset = ui.dayDiff(timelineStart, today) * DAY_WIDTH + DAY_WIDTH / 2;

  const visibleGroupIds = new Set();
  sortedEmployees.forEach((employee) => {
    const group = primaryGroupForEmployee(employee.id);
    if (group) visibleGroupIds.add(group.id);
  });
  const visibleGroups = cachedGroups
    .filter((group) => visibleGroupIds.has(group.id))
    .sort((a, b) => a.name.localeCompare(b.name, undefined, { sensitivity: "base" }));
  renderTimelineLegend(visibleGroups);

  els.timelineDays.style.width = `${totalWidth}px`;
  els.timelineDays.innerHTML = "";
  for (let i = 0; i < timelineDayCount; i += 1) {
    const date = dateAtOffset(timelineStart, i);
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
      ${showMonth ? `<span class="muted" style="font-size:0.58rem">${parts.month}</span>` : ""}`;
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
  // Selection nodes were wiped with innerHTML; recreate on next ensure.
  selectionLayer = null;
  selectionRangeEl = null;
  selectionStartHandle = null;
  selectionEndHandle = null;

  const todayMarker = document.createElement("div");
  todayMarker.className = "timeline-today-marker";
  todayMarker.style.left = `${todayOffset}px`;
  els.timelineRows.appendChild(todayMarker);

  sortedEmployees.forEach((employee) => {
    const isSelf = profile?.id === employee.id;
    const primaryGroup = primaryGroupForEmployee(employee.id);
    const groupColor = groupColorForId(primaryGroup?.id);
    const nameRow = document.createElement("div");
    nameRow.className = `timeline-name-row${isSelf ? " is-self" : ""}`;
    const groupDot =
      colorMatchGroups
        ? `<span class="group-dot" style="background:${groupColor.bg};border-color:${groupColor.border}" title="${ui.escapeHtml(primaryGroup?.name || "Ungrouped")}"></span>`
        : "";
    nameRow.innerHTML = `
      <strong>${groupDot}${ui.escapeHtml(employee.username)}</strong>
      <span class="muted">${ui.escapeHtml(employee.email)}</span>`;
    els.timelineNameRows.appendChild(nameRow);

    const track = document.createElement("div");
    track.className = `timeline-track${isSelf ? " is-self" : ""}`;
    track.style.width = `${totalWidth}px`;

    const employeeVacations = vacationsByEmployee.get(employee.id) || [];
    employeeVacations.forEach((vacation) => {
      if (!canSeeWaitingVacation(vacation)) return;

      const start = ui.parseAppDate(vacation.start);
      const end = ui.parseAppDate(vacation.end);
      const startIndex = ui.dayDiff(timelineStart, start);
      const endIndex = ui.dayDiff(timelineStart, end);
      const lastVisible = timelineDayCount - 1;

      if (endIndex < 0 || startIndex > lastVisible) return;

      const visibleStart = Math.max(0, startIndex);
      const visibleEnd = Math.min(lastVisible, endIndex);
      const spanDays = visibleEnd - visibleStart + 1;
      const left = visibleStart * DAY_WIDTH + 2;
      const width = spanDays * DAY_WIDTH - 4;

      const bar = document.createElement("div");
      const statusClass = ui.statusClass(vacation.status);
      bar.className = `timeline-bar ${statusClass}${isSelf ? " is-mine" : ""}${
        colorMatchGroups ? " group-colored" : ""
      }`;
      if (colorMatchGroups) {
        bar.style.background = groupColor.bg;
        bar.style.borderColor = groupColor.border;
      }
      bar.style.left = `${left}px`;
      bar.style.width = `${Math.max(width, 28)}px`;
      bar.dataset.openVacation = String(vacation.id);
      bar.setAttribute("role", "button");
      bar.tabIndex = 0;
      const statusLabel = ui.formatStatus(vacation.status);
      const groupLabel = primaryGroup ? primaryGroup.name : "Ungrouped";
      const label =
        width >= 88
          ? `${ui.formatAppDate(vacation.start)} – ${ui.formatAppDate(vacation.end)}`
          : width >= 48
            ? statusLabel
            : "";
      bar.innerHTML = `${avatarMarkup(employee.username, employee.avatar_url)}${
        label ? `<span class="timeline-bar-label">${ui.escapeHtml(label)}</span>` : ""
      }`;
      bar.title = `${employee.username}: ${ui.formatAppDate(vacation.start)} → ${ui.formatAppDate(
        vacation.end
      )} (${statusLabel}${colorMatchGroups ? ` · ${groupLabel}` : ""}) — click for details`;
      bar.addEventListener("click", (event) => {
        event.stopPropagation();
        openVacationDetail(vacation.id);
      });
      bar.addEventListener("keydown", (event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          openVacationDetail(vacation.id);
        }
      });
      track.appendChild(bar);
    });

    els.timelineRows.appendChild(track);
  });

  ensureSelectionLayer();
  setSelectionEnabled(requestModeActive && canRequestVacation);
  if (requestModeActive) syncSelectionFromInputs();
  else clearSelectionRange();

  if (els.timelineRangeLabel) {
    const rangeStart = timelineStart;
    const rangeEnd = dateAtOffset(timelineStart, timelineDayCount - 1);
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
    if (pendingTimelineScrollIndex != null) {
      scrollTimelineToIndex(pendingTimelineScrollIndex, false);
      pendingTimelineScrollIndex = null;
      skipScrollToTodayOnce = false;
      return;
    }
    if (skipScrollToTodayOnce) {
      skipScrollToTodayOnce = false;
      return;
    }
    scrollTimelineToToday(false);
  });
}

async function loadProfile() {
  profile = await ui.api("/employees/me");
  orgId = profile.organization_id;

  if (els.accountAvatar) setAvatarElement(els.accountAvatar, profile.username, profile.avatar_url);
  if (els.accountName) els.accountName.textContent = profile.username;
  if (els.accountEmail) els.accountEmail.textContent = profile.email;
  if (els.settingsRootAvatar) {
    setAvatarElement(els.settingsRootAvatar, profile.username, profile.avatar_url);
  }
  if (els.settingsRootName) els.settingsRootName.textContent = profile.username;
  if (els.settingsRootEmail) els.settingsRootEmail.textContent = profile.email || "Account Settings";
  if (els.accountUsername) els.accountUsername.value = profile.username;
  if (els.accountEmailInput) els.accountEmailInput.value = profile.email;
  if (els.accountPassword) els.accountPassword.value = "";
  if (els.accountPasswordEmail) els.accountPasswordEmail.value = profile.email || "";
  if (els.accountAvatarInput) els.accountAvatarInput.value = "";
  setPasswordChangeOpen(false);

  const canEditAccount = Boolean(profile.permissions.edit_own_account);
  [
    els.accountUsername,
    els.accountEmailInput,
    els.accountPassword,
    els.accountPasswordEmail,
    els.accountAvatarInput,
    els.togglePasswordChange,
  ].forEach((input) => {
    if (input) input.disabled = !canEditAccount;
  });
  if (els.accountAvatarBtn) els.accountAvatarBtn.disabled = !canEditAccount;
  els.accountReadonlyNote?.classList.toggle("hidden", canEditAccount);
  els.accountPhotoActions?.classList.toggle("hidden", !canEditAccount);
  els.accountRemoveAvatar?.classList.toggle(
    "hidden",
    !canEditAccount || !profile.avatar_url
  );
  accountBaseline = accountFormSnapshot();
  updateAccountSaveBar();

  els.profileSummary.innerHTML = `
    <p>Employee ID: <strong>${profile.id}</strong></p>
    <p>Organization: ${profile.organization_id ?? "—"}</p>
    <p>Groups: ${profile.group_ids.join(", ") || "none"}</p>`;
  els.permissionsSummary.innerHTML = `<p>${permFlags(profile.permissions)}</p>`;

  canApproveVacations = Boolean(
    profile.permissions.approve_vacations ||
      profile.permissions.approve_own_vacations ||
      [...myGroupPermsById.values()].some(
        (perms) => perms.can_approve_vacations || perms.can_approve_own_vacations
      )
  );
  canRequestVacation = Boolean(profile.permissions.request_vacation);
  canEditVacations = Boolean(profile.permissions.edit_vacations);

  els.openVacationRequest?.classList.toggle("hidden", !canRequestVacation);
  els.openApprovalsPanel?.classList.toggle("hidden", !canApproveVacations);
  if (!canApproveVacations) setPanelOpen(els.approvalsPanel, false);
  const showGroupSettings = canAdministerGroups() || Boolean(profile.group_ids?.length);
  els.settingsOpenGroups?.classList.toggle("hidden", !showGroupSettings);
  els.settingsOpenGroups?.closest(".ios-settings-group")?.previousElementSibling?.classList.toggle(
    "hidden",
    !showGroupSettings
  );
  if ((settingsView === "groups" || settingsView === "group-detail") && !showGroupSettings) {
    activeGroupId = null;
    showSettingsView("root");
  }
  if (!canRequestVacation) closeVacationModal();
  else setSelectionEnabled(requestModeActive);
  if (settingsView === "groups") renderGroupsSettingsPanel();
  if (settingsView === "group-detail" && activeGroupId != null) {
    renderMyGroupDetail().catch(() => {});
  }
}

async function refreshVacations() {
  const [mine, all, employees] = await Promise.all([
    ui.api("/vacations/mine"),
    ui.api("/vacations?role=employee"),
    ui.api("/employees?role=employee"),
  ]);

  employeesById = new Map(employees.map((employee) => [employee.id, employee]));
  cachedEmployees = employees;
  vacationsById = new Map(all.map((vacation) => [vacation.id, vacation]));
  mine.forEach((vacation) => vacationsById.set(vacation.id, vacation));
  renderTimeline(employees, all);
  renderMyVacationCards(mine);

  const pending = all.filter(
    (v) =>
      String(v.status).includes("Awaiting") || String(v.status).includes("CancellationRequested")
  );
  if (canApproveVacations) {
    renderApprovalCards(pending);
  } else if (els.approvalsCount) {
    els.approvalsCount.classList.add("hidden");
    els.openApprovalsPanel?.classList.remove("has-pending");
  }
}

async function actOnVacation(id, action) {
  try {
    const path = `/vacations/${id}/${action}`;
    let body = "{}";
    if (action === "deny") {
      const reason = window.prompt("Reason for denial (optional):", "");
      if (reason === null) return;
      body = JSON.stringify({ reason: String(reason).trim() });
    }
    const updated = await ui.api(path, { method: "POST", body });
    if (action === "approve" && updated && String(updated.status).includes("Awaiting")) {
      const pending = pendingGroupApprovals(updated).length;
      ui.toast(
        pending
          ? `Group vote recorded. Still waiting on ${pending} group${pending === 1 ? "" : "s"}.`
          : "Group vote recorded."
      );
    } else {
      ui.toast(`Request ${action === "approve" ? "approved" : "denied"}`);
    }
    if (activeVacationId === id) {
      vacationsById.set(id, updated);
      renderVacationOverview(updated);
      renderVacationOptions(updated);
    }
    await refreshVacations();
  } catch (err) {
    ui.toast(err.message, true);
  }
}

async function refreshGroups() {
  if (orgId === null || orgId === undefined) return;
  cachedGroups = await ui.api(`/organizations/${orgId}/groups?role=employee`);
  if (!cachedEmployees.length) {
    cachedEmployees = await ui.api("/employees");
    employeesById = new Map(cachedEmployees.map((employee) => [employee.id, employee]));
  }
  await loadMyGroupPermissions();
  if (settingsView === "group-detail" && activeGroupId != null) {
    await renderMyGroupDetail();
  } else if (settingsView === "groups") {
    renderGroupsSettingsPanel();
  }
}

async function prepareGroupsSettingsView() {
  cachedEmployees = await ui.api("/employees");
  employeesById = new Map(cachedEmployees.map((employee) => [employee.id, employee]));
  await refreshGroups();
  renderGroupsSettingsPanel();
}

function renderGroupsSettingsPanel() {
  const memberGroups = myMemberGroups();
  if (els.groupsOrgSummary) {
    els.groupsOrgSummary.textContent = profile
      ? `You belong to ${memberGroups.length} group${memberGroups.length === 1 ? "" : "s"}.`
      : "Loading…";
  }

  els.empToggleCreateGroup?.classList.toggle("hidden", !canCreateGroups());
  els.empEmployeesSection?.classList.toggle("hidden", !canAdministerGroups());

  renderEmpGroupsList();
  if (canAdministerGroups()) renderEmpEmployeesList();
}

function renderEmpGroupsList() {
  if (!els.empGroupsList) return;
  const query = (els.empGroupsSearch?.value || "").trim().toLowerCase();
  const sourceGroups = myMemberGroups();
  const groups = sourceGroups.filter((group) => {
    if (!query) return true;
    return `${group.name} #${group.id}`.toLowerCase().includes(query);
  });

  if (!groups.length) {
    els.empGroupsList.innerHTML = `<p class="muted org-scroll-empty">${
      sourceGroups.length
        ? "No groups match this search."
        : canCreateGroups()
          ? "You are not in any groups yet. Create one to get started."
          : "You are not a member of any groups yet."
    }</p>`;
    return;
  }

  els.empGroupsList.innerHTML = groups
    .map((group) => {
      const memberCount = (group.member_ids || []).length;
      const color = groupColorForId(group.id);
      const status = groupStatusLabel(group.id);
      const statusClass = status === "Creator" ? "status-creator" : "status-member";
      return `
        <button type="button" class="my-group-row" data-open-my-group="${group.id}">
          <div class="my-group-row-main">
            <strong><span class="group-dot" style="background:${color.bg};border-color:${color.border}"></span>${ui.escapeHtml(group.name)}</strong>
            <span class="muted">${memberCount} member${memberCount === 1 ? "" : "s"}</span>
          </div>
          <span class="my-group-status ${statusClass}">${status}</span>
          <span class="ios-chevron" aria-hidden="true">›</span>
        </button>`;
    })
    .join("");

  els.empGroupsList.querySelectorAll("[data-open-my-group]").forEach((btn) => {
    btn.addEventListener("click", () => openMyGroupDetail(Number(btn.dataset.openMyGroup)));
  });
}

function renderEmpEmployeesList() {
  if (!els.empEmployeesList) return;
  const query = (els.empEmployeesSearch?.value || "").trim().toLowerCase();
  const employees = cachedEmployees.filter((employee) => {
    if (!query) return true;
    return `${employee.username} ${employee.email} #${employee.id}`.toLowerCase().includes(query);
  });

  if (!employees.length) {
    els.empEmployeesList.innerHTML = `<p class="muted org-scroll-empty">No employees found.</p>`;
    return;
  }

  els.empEmployeesList.innerHTML = employees
    .map((employee) => {
      const groups =
        cachedGroups
          .filter((group) => (group.member_ids || []).includes(employee.id))
          .map((group) => group.name)
          .join(", ") || "No groups";
      return `
        <div class="groups-editor-row" data-employee-id="${employee.id}">
          <div>
            <strong>${ui.escapeHtml(employee.username)}</strong>
            <span class="muted">${ui.escapeHtml(employee.email)} · #${employee.id} · ${ui.escapeHtml(groups)}</span>
          </div>
          <button type="button" class="icon-btn" data-emp-employee-perms="${employee.id}" title="Employee permissions" aria-label="Employee permissions">
            ${GEAR_ICON}
          </button>
        </div>`;
    })
    .join("");

  els.empEmployeesList.querySelectorAll("[data-emp-employee-perms]").forEach((btn) => {
    btn.addEventListener("click", () => openEmployeePermissionsModal(Number(btn.dataset.empEmployeePerms)));
  });
}

async function openEmployeePermissionsModal(employeeId) {
  const employee = cachedEmployees.find((item) => item.id === employeeId) || employeesById.get(employeeId);
  if (!employee) {
    ui.toast("Employee not found", true);
    return;
  }
  try {
    selectedEmployeePerms = await ui.api(`/employees/${employeeId}/permissions?role=employee`);
  } catch (err) {
    ui.toast(err.message, true);
    return;
  }
  if (els.permEmployeeId) els.permEmployeeId.value = String(employeeId);
  fillGlobalPermissionCheckboxes(selectedEmployeePerms);
  if (els.employeePermsTitle) els.employeePermsTitle.textContent = employee.username;
  if (els.employeePermsSubtitle) {
    els.employeePermsSubtitle.textContent = `${employee.email} · id ${employee.id}`;
  }
  setModalOpen(els.employeePermsModal, true);
}

async function openMyGroupDetail(groupId) {
  activeGroupId = groupId;
  membersPage = 0;
  participantsFilter = "";
  closeMemberPermsModal();
  closeGroupEditModal();
  showSettingsView("group-detail");
  await renderMyGroupDetail();
}

async function renderMyGroupDetail() {
  if (!els.myGroupDetail) return;
  const group = cachedGroups.find((item) => item.id === activeGroupId);
  if (!group) {
    activeGroupId = null;
    showSettingsView("groups");
    renderGroupsSettingsPanel();
    return;
  }

  let myGroupPerms = myGroupPermsById.get(activeGroupId) || emptyGroupPerms();
  if (profile && (group.member_ids || []).includes(profile.id)) {
    try {
      myGroupPerms = await ui.api(`/groups/${activeGroupId}/members/${profile.id}/permissions?role=employee`);
      myGroupPermsById.set(activeGroupId, myGroupPerms);
    } catch {
      /* keep cached/default */
    }
  }

  const memberIds = group.member_ids || [];
  const members = memberIds
    .map((id) => cachedEmployees.find((employee) => employee.id === id) || employeesById.get(id))
    .filter(Boolean)
    .sort((a, b) => String(a.username).localeCompare(String(b.username)));

  const filter = participantsFilter.trim().toLowerCase();
  const filteredMembers = filter
    ? members.filter((member) =>
        `${member.username} ${member.email} #${member.id}`.toLowerCase().includes(filter)
      )
    : members;

  const canEdit = Boolean(myGroupPerms.can_edit_group);
  const canAdd = Boolean(myGroupPerms.can_add_members);
  const canDelete = Boolean(myGroupPerms.can_delete_group);
  const canRemove = Boolean(myGroupPerms.can_remove_members);
  const canManageMemberPerms = canEdit;
  const status = groupStatusLabel(group.id);

  if (els.settingsTitle) els.settingsTitle.textContent = group.name;

  const detailsNote = canEdit
    ? "Update the group name and review its participants."
    : "View group information and participants in this group.";

  els.myGroupDetail.innerHTML = `
    <div class="card settings-org-card">
      <h2>Group details</h2>
      <p class="muted section-note">${detailsNote}</p>
      ${
        canEdit
          ? `<form id="group-info-form" class="org-details-form">
              <div class="field">
                <label for="group-info-name">Name</label>
                <input id="group-info-name" name="name" required maxlength="120" value="${ui.escapeHtml(group.name)}" />
              </div>
              <button type="submit" class="btn-primary">Save details</button>
            </form>`
          : `<p class="org-details-form"><strong>${ui.escapeHtml(group.name)}</strong></p>`
      }
      <p id="group-details-meta" class="muted">
        ${members.length} participant${members.length === 1 ? "" : "s"} · your status:
        <span class="my-group-status ${status === "Creator" ? "status-creator" : "status-member"}">${status}</span>
      </p>
      ${
        canDelete
          ? `<div class="btn-row" style="margin-top:0.75rem">
              <button type="button" class="btn-danger" id="delete-group-btn">Delete group</button>
            </div>`
          : ""
      }

      ${
        canAdd
          ? `<hr class="soft-rule">
            <div class="section-head">
              <div>
                <h3 class="org-section-title">Add people</h3>
                <p class="muted section-note">Search and add employees to this group.</p>
              </div>
            </div>
            <div class="field">
              <label for="member-search">Search employees</label>
              <input id="member-search" type="search" placeholder="Filter by name or email…" autocomplete="off" />
            </div>
            <div id="member-search-results" class="search-results"></div>`
          : ""
      }

      <hr class="soft-rule">

      <div class="section-head">
        <div>
          <h3 class="org-section-title">Participants</h3>
          <p class="muted section-note">Search and open settings to edit permissions${
            canRemove ? " or remove people" : ""
          }.</p>
        </div>
      </div>
      <div class="field">
        <label for="participants-search">Search participants</label>
        <input id="participants-search" type="search" placeholder="Filter by name or email…" autocomplete="off" value="${ui.escapeHtml(participantsFilter)}" />
      </div>
      <div id="group-members-list" class="org-scroll-list my-participants-list">
        ${
          filteredMembers.length
            ? filteredMembers
                .map((member) => {
                  const isSelf = profile && member.id === profile.id;
                  const memberStatus = isSelf ? groupStatusLabel(group.id) : "";
                  const statusHtml = memberStatus
                    ? `<span class="my-group-status ${
                        memberStatus === "Creator" ? "status-creator" : "status-member"
                      }">${memberStatus}</span>`
                    : "";
                  return `
                    <div class="groups-editor-row" data-member-id="${member.id}">
                      <div>
                        <strong>${ui.escapeHtml(member.username)}${isSelf ? " (you)" : ""}${statusHtml ? ` ${statusHtml}` : ""}</strong>
                        <span class="muted">${ui.escapeHtml(member.email)} · #${member.id}</span>
                      </div>
                      <div class="row-actions">
                        ${
                          canManageMemberPerms
                            ? `<button type="button" class="icon-btn" data-member-settings="${member.id}" title="Member settings" aria-label="Member settings">
                                ${GEAR_ICON}
                              </button>`
                            : ""
                        }
                        ${
                          canRemove
                            ? `<button type="button" class="btn-danger btn-small" data-remove-member="${member.id}">Remove</button>`
                            : ""
                        }
                      </div>
                    </div>`;
                })
                .join("")
            : `<p class="muted org-scroll-empty">${
                members.length ? "No participants match this search." : "No participants yet."
              }</p>`
        }
      </div>
    </div>
  `;

  document.getElementById("group-info-form")?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    try {
      await ui.api(`/groups/${activeGroupId}?role=employee`, {
        method: "PUT",
        body: JSON.stringify({ name: String(form.get("name") || "").trim() }),
      });
      ui.toast("Group updated");
      await refreshGroups();
    } catch (err) {
      ui.toast(err.message, true);
    }
  });

  const searchInput = document.getElementById("member-search");
  const searchResults = document.getElementById("member-search-results");
  if (searchInput && searchResults) {
    const renderSearch = () => {
      const query = searchInput.value.trim().toLowerCase();
      if (!query) {
        searchResults.innerHTML = "";
        return;
      }
      const memberSet = new Set(memberIds);
      const matches = cachedEmployees
        .filter((employee) => !memberSet.has(employee.id))
        .filter((employee) => {
          const hay = `${employee.username} ${employee.email}`.toLowerCase();
          return hay.includes(query);
        })
        .slice(0, 8);
      if (!matches.length) {
        searchResults.innerHTML = `<p class="muted">No matching employees</p>`;
        return;
      }
      searchResults.innerHTML = matches
        .map(
          (employee) => `
            <div class="search-result-item">
              <div>
                <strong>${ui.escapeHtml(employee.username)}</strong>
                <span class="muted">${ui.escapeHtml(employee.email)}</span>
              </div>
              <button type="button" class="btn-primary btn-small" data-add-member="${employee.id}">Add</button>
            </div>
          `
        )
        .join("");
      searchResults.querySelectorAll("[data-add-member]").forEach((btn) => {
        btn.addEventListener("click", async () => {
          try {
            await ui.api(`/groups/${activeGroupId}/members?role=employee`, {
              method: "POST",
              body: JSON.stringify({ employee_id: Number(btn.dataset.addMember) }),
            });
            ui.toast("Member added");
            searchInput.value = "";
            searchResults.innerHTML = "";
            await refreshGroups();
          } catch (err) {
            ui.toast(err.message, true);
          }
        });
      });
    };
    searchInput.addEventListener("input", renderSearch);
  }

  document.getElementById("participants-search")?.addEventListener("input", (event) => {
    participantsFilter = event.currentTarget.value || "";
    const caret = event.currentTarget.selectionStart;
    renderMyGroupDetail().then(() => {
      const input = document.getElementById("participants-search");
      if (!input) return;
      input.focus();
      try {
        input.setSelectionRange(caret, caret);
      } catch {
        /* ignore */
      }
    });
  });

  els.myGroupDetail.querySelectorAll("[data-remove-member]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      if (!window.confirm("Remove this person from the group?")) return;
      try {
        await ui.api(`/groups/${activeGroupId}/members/${btn.dataset.removeMember}?role=employee`, {
          method: "DELETE",
        });
        ui.toast("Member removed");
        await refreshGroups();
      } catch (err) {
        ui.toast(err.message, true);
      }
    });
  });

  els.myGroupDetail.querySelectorAll("[data-member-settings]").forEach((btn) => {
    btn.addEventListener("click", () => openMemberPermissions(Number(btn.dataset.memberSettings)));
  });

  document.getElementById("delete-group-btn")?.addEventListener("click", async () => {
    if (!window.confirm(`Delete group "${group.name}"?`)) return;
    try {
      await ui.api(`/groups/${activeGroupId}?role=employee`, { method: "DELETE" });
      ui.toast("Group deleted");
      activeGroupId = null;
      participantsFilter = "";
      await refreshGroups();
      showSettingsView("groups");
      renderGroupsSettingsPanel();
    } catch (err) {
      ui.toast(err.message, true);
    }
  });
}

async function openMemberPermissions(employeeId) {
  const employee =
    cachedEmployees.find((item) => item.id === employeeId) || employeesById.get(employeeId);
  if (!employee) return;

  let perms = emptyGroupPerms();
  try {
    perms = ui.mergePermissions(
      ui.GROUP_MEMBER_PERMISSION_LABELS,
      await ui.api(`/groups/${activeGroupId}/members/${employeeId}/permissions?role=employee`)
    );
  } catch (err) {
    ui.toast(err.message, true);
    return;
  }

  if (els.memberPermsTitle) els.memberPermsTitle.textContent = employee.username;
  if (els.memberPermsSubtitle) {
    els.memberPermsSubtitle.textContent = `Group-specific permissions for group #${activeGroupId}`;
  }
  els.memberPermsBody.innerHTML = `
    <p class="muted" style="margin:0 0 0.85rem">
      These apply only inside this group. Creating new groups is a global permission set under Settings → My Groups.
    </p>
    <form id="member-perms-form">
      ${ui.renderPermissionCheckboxes(perms, ui.GROUP_MEMBER_PERMISSION_LABELS)}
      <div class="btn-row"><button class="btn-primary" type="submit">Save group permissions</button></div>
    </form>
  `;
  setModalOpen(els.memberPermsModal, true);

  document.getElementById("member-perms-form").addEventListener("submit", async (event) => {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    try {
      await ui.api(`/groups/${activeGroupId}/members/${employeeId}/permissions?role=employee`, {
        method: "PUT",
        body: JSON.stringify(ui.readPermissionForm(form, ui.GROUP_MEMBER_PERMISSION_LABELS, perms)),
      });
      ui.toast("Member permissions saved");
      closeMemberPermsModal();
    } catch (err) {
      ui.toast(err.message, true);
    }
  });
}

async function refreshAll() {
  await loadProfile();
  await refreshGroups();
  await refreshVacations();
}

function fillGlobalPermissionCheckboxes(perms) {
  if (!els.permissionsForm) return;
  const defaults = ui.mergePermissions(ui.EMPLOYEE_PERMISSION_LABELS, {
    request_vacation: true,
    edit_own_account: true,
  });
  const source = ui.mergePermissions(ui.EMPLOYEE_PERMISSION_LABELS, perms || defaults);
  const host = els.permissionsForm.querySelector("[data-permission-checkboxes]");
  if (!host) return;
  host.innerHTML = ui.renderPermissionCheckboxes(source, ui.EMPLOYEE_PERMISSION_LABELS);
}

els.settingsBack?.addEventListener("click", settingsGoBack);
els.settingsPanel?.querySelectorAll("[data-settings-open]").forEach((btn) => {
  btn.addEventListener("click", () => {
    const view = btn.dataset.settingsOpen;
    if (view === "groups") {
      prepareGroupsSettingsView()
        .then(() => showSettingsView("groups"))
        .catch((err) => ui.toast(err.message, true));
      return;
    }
    showSettingsView(view);
  });
});

els.openVacationsPanel?.addEventListener("click", openVacationsPanel);
els.openApprovalsPanel?.addEventListener("click", () => {
  if (!canApproveVacations) return;
  openApprovalsPanel();
});
els.openSettingsPanel?.addEventListener("click", () => openSettingsPanel("root"));

els.empGroupsSearch?.addEventListener("input", renderEmpGroupsList);
els.empEmployeesSearch?.addEventListener("input", renderEmpEmployeesList);
els.empToggleCreateGroup?.addEventListener("click", () => {
  els.empCreateGroupCard?.classList.toggle("hidden");
  if (!els.empCreateGroupCard?.classList.contains("hidden")) {
    document.getElementById("emp-new-group-name")?.focus();
  }
});
els.empCreateGroupForm?.addEventListener("submit", async (event) => {
  event.preventDefault();
  const form = new FormData(event.currentTarget);
  try {
    const group = await ui.api(`/organizations/${orgId}/groups?role=employee`, {
      method: "POST",
      body: JSON.stringify({ name: String(form.get("name") || "").trim() }),
    });
    ui.toast("Group created");
    closeCreateGroupForm();
    await refreshGroups();
    await openMyGroupDetail(group.id);
  } catch (err) {
    ui.toast(err.message, true);
  }
});

els.colorMatchGroups?.addEventListener("change", () => {
  setColorMatchGroups(els.colorMatchGroups.checked);
});
if (els.colorMatchGroups) els.colorMatchGroups.checked = colorMatchGroups;

els.vacationsPanel?.querySelectorAll("[data-close-vacations-panel]").forEach((el) => {
  el.addEventListener("click", () => setPanelOpen(els.vacationsPanel, false));
});
els.approvalsPanel?.querySelectorAll("[data-close-approvals-panel]").forEach((el) => {
  el.addEventListener("click", () => setPanelOpen(els.approvalsPanel, false));
});
els.settingsPanel?.querySelectorAll("[data-close-settings]").forEach((el) => {
  el.addEventListener("click", () => {
    activeGroupId = null;
    participantsFilter = "";
    showSettingsView("root");
    setPanelOpen(els.settingsPanel, false);
  });
});
document.querySelectorAll("[data-close-group-edit]").forEach((el) => {
  el.addEventListener("click", closeGroupEditModal);
});
document.querySelectorAll("[data-close-employee-perms]").forEach((el) => {
  el.addEventListener("click", closeEmployeePermsModal);
});
document.querySelectorAll("[data-close-member-perms]").forEach((el) => {
  el.addEventListener("click", closeMemberPermsModal);
});

els.timelineToday?.addEventListener("click", () => {
  timelineWindowOffset = 0;
  skipScrollToTodayOnce = false;
  if (lastTimelineEmployees.length) {
    renderTimeline(lastTimelineEmployees, lastTimelineVacations || []);
  } else {
    scrollTimelineToToday(true);
  }
});
els.timelinePrev?.addEventListener("click", () => shiftTimeline(-TIMELINE_SCROLL_DAYS));
els.timelineNext?.addEventListener("click", () => shiftTimeline(TIMELINE_SCROLL_DAYS));
els.timelinePane?.addEventListener("scroll", onTimelinePaneScroll, { passive: true });

(function bindTimelineVerticalSync() {
  const names = document.querySelector(".timeline-names");
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

els.openVacationRequest?.addEventListener("click", () => {
  openVacationModal();
  els.vacationStart?.focus();
});

els.vacationModalClose?.addEventListener("click", closeVacationModal);
els.vacationModal?.querySelectorAll("[data-close-vacation-modal]").forEach((el) => {
  el.addEventListener("click", closeVacationModal);
});

els.vacationDetailTabs.forEach((tab) => {
  tab.addEventListener("click", () => showDetailTab(tab.dataset.detailTab));
});

els.vacationDetailModal?.querySelectorAll("[data-close-vacation-detail]").forEach((el) => {
  el.addEventListener("click", closeVacationDetailModal);
});

document.addEventListener("keydown", (event) => {
  if (event.key !== "Escape") return;
  if (els.memberPermsModal && !els.memberPermsModal.classList.contains("hidden")) {
    closeMemberPermsModal();
    return;
  }
  if (els.employeePermsModal && !els.employeePermsModal.classList.contains("hidden")) {
    closeEmployeePermsModal();
    return;
  }
  if (els.groupEditModal && !els.groupEditModal.classList.contains("hidden")) {
    closeGroupEditModal();
    return;
  }
  if (els.vacationDetailModal && !els.vacationDetailModal.classList.contains("hidden")) {
    closeVacationDetailModal();
    return;
  }
  if (els.vacationModal && !els.vacationModal.classList.contains("hidden")) {
    closeVacationModal();
    return;
  }
  if (els.settingsPanel && !els.settingsPanel.classList.contains("hidden")) {
    settingsGoBack();
    return;
  }
  if (els.vacationsPanel && !els.vacationsPanel.classList.contains("hidden")) {
    setPanelOpen(els.vacationsPanel, false);
    return;
  }
  if (els.approvalsPanel && !els.approvalsPanel.classList.contains("hidden")) {
    setPanelOpen(els.approvalsPanel, false);
  }
});

window.addEventListener("resize", () => {
  cancelAnimationFrame(resizeFrame);
  resizeFrame = requestAnimationFrame(handleTimelineResize);
});

els.vacationStart?.addEventListener("change", syncSelectionFromInputs);
els.vacationEnd?.addEventListener("change", syncSelectionFromInputs);

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
        role: "employee",
      }),
    });
    updateChrome();
    await refreshAll();
    ui.toast("Signed in as employee");
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

els.logoutBtn?.addEventListener("click", async () => {
  try {
    await ui.logout(ROLE);
  } catch {
    /* ignore */
  }
  session = null;
  profile = null;
  showLoginForm();
  updateChrome();
  ui.toast("Signed out");
});

els.vacationForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  const form = new FormData(els.vacationForm);
  const startIso = form.get("start");
  const endIso = form.get("end");
  const hint = document.getElementById("vacation-date-hint");

  try {
    ui.assertVacationRange(startIso, endIso);
    if (hint) hint.classList.add("hidden");
  } catch (err) {
    if (hint) {
      hint.textContent = err.message;
      hint.classList.remove("hidden");
    }
    ui.toast(err.message, true);
    return;
  }

  const start = ui.dateToParts(startIso);
  const end = ui.dateToParts(endIso);
  try {
    const created = await ui.api("/vacations", {
      method: "POST",
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
    const autoApproved =
      created &&
      (String(created.status || "").includes("Approved") ||
        String(created.status || "").toLowerCase() === "approved");
    ui.toast(autoApproved ? "Vacation approved (no approver configured)" : "Vacation request submitted");
    els.vacationForm.reset();
    if (els.vacationDetails) els.vacationDetails.value = "";
    clearSelectionRange();
    closeVacationModal();
    await refreshVacations();
  } catch (err) {
    ui.toast(err.message, true);
  }
});

els.accountAvatarBtn?.addEventListener("click", () => openAvatarPicker());

els.togglePasswordChange?.addEventListener("click", () => {
  setPasswordChangeOpen(!passwordChangeOpen);
});

els.accountForm?.addEventListener("input", updateAccountSaveBar);
els.accountForm?.addEventListener("change", updateAccountSaveBar);

els.accountAvatarInput?.addEventListener("change", async () => {
  const file = els.accountAvatarInput?.files?.[0];
  if (!file || !els.accountAvatar) return;
  try {
    const previewUrl = await fileToAvatarDataUrl(file);
    setAvatarElement(els.accountAvatar, profile?.username || "?", previewUrl);
    els.accountRemoveAvatar?.classList.remove("hidden");
    updateAccountSaveBar();
  } catch (err) {
    ui.toast(err.message, true);
  }
});

els.accountForm?.addEventListener("submit", async (event) => {
  event.preventDefault();
  if (!profile?.permissions?.edit_own_account) return;
  const form = new FormData(els.accountForm);
  const password = passwordChangeOpen ? String(form.get("password") || "").trim() : "";
  const passwordEmail = passwordChangeOpen
    ? String(form.get("password_email") || "").trim()
    : "";
  if (password && !passwordEmail) {
    ui.toast("Enter an email for password change confirmation", true);
    return;
  }
  const avatarFile = els.accountAvatarInput?.files?.[0];
  try {
    await ui.api("/employees/me", {
      method: "PUT",
      body: JSON.stringify({
        username: String(form.get("username") || "").trim(),
        email: String(form.get("email") || "").trim(),
        password: password || null,
        password_change_email: password ? passwordEmail || null : null,
      }),
    });
    if (avatarFile) {
      const avatarUrl = await fileToAvatarDataUrl(avatarFile);
      await ui.api("/employees/me/avatar", {
        method: "PUT",
        body: JSON.stringify({ avatar_url: avatarUrl }),
      });
    }
    ui.toast(password ? "Details updated · password change email sent" : "Details updated");
    await refreshAll();
  } catch (err) {
    ui.toast(err.message, true);
  }
});

els.accountRemoveAvatar?.addEventListener("click", async () => {
  if (!profile?.permissions?.edit_own_account) return;
  try {
    await ui.api("/employees/me/avatar", {
      method: "PUT",
      body: JSON.stringify({ avatar_url: null }),
    });
    if (els.accountAvatarInput) els.accountAvatarInput.value = "";
    ui.toast("Profile picture removed");
    await refreshAll();
  } catch (err) {
    ui.toast(err.message, true);
  }
});

els.permissionsForm?.addEventListener("submit", async (event) => {
  event.preventDefault();
  const form = new FormData(els.permissionsForm);
  const employeeId = form.get("employee_id");
  if (!employeeId) {
    ui.toast("Employee not selected", true);
    return;
  }
  const baseline = selectedEmployeePerms || {};
  const nextPerms = ui.readPermissionForm(form, ui.EMPLOYEE_PERMISSION_LABELS, baseline);
  try {
    await ui.api(`/employees/${employeeId}/permissions?role=employee`, {
      method: "PUT",
      body: JSON.stringify(nextPerms),
    });
    ui.toast("Permissions updated");
    closeEmployeePermsModal();
    selectedEmployeePerms = { ...baseline, ...nextPerms };
    await refreshAll();
  } catch (err) {
    ui.toast(err.message, true);
  }
});

ui.bindVacationDateRange("vacation-start", "vacation-end", "vacation-date-hint");

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
