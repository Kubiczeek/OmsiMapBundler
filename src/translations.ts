// Translation strings for the application

export const translations: Record<string, Record<string, string>> = {
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
