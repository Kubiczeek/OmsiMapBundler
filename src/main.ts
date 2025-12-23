import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";

// Types
interface ValidationResult {
  valid: boolean;
  missing_files: string[];
  error?: string;
}

interface DependencyResult {
  sceneryobjects: string[];
  splines: string[];
  textures: string[];
  humans: string[];
  vehicles: string[];
  tile_maps: string[];
  error?: string;
}

interface BundleRequest {
  map_folder: string;
  output_folder?: string;
  zip_name?: string;
  readme_path?: string;
}

interface BundleResult {
  success: boolean;
  output_path?: string;
  error?: string;
}

// Translations
const translations: Record<string, Record<string, string>> = {
  en: {
    mapFolder: "Map Folder",
    readmePath: "README Path (Optional)",
    outputFolder: "Output Folder (Optional)",
    zipName: "ZIP Name (Optional)",
    browse: "Browse",
    createBundle: "Create Bundle",
    selectMapFolder: "Select map folder...",
    selectReadme: "Select README file...",
    selectOutput: "Select output folder...",
    zipNamePlaceholder: "my-map-bundle",
    errorSelectMap: "Please select a map folder",
    successBundle: "Bundle created successfully!",
    errorBundle: "Error creating bundle",
    processing: "Creating bundle... This may take a moment.",
    validatingMap: "Validating map folder...",
    missingFiles: "Missing required files",
    invalidFolder: "Invalid map folder",
    aboutTitle: "About OMSI Map Bundler",
    aboutDescription:
      "This tool helps you bundle your OMSI 2 maps with all dependencies into a single ZIP file. It automatically detects and includes sceneryobjects, splines, textures, humans, vehicles, and trains.",
    howToUse: "How to Use:",
    step1: "Select your map folder (located in OMSI 2/maps/)",
    step2: "Optionally add a README file",
    step3: "Optionally set a custom output folder and ZIP name",
    step4: 'Click "Create Bundle" and wait for the process to complete',
    noteWarning:
      "The bundling process may take several minutes depending on the map size.",
    importantNote: "Important:",
    fontsNotIncluded:
      "Fonts are NOT included in the bundle. You need to distribute them separately.",
    vehicleFolders:
      "Due to the complexity of vehicle files, entire vehicle parent folders are bundled. Be aware that this includes all files in those folders.",
    supportContact:
      "If you encounter any issues, please contact me on Discord: kubiczeek",
    disclaimerTitle: "Disclaimer",
    disclaimerText:
      'This tool is provided "as is" without warranty of any kind. Always backup your files before using this tool. The author is not responsible for any loss of data or damages that may occur from using this software. Always check the bundled files before distribution.',
  },
  cs: {
    mapFolder: "Složka s Mapou",
    readmePath: "Cesta k README (Volitelné)",
    outputFolder: "Výstupní Složka (Volitelné)",
    zipName: "Název ZIP (Volitelné)",
    browse: "Procházet",
    createBundle: "Vytvořit Balíček",
    selectMapFolder: "Vyberte složku s mapou...",
    selectReadme: "Vyberte README soubor...",
    selectOutput: "Vyberte výstupní složku...",
    zipNamePlaceholder: "muj-balik-mapy",
    errorSelectMap: "Prosím vyberte složku s mapou",
    successBundle: "Balíček úspěšně vytvořen!",
    errorBundle: "Chyba při vytváření balíčku",
    processing: "Vytvářím balíček... Může to chvíli trvat.",
    validatingMap: "Ověřuji složku s mapou...",
    missingFiles: "Chybí povinné soubory",
    invalidFolder: "Neplatná složka s mapou",
    aboutTitle: "O aplikaci OMSI Map Bundler",
    aboutDescription:
      "Tento nástroj vám pomůže zabalit OMSI 2 mapy se všemi závislostmi do jediného ZIP souboru. Automaticky detekuje a zahrnuje sceneryobjects, splines, textury, lidi, vozidla a vlaky.",
    howToUse: "Jak používat:",
    step1: "Vyberte složku s mapou (umístěnou v OMSI 2/maps/)",
    step2: "Volitelně přidejte README soubor",
    step3: "Volitelně nastavte vlastní výstupní složku a název ZIP",
    step4: 'Klikněte na "Vytvořit Balíček" a počkejte na dokončení procesu',
    noteWarning:
      "Poznámka: Proces balení může trvat několik minut v závislosti na velikosti mapy.",
    importantNote: "Důležité:",
    fontsNotIncluded:
      "Fonty NEJSOU součástí balíčku. Musíte je distribuovat samostatně.",
    vehicleFolders:
      "Vzhledem ke složitosti souborů vozidel jsou baleny celé nadřazené složky vozidel. Pamatujte, že to zahrnuje všechny soubory v těchto složkách.",
    supportContact:
      "Pokud narazíte na problémy, kontaktujte mě na Discordu: Kubiczeek",
    disclaimerTitle: "Prohlášení o vyloučení odpovědnosti",
    disclaimerText:
      'Tento nástroj je poskytován "jak je" bez jakékoli záruky. Vždy si před použitím zálohujte své soubory. Autor nenese odpovědnost za ztrátu dat nebo škody, které mohou vzniknout používáním tohoto softwaru. Před distribucí vždy zkontrolujte výsledný balíček.',
  },
  de: {
    mapFolder: "Karten-Ordner",
    readmePath: "README-Pfad (Optional)",
    outputFolder: "Ausgabeordner (Optional)",
    zipName: "ZIP-Name (Optional)",
    browse: "Durchsuchen",
    createBundle: "Paket Erstellen",
    selectMapFolder: "Kartenordner auswählen...",
    selectReadme: "README-Datei auswählen...",
    selectOutput: "Ausgabeordner auswählen...",
    zipNamePlaceholder: "mein-karten-paket",
    errorSelectMap: "Bitte wählen Sie einen Kartenordner",
    successBundle: "Paket erfolgreich erstellt!",
    errorBundle: "Fehler beim Erstellen des Pakets",
    processing: "Paket wird erstellt... Dies kann einen Moment dauern.",
    validatingMap: "Kartenordner wird validiert...",
    missingFiles: "Erforderliche Dateien fehlen",
    invalidFolder: "Ungültiger Kartenordner",
    aboutTitle: "Über OMSI Map Bundler",
    aboutDescription:
      "Dieses Tool hilft Ihnen, Ihre OMSI 2 Karten mit allen Abhängigkeiten in eine einzige ZIP-Datei zu bündeln. Es erkennt und enthält automatisch Sceneryobjects, Splines, Texturen, Menschen, Fahrzeuge und Züge.",
    howToUse: "Anleitung:",
    step1: "Wählen Sie Ihren Kartenordner (unter OMSI 2/maps/)",
    step2: "Optional: README-Datei hinzufügen",
    step3:
      "Optional: Benutzerdefinierten Ausgabeordner und ZIP-Namen festlegen",
    step4: 'Klicken Sie auf "Paket Erstellen" und warten Sie auf den Abschluss',
    noteWarning:
      "Hinweis: Der Bündelungsvorgang kann je nach Kartengröße mehrere Minuten dauern.",
    importantNote: "Wichtig:",
    fontsNotIncluded:
      "Schriftarten sind NICHT im Paket enthalten. Sie müssen separat verteilt werden.",
    vehicleFolders:
      "Aufgrund der Komplexität von Fahrzeugdateien werden ganze übergeordnete Fahrzeugordner gebündelt. Beachten Sie, dass dies alle Dateien in diesen Ordnern umfasst.",
    supportContact:
      "Wenn Sie auf Probleme stoßen, kontaktieren Sie mich bitte auf Discord: Kubiczeek",
    disclaimerTitle: "Haftungsausschluss",
    disclaimerText:
      'Dieses Tool wird "wie besehen" ohne jegliche Garantie bereitgestellt. Sichern Sie immer Ihre Dateien, bevor Sie dieses Tool verwenden. Der Autor ist nicht verantwortlich für Datenverluste oder Schäden, die durch die Verwendung dieser Software entstehen können. Überprüfen Sie das gebündelte Paket vor der Verteilung stets.',
  },
};

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
    } else {
      showStatus(
        `${translations[currentLang].errorBundle}: ${
          result.error || "Unknown error"
        }`,
        "error"
      );
    }
  } catch (error) {
    showStatus(`${translations[currentLang].errorBundle}: ${error}`, "error");
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

// Initialize
window.addEventListener("DOMContentLoaded", () => {
  // Load saved theme
  const savedTheme = localStorage.getItem("theme") || "light";
  document.documentElement.setAttribute("data-theme", savedTheme);

  // Load saved language
  const savedLang = localStorage.getItem("lang") || "en";
  currentLang = savedLang;
  updateLanguage();

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
