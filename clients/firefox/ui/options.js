const $ = (selector) => document.querySelector(selector);

const COOKIE_PERMISSION = Object.freeze({
  permissions: ["cookies"],
  origins: ["<all_urls>"]
});

let cookiePermissionGranted = false;
let permissionBusy = false;

function nativeModeSelected() {
  return $("#mode").value === "native";
}

function updateNativeUi() {
  const nativeMode = nativeModeSelected();
  const forwardingRequested = $("#forwardCookies").checked;

  for (const id of ["segments", "retryCount", "nativeDirectory", "forwardCookies"]) {
    $("#" + id).disabled = !nativeMode;
  }

  $("#nativeDetails").hidden = !nativeMode;
  $("#nativeSummary").hidden = nativeMode;
  $("#nativeState").textContent = nativeMode ? "On" : "Off";
  $("#nativeState").classList.toggle("active", nativeMode);

  const showGrant = nativeMode && forwardingRequested && !cookiePermissionGranted;
  const showRevoke = cookiePermissionGranted;
  $("#permissionActions").hidden = !(showGrant || showRevoke);
  $("#grantCookies").hidden = !showGrant;
  $("#grantCookies").disabled = permissionBusy || !showGrant;
  $("#revokeCookies").hidden = !showRevoke;
  $("#revokeCookies").disabled = permissionBusy || !showRevoke;
  $("#test").disabled = permissionBusy || !nativeMode;

  if (cookiePermissionGranted) {
    $("#cookieStatus").textContent = nativeMode && forwardingRequested
      ? "Cookie access granted."
      : "Cookie access is granted but not currently in use.";
  } else if (!forwardingRequested) {
    $("#cookieStatus").textContent = "Cookie forwarding is off.";
  } else {
    $("#cookieStatus").textContent = "Allow cookie access before saving authenticated native downloads.";
  }
}

async function refreshCookiePermission() {
  cookiePermissionGranted = await browser.permissions.contains(COOKIE_PERMISSION);
  updateNativeUi();
  return cookiePermissionGranted;
}

async function load() {
  const settings = await browser.runtime.sendMessage({ type: "get-settings" });
  for (const key of ["mode", "segments", "maxConcurrent", "retryCount", "nativeDirectory"]) {
    $("#" + key).value = settings[key] ?? "";
  }
  $("#forwardCookies").checked = Boolean(settings.forwardCookies);
  $("#completionNotifications").checked = settings.completionNotifications !== false;
  $("#captureFirefoxDownloads").checked = settings.captureFirefoxDownloads !== false;
  await refreshCookiePermission();
}

$("#mode").addEventListener("change", () => {
  updateNativeUi();
  $("#status").textContent = nativeModeSelected()
    ? "Native acceleration settings are available. Save changes to apply this engine."
    : "Firefox downloads selected.";
});

$("#forwardCookies").addEventListener("change", () => {
  updateNativeUi();
  $("#status").textContent = $("#forwardCookies").checked
    ? "Cookie forwarding selected. Allow cookie access before saving if permission has not already been granted."
    : "Cookie forwarding is off.";
});

$("#grantCookies").addEventListener("click", () => {
  // Firefox requires permissions.request() to run directly from a user-action
  // handler. Keep the request synchronous with this deliberate click.
  const nativeMode = nativeModeSelected();
  const forwardingRequested = $("#forwardCookies").checked;
  if (!nativeMode || !forwardingRequested) {
    updateNativeUi();
    $("#status").textContent = nativeMode
      ? "Turn on cookie forwarding before requesting website-data access."
      : "Select the native segmented helper before requesting website-data access.";
    return;
  }

  let request;
  try {
    permissionBusy = true;
    updateNativeUi();
    $("#status").textContent = "Waiting for Firefox permission decision…";
    request = browser.permissions.request(COOKIE_PERMISSION);
  } catch (error) {
    permissionBusy = false;
    updateNativeUi();
    $("#status").textContent = `Permission request failed: ${error.message || String(error)}`;
    return;
  }

  request.then((granted) => {
    cookiePermissionGranted = Boolean(granted);
    $("#status").textContent = granted
      ? "Cookie access granted. Save changes to enable forwarding."
      : "Firefox did not grant the optional cookie permission.";
  }).catch((error) => {
    $("#status").textContent = `Permission request failed: ${error.message || String(error)}`;
  }).finally(() => {
    permissionBusy = false;
    refreshCookiePermission().catch(() => updateNativeUi());
  });
});

$("#revokeCookies").addEventListener("click", () => {
  permissionBusy = true;
  updateNativeUi();
  $("#status").textContent = "Removing optional cookie access…";

  browser.permissions.remove(COOKIE_PERMISSION).then((removed) => {
    if (removed) {
      cookiePermissionGranted = false;
      $("#forwardCookies").checked = false;
      $("#status").textContent = "Cookie access revoked. Save changes to keep cookie forwarding off.";
    } else {
      $("#status").textContent = "Cookie access was already absent.";
    }
  }).catch((error) => {
    $("#status").textContent = `Permission removal failed: ${error.message || String(error)}`;
  }).finally(() => {
    permissionBusy = false;
    refreshCookiePermission().catch(() => updateNativeUi());
  });
});

$("#save").addEventListener("click", async () => {
  $("#save").disabled = true;
  try {
    const mode = $("#mode").value;
    const forwardCookies = $("#forwardCookies").checked;
    if (mode === "native" && forwardCookies && !(await refreshCookiePermission())) {
      $("#status").textContent = "Cookie forwarding was not saved. Turn it on, choose Allow cookie access, approve Firefox's prompt, then save again.";
      return;
    }

    const settings = {
      mode,
      segments: Number($("#segments").value),
      maxConcurrent: Number($("#maxConcurrent").value),
      retryCount: Number($("#retryCount").value),
      nativeDirectory: $("#nativeDirectory").value.trim(),
      forwardCookies,
      completionNotifications: $("#completionNotifications").checked,
      captureFirefoxDownloads: $("#captureFirefoxDownloads").checked
    };
    await browser.runtime.sendMessage({ type: "save-settings", settings });
    $("#status").textContent = "Changes saved.";
    setTimeout(() => { $("#status").textContent = ""; }, 2200);
  } finally {
    $("#save").disabled = false;
  }
});

$("#test").addEventListener("click", async () => {
  $("#test").disabled = true;
  try {
    const result = await browser.runtime.sendMessage({ type: "native-status" });
    if (result.available && result.compatible !== false) {
      const version = result.helperVersion || "unknown";
      const protocol = result.protocolVersion ?? "unknown";
      $("#status").textContent = `Native helper ${version} · protocol ${protocol} ready.`;
    } else {
      $("#status").textContent = result.error || "Native helper unavailable. Install or repair the current native host first.";
    }
  } finally {
    updateNativeUi();
  }
});

browser.permissions.onAdded.addListener(() => {
  refreshCookiePermission().catch(() => {});
});

browser.permissions.onRemoved.addListener(() => {
  refreshCookiePermission().catch(() => {});
});

load();
