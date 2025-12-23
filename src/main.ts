import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { translations } from "./translations";
import type {
  ValidationResult,
  DependencyResult,
  BundleRequest,
  BundleResult,
} from "./types";

let currentLang = "en";

// State
let mapFolderPath = "";
let readmePath = "";
let outputFolderPath = "";

// Elements
const mapFolderInput = document.getElementById(
  "map-folder"
) as HTMLInputElement;
const readmeInput = document.getElementById("readme-path") as HTMLInputElement;
const outputFolderInput = document.getElementById(
  "output-folder"
) as HTMLInputElement;
const zipNameInput = document.getElementById("zip-name") as HTMLInputElement;
const compressionToggle = document.getElementById(
  "compression-toggle"
) as HTMLButtonElement;
const compressionCurrent = document.getElementById(
  "compression-current"
) as HTMLSpanElement;
const compressionDropdown = document.getElementById(
  "compression-dropdown"
) as HTMLDivElement;
const compressionSelector = document.querySelector(
  ".compression-selector"
) as HTMLDivElement;
const progressContainer = document.getElementById(
  "progress-container"
) as HTMLDivElement;
const progressFill = document.getElementById("progress-fill") as HTMLDivElement;
const progressText = document.getElementById("progress-text") as HTMLDivElement;

interface ProgressPayload {
  message: string;
  progress: number;
}

// Get current compression value from active option
function getCurrentCompression(): string {
  const activeOption = compressionDropdown?.querySelector(
    ".compression-option.active"
  ) as HTMLButtonElement;
  return activeOption?.dataset.value || "fast";
}

// Set compression value and update UI
function setCompressionValue(value: string): void {
  const options = compressionDropdown?.querySelectorAll(".compression-option");
  options?.forEach((option) => {
    const btn = option as HTMLButtonElement;
    if (btn.dataset.value === value) {
      btn.classList.add("active");
      compressionCurrent.textContent = btn.textContent || "";
    } else {
      btn.classList.remove("active");
    }
  });
}

const bundleBtn = document.getElementById("bundle-btn") as HTMLButtonElement;
const statusMessage = document.getElementById(
  "status-message"
) as HTMLDivElement;
const themeToggle = document.getElementById(
  "theme-toggle"
) as HTMLButtonElement;
const helpToggle = document.getElementById("help-toggle") as HTMLButtonElement;
const infoModal = document.getElementById("info-modal") as HTMLDivElement;
const modalClose = document.getElementById("modal-close") as HTMLButtonElement;
const langToggle = document.getElementById("lang-toggle") as HTMLButtonElement;
const languageSwitcher = document.querySelector(
  ".language-switcher"
) as HTMLDivElement;

// Select Map Folder
document
  .getElementById("select-map-folder")
  ?.addEventListener("click", async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: translations[currentLang].selectMapFolder,
    });

    if (selected && typeof selected === "string") {
      mapFolderPath = selected;
      mapFolderInput.value = selected;
    }
  });

// Select README
document
  .getElementById("select-readme")
  ?.addEventListener("click", async () => {
    const selected = await open({
      multiple: false,
      title: translations[currentLang].selectReadme,
      filters: [
        {
          name: "Text Files",
          extensions: ["txt", "md", "pdf"],
        },
      ],
    });

    if (selected && typeof selected === "string") {
      readmePath = selected;
      readmeInput.value = selected;
    }
  });

// Select Output Folder
document
  .getElementById("select-output-folder")
  ?.addEventListener("click", async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: translations[currentLang].selectOutput,
    });

    if (selected && typeof selected === "string") {
      outputFolderPath = selected;
      outputFolderInput.value = selected;
    }
  });

// Bundle Button
bundleBtn?.addEventListener("click", async () => {
  if (!mapFolderPath) {
    showStatus(translations[currentLang].errorSelectMap, "error");
    return;
  }

  try {
    bundleBtn.disabled = true;
    showStatus(translations[currentLang].validatingMap, "processing");
    setProgress(0.02, translations[currentLang].processing);

    // Validate map folder
    const validation = await invoke<ValidationResult>("validate_map_folder", {
      mapFolder: mapFolderPath,
    });

    if (!validation.valid) {
      if (validation.error) {
        showStatus(
          `${translations[currentLang].invalidFolder}: ${validation.error}`,
          "error"
        );
      } else {
        showStatus(
          `${
            translations[currentLang].missingFiles
          }: ${validation.missing_files.join(", ")}`,
          "error"
        );
      }
      return;
    }

    showStatus(translations[currentLang].processing, "processing");
    setProgress(0.08, translations[currentLang].processing);

    // Extract all dependencies
    console.log("Extracting dependencies from map folder...");
    const dependencies = await invoke<DependencyResult>(
      "extract_dependencies",
      {
        mapFolder: mapFolderPath,
      }
    );

    console.log("=== MAP DEPENDENCIES ===");
    console.log(
      `Found ${dependencies.tile_maps.length} tile maps:`,
      dependencies.tile_maps
    );
    console.log(
      `Found ${dependencies.sceneryobjects.length} scenery objects:`,
      dependencies.sceneryobjects
    );
    console.log(
      `Found ${dependencies.splines.length} splines:`,
      dependencies.splines
    );
    console.log(
      `Found ${dependencies.textures.length} textures:`,
      dependencies.textures
    );
    console.log(
      `Found ${dependencies.humans.length} human models:`,
      dependencies.humans
    );
    console.log(
      `Found ${dependencies.vehicles.length} vehicles:`,
      dependencies.vehicles
    );
    console.log("=======================");

    // Call create_bundle API
    const bundleRequest: BundleRequest = {
      map_folder: mapFolderPath,
      readme_path: readmePath || undefined,
      output_folder: outputFolderPath || undefined,
      zip_name: zipNameInput.value || undefined,
      ...mapCompression(getCurrentCompression()),
    };

    console.log("Creating bundle with parameters:", bundleRequest);

    const result = await invoke<BundleResult>("create_bundle", {
      request: bundleRequest,
    });

    if (result.success) {
      showStatus(
        `${translations[currentLang].successBundle}\n${
          result.output_path || ""
        }`,
        "success"
      );
      setProgress(1, translations[currentLang].successBundle);
    } else {
      showStatus(
        `${translations[currentLang].errorBundle}: ${
          result.error || "Unknown error"
        }`,
        "error"
      );
      setProgress(0, "");
    }
  } catch (error) {
    showStatus(`${translations[currentLang].errorBundle}: ${error}`, "error");
    setProgress(0, "");
  } finally {
    bundleBtn.disabled = false;
  }
});

// Theme Toggle
themeToggle?.addEventListener("click", () => {
  const html = document.documentElement;
  const currentTheme = html.getAttribute("data-theme");
  const newTheme = currentTheme === "light" ? "dark" : "light";
  html.setAttribute("data-theme", newTheme);
  localStorage.setItem("theme", newTheme);
});

// Language Switcher Dropdown
const languageFlags: Record<string, string> = {
  en: `<rect width="20" height="15" fill="#012169" />
    <path d="M0,0 L20,15 M20,0 L0,15" stroke="#FFF" stroke-width="3" />
    <path d="M0,0 L20,15 M20,0 L0,15" stroke="#C8102E" stroke-width="1.5" />
    <path d="M10,0 V15 M0,7.5 H20" stroke="#FFF" stroke-width="5" />
    <path d="M10,0 V15 M0,7.5 H20" stroke="#C8102E" stroke-width="3" />`,
  cs: `<rect width="20" height="7.5" fill="#FFF" />
    <rect y="7.5" width="20" height="7.5" fill="#D7141A" />
    <path d="M0,0 L10,7.5 L0,15 Z" fill="#11457E" />`,
  de: `<rect width="20" height="5" fill="#000" />
    <rect y="5" width="20" height="5" fill="#D00" />
    <rect y="10" width="20" height="5" fill="#FFCE00" />`,
};

langToggle?.addEventListener("click", (e) => {
  e.stopPropagation();
  languageSwitcher?.classList.toggle("open");
});

// Close dropdown when clicking outside
document.addEventListener("click", (e) => {
  if (!languageSwitcher?.contains(e.target as Node)) {
    languageSwitcher?.classList.remove("open");
  }
});

// Language options
document.querySelectorAll(".lang-option").forEach((btn) => {
  btn.addEventListener("click", (e) => {
    const target = e.currentTarget as HTMLButtonElement;
    const lang = target.getAttribute("data-lang");

    if (lang) {
      currentLang = lang;
      updateLanguage();

      // Update flag in toggle button
      const currentFlagSvg = document.getElementById("current-flag-svg");
      if (currentFlagSvg && languageFlags[lang]) {
        currentFlagSvg.innerHTML = languageFlags[lang];
      }

      // Update active state
      document
        .querySelectorAll(".lang-option")
        .forEach((b) => b.classList.remove("active"));
      target.classList.add("active");

      localStorage.setItem("lang", lang);
      languageSwitcher?.classList.remove("open");
    }
  });
});

// Old lang-btn handler (keeping for backward compatibility if any)
document.querySelectorAll(".lang-btn").forEach((btn) => {
  btn.addEventListener("click", (e) => {
    const target = e.currentTarget as HTMLButtonElement;
    const lang = target.getAttribute("data-lang");

    if (lang) {
      currentLang = lang;
      updateLanguage();

      document
        .querySelectorAll(".lang-btn")
        .forEach((b) => b.classList.remove("active"));
      target.classList.add("active");
      localStorage.setItem("lang", lang);
    }
  });
});

// Update Language
function updateLanguage() {
  document.querySelectorAll("[data-i18n]").forEach((el) => {
    const key = el.getAttribute("data-i18n");
    if (key && translations[currentLang][key]) {
      el.textContent = translations[currentLang][key];
    }
  });

  document.querySelectorAll("[data-i18n-placeholder]").forEach((el) => {
    const key = el.getAttribute("data-i18n-placeholder");
    if (key && translations[currentLang][key]) {
      (el as HTMLInputElement).placeholder = translations[currentLang][key];
    }
  });

  // Update compression dropdown options text
  const currentValue = getCurrentCompression();
  compressionDropdown
    ?.querySelectorAll(".compression-option")
    .forEach((option) => {
      const btn = option as HTMLButtonElement;
      const value = btn.dataset.value;
      if (value) {
        const key = `compression${
          value.charAt(0).toUpperCase() + value.slice(1)
        }`;
        if (translations[currentLang][key]) {
          btn.textContent = translations[currentLang][key];
          if (value === currentValue) {
            compressionCurrent.textContent = translations[currentLang][key];
          }
        }
      }
    });
}

// Show Status
function showStatus(
  message: string,
  type: "error" | "success" | "info" | "processing"
) {
  statusMessage.innerHTML = "";
  statusMessage.className = `status-message visible ${type}`;

  if (type === "processing") {
    const spinner = document.createElement("div");
    spinner.className = "spinner";
    statusMessage.appendChild(spinner);
  }

  const textSpan = document.createElement("span");
  textSpan.className = "status-text";
  textSpan.textContent = message;
  statusMessage.appendChild(textSpan);

  if (type === "success") {
    setTimeout(() => {
      statusMessage.classList.remove("visible");
    }, 5000);
  }
}

function mapCompression(value: string | undefined) {
  switch (value) {
    case "none":
      return { compression_method: "stored", compression_level: 0 };
    case "fast":
      return { compression_method: "deflate", compression_level: 1 };
    case "balanced":
      return { compression_method: "deflate", compression_level: 6 };
    case "max":
      return { compression_method: "deflate", compression_level: 9 };
    default:
      return { compression_method: "deflate", compression_level: 1 };
  }
}

function setProgress(progress: number, message: string) {
  if (!progressContainer || !progressFill || !progressText) return;

  const clamped = Math.max(0, Math.min(progress, 1));
  progressContainer.classList.add("visible");
  progressFill.style.width = `${Math.round(clamped * 100)}%`;
  progressText.textContent = `${Math.round(clamped * 100)}% — ${message}`;

  if (clamped === 0 || !message) {
    progressContainer.classList.remove("visible");
    progressFill.style.width = "0%";
    progressText.textContent = "";
  }
}

// Initialize
window.addEventListener("DOMContentLoaded", () => {
  // Load saved theme
  const savedTheme = localStorage.getItem("theme") || "light";
  document.documentElement.setAttribute("data-theme", savedTheme);

  // Load saved language
  const savedLang = localStorage.getItem("lang") || "en";
  currentLang = savedLang;
  updateLanguage();

  // Load saved compression preference
  const savedCompression = localStorage.getItem("compression") || "balanced";
  setCompressionValue(savedCompression);

  // Listen for backend progress events
  listen<ProgressPayload>("bundle-progress", (event) => {
    const { message, progress } = event.payload;
    setProgress(progress, message);
  });

  // Set active language option in dropdown
  document
    .querySelector(`.lang-option[data-lang="${savedLang}"]`)
    ?.classList.add("active");

  // Update flag in toggle button
  const currentFlagSvg = document.getElementById("current-flag-svg");
  if (currentFlagSvg && languageFlags[savedLang]) {
    currentFlagSvg.innerHTML = languageFlags[savedLang];
  }

  // Set active language button (legacy)
  document
    .querySelector(`.lang-btn[data-lang="${savedLang}"]`)
    ?.classList.add("active");

  // Check if first visit
  const hasVisited = localStorage.getItem("hasVisited");
  if (!hasVisited) {
    infoModal?.classList.add("visible");
    localStorage.setItem("hasVisited", "true");
  }
});

// Compression Dropdown Toggle
compressionToggle?.addEventListener("click", (e) => {
  e.stopPropagation();
  compressionSelector?.classList.toggle("open");
});

// Compression Option Selection
compressionDropdown
  ?.querySelectorAll(".compression-option")
  .forEach((option) => {
    option.addEventListener("click", () => {
      const btn = option as HTMLButtonElement;
      const value = btn.dataset.value;
      if (value) {
        setCompressionValue(value);
        localStorage.setItem("compression", value);
        compressionSelector?.classList.remove("open");
      }
    });
  });

// Close compression dropdown when clicking outside
document.addEventListener("click", (e) => {
  if (!compressionSelector?.contains(e.target as Node)) {
    compressionSelector?.classList.remove("open");
  }
});

// Info Modal Controls
helpToggle?.addEventListener("click", () => {
  infoModal?.classList.add("visible");
});

modalClose?.addEventListener("click", () => {
  infoModal?.classList.remove("visible");
});

// Close modal on outside click
infoModal?.addEventListener("click", (e) => {
  if (e.target === infoModal) {
    infoModal.classList.remove("visible");
  }
});
