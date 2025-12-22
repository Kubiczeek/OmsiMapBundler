---
description: "Describe what this custom agent does and when to use it."
tools:
  [
    "vscode",
    "execute",
    "read",
    "edit",
    "search",
    "web",
    "agent",
    "app-modernization-deploy/*",
    "copilot-container-tools/*",
    "gitkraken/*",
    "ms-python.python/getPythonEnvironmentInfo",
    "ms-python.python/getPythonExecutableCommand",
    "ms-python.python/installPythonPackage",
    "ms-python.python/configurePythonEnvironment",
    "vscjava.migrate-java-to-azure/appmod-install-appcat",
    "vscjava.migrate-java-to-azure/appmod-precheck-assessment",
    "vscjava.migrate-java-to-azure/appmod-run-assessment",
    "vscjava.migrate-java-to-azure/appmod-get-vscode-config",
    "vscjava.migrate-java-to-azure/appmod-preview-markdown",
    "vscjava.migrate-java-to-azure/migration_assessmentReport",
    "vscjava.migrate-java-to-azure/uploadAssessSummaryReport",
    "vscjava.migrate-java-to-azure/appmod-search-knowledgebase",
    "vscjava.migrate-java-to-azure/appmod-search-file",
    "vscjava.migrate-java-to-azure/appmod-fetch-knowledgebase",
    "vscjava.migrate-java-to-azure/appmod-create-migration-summary",
    "vscjava.migrate-java-to-azure/appmod-run-task",
    "vscjava.migrate-java-to-azure/appmod-consistency-validation",
    "vscjava.migrate-java-to-azure/appmod-completeness-validation",
    "vscjava.migrate-java-to-azure/appmod-version-control",
    "vscjava.migrate-java-to-azure/appmod-python-setup-env",
    "vscjava.migrate-java-to-azure/appmod-python-validate-syntax",
    "vscjava.migrate-java-to-azure/appmod-python-validate-lint",
    "vscjava.migrate-java-to-azure/appmod-python-run-test",
    "vscjava.vscode-java-upgrade/list_jdks",
    "vscjava.vscode-java-upgrade/list_mavens",
    "vscjava.vscode-java-upgrade/install_jdk",
    "vscjava.vscode-java-upgrade/install_maven",
    "todo",
  ]
---

# Instructions

Dělám omsi map bundler - aplikace, ve které vybereš složku s mapu a ono to vezme mapu, všechny dependencies a dá je to do jednoho zipu.

Používám Tauri + Vanilla TS.

Tady mám nějakou design paletu na light theme:

- Primary: #F1F3E0 - hlavní brand/background barva, zabírá největší plochu
- Secondary: #D2DCB6 - doplňuje primary, používá se častěji v blocích, sekcích
- Accent: #A1BC98 - zvýraznění tlačítek, odkazů, důležitých prvků
- Neutral: #778873 - text na světlém pozadí, hlavičky, patičky

Potřebuji aby design byl jednoduchý, aby si člověk u té aplikace řekl, ano, toto vypadá solidně, seriózně a 100% trusted a oficiálně. Aby člověk věděl kam má klikat, aby to na něj nebylo složité.
Nikdy nesmíš používat emoji. Místo toho přidávej ikonky, které se hodí k tématu tlačítka/akce.
Vždy defaultně používej anglický jazyk.
Nejradši bych měl takový více hranatý design, ne moc zaoblený.
