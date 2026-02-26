#!/usr/bin/env node
/**
 * Pre-flight dependency check for `tauri dev` / `tauri build`.
 *
 * Detects missing system dependencies and offers to install them
 * interactively. In CI or non-interactive shells, prints what is
 * missing and exits 1 without prompting.
 *
 * Usage:  node scripts/check-deps.mjs
 */

import { execSync, execFileSync } from "node:child_process";
import { createInterface } from "node:readline";
import { readFileSync, accessSync, constants } from "node:fs";

// ── Colour helpers (disabled when stdout is not a TTY) ──────────────

const USE_COLOR = process.stdout.isTTY ?? false;
const c = {
  green: (s) => (USE_COLOR ? `\x1b[32m${s}\x1b[0m` : s),
  red: (s) => (USE_COLOR ? `\x1b[31m${s}\x1b[0m` : s),
  yellow: (s) => (USE_COLOR ? `\x1b[33m${s}\x1b[0m` : s),
  bold: (s) => (USE_COLOR ? `\x1b[1m${s}\x1b[0m` : s),
  dim: (s) => (USE_COLOR ? `\x1b[2m${s}\x1b[0m` : s),
};

// ── Platform detection ──────────────────────────────────────────────

const PLATFORM = process.platform; // "linux" | "darwin" | "win32"

function isWSL2() {
  if (PLATFORM !== "linux") return false;
  try {
    const ver = readFileSync("/proc/version", "utf8");
    return /microsoft/i.test(ver);
  } catch {
    return false;
  }
}

const IS_WSL2 = isWSL2();
const IS_LINUX = PLATFORM === "linux";
const IS_MACOS = PLATFORM === "darwin";
const IS_INTERACTIVE = (process.stdin.isTTY ?? false) && !process.env.CI;

// ── Utility ─────────────────────────────────────────────────────────

/** Run a command silently; return stdout or null on failure. */
function run(cmd, args = []) {
  try {
    return execFileSync(cmd, args, {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
      timeout: 15_000,
    }).trim();
  } catch {
    return null;
  }
}

/** True if the command exists on PATH. */
function commandExists(cmd) {
  return run("which", [cmd]) !== null;
}

/**
 * Parse a semver-ish string into [major, minor, patch].
 * Accepts "1.77.2", "v1.77.2", "rustc 1.77.2 (…)", etc.
 */
function parseSemver(raw) {
  const m = raw?.match(/(\d+)\.(\d+)\.(\d+)/);
  if (!m) return null;
  return [Number(m[1]), Number(m[2]), Number(m[3])];
}

/** True if `a >= b` in semver. */
function semverGte(a, b) {
  for (let i = 0; i < 3; i++) {
    if (a[i] > b[i]) return true;
    if (a[i] < b[i]) return false;
  }
  return true; // equal
}

/** Check whether a dpkg package is installed. */
function dpkgInstalled(pkg) {
  try {
    const out = execFileSync("dpkg-query", ["-W", "-f", "${Status}", pkg], {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
      timeout: 5_000,
    });
    return out.includes("install ok installed");
  } catch {
    return false;
  }
}

/** Prompt the user with a yes/no question. Returns true for yes. */
async function confirm(question) {
  if (!IS_INTERACTIVE) return false;

  const rl = createInterface({ input: process.stdin, output: process.stdout });
  return new Promise((resolve) => {
    rl.question(`${question} [Y/n] `, (answer) => {
      rl.close();
      const a = answer.trim().toLowerCase();
      resolve(a === "" || a === "y" || a === "yes");
    });
  });
}

/** Run a shell command visibly (inherit stdio). */
function execVisible(cmd) {
  execSync(cmd, { stdio: "inherit", timeout: 300_000 });
}

// ── Check definitions ───────────────────────────────────────────────

/**
 * Each check returns { name, status, message, fix? }
 *   status: "pass" | "fail" | "warn"
 *   fix:    async () => void  — called when user accepts auto-install
 */

async function checkCargo() {
  const MIN_VERSION = [1, 77, 2];
  const name = "cargo";

  if (!commandExists("cargo")) {
    return {
      name,
      status: "fail",
      message: "cargo not found on PATH",
      installLabel: "Install Rust via rustup",
      fix: async () => {
        execVisible('curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y');
        console.log(
          c.yellow(
            "\n  Rust was installed, but the current shell may not have the updated PATH.\n" +
              "  If the next verification fails, restart your terminal and re-run."
          )
        );
      },
      verify: () => {
        // After installing, cargo may not be on PATH in this process.
        // Check the default install location directly.
        const home = process.env.HOME ?? process.env.USERPROFILE;
        const cargoPath = `${home}/.cargo/bin/cargo`;
        try {
          accessSync(cargoPath, constants.X_OK);
          return true;
        } catch {
          return commandExists("cargo");
        }
      },
    };
  }

  const raw = run("cargo", ["--version"]);
  const ver = parseSemver(raw);
  if (!ver) {
    return { name, status: "warn", message: `Could not parse cargo version: ${raw}` };
  }

  if (!semverGte(ver, MIN_VERSION)) {
    return {
      name,
      status: "fail",
      message: `cargo ${ver.join(".")} found, need >= ${MIN_VERSION.join(".")}`,
      installLabel: "Update Rust via rustup",
      fix: async () => {
        execVisible("rustup update stable");
      },
    };
  }

  return { name, status: "pass", message: `cargo ${ver.join(".")}` };
}

function checkXcodeTools() {
  if (!IS_MACOS) return null;
  const name = "Xcode CLT";

  // `xcode-select -p` exits 0 when tools are installed
  const path = run("xcode-select", ["-p"]);
  if (path) {
    return { name, status: "pass", message: path };
  }

  return {
    name,
    status: "fail",
    message:
      "Xcode Command Line Tools not installed.\n" +
      "         Run:  xcode-select --install",
  };
}

function checkPkgConfig() {
  if (!IS_LINUX) return null;
  const name = "pkg-config";

  if (commandExists("pkg-config")) {
    return { name, status: "pass", message: "found" };
  }

  return {
    name,
    status: "fail",
    message: "pkg-config not found",
    installLabel: "Install pkg-config via apt",
    fix: async () => {
      execVisible("sudo apt-get update -qq && sudo apt-get install -y pkg-config");
    },
  };
}

function checkLinuxLib(dpkgName, friendlyName) {
  if (!IS_LINUX) return null;
  const name = friendlyName ?? dpkgName;

  if (dpkgInstalled(dpkgName)) {
    return { name, status: "pass", message: "installed" };
  }

  return {
    name,
    status: "fail",
    message: `${dpkgName} not installed`,
    installLabel: `Install ${dpkgName} via apt`,
    fix: async () => {
      execVisible(`sudo apt-get update -qq && sudo apt-get install -y ${dpkgName}`);
    },
  };
}

function checkInotifyInstances() {
  if (!IS_WSL2) return null;
  const name = "inotify limit";

  try {
    const raw = readFileSync("/proc/sys/fs/inotify/max_user_instances", "utf8").trim();
    const val = Number(raw);
    if (val >= 512) {
      return { name, status: "pass", message: `max_user_instances = ${val}` };
    }
    return {
      name,
      status: "warn",
      message:
        `max_user_instances = ${val} (recommend >= 512)\n` +
        "         Fix:  sudo sysctl -w fs.inotify.max_user_instances=1024\n" +
        "         Persist in /etc/sysctl.conf:  fs.inotify.max_user_instances=1024",
    };
  } catch {
    return { name, status: "warn", message: "Could not read inotify limit" };
  }
}

// ── Linux libraries needed by Tauri ─────────────────────────────────

const TAURI_LINUX_LIBS = [
  "libssl-dev",
  "libgtk-3-dev",
  "libwebkit2gtk-4.1-dev",
  "libappindicator3-dev",
  "librsvg2-dev",
  "patchelf",
];

// ── Main ────────────────────────────────────────────────────────────

async function main() {
  const results = [];

  // Cargo (all platforms)
  results.push(await checkCargo());

  // macOS: Xcode CLT
  const xcode = checkXcodeTools();
  if (xcode) results.push(xcode);

  // Linux: pkg-config
  const pkgCfg = checkPkgConfig();
  if (pkgCfg) results.push(pkgCfg);

  // Linux: Tauri system libs
  for (const lib of TAURI_LINUX_LIBS) {
    const r = checkLinuxLib(lib);
    if (r) results.push(r);
  }

  // WSL2: inotify
  const inot = checkInotifyInstances();
  if (inot) results.push(inot);

  // ── Print summary ───────────────────────────────────────────────

  const failures = results.filter((r) => r.status === "fail");
  const warnings = results.filter((r) => r.status === "warn");
  const passes = results.filter((r) => r.status === "pass");

  // Fast path: everything OK
  if (failures.length === 0 && warnings.length === 0) {
    console.log(c.green("All dependencies satisfied."));
    process.exit(0);
  }

  console.log(c.bold("\n  Dependency check\n"));

  for (const r of results) {
    const icon =
      r.status === "pass" ? c.green("PASS") : r.status === "warn" ? c.yellow("WARN") : c.red("FAIL");
    console.log(`  [${icon}]  ${c.bold(r.name)}  ${c.dim(r.message)}`);
  }

  console.log(); // blank line

  if (warnings.length > 0 && failures.length === 0) {
    // Warnings only — continue
    console.log(c.yellow("Warnings above are non-blocking. Continuing.\n"));
    process.exit(0);
  }

  if (failures.length === 0) {
    process.exit(0);
  }

  // ── Non-interactive: just report and exit ───────────────────────

  if (!IS_INTERACTIVE) {
    console.log(c.red(`${failures.length} missing dependency(ies). Install them and retry.\n`));
    process.exit(1);
  }

  // ── Interactive: offer to fix ───────────────────────────────────

  // Batch Linux apt packages into a single install when possible
  const aptFixable = failures.filter(
    (r) => r.fix && r.installLabel?.includes("via apt")
  );
  const otherFixable = failures.filter(
    (r) => r.fix && !r.installLabel?.includes("via apt")
  );
  const unfixable = failures.filter((r) => !r.fix);

  let installed = 0;

  // Handle non-apt fixes first (e.g. rustup)
  for (const r of otherFixable) {
    const yes = await confirm(`  ${r.installLabel}?`);
    if (yes) {
      try {
        await r.fix();
        installed++;
      } catch (e) {
        console.error(c.red(`  Failed to install ${r.name}: ${e.message}`));
      }
    }
  }

  // Batch apt installs
  if (aptFixable.length > 0) {
    const pkgs = aptFixable.map((r) => {
      // Extract dpkg package name from the message "xxx not installed"
      const m = r.message.match(/^(\S+) not/);
      return m ? m[1] : r.name;
    });

    console.log(`\n  The following packages can be installed via apt:`);
    for (const p of pkgs) {
      console.log(`    - ${p}`);
    }

    const yes = await confirm(`\n  Install all via sudo apt-get install?`);
    if (yes) {
      try {
        execVisible(`sudo apt-get update -qq && sudo apt-get install -y ${pkgs.join(" ")}`);
        installed += aptFixable.length;
      } catch (e) {
        console.error(c.red(`  apt install failed: ${e.message}`));
      }
    }
  }

  // Report unfixable
  for (const r of unfixable) {
    console.log(c.yellow(`  Manual action required for ${c.bold(r.name)}:`));
    console.log(`    ${r.message}\n`);
  }

  // ── Re-verify after installs ────────────────────────────────────

  if (installed > 0) {
    console.log(c.bold("\n  Re-verifying...\n"));

    let stillFailing = 0;

    for (const r of failures) {
      if (r.verify) {
        if (!r.verify()) {
          console.log(`  [${c.red("FAIL")}]  ${r.name} — still not available`);
          stillFailing++;
        } else {
          console.log(`  [${c.green("PASS")}]  ${r.name}`);
        }
      } else if (r.fix) {
        // For apt packages, re-check via dpkg
        const m = r.message.match(/^(\S+) not/);
        const pkg = m ? m[1] : null;
        if (pkg && dpkgInstalled(pkg)) {
          console.log(`  [${c.green("PASS")}]  ${r.name}`);
        } else if (commandExists(r.name)) {
          console.log(`  [${c.green("PASS")}]  ${r.name}`);
        } else {
          console.log(`  [${c.red("FAIL")}]  ${r.name} — still not available`);
          stillFailing++;
        }
      }
    }

    console.log();
    if (stillFailing > 0) {
      console.log(
        c.yellow(
          "  Some dependencies are still missing. You may need to restart your terminal\n" +
            "  for PATH changes to take effect, then re-run.\n"
        )
      );
      process.exit(1);
    }

    console.log(c.green("  All dependencies satisfied after install.\n"));
    process.exit(0);
  }

  // Nothing was installed and there are still failures
  console.log(c.red(`\n  ${failures.length} missing dependency(ies). Install them and retry.\n`));
  process.exit(1);
}

main().catch((err) => {
  console.error(c.red(`check-deps failed: ${err.message}`));
  process.exit(1);
});
