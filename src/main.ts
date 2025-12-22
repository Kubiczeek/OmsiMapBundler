import { open } from "@tauri-apps/plugin-dialog";

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
    processing: "Processing...",
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
    processing: "Zpracovávám...",
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
    processing: "Verarbeitung...",
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
    showStatus(translations[currentLang].processing, "info");

    // TODO: Call Rust backend to create bundle
    // For now, we'll log the parameters that will be sent
    console.log("Bundle parameters:", {
      mapFolder: mapFolderPath,
      readmePath: readmePath || null,
      outputFolder: outputFolderPath || null,
      zipName: zipNameInput.value || null,
    });

    // await invoke('create_bundle', {
    //   mapFolder: mapFolderPath,
    //   readmePath: readmePath || null,
    //   outputFolder: outputFolderPath || null,
    //   zipName: zipNameInput.value || null
    // });

    // Simulate processing for now
    await new Promise((resolve) => setTimeout(resolve, 1500));

    showStatus(translations[currentLang].successBundle, "success");
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

// Language Switcher
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
function showStatus(message: string, type: "error" | "success" | "info") {
  statusMessage.textContent = message;
  statusMessage.className = `status-message visible ${type}`;

  if (type === "success" || type === "info") {
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

  // Set active language button
  document
    .querySelector(`.lang-btn[data-lang="${savedLang}"]`)
    ?.classList.add("active");
});
