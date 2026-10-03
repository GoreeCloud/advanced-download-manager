#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");

const root = path.resolve(__dirname, "..");
const EXTENSION_ID = "download-manager@goreecloud.com";

function event() {
  const listeners = [];
  return {
    listeners,
    addListener(fn) { listeners.push(fn); },
    async emit(...args) { for (const fn of listeners) await fn(...args); }
  };
}

function createHarness() {
  const storage = new Map();
  const downloadItems = new Map();
  const downloadCalls = [];
  const pauseCalls = [];
  const notifications = [];
  const onCreated = event();
  const onChanged = event();
  const onMessage = event();
  let uuid = 1;

  const local = {
    async get(query) {
      if (query == null) return Object.fromEntries(storage.entries());
      if (typeof query === "string") return storage.has(query) ? { [query]: storage.get(query) } : {};
      if (Array.isArray(query)) {
        const out = {};
        for (const key of query) if (storage.has(key)) out[key] = storage.get(key);
        return out;
      }
      const out = {};
      for (const [key, fallback] of Object.entries(query || {})) out[key] = storage.has(key) ? storage.get(key) : fallback;
      return out;
    },
    async set(values) { for (const [key, value] of Object.entries(values)) storage.set(key, value); },
    async remove(keys) { for (const key of Array.isArray(keys) ? keys : [keys]) storage.delete(key); }
  };

  const browser = {
    storage: { local },
    runtime: {
      id: EXTENSION_ID,
      lastError: null,
      onInstalled: event(),
      onStartup: event(),
      onMessage,
      async sendMessage() { return undefined; },
      getURL(value) { return `moz-extension://test/${value}`; },
      connectNative() {
        return {
          onMessage: event(),
          onDisconnect: event(),
          postMessage() {}
        };
      }
    },
    downloads: {
      onCreated,
      onChanged,
      async download(options) {
        downloadCalls.push(options);
        throw new Error("auto-capture must not replay a Firefox-started request");
      },
      async search({ id }) {
        const item = downloadItems.get(id);
        return item ? [{ ...item }] : [];
      },
      async pause(id) {
        pauseCalls.push(id);
        const item = downloadItems.get(id);
        assert(item);
        item.paused = true;
        await onChanged.emit({ id, paused: { current: true } });
      },
      async resume() {},
      async cancel() {}
    },
    notifications: {
      async create(...args) {
        notifications.push(args);
        return String(notifications.length);
      }
    },
    contextMenus: { onClicked: event(), async removeAll() {}, create() {} },
    commands: { onCommand: event() },
    tabs: { async create() {} },
    permissions: { async contains() { return false; }, async request() { return false; } },
    cookies: { async getAll() { return []; } }
  };

  const context = vm.createContext({
    browser,
    URL,
    console,
    setTimeout,
    clearTimeout,
    Promise,
    Date,
    performance: { now: () => Date.now() },
    crypto: { randomUUID: () => `captured-job-${uuid++}` }
  });
  context.globalThis = context;

  for (const file of ["background.js", "scheduler_hardening.js"]) {
    vm.runInContext(fs.readFileSync(path.join(root, file), "utf8"), context, { filename: file });
  }

  const messageListener = onMessage.listeners.at(-1);
  assert.equal(typeof messageListener, "function");
  assert.equal(onCreated.listeners.length, 1, "Firefox download creation listener must be registered");

  async function message(value) { return messageListener(value, {}); }
  async function settle() { for (let i = 0; i < 12; i += 1) await new Promise((resolve) => setTimeout(resolve, 0)); }
  async function jobs() { await settle(); return message({ type: "list-jobs" }); }

  async function createFirefoxDownload(item) {
    downloadItems.set(item.id, { ...item });
    await onCreated.emit({ ...item });
    await settle();
  }

  return {
    browser,
    storage,
    downloadItems,
    downloadCalls,
    pauseCalls,
    notifications,
    message,
    jobs,
    settle,
    createFirefoxDownload
  };
}

async function main() {
  const h = createHarness();

  await h.createFirefoxDownload({
    id: 42,
    url: "https://downloads.example.test/archive.zip?token=short-lived",
    filename: "/home/test/Downloads/archive.zip",
    state: "in_progress",
    paused: false,
    bytesReceived: 4096,
    totalBytes: 8192,
    startTime: "2026-10-03T18:00:00.000Z"
  });

  let jobs = await h.jobs();
  assert.equal(jobs.length, 1, "ordinary Firefox download should be adopted automatically");
  let adopted = jobs[0];
  assert.equal(adopted.downloadId, 42);
  assert.equal(adopted.url, "https://downloads.example.test/archive.zip?token=short-lived");
  assert.equal(adopted.engine, "browser", "captured request must stay on Firefox's original engine");
  assert.equal(adopted.native, false);
  assert.equal(adopted.autoCaptured, true);
  assert.equal(adopted.acquisition, "firefox-auto-capture");
  assert.equal(h.downloadCalls.length, 0, "capture must not replay the request");

  await h.message({ type: "pause-job", id: adopted.id });
  await h.settle();
  assert.deepEqual(h.pauseCalls, [42], "adopted download must remain controllable through Firefox's download ID");

  await h.createFirefoxDownload({
    id: 43,
    byExtensionId: EXTENSION_ID,
    url: "https://downloads.example.test/extension-owned.zip",
    filename: "/home/test/Downloads/extension-owned.zip",
    state: "in_progress",
    paused: false,
    bytesReceived: 0,
    totalBytes: 1024,
    startTime: "2026-10-03T18:01:00.000Z"
  });
  jobs = await h.jobs();
  assert.equal(jobs.length, 1, "extension-started downloads must not be duplicated by auto-capture");

  await h.createFirefoxDownload({
    id: 44,
    url: "blob:https://example.test/2bdc",
    filename: "/home/test/Downloads/blob.bin",
    state: "in_progress",
    paused: false,
    bytesReceived: 0,
    totalBytes: 128,
    startTime: "2026-10-03T18:02:00.000Z"
  });
  jobs = await h.jobs();
  assert.equal(jobs.length, 1, "non-HTTP(S) downloads must remain outside automatic capture");

  await h.message({
    type: "save-settings",
    settings: {
      mode: "native",
      segments: 8,
      maxConcurrent: 3,
      retryCount: 3,
      nativeDirectory: "",
      forwardCookies: false,
      completionNotifications: true,
      captureFirefoxDownloads: false
    }
  });
  await h.createFirefoxDownload({
    id: 45,
    url: "https://downloads.example.test/disabled.zip",
    filename: "/home/test/Downloads/disabled.zip",
    state: "in_progress",
    paused: false,
    bytesReceived: 0,
    totalBytes: 2048,
    startTime: "2026-10-03T18:03:00.000Z"
  });
  jobs = await h.jobs();
  assert.equal(jobs.length, 1, "user opt-out must prevent automatic adoption");

  await h.message({
    type: "save-settings",
    settings: {
      mode: "native",
      segments: 8,
      maxConcurrent: 3,
      retryCount: 3,
      nativeDirectory: "",
      forwardCookies: false,
      completionNotifications: true,
      captureFirefoxDownloads: true
    }
  });
  await h.createFirefoxDownload({
    id: 46,
    url: "https://downloads.example.test/native-setting.zip",
    filename: "/home/test/Downloads/native-setting.zip",
    state: "in_progress",
    paused: false,
    bytesReceived: 0,
    totalBytes: 4096,
    startTime: "2026-10-03T18:04:00.000Z"
  });
  jobs = await h.jobs();
  const nativeSettingCapture = jobs.find((job) => job.downloadId === 46);
  assert(nativeSettingCapture, "capture should remain enabled when native mode is selected");
  assert.equal(nativeSettingCapture.engine, "browser", "automatic capture must not convert an already-started request into a native replay");
  assert.equal(nativeSettingCapture.native, false);
  assert.equal(h.downloadCalls.length, 0);

  console.log("FIREFOX AUTOMATIC DOWNLOAD CAPTURE: PASS");
  console.log("- default-on ordinary HTTP/HTTPS adoption: PASS");
  console.log("- no request cancellation or replay: PASS");
  console.log("- extension-owned download de-duplication: PASS");
  console.log("- non-HTTP(S) exclusion: PASS");
  console.log("- explicit user opt-out: PASS");
  console.log("- native-mode capture remains Firefox-engine safe: PASS");
  console.log("- adopted download control by original Firefox ID: PASS");
}

main().catch((error) => {
  console.error(error.stack || error);
  process.exitCode = 1;
});
