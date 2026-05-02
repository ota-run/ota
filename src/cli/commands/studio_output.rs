//                █████
//               ░░███
//       ██████  ███████    ██████
//      ███░░███░░░███░    ░░░░░███
//     ░███ ░███  ░███      ███████
//     ░███ ░███  ░███ ███ ███░░███
//     ░░██████   ░░█████ ░░████████
//      ░░░░░░     ░░░░░   ░░░░░░░░
//
//   Copyright (C) 2026 — 2026, Ota. All Rights Reserved.
//
//   DO NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.
//
//   Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.
//   You may not use this file except in compliance with that License.
//   Unless required by applicable law or agreed to in writing, software distributed under the
//   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
//   either express or implied. See the License for the specific language governing permissions
//   and limitations under the License.
//
//   If you need additional information or have any questions, please email: os@ota.run

use serde_json::Value as JsonValue;

pub(crate) fn render_studio_snapshot_html(snapshot: &JsonValue) -> String {
    let snapshot_json = serde_json::to_string(snapshot)
        .expect("serializing studio snapshot should not fail")
        .replace("</", "<\\/");
    let template = r##"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Ota Studio</title>
    <style>
      :root {
        --bg: #f4efe6;
        --panel: #fffaf1;
        --panel-strong: #fffdf8;
        --ink: #1f262d;
        --muted: #66717d;
        --line: #d8c8ae;
        --accent: #235c87;
        --accent-soft: #dce9f3;
        --success: #1d7c62;
        --warning: #996d18;
        --error: #a4372f;
        --shadow: 0 18px 42px rgba(44, 35, 20, 0.08);
      }

      * { box-sizing: border-box; }
      body {
        margin: 0;
        font-family: "SF Pro Text", "Avenir Next", "Segoe UI", sans-serif;
        color: var(--ink);
        background:
          radial-gradient(circle at top left, rgba(35, 92, 135, 0.10), transparent 28rem),
          linear-gradient(180deg, #fbf6ee 0%, var(--bg) 100%);
      }

      .shell {
        max-width: 1280px;
        margin: 0 auto;
        padding: 40px 24px 72px;
      }

      .hero {
        display: grid;
        gap: 16px;
        padding: 28px 28px 24px;
        border: 1px solid var(--line);
        border-radius: 28px;
        background: linear-gradient(180deg, rgba(255,255,255,0.92), rgba(255,250,241,0.92));
        box-shadow: var(--shadow);
      }

      .eyebrow {
        font-size: 12px;
        letter-spacing: 0.18em;
        text-transform: uppercase;
        color: var(--muted);
      }

      h1 {
        margin: 0;
        font-size: clamp(2rem, 4vw, 3.2rem);
        line-height: 1.02;
      }

      .hero-subtitle {
        margin: 0;
        max-width: 52rem;
        color: var(--muted);
        font-size: 1rem;
        line-height: 1.6;
      }

      .hero-meta {
        display: flex;
        flex-wrap: wrap;
        gap: 12px;
      }

      .chip {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        padding: 10px 14px;
        border-radius: 999px;
        border: 1px solid var(--line);
        background: rgba(255,255,255,0.72);
        color: var(--muted);
        font-size: 0.95rem;
      }

      .chip strong { color: var(--ink); }

      .cards {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(210px, 1fr));
        gap: 14px;
        margin-top: 20px;
      }

      .card {
        padding: 18px 18px 16px;
        border-radius: 22px;
        border: 1px solid var(--line);
        background: var(--panel-strong);
        box-shadow: var(--shadow);
      }

      .card-label {
        color: var(--muted);
        font-size: 0.82rem;
        text-transform: uppercase;
        letter-spacing: 0.12em;
      }

      .card-value {
        margin-top: 8px;
        font-size: 1.8rem;
        font-weight: 700;
      }

      .grid {
        display: grid;
        grid-template-columns: 1.15fr 0.85fr;
        gap: 18px;
        margin-top: 18px;
      }

      .panel {
        min-width: 0;
        padding: 22px;
        border-radius: 24px;
        border: 1px solid var(--line);
        background: var(--panel);
        box-shadow: var(--shadow);
      }

      .panel h2 {
        margin: 0 0 12px;
        font-size: 1.15rem;
      }

      .status {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        padding: 8px 12px;
        border-radius: 999px;
        font-size: 0.92rem;
        font-weight: 600;
      }

      .status-ready { background: rgba(29,124,98,0.12); color: var(--success); }
      .status-risky { background: rgba(153,109,24,0.14); color: var(--warning); }
      .status-blocked { background: rgba(164,55,47,0.12); color: var(--error); }

      .stack { display: grid; gap: 14px; }

      .finding, .list-item {
        padding: 14px 16px;
        border-radius: 18px;
        border: 1px solid rgba(216, 200, 174, 0.9);
        background: rgba(255, 255, 255, 0.62);
      }

      .note {
        padding: 16px 18px;
        border-radius: 18px;
        border: 1px solid rgba(35, 92, 135, 0.18);
        background: rgba(220, 233, 243, 0.42);
        color: var(--ink);
      }

      .finding-title, .list-title {
        font-weight: 700;
        margin: 0 0 6px;
      }

      .finding-body, .list-meta {
        margin: 0;
        color: var(--muted);
        line-height: 1.55;
        font-size: 0.95rem;
      }

      .list-inline {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
        margin-top: 8px;
      }

      .pill {
        padding: 6px 10px;
        border-radius: 999px;
        background: var(--accent-soft);
        color: var(--accent);
        font-size: 0.86rem;
      }

      pre {
        margin: 0;
        padding: 16px;
        border-radius: 20px;
        border: 1px solid rgba(216, 200, 174, 0.9);
        background: #fcfaf6;
        color: #203040;
        overflow: auto;
        font: 13px/1.55 "SFMono-Regular", "JetBrains Mono", "Menlo", monospace;
      }

      .muted { color: var(--muted); }

      .detail-grid {
        display: grid;
        gap: 12px;
        margin-top: 12px;
      }

      .detail-row {
        display: grid;
        gap: 6px;
      }

      .detail-label {
        color: var(--muted);
        font-size: 0.8rem;
        letter-spacing: 0.08em;
        text-transform: uppercase;
      }

      .detail-value {
        color: var(--ink);
        font-size: 0.95rem;
        line-height: 1.55;
      }

      .split {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 14px;
        margin-top: 14px;
      }

      .mini-panel {
        min-width: 0;
        padding: 16px;
        border-radius: 20px;
        border: 1px solid rgba(216, 200, 174, 0.9);
        background: rgba(255, 255, 255, 0.62);
      }

      .mini-title {
        margin: 0 0 10px;
        font-size: 0.92rem;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        color: var(--muted);
      }

      .command-box {
        margin-top: 10px;
        padding: 12px 14px;
        border-radius: 16px;
        border: 1px solid rgba(35, 92, 135, 0.18);
        background: rgba(220, 233, 243, 0.34);
        color: #203040;
        font: 13px/1.45 "SFMono-Regular", "JetBrains Mono", "Menlo", monospace;
        overflow: auto;
      }

      .command-actions {
        display: flex;
        justify-content: flex-end;
        margin-top: 10px;
      }

      .action-button {
        border: 1px solid rgba(35, 92, 135, 0.20);
        background: rgba(255,255,255,0.82);
        color: var(--accent);
        border-radius: 999px;
        padding: 8px 12px;
        font-size: 0.82rem;
        font-weight: 600;
        cursor: pointer;
      }

      .action-button:hover {
        background: rgba(220, 233, 243, 0.52);
      }

      .scan-strip {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
        margin-top: 10px;
      }

      .timeline-strip {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
        margin-top: 10px;
      }

      .toolbar {
        display: flex;
        flex-wrap: wrap;
        gap: 10px;
        margin-top: 10px;
      }

      .toolbar-button {
        border: 1px solid rgba(216, 200, 174, 0.9);
        background: rgba(255,255,255,0.72);
        color: var(--muted);
        border-radius: 999px;
        padding: 7px 12px;
        font-size: 0.82rem;
        font-weight: 600;
        cursor: pointer;
      }

      .toolbar-button.is-active {
        background: var(--accent-soft);
        color: var(--accent);
        border-color: rgba(35, 92, 135, 0.20);
      }

      .timeline-step {
        padding: 6px 10px;
        border-radius: 999px;
        font-size: 0.82rem;
        font-weight: 600;
        border: 1px solid rgba(216, 200, 174, 0.9);
        background: rgba(255, 255, 255, 0.62);
        color: var(--muted);
      }

      .timeline-step-ready {
        background: rgba(29,124,98,0.12);
        color: var(--success);
      }

      .timeline-step-running {
        background: rgba(35, 92, 135, 0.12);
        color: var(--accent);
      }

      .timeline-step-risky {
        background: rgba(153,109,24,0.14);
        color: var(--warning);
      }

      .timeline-step-blocked {
        background: rgba(164,55,47,0.12);
        color: var(--error);
      }

      .subheading {
        margin: 18px 0 8px;
        color: var(--muted);
        font-size: 0.82rem;
        letter-spacing: 0.12em;
        text-transform: uppercase;
      }

      .explain-group {
        padding: 0;
        overflow: hidden;
      }

      .explain-group > summary {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 14px 16px;
      }

      .explain-group-body {
        display: grid;
        gap: 12px;
        padding: 0 16px 16px;
      }

      details {
        margin-top: 12px;
      }

      summary {
        cursor: pointer;
        color: var(--accent);
        font-weight: 600;
      }

      @media (max-width: 980px) {
        .grid {
          grid-template-columns: 1fr;
        }

        .split {
          grid-template-columns: 1fr;
        }
      }
    </style>
  </head>
  <body>
    <div class="shell">
      <section class="hero">
        <div class="eyebrow">Ota Studio</div>
        <h1 id="hero-title">Read-only repo snapshot</h1>
        <p class="hero-subtitle">
          A local Studio preview built from Ota’s canonical read surfaces. The snapshot stays contract-native:
          no parallel model, no hidden execution logic, and no writes back into the repo.
        </p>
        <div class="hero-meta" id="hero-meta"></div>
      </section>

      <section class="cards" id="summary-cards"></section>

      <section class="grid">
        <div class="stack">
          <article class="panel">
            <h2>Scope</h2>
            <div class="note">
              Repo-first and read-only. This snapshot helps inspect readiness, inferred contract data,
              and declared topology visually. Workspace coverage should arrive later as an explicit
              `ota workspace studio` surface rather than quietly overloading repo Studio.
            </div>
          </article>

          <article class="panel">
            <h2>Readiness</h2>
            <div id="doctor-status"></div>
            <div class="stack" id="doctor-findings" style="margin-top:14px;"></div>
          </article>

          <article class="panel">
            <h2>Contract Review</h2>
            <p class="muted" id="review-summary"></p>
            <div class="cards" id="review-cards" style="margin-top:14px;"></div>
            <div class="split" id="review-contracts"></div>
            <div class="stack" id="review-diff" style="margin-top:14px;"></div>
            <div class="split" id="review-outcomes" style="margin-top:14px;"></div>
            <div class="stack" id="review-apply" style="margin-top:14px;"></div>
            <details>
              <summary>Show raw detect payload</summary>
              <pre id="detect-config" style="margin-top:12px;"></pre>
            </details>
          </article>
        </div>

        <div class="stack">
          <article class="panel">
            <h2>Recent Activity</h2>
            <div class="stack" id="activity-list"></div>
          </article>

          <article class="panel">
            <h2>Why</h2>
            <div class="stack" id="explain-list"></div>
          </article>

          <article class="panel">
            <h2>Topology</h2>
            <div class="stack" id="topology-summary"></div>
          </article>

          <article class="panel">
            <h2>Tasks</h2>
            <div class="stack" id="task-list"></div>
          </article>

          <article class="panel">
            <h2>Services</h2>
            <div class="stack" id="service-list"></div>
          </article>

          <article class="panel">
            <h2>Shared Backends</h2>
            <div class="stack" id="shared-backend-list"></div>
          </article>
        </div>
      </section>
    </div>

    <script id="ota-studio-data" type="application/json">__OTA_STUDIO_SNAPSHOT__</script>
    <script>
      const initialSnapshot = JSON.parse(document.getElementById("ota-studio-data").textContent);
      let currentServedMode = false;
      let currentActionBaseUrl = "";
      let currentSnapshotSignature = "";
      let currentActivityFilter = "all";
      let snapshotPollTimer = null;
      let snapshotPollInFlight = false;

      const text = (value, fallback = "—") => {
        if (value === undefined || value === null || value === "") return fallback;
        return String(value);
      };

      const pretty = (value) => JSON.stringify(value, null, 2);
      const count = (value) => Array.isArray(value) ? value.length : 0;
      const renderPills = (items) => items.filter(Boolean).map((item) => `<span class="pill">${item}</span>`).join("");
      const renderDetailRow = (label, value) => `
        <div class="detail-row">
          <div class="detail-label">${label}</div>
          <div class="detail-value">${value}</div>
        </div>
      `;
      const renderCommandBox = (value, buttonLabel = "Copy command", action = null, actionLabel = "Apply in Studio") => `
        <div class="command-box">${text(value)}</div>
        <div class="command-actions">
          ${currentServedMode && action ? `<button class="action-button" type="button" data-studio-action="${action}">${actionLabel}</button>` : ""}
          <button class="action-button" type="button" data-copy-command="${text(value).replace(/"/g, "&quot;")}">${buttonLabel}</button>
        </div>
      `;
      const renderCopyButtons = (items) => `
        <div class="command-actions" style="justify-content:flex-start;gap:8px;flex-wrap:wrap;">
          ${items.filter((item) => item && item.value).map((item) => `
            <button class="action-button" type="button" data-copy-command="${text(item.value).replace(/"/g, "&quot;")}">${text(item.label)}</button>
          `).join("")}
        </div>
      `;
      const renderContractPreview = (title, value, emptyCopy) => `
        <article class="mini-panel">
          <p class="mini-title">${title}</p>
          ${value ? `<pre>${value}</pre>` : `<p class="muted">${emptyCopy}</p>`}
        </article>
      `;

      const activationWhy = (mode) => {
        switch (mode) {
          case "manual":
            return "Ota resolves the target but will not auto-start the producer.";
          case "ensure_started":
            return "Ota may start the producer and return once startup is handed off.";
          case "restart_ready":
            return "Ota may bounce a reachable producer, then wait for it to become ready again.";
          case "ensure_running":
            return "Ota waits for the declared target listener to become reachable.";
          case "ensure_ready":
            return "Ota waits for the stronger declared readiness signal when one exists.";
          default:
            return `Activation mode: ${text(mode)}.`;
        }
      };

      const addressViewWhy = (view) => {
        switch (view) {
          case "host":
            return "This target follows the producer's published host-facing address.";
          case "topology":
            return "This target follows the producer's truthful shared-topology address.";
          case "internal":
            return "This target follows the producer's in-backend bind endpoint.";
          default:
            return `Address view: ${text(view)}.`;
        }
      };

      const listenerWhy = (listener) => {
        const hostProjection = listener.host_projection;
        if (hostProjection?.primary) {
          return "This listener projects the primary host-facing endpoint that downstream tasks and humans are most likely to use.";
        }
        if (hostProjection) {
          return "This listener projects a host-facing endpoint from the declared runtime bind.";
        }
        return "This listener stays internal to the declared runtime and is not projected back to the host.";
      };

      const targetWhy = (target) => {
        const service = target.service || null;
        if (!service) {
          return "This target stays manual. Ota can pass the value through, but it will not derive or auto-start a producer from a raw URL target.";
        }
        const overrideWhy = target.override_input
          ? ` If \`${text(target.override_input)}\` is set, that explicit input still wins.`
          : "";
        return `${addressViewWhy(service.address_view)} ${activationWhy(target.activation_mode)}${overrideWhy}`;
      };

      const taskWhy = (task) => {
        const runtime = task.runtime || null;
        const readiness = runtime?.readiness || null;
        const backendWhy = runtime?.backend_binding
          ? `Runtime work is anchored to shared backend \`${text(runtime.backend_binding)}\`.`
          : runtime
            ? `Runtime kind is \`${text(runtime.kind)}\` with no shared backend binding.`
            : "No runtime block is declared for this task.";
        const readinessWhy = readiness
          ? ` Readiness is checked through \`${text(readiness.kind)}\`${readiness.listener ? ` on \`${text(readiness.listener)}\`` : ""}.`
          : " No readiness contract is declared.";
        return `${backendWhy}${readinessWhy}`;
      };

      const badgeClass = (doctor) => {
        const verdict = text(doctor.summary?.verdict, "unknown");
        if (verdict === "ready") return "status status-ready";
        if (verdict === "risky") return "status status-risky";
        return "status status-blocked";
      };

      const activityStatusClass = (entry) => {
        const status = text(entry.status, entry.ok ? "READY" : "NOT READY").toLowerCase();
        if (entry.ok && status.includes("ready")) return "status status-ready";
        if (status.includes("warn") || status.includes("risky")) return "status status-risky";
        return "status status-blocked";
      };

      const activityStepClass = (status) => {
        const normalized = text(status, "").toLowerCase();
        if (normalized.includes("ready") || normalized.includes("success")) return "timeline-step timeline-step-ready";
        if (normalized.includes("running") || normalized.includes("starting") || normalized.includes("planned")) return "timeline-step timeline-step-running";
        if (normalized.includes("warn") || normalized.includes("risky")) return "timeline-step timeline-step-risky";
        if (normalized.includes("fail") || normalized.includes("blocked") || normalized.includes("not ready") || normalized.includes("interrupted")) {
          return "timeline-step timeline-step-blocked";
        }
        return "timeline-step";
      };

      const activityReadinessLabel = (entry) => {
        if (entry.ok) {
          return "Receipt archived this run as ready.";
        }
        if (entry.status) {
          return `Receipt archived this run as ${String(entry.status).toLowerCase()}.`;
        }
        return "Receipt archived this run without a ready result.";
      };

      const activityProvenanceLabel = (entry, hasContract) => {
        if (!hasContract) {
          return "Archived receipt";
        }
        if (entry.matches_current_contract) {
          return "Current contract receipt";
        }
        return "Older contract receipt";
      };

      const activityAgeLabel = (entries, entry, hasContract) => {
        if (!hasContract) {
          if (entries[0] === entry) {
            return "Most recent archived receipt";
          }
          return "Older archived receipt";
        }
        const matches = entries.filter((candidate) => Boolean(candidate.matches_current_contract) === Boolean(entry.matches_current_contract));
        if (!matches.length) return "Archived receipt";
        if (matches[0] === entry) {
          return entry.matches_current_contract
            ? "Most recent current-contract receipt"
            : "Most recent older-contract receipt";
        }
        return entry.matches_current_contract
          ? "Older current-contract receipt"
          : "Older archived receipt";
      };

      const activityFilterMatches = (entry, filter) => {
        if (filter === "failures") return !entry.ok;
        if (filter === "ready") return Boolean(entry.ok);
        if (filter === "current") return Boolean(entry.matches_current_contract);
        return true;
      };

      const renderActivityTimeline = (entry) => {
        const steps = entry.steps || [];
        if (!steps.length) {
          return `<div class="timeline-strip"><span class="${activityStepClass(entry.status)}">${text(entry.status, entry.ok ? "READY" : "NOT READY")}</span></div>`;
        }
        return `
          <div class="timeline-strip">
            ${steps.slice(0, 6).map((step) => `<span class="${activityStepClass(step.status)}">${text(step.label)} · ${text(step.status)}</span>`).join("")}
            <span class="${activityStepClass(entry.status)}">receipt · ${text(entry.status, entry.ok ? "READY" : "NOT READY")}</span>
          </div>
        `;
      };

      const syncSnapshotPolling = () => {
        if (snapshotPollTimer) {
          window.clearInterval(snapshotPollTimer);
          snapshotPollTimer = null;
        }
        if (!currentServedMode || !currentActionBaseUrl) return;
        snapshotPollTimer = window.setInterval(async () => {
          if (document.hidden || snapshotPollInFlight) return;
          snapshotPollInFlight = true;
          try {
            const response = await fetch(`${currentActionBaseUrl.replace(/\/$/, "")}/api/snapshot`);
            const nextSnapshot = await response.json();
            if (!response.ok) return;
            const nextSignature = JSON.stringify(nextSnapshot);
            if (nextSignature === currentSnapshotSignature) return;
            document.getElementById("ota-studio-data").textContent = nextSignature;
            renderSnapshot(nextSnapshot);
          } catch (_error) {
          } finally {
            snapshotPollInFlight = false;
          }
        }, 4000);
      };

      const bindStudioInteractions = () => {
        document.querySelectorAll("[data-copy-command]").forEach((button) => {
          button.addEventListener("click", async () => {
            const value = button.getAttribute("data-copy-command") || "";
            try {
              await navigator.clipboard.writeText(value);
              button.textContent = "Copied";
              setTimeout(() => {
                button.textContent = button.getAttribute("data-copy-command-label") || "Copy command";
              }, 1200);
            } catch (_error) {
              button.textContent = "Copy failed";
            }
          });
          button.setAttribute("data-copy-command-label", button.textContent);
        });

        document.querySelectorAll("[data-studio-action]").forEach((button) => {
          button.addEventListener("click", async () => {
            const action = button.getAttribute("data-studio-action");
            if (!action || !currentActionBaseUrl) return;
            const confirmed = window.confirm(`Apply the reviewed ${action} path through Ota core?`);
            if (!confirmed) return;
            button.textContent = "Applying…";
            button.setAttribute("disabled", "true");
            try {
              const response = await fetch(`${currentActionBaseUrl.replace(/\/$/, "")}/api/actions/${action}`, {
                method: "POST",
              });
              const payload = await response.json();
              if (!response.ok || payload.ok !== true) {
                throw new Error(payload.error || "Studio action failed");
              }
              const refreshed = await fetch(`${currentActionBaseUrl.replace(/\/$/, "")}/api/snapshot`);
              const nextSnapshot = await refreshed.json();
              if (!refreshed.ok) {
                throw new Error(nextSnapshot.error || "Studio refresh failed");
              }
              document.getElementById("ota-studio-data").textContent = JSON.stringify(nextSnapshot);
              renderSnapshot(nextSnapshot);
              window.alert(`Applied ${action} successfully.`);
            } catch (error) {
              window.alert(error.message || "Studio action failed");
              button.textContent = action === "starter" ? "Apply starter contract" : "Apply additive changes";
              button.removeAttribute("disabled");
            }
          });
        });

        document.querySelectorAll("[data-activity-filter]").forEach((button) => {
          button.addEventListener("click", () => {
            currentActivityFilter = button.getAttribute("data-activity-filter") || "all";
            document.querySelectorAll("[data-activity-filter]").forEach((candidate) => {
              candidate.classList.toggle(
                "is-active",
                candidate.getAttribute("data-activity-filter") === currentActivityFilter
              );
            });
            const nextSnapshot = JSON.parse(document.getElementById("ota-studio-data").textContent);
            renderSnapshot(nextSnapshot);
          });
        });
      };

      const renderSnapshot = (snapshot) => {
        const studio = snapshot.studio || {};
        const activity = snapshot.activity || {};
        const doctor = (snapshot.doctor || {}).data || {};
        const detect = (snapshot.detect || {}).data || {};
        const topology = (snapshot.topology || {}).data || {};
        const detectContractText = snapshot.detect_contract_text || "";
        const reviewContracts = snapshot.review_contracts || {};
        currentSnapshotSignature = JSON.stringify(snapshot);
        currentServedMode = studio.mode === "interactive_server" && window.location.protocol.startsWith("http");
        currentActionBaseUrl = studio.action_base_url || "";
        const hasContract = Boolean(studio.contract_path);

        document.getElementById("hero-title").textContent =
          `${text(topology.contract_identity?.project?.name, "repo")} Studio`;

        document.getElementById("hero-meta").innerHTML = `
          <span class="chip"><strong>Mode</strong> ${text(studio.mode, "read_only_snapshot")}</span>
          <span class="chip"><strong>Repo</strong> ${text(studio.repo_root)}</span>
          <span class="chip"><strong>Contract</strong> ${text(studio.contract_path, "not yet declared")}</span>
          <span class="chip"><strong>Generated</strong> ${text(studio.generated_at)}</span>
        `;

        const summaryDetectChanges = detect.comparison?.changes || [];
        const summaryDetectRemovals = detect.comparison?.removals || [];
        const reviewRollup = !hasContract
          ? "starter review"
          : summaryDetectChanges.length || summaryDetectRemovals.length
            ? "draft differs"
            : "draft aligned";
        const summaryActivityEntries = activity.entries || [];
        const currentContractActivityEntries = hasContract
          ? summaryActivityEntries.filter((entry) => entry.matches_current_contract)
          : [];
        const summaryActivityBase = currentContractActivityEntries.length
          ? currentContractActivityEntries
          : summaryActivityEntries;
        const latestActivity = summaryActivityBase[0] || null;
        const activityRollup = !latestActivity
          ? "no activity yet"
          : latestActivity.ok
            ? "recent ready"
            : "recent failure";
        const latestFailureActivity = summaryActivityBase.find((entry) => !entry.ok) || null;
        const actionNeeded = doctor.summary?.primary_blocker?.next
          || latestFailureActivity?.next
          || "no immediate action";
        const cards = [
          ["Doctor verdict", text(doctor.summary?.verdict, "unknown")],
          ["Contract review", reviewRollup],
          ["Activity", activityRollup],
          ["Action needed", text(actionNeeded)],
          ["Errors", text(doctor.summary?.error_count, "0")],
          ["Tasks", text(topology.contract_identity?.counts?.tasks, "0")],
          ["Services", text(topology.contract_identity?.counts?.services, "0")],
          ["Shared backends", text((topology.shared_backends || []).length, "0")],
        ];
        document.getElementById("summary-cards").innerHTML = cards.map(([label, value]) => `
          <article class="card">
            <div class="card-label">${label}</div>
            <div class="card-value">${value}</div>
          </article>
        `).join("");

        document.getElementById("doctor-status").innerHTML =
          `<span class="${badgeClass(doctor)}">${text(doctor.summary?.verdict, "unknown")}</span>`;

        const findings = doctor.findings || [];
        document.getElementById("doctor-findings").innerHTML = findings.length
          ? findings.slice(0, 8).map((finding) => `
              <article class="finding">
                <p class="finding-title">${text(finding.summary)}</p>
                <p class="finding-body">${text(finding.why)}</p>
                <p class="finding-body" style="margin-top:8px;"><strong>Next:</strong> ${text(finding.next)}</p>
              </article>
            `).join("")
          : `<p class="muted">No readiness findings surfaced in this snapshot.</p>`;

        const activityEntries = activity.entries || [];
        const activityCards = [];
        if (activity.error) {
          activityCards.push(`
            <article class="list-item">
              <p class="list-title">Recent activity unavailable</p>
              <p class="list-meta">${text(activity.error)}</p>
            </article>
          `);
        } else if (activityEntries.length) {
          const activityBaseEntries = currentContractActivityEntries.length
            ? currentContractActivityEntries
            : activityEntries;
          const latestFailure = activityBaseEntries.find((entry) => !entry.ok) || null;
          const latestReady = activityBaseEntries.find((entry) => entry.ok) || null;
          const filteredEntries = activityEntries.filter((entry) =>
            activityFilterMatches(entry, currentActivityFilter)
          );
          activityCards.push(`
            <article class="list-item">
              <p class="list-title">Activity focus</p>
              <p class="list-meta">Filter recent receipt-backed activity without changing the underlying artifact history.</p>
              <div class="toolbar">
                <button type="button" class="toolbar-button${currentActivityFilter === "all" ? " is-active" : ""}" data-activity-filter="all">All</button>
                <button type="button" class="toolbar-button${currentActivityFilter === "failures" ? " is-active" : ""}" data-activity-filter="failures">Failures</button>
                <button type="button" class="toolbar-button${currentActivityFilter === "ready" ? " is-active" : ""}" data-activity-filter="ready">Ready</button>
                <button type="button" class="toolbar-button${currentActivityFilter === "current" ? " is-active" : ""}" data-activity-filter="current">Current contract</button>
              </div>
              <div class="detail-grid">
                ${latestFailure ? renderDetailRow("Latest failure", `${text(latestFailure.archived_at, "unknown time")} · ${text(latestFailure.failed_task || latestFailure.status, "recent failure")}`) : renderDetailRow("Latest failure", "none archived")}
                ${latestReady ? renderDetailRow("Latest ready", `${text(latestReady.archived_at, "unknown time")} · ${text(latestReady.status, "READY")}`) : renderDetailRow("Latest ready", "none archived")}
              </div>
              ${latestFailure ? `
                <div class="subheading">Failure summary</div>
                <div class="detail-grid">
                  ${renderDetailRow("Most recent failure", `${text(latestFailure.status, "FAILED")} · ${text(latestFailure.failed_task || latestFailure.failed_dependency || "recent task")}`)}
                  ${latestFailure.failure_origin ? renderDetailRow("Failure origin", text(latestFailure.failure_origin)) : ""}
                  ${latestFailure.next ? renderDetailRow("Recovery", text(latestFailure.next)) : ""}
                </div>
              ` : ""}
              ${latestReady ? `
                <div class="subheading">Ready summary</div>
                <div class="detail-grid">
                  ${renderDetailRow("Most recent ready run", `${text(latestReady.status, "READY")} · ${text(latestReady.archived_at, "unknown time")}`)}
                  ${latestReady.context ? renderDetailRow("Ready context", text(latestReady.context)) : ""}
                  ${latestReady.backend ? renderDetailRow("Ready backend", text(latestReady.backend)) : ""}
                </div>
              ` : ""}
            </article>
          `);
          activityCards.push(...filteredEntries.map((entry) => {
            const steps = (entry.steps || []).slice(0, 4);
            const stepDetails = entry.steps || [];
            const logs = entry.logs || null;
            const summary = entry.summary || {};
            const findings = (entry.findings || []).slice(0, 6);
            const ageLabel = activityAgeLabel(activityEntries, entry, hasContract);
            const failedLabel = entry.failed_task
              ? `failed task ${text(entry.failed_task)}`
              : steps.length
                ? `latest step ${text(steps[steps.length - 1].label)}`
                : "recent execution";
            return `
              <article class="list-item">
                <div style="display:flex;justify-content:space-between;gap:12px;align-items:flex-start;">
                  <div>
                    <p class="list-title">${text(entry.archived_at, "unknown time")}</p>
                    <p class="list-meta">${failedLabel} · ${text(entry.contract)}</p>
                  </div>
                  <span class="${activityStatusClass(entry)}">${text(entry.status, entry.ok ? "READY" : "NOT READY")}</span>
                </div>
                <div class="list-inline">
                  ${renderPills([
                    entry.backend ? `backend:${entry.backend}` : "",
                    entry.context ? `context:${entry.context}` : "",
                    entry.lifecycle ? `lifecycle:${entry.lifecycle}` : "",
                    entry.target ? `target:${entry.target}` : "",
                    entry.provider ? `provider:${entry.provider}` : "",
                    activityProvenanceLabel(entry, hasContract).toLowerCase(),
                    ageLabel.toLowerCase(),
                  ])}
                </div>
                ${renderActivityTimeline(entry)}
                <div class="detail-grid">
                  ${renderDetailRow("Provenance", activityProvenanceLabel(entry, hasContract))}
                  ${renderDetailRow("Archived age", ageLabel)}
                  ${renderDetailRow("Readiness", activityReadinessLabel(entry))}
                  ${renderDetailRow("Summary", `errors=${text(summary.error_count, "0")}, warnings=${text(summary.warn_count, "0")}, info=${text(summary.info_count, "0")}, steps=${text(summary.step_count, "0")}`)}
                  ${steps.length ? renderDetailRow("Execution timeline", steps.map((step) => `${text(step.label)} (${text(step.status)})`).join(", ")) : ""}
                  ${logs ? renderDetailRow("Logs", `${text(logs.dir)} · stdout=${text(logs.stdout)} · stderr=${text(logs.stderr)}`) : ""}
                  ${entry.next ? renderDetailRow("Next", text(entry.next)) : ""}
                </div>
                <details>
                  <summary>Receipt details</summary>
                  <div class="stack" style="margin-top:12px;">
                    ${stepDetails.length ? `
                      <article class="list-item">
                        <p class="list-title">Archived steps</p>
                        <div class="stack" style="margin-top:10px;">
                          ${stepDetails.map((step) => `
                            <article class="list-item">
                              <p class="list-title">${text(step.label)} · ${text(step.status)}</p>
                              <p class="list-meta">order=${text(step.order, "n/a")}${step.exit_code !== undefined && step.exit_code !== null ? ` · exit=${step.exit_code}` : ""}</p>
                              ${step.detail ? `<p class="list-meta" style="margin-top:8px;">${text(step.detail)}</p>` : ""}
                            </article>
                          `).join("")}
                        </div>
                      </article>
                    ` : ""}
                    ${findings.length ? `
                      <article class="list-item">
                        <p class="list-title">Archived findings</p>
                        <div class="stack" style="margin-top:10px;">
                          ${findings.map((finding) => `
                            <article class="list-item">
                              <p class="list-title">${text(finding.summary)}</p>
                              <p class="list-meta">${text(finding.why)}</p>
                              ${finding.next ? `<p class="list-meta" style="margin-top:8px;"><strong>Next:</strong> ${text(finding.next)}</p>` : ""}
                            </article>
                          `).join("")}
                        </div>
                      </article>
                    ` : ""}
                    ${logs ? `
                      <article class="list-item">
                        <p class="list-title">Durable log paths</p>
                        <p class="list-meta">Copy the repo-local log artifact paths to inspect them from the shell or editor.</p>
                        <div class="detail-grid">
                          ${renderDetailRow("Directory", text(logs.dir))}
                          ${renderDetailRow("Stdout", text(logs.stdout))}
                          ${renderDetailRow("Stderr", text(logs.stderr))}
                        </div>
                        ${renderCopyButtons([
                          { label: "Copy log dir", value: logs.dir },
                          { label: "Copy stdout path", value: logs.stdout },
                          { label: "Copy stderr path", value: logs.stderr },
                        ])}
                      </article>
                    ` : ""}
                    ${!stepDetails.length && !findings.length && !logs ? `
                      <article class="list-item">
                        <p class="list-title">No extra archived detail</p>
                        <p class="list-meta">This receipt did not carry step or finding detail beyond the top-level archived summary.</p>
                      </article>
                    ` : ""}
                  </div>
                </details>
              </article>
            `;
          }));
          if (!filteredEntries.length) {
            activityCards.push(`
              <article class="list-item">
                <p class="list-title">No activity matches this filter</p>
                <p class="list-meta">Try another activity view or run more repo-scoped Ota commands so new receipts land under \`.ota/receipts\`.</p>
              </article>
            `);
          }
        } else {
          activityCards.push(`
            <article class="list-item">
              <p class="list-title">No archived repo activity yet</p>
              <p class="list-meta">Recent activity appears here after repo-scoped Ota runs write receipts and log metadata under \`.ota/receipts\` and \`.ota/state/logs\`.</p>
            </article>
          `);
        }
        if (!activity.error && activity.invalid_archive_count) {
          activityCards.push(`
            <article class="list-item">
              <p class="list-title">Some archived receipts were skipped</p>
              <p class="list-meta">${text(activity.invalid_archive_count)} archived receipt file(s) could not be read cleanly for this snapshot.</p>
            </article>
          `);
        }
        document.getElementById("activity-list").innerHTML = activityCards.join("");

        const detectChanges = summaryDetectChanges;
        const detectRemovals = summaryDetectRemovals;
        const detectInferred = detect.inferred || [];
        const currentContractText =
          snapshot.contract_text || "# No ota.yaml detected yet\n# Start with `ota doctor`\n# Then review `ota detect --dry-run .`\n# Then review `ota init --dry-run`";
        const reviewSummary = detect.ok === false && detect.error
          ? detect.error
          : (!hasContract
              ? "No contract yet. Review the inferred draft, then move through the init path deliberately."
              : detectChanges.length || detectRemovals.length
                ? "The current contract and the dry-run draft differ. Review the semantic comparison before applying anything."
                : "The current contract and the inferred dry-run draft are aligned enough that no semantic drift is surfaced here.");
        document.getElementById("review-summary").textContent = reviewSummary;
        const reviewCards = [
          ["Inferred fields", String(count(detectInferred))],
          ["Detected changes", String(count(detectChanges))],
          ["Potential removals", String(count(detectRemovals))],
          ["Existing contract", detect.comparison?.existing_contract ? "yes" : "no"],
        ];
        document.getElementById("review-cards").innerHTML = reviewCards.map(([label, value]) => `
          <article class="card">
            <div class="card-label">${label}</div>
            <div class="card-value">${value}</div>
          </article>
        `).join("");
        document.getElementById("review-contracts").innerHTML = `
          <article class="mini-panel">
            <p class="mini-title">Current contract</p>
            <pre id="review-current-contract">${currentContractText}</pre>
          </article>
          <article class="mini-panel">
            <p class="mini-title">Inferred draft</p>
            <pre id="review-inferred-contract">${detectContractText || (detect.config ? pretty(detect.config) : pretty(detect))}</pre>
          </article>
        `;

        const changeItems = detectChanges.slice(0, 8).map((change) => `
          <article class="list-item">
            <p class="list-title">${text(change.field)}</p>
            <p class="list-meta">status=${text(change.status)} · confidence=${text(change.confidence, "n/a")} · source=${text(change.source, "detector")}</p>
            <div class="list-inline">
              <span class="pill">detected:${text(change.detected)}</span>
              ${change.existing ? `<span class="pill">existing:${text(change.existing)}</span>` : ""}
            </div>
          </article>
        `);
        const removalItems = detectRemovals.slice(0, 8).map((removal) => `
          <article class="list-item">
            <p class="list-title">${text(removal.field)}</p>
            <p class="list-meta">would be removed from the current contract if you choose a full rewrite path.</p>
            <div class="list-inline">
              <span class="pill">existing:${text(removal.existing)}</span>
            </div>
          </article>
        `);
        document.getElementById("review-diff").innerHTML =
          changeItems.length || removalItems.length
            ? [
                changeItems.length ? `<div class="subheading">Changes</div>${changeItems.join("")}` : "",
                removalItems.length ? `<div class="subheading">Potential removals</div>${removalItems.join("")}` : "",
              ].join("")
            : detectInferred.length
              ? detectInferred.slice(0, 8).map((inference) => `
                  <article class="list-item">
                    <p class="list-title">${text(inference.field)}</p>
                    <p class="list-meta">confidence=${text(inference.confidence, "n/a")} · source=${text(inference.source, "detector")}</p>
                    <div class="list-inline">
                      <span class="pill">value:${text(inference.value)}</span>
                    </div>
                  </article>
                `).join("")
              : `<p class="muted">No contract diff or inferred draft data surfaced in this snapshot.</p>`;

        const reviewOutcomeCards = [];
        if (reviewContracts.merge_contract_text || reviewContracts.error) {
          reviewOutcomeCards.push(renderContractPreview(
            "Reviewed merge result",
            reviewContracts.merge_contract_text,
            reviewContracts.error || "No merge-ready contract output is available for this snapshot."
          ));
        }
        if (reviewContracts.rewrite_contract_text) {
          reviewOutcomeCards.push(renderContractPreview(
            "Reviewed rewrite result",
            reviewContracts.rewrite_contract_text,
            "No rewrite-ready contract output is available for this snapshot."
          ));
        }
        document.getElementById("review-outcomes").innerHTML = reviewOutcomeCards.join("");

        const reviewApplyItems = !hasContract
          ? [
              {
                title: "Review starter draft",
                body: "Stay on the doctor-first path, then inspect the first contract write before any file changes.",
                command: "ota init --dry-run",
                applyLabel: "Copy starter review",
              },
              {
                title: "Write starter contract",
                body: "Apply the reviewed starter once the inferred draft looks right for this repo.",
                command: "ota init",
                applyLabel: "Copy starter write",
                action: "starter",
                actionLabel: "Apply starter contract",
              },
            ]
          : detectChanges.length || detectRemovals.length
            ? [
                {
                  title: "Review additive changes",
                  body: "Use merge review first when the current contract should keep its existing manual structure and only pick up eligible additions.",
                  command: "ota detect --merge --dry-run .",
                  applyLabel: "Copy merge review",
                },
                ...(detectChanges.length ? [{
                  title: "Apply additive changes",
                  body: "This is the first safe apply path when the current contract only needs eligible detect-backed additions.",
                  command: "ota detect --merge .",
                  applyLabel: "Copy merge apply",
                  action: "merge",
                  actionLabel: "Apply additive changes",
                }] : []),
                ...(detectRemovals.length ? [{
                  title: "Review full rewrite",
                  body: "Removals mean the inferred draft wants to replace or drop current contract structure. Review the rewrite path explicitly before any replacement.",
                  command: "ota detect --rewrite --dry-run .",
                  applyLabel: "Copy rewrite review",
                }] : []),
                ...(detectRemovals.length ? [{
                  title: "Apply full rewrite",
                  body: "Only use rewrite after reviewing the exact replacement contract. This replaces the current `ota.yaml` after backing it up.",
                  command: "ota detect --rewrite --yes .",
                  applyLabel: "Copy rewrite apply",
                }] : []),
              ]
            : [
                {
                  title: "Validate current contract",
                  body: "No semantic diff is surfaced here, so the next useful step is to validate and keep running through the declared contract.",
                  command: "ota validate",
                  applyLabel: "Copy validate",
                },
              ];
        document.getElementById("review-apply").innerHTML = reviewApplyItems.map((item) => `
          <article class="list-item">
            <p class="list-title">${text(item.title)}</p>
            <p class="list-meta">${text(item.body)}</p>
            ${renderCommandBox(item.command, item.applyLabel || "Copy command", item.action, item.actionLabel)}
          </article>
        `).join("");
        document.getElementById("detect-config").textContent =
          detect.config ? pretty(detect.config) : pretty(detect);

        const explanationGroups = [];
        const onboardingItems = [];
        const blockerItems = [];
        const activationItems = [];
        const sharedBackendItems = [];
        if (!hasContract) {
          onboardingItems.push({
            title: "No contract yet",
            body: "Studio is still useful before `ota.yaml` exists. Start with `ota doctor`, review `ota detect --dry-run .`, then review `ota init --dry-run` before writing anything.",
          });
        }
        const primaryBlocker = doctor.summary?.primary_blocker;
        if (primaryBlocker) {
          blockerItems.push({
            title: text(primaryBlocker.summary),
            body: `${text(primaryBlocker.why)} Next: ${text(primaryBlocker.next)}`,
          });
        }
        (topology.tasks || []).forEach((task) => {
          (task.targets || []).forEach((target) => {
            const service = target.service || null;
            if (!service) return;
            activationItems.push({
              title: `${text(task.name)} follows target ${text(target.name)}`,
              body: `${addressViewWhy(service.address_view)} ${activationWhy(target.activation_mode)}`,
            });
          });
        });
        (topology.shared_backends || []).forEach((backend) => {
          const boundTaskCount = (topology.tasks || []).filter((task) => task.runtime?.backend_binding === backend.name).length;
          sharedBackendItems.push({
            title: `${text(backend.name)} is a shared backend boundary`,
            body: `${boundTaskCount} task(s) bind here. Lifecycle is ${text(backend.lifecycle)} and fulfillment is ${text(backend.fulfillment, "none")}.`,
          });
        });
        const pushExplainGroup = (heading, items, options = {}) => {
          if (!items.length) return;
          const countLabel = `${items.length} item${items.length === 1 ? "" : "s"}`;
          explanationGroups.push(`
            <details class="list-item explain-group"${options.open ? " open" : ""}>
              <summary><span>${heading}</span><span class="muted">${countLabel}</span></summary>
              <div class="explain-group-body">
                ${items.slice(0, 6).map((item) => `
                  <article class="list-item">
                    <p class="list-title">${text(item.title)}</p>
                    <p class="list-meta">${text(item.body)}</p>
                  </article>
                `).join("")}
              </div>
            </details>
          `);
        };
        pushExplainGroup("Blocker", blockerItems, { open: true });
        pushExplainGroup("Onboarding", onboardingItems, { open: !blockerItems.length });
        pushExplainGroup("Activation", activationItems);
        pushExplainGroup("Shared backend", sharedBackendItems);
        document.getElementById("explain-list").innerHTML = explanationGroups.length
          ? explanationGroups.join("")
          : `<article class="list-item"><p class="list-title">No extra explanation needed</p><p class="list-meta">This snapshot did not surface blockers or topology relationships that need extra translation beyond the declared contract and detect review.</p></article>`;

        const topologySummary = [];
        if (topology.declared_execution) {
          topologySummary.push(`
            <article class="list-item">
              <p class="list-title">Declared execution</p>
              <p class="list-meta">Default context: ${text(topology.declared_execution.default_context)}</p>
              <div class="list-inline">
                ${(topology.declared_execution.supported || []).map((item) => `<span class="pill">${item}</span>`).join("")}
              </div>
            </article>
          `);
          const contexts = topology.declared_execution.contexts || [];
          if (contexts.length) {
            topologySummary.push(`
              <article class="list-item">
                <p class="list-title">Contexts</p>
                <p class="list-meta">Declared execution contexts and their concrete backend shape.</p>
                <div class="list-inline">
                  ${contexts.map((context) => `<span class="pill">${context.name}:${context.backend}${context.lifecycle ? `:${context.lifecycle}` : ""}</span>`).join("")}
                </div>
              </article>
            `);
          }
        } else {
          topologySummary.push(`
            <article class="list-item">
              <p class="list-title">Topology unavailable</p>
              <p class="list-meta">Execution topology only appears after a valid contract exists.</p>
            </article>
          `);
        }
        document.getElementById("topology-summary").innerHTML = topologySummary.join("");

        const tasks = topology.tasks || [];
        document.getElementById("task-list").innerHTML = tasks.length
          ? tasks.map((task) => {
              const listeners = task.runtime?.listeners ? Object.keys(task.runtime.listeners) : [];
              const targets = task.targets || [];
              const runtime = task.runtime || null;
              const readiness = runtime?.readiness || null;
              const listenerDetails = listeners.length
                ? listeners.map((listenerName) => {
                    const listener = runtime.listeners[listenerName] || {};
                    const hostProjection = listener.host_projection;
                    const bind = `${text(listener.bind_address)}:${listener.bind_port_mode === "fixed" && listener.bind_port_value ? listener.bind_port_value : listener.bind_port_mode}`;
                    const projection = hostProjection
                      ? `${text(hostProjection.address)}:${hostProjection.port_mode === "fixed" && hostProjection.port_value ? hostProjection.port_value : hostProjection.port_mode}${hostProjection.path ? hostProjection.path : ""}${hostProjection.primary ? " (primary)" : ""}`
                      : "not projected";
                    return `
                      <article class="list-item">
                        <p class="list-title">${listenerName}</p>
                        <div class="detail-grid">
                          ${renderDetailRow("Protocol", text(listener.protocol))}
                          ${renderDetailRow("Bind", bind)}
                          ${renderDetailRow("Host projection", projection)}
                          ${renderDetailRow("Why", listenerWhy(listener))}
                        </div>
                      </article>
                    `;
                  }).join("")
                : `<p class="muted">No declared listeners.</p>`;
              const targetDetails = targets.length
                ? targets.map((target) => {
                    const service = target.service || null;
                    const resolution = service
                      ? `${service.member ? `${service.member}:` : ""}${text(service.task)}${service.listener ? `:${service.listener}` : ""} via ${text(service.address_view)}`
                      : text(target.url, "manual value");
                    return `
                      <article class="list-item">
                        <p class="list-title">${text(target.name)}</p>
                        <div class="detail-grid">
                          ${renderDetailRow("Kind", text(target.kind))}
                          ${renderDetailRow("Activation", text(target.activation_mode))}
                          ${renderDetailRow("Resolution", resolution)}
                          ${renderDetailRow("Why", targetWhy(target))}
                          ${target.override_input ? renderDetailRow("Override input", text(target.override_input)) : ""}
                        </div>
                      </article>
                    `;
                  }).join("")
                : `<p class="muted">No declared targets.</p>`;
              const servedBy = listeners.length ? listeners.join(", ") : "no listeners";
              const targetsSummary = targets.length
                ? targets.map((target) => `${target.name}:${target.activation_mode}`).join(", ")
                : "no targets";
              return `
                <details class="list-item">
                  <summary>${text(task.name)} · ${text(task.kind)} · ${text(task.context)}</summary>
                  <p class="list-title">${text(task.name)}</p>
                  <p class="list-meta">kind=${text(task.kind)} · context=${text(task.context)}</p>
                  <div class="scan-strip">
                    <span class="pill">serves:${servedBy}</span>
                    <span class="pill">targets:${targetsSummary}</span>
                    <span class="pill">backend:${runtime?.backend_binding ? text(runtime.backend_binding) : "none"}</span>
                  </div>
                  <div class="list-inline">
                    ${task.default_mode ? `<span class="pill">default:${task.default_mode}</span>` : ""}
                    ${(runtime?.backend_binding ? [`<span class="pill">binding:${runtime.backend_binding}</span>`] : []).join("")}
                    ${listeners.map((listener) => `<span class="pill">listener:${listener}</span>`).join("")}
                    ${targets.map((target) => `<span class="pill">target:${target.name}:${target.activation_mode}</span>`).join("")}
                  </div>
                  <div class="detail-grid">
                    ${renderDetailRow("Runtime", runtime ? `${text(runtime.kind)}${runtime.backend_binding ? ` via ${runtime.backend_binding}` : ""}` : "no runtime block")}
                    ${renderDetailRow("Readiness", readiness ? `${text(readiness.kind)}${readiness.listener ? ` on ${readiness.listener}` : ""}${readiness.path ? ` ${readiness.path}` : ""}` : "no readiness contract")}
                    ${renderDetailRow("Why", taskWhy(task))}
                  </div>
                  <details>
                    <summary>Listeners</summary>
                    <div class="stack" style="margin-top:12px;">${listenerDetails}</div>
                  </details>
                  <details>
                    <summary>Targets</summary>
                    <div class="stack" style="margin-top:12px;">${targetDetails}</div>
                  </details>
                </details>
              `;
            }).join("")
          : `<p class="muted">No declared tasks.</p>`;

        const services = topology.services || [];
        document.getElementById("service-list").innerHTML = services.length
          ? services.map((service) => {
              const endpoints = Object.entries(service.endpoints || {});
              const dependencies = service.depends_on || [];
              return `
                <details class="list-item">
                  <summary>${text(service.name)} · ${service.required ? "required" : "optional"}</summary>
                  <p class="list-title">${text(service.name)}</p>
                  <p class="list-meta">${service.required ? "required for readiness" : "advisory infrastructure"}</p>
                  <div class="list-inline">
                    ${renderPills([
                      service.manager ? `manager:${service.manager.kind}${service.manager.name ? `:${service.manager.name}` : ""}` : "",
                      service.provider ? `provider:${service.provider}` : "",
                      service.timeout ? `timeout:${service.timeout}s` : "",
                    ])}
                  </div>
                  <div class="detail-grid">
                    ${renderDetailRow("Start", text(service.start, "not declared"))}
                    ${renderDetailRow("Stop", text(service.stop, "not declared"))}
                    ${renderDetailRow("Healthcheck", text(service.healthcheck, "not declared"))}
                    ${renderDetailRow("Readiness", service.readiness ? `${text(service.readiness.from)} → ${text(service.readiness.run)}` : "not declared")}
                    ${renderDetailRow("Depends on", dependencies.length ? dependencies.join(", ") : "none")}
                  </div>
                  <details>
                    <summary>Endpoints</summary>
                    <div class="stack" style="margin-top:12px;">
                      ${endpoints.length ? endpoints.map(([context, endpoint]) => `
                        <article class="list-item">
                          <p class="list-title">${context}</p>
                          <div class="detail-grid">
                            ${renderDetailRow("Address", text(endpoint.address))}
                            ${renderDetailRow("Port", String(endpoint.port))}
                          </div>
                        </article>
                      `).join("") : `<p class="muted">No projected endpoints.</p>`}
                    </div>
                  </details>
                </details>
              `;
            }).join("")
          : `<p class="muted">No declared services.</p>`;

        const sharedBackends = topology.shared_backends || [];
        document.getElementById("shared-backend-list").innerHTML = sharedBackends.length
          ? sharedBackends.map((backend) => `
              <article class="list-item">
                <p class="list-title">${text(backend.name)}</p>
                <p class="list-meta">${text(backend.scope)} · ${text(backend.backend)} · ${text(backend.lifecycle)}</p>
                <div class="list-inline">
                  ${backend.context ? `<span class="pill">context:${backend.context}</span>` : ""}
                  ${backend.fulfillment ? `<span class="pill">fulfillment:${backend.fulfillment}</span>` : ""}
                  ${backend.environment?.profile ? `<span class="pill">profile:${backend.environment.profile}</span>` : ""}
                </div>
              </article>
            `).join("")
          : `<p class="muted">No declared shared backends.</p>`;

        bindStudioInteractions();
        syncSnapshotPolling();
      };

      renderSnapshot(initialSnapshot);
    </script>
  </body>
</html>"##;
    template.replace("__OTA_STUDIO_SNAPSHOT__", &snapshot_json)
}
