# Dokumentace extrakce závislostí - SOUČASNÝ STAV

Tento dokument popisuje **jak aktuálně funguje** extrakce závislostí v kódu (ne jak by měla).

---

## Architektura - jak to funguje TEĎKA

### Hlavní tok zpracování

```
extraction.rs (extract_dependencies)
  ↓
  Parsuje mapové soubory (.map, global.cfg, ailists.cfg, parklist_p.txt)
  ↓
  Vytváří seznamy: sceneryobjects, splines, humans, vehicles
  ↓
dependencies/mod.rs (extract_nested_dependencies)
  ↓
  Pro každý typ volá příslušnou funkci:
    - sceneryobject::extract_sceneryobject_dependencies()
    - sceneryobject::extract_ovh_dependencies()  ← DUPLIKACE!
    - vehicle::extract_vehicle_dependencies()
    - train::extract_train_dependencies()
    - spline::extract_spline_dependencies()
    - human::extract_human_dependencies()
```

### PROBLÉM: Kontextové zpracování

`.ovh` soubory se zpracovávají **DVAKRÁT** různě:

- V `sceneryobject.rs` → `extract_ovh_dependencies()` - parsuje jednotlivé sekce
- V `vehicle.rs` → `extract_vehicle_dependencies()` - kopíruje celou složku

Rozhodování podle kontextu:

```rust
// v mod.rs
"sceneryobject" => {
    if asset_path.ends_with(".ovh") {
        extract_ovh_dependencies()  // detailní parsing
    } else {
        extract_sceneryobject_dependencies()
    }
}
"vehicle" => {
    if .ends_with(".bus") || .ends_with(".ovh") {
        extract_vehicle_dependencies()  // celá složka
    }
}
```

---

## 📍 Mapové soubory (extraction.rs)

### `.map` - Map Tiles

**Lokace**: `{map_folder}/*.map` + `{map_folder}/Chrono/*/*.map`

**Parsování**:

```rust
while let Some(line) = lines.next() {
    if trimmed == "[spline]" {
        lines.next(); // skip ID
        path = lines.next(); // cesta k .sli
        splines.insert(path);
    }

    if trimmed == "[object]" || trimmed == "[splineAttachement]" {
        lines.next(); // skip ID
        path = lines.next(); // cesta k .sco nebo .ovh
        if path.ends_with(".sco") || path.ends_with(".ovh") {
            sceneryobjects.insert(path);
        }
    }
}
```

### `global.cfg`

**Parsování**:

```rust
if trimmed == "[groundtex]" {
    tex_path = lines.next();
    textures.insert(tex_path);
}

if trimmed == "[humans]" {
    hum_path = lines.next();
    humans.insert(hum_path);
}

if trimmed == "[spline]" {
    sli_path = lines.next();
    splines.insert(sli_path);
}
```

### `ailists.cfg`

**Parsování**:

```rust
while let Some(line) = lines.next() {
    if trimmed.starts_with("[aigroup_depot_typgroup") {
        in_depot_typgroup = true;
    }

    if in_depot_typgroup {
        if trimmed.ends_with(".bus|.ovh|.zug|.sco") {
            vehicles.insert(trimmed);
            in_depot_typgroup = false;
        }
    } else {
        // Normální řádky: "cesta<whitespace>počet"
        first_part = trimmed.split_whitespace().next();
        if first_part.ends_with(".bus|.ovh|.zug|.sco") {
            vehicles.insert(first_part);
        }
    }
}
```

### `parklist_p.txt`

**Parsování**:

```rust
for line in content.lines() {
    if line.ends_with(".sco") || line.ends_with(".ovh") {
        if line.starts_with("vehicles\\") || line.starts_with("vehicles/") {
            vehicles.insert(line);  // static vehicle
        } else {
            sceneryobjects.insert(line);
        }
    }
}
```

---

## 🏢 `.sco` - Scenery Objects (sceneryobject.rs)

**Funkce**: `extract_sceneryobject_dependencies()`

**Kódování**: Windows-1252

**Parsované sekce**:

### `[mesh]`

```rust
lines.next(); // název mesh souboru (.o3d nebo .x)
mesh_file = line.trim();

// Hledání v lokacích (v tomto pořadí):
// 1. {sco_folder}/model/{mesh_file}
// 2. {sco_folder}/{mesh_file}
// 3. {mesh_file} (as-is)

dependencies.insert(mesh_path);

// Extrakce textur z meshu
if mesh_file.ends_with(".o3d") {
    textures = extract_o3d_textures(mesh_path);
} else if mesh_file.ends_with(".x") {
    textures = extract_x_textures(mesh_path);
}

// Pro každou texturu → add_texture_variants(base_name)
```

### `[collision_mesh]`

- **Identická logika jako `[mesh]`**
- Collision mesh může mít stejné textury

### `[matl]`, `[matl_change]`, `[matl_lightmap]`

```rust
tex_file = lines.next(); // cesta k textuře
base_name = tex_file.strip_extension();
add_texture_variants(base_name, sco_folder, omsi_root);
```

### `[matl_envmap]`

```rust
tex_file = lines.next(); // environment map textura
lines.next(); // skip číselná hodnota (např. 0.85)
base_name = tex_file.strip_extension();
add_texture_variants(base_name, sco_folder, omsi_root);
```

### `[CTCTexture]`

```rust
lines.next(); // skip název proměnné
tex_file = lines.next(); // cesta k textuře
base_name = tex_file.strip_extension();
add_texture_variants(base_name, sco_folder, omsi_root);
```

### `[CTC]`

```rust
lines.next(); // skip proměnná
folder_path = lines.next(); // cesta ke složce
// Přidat VŠECHNY soubory z této složky
for entry in read_dir(folder_path) {
    dependencies.insert(entry.path());
}
```

### `[script]`

```rust
lines.next(); // skip count
script_file = lines.next(); // cesta k .osc

// Hledání (v pořadí):
// 1. {sco_folder}/script/{script_file}
// 2. {sco_folder}/{script_file}
// 3. {script_file} (as-is)
// Přidá se pouze pokud existuje
```

### `[varnamelist]`

```rust
lines.next(); // skip count
varlist_file = lines.next(); // cesta k .txt

// Hledání (v pořadí):
// 1. {sco_folder}/script/{varlist_file}
// 2. {sco_folder}/{varlist_file}
// 3. {varlist_file} (as-is)
```

### `[sound]`

```rust
sound_file = lines.next(); // cesta k .cfg

// Hledání (v pořadí):
// 1. {sco_folder}/sound/{sound_file}
// 2. {sco_folder}/{sound_file}
// 3. {sound_file} (as-is)

dependencies.insert(sound_path);

// Parsovat sound config → extract_sound_config_dependencies()
// → hledá .wav soubory uvnitř
```

### `[passengercabin]`

```rust
cabin_file = lines.next(); // cesta k .cfg

// Hledání (v pořadí):
// 1. {sco_folder}/{cabin_file}
// 2. {sco_folder}/model/{cabin_file}
// 3. {cabin_file} (as-is)
// Přidá se pouze pokud existuje
```

### Speciální: Prefix matching

```rust
// Na konci funkce:
sco_filename = Path::file_stem(sco_path); // např. "Dum_cetkovice4"
search_textures_by_prefix(sco_filename, sco_folder, omsi_root);
// Hledá všechny textury začínající na "Dum_cetkovice4"
```

---

## 🚗 `.ovh` - AI Vehicles v Sceneryobjects (sceneryobject.rs)

**Funkce**: `extract_ovh_dependencies()`

**Kódování**: Windows-1252

**DŮLEŽITÉ**: Toto se volá **pouze** pro .ovh v Sceneryobjects!

**Parsované sekce**:

### `[model]`

```rust
model_file = lines.next(); // cesta k .cfg (např. "model/Golf_V.cfg")

// Pokud začíná "..\\" → resolve relativně: {ovh_folder}/{model_file}
// Jinak hledání (v pořadí):
// 1. {ovh_folder}/model/{model_file}
// 2. {ovh_folder}/{model_file}
// 3. Resolved relativní cesta

// Přidá se pouze pokud existuje
```

### `[sound]`

```rust
sound_file = lines.next(); // např. "..\..\Sounds\AI_Cars\sound.cfg"

// Pokud začíná "..\\" → resolve: {ovh_folder}/{sound_file}
// Jinak hledání:
// 1. {ovh_folder}/sound/{sound_file}
// 2. {ovh_folder}/{sound_file}

dependencies.insert(sound_path);
// Parsovat sound config → extract_sound_config_dependencies()
```

### `[varnamelist]`

```rust
count = lines.next().parse(); // kolik souborů
for i in 0..count {
    varlist_file = lines.next(); // např. "..\..\Scripts\AI_Cars\AI_varlist.txt"

    // Pokud začíná "..\\" → resolve: {ovh_folder}/{varlist_file}
    // Jinak hledání:
    // 1. {ovh_folder}/script/{varlist_file}
    // 2. {ovh_folder}/{varlist_file}

    // Přidá se pouze pokud existuje
}
```

### `[script]`

```rust
count = lines.next().parse(); // kolik scriptů
for i in 0..count {
    script_file = lines.next(); // např. "..\..\Scripts\AI_Cars\main_AI.osc"

    // Pokud začíná "..\\" → resolve: {ovh_folder}/{script_file}
    // Jinak hledání:
    // 1. {ovh_folder}/script/{script_file}
    // 2. {ovh_folder}/{script_file}

    // Přidá se pouze pokud existuje
}
```

### `[constfile]`

```rust
count = lines.next().parse(); // kolik const files
for i in 0..count {
    const_file = lines.next(); // např. "script\AI_constfile.txt"

    // Hledání:
    // 1. {ovh_folder}/script/{const_file}
    // 2. {ovh_folder}/{const_file}

    // Přidá se pouze pokud existuje
}
```

---

## 🚌 `.bus` / 🚗 `.ovh` v Vehicles (vehicle.rs)

**Funkce**: `extract_vehicle_dependencies()`

**STRATEGIE**: **KOPÍRUJE CELOU SLOŽKU!**

```rust
vehicle_folder = Path::parent(vehicle_path); // např. "Vehicles\MAN\Lions_City"

// Safety check:
if !folder.is_empty() && folder != "\\" && folder.contains("\\") {
    dependencies.insert("FOLDER:" + folder);
    println!("Will copy vehicle folder: {}", folder);
}
```

**POZNÁMKA**: Neextrahuje jednotlivé závislosti, jen označí že celá složka má být zkopírována!

---

## 🚊 `.zug` - Vlaky (train.rs)

**Funkce**: `extract_train_dependencies()`

**STRATEGIE**: **KOPÍRUJE CELÉ SLOŽKY PRO KAŽDÝ WAGON!**

```rust
lines = zug_content.lines();
i = 0;

while i < lines.len() {
    line = lines[i];

    if line.ends_with(".ovh") || line.ends_with(".bus") {
        vehicle_folder = Path::parent(line);

        if !folder.is_empty() && folder != "\\" && folder.contains("\\") {
            dependencies.insert("FOLDER:" + folder);
            println!("Will copy vehicle folder: {}", folder);
        }

        i += 2; // skip další řádek (config číslo)
    } else {
        i += 1;
    }
}
```

**Formát .zug souboru**:

```
Vehicles\Tramway\Wagon1.ovh
1
Vehicles\Tramway\Wagon2.ovh
2
```

---

## 🛤️ `.sli` - Splines (spline.rs)

**Funkce**: `extract_spline_dependencies()`

**Kódování**: Windows-1252

**Parsované sekce**:

### `[texture]`

```rust
tex_file = lines.next(); // např. "asphalt.dds"
base_name = tex_file.strip_extension();
add_texture_variants(base_name, sli_folder, omsi_root);
```

### `add_texture_variants()` - hledání variant

**Hledání v lokacích**:

1. `{sli_folder}/texture/`
2. `{sli_folder}/`
3. `Texture/` (globální)

**Pro každou lokaci**:

- Hledá všechny přípony: `.jpg`, `.bmp`, `.dds`, `.png`, `.tga`
- Hledá v podsložkách (všechny, ne jen seasonal)
- Přidává `.cfg` a `.surf` varianty
- Case-insensitive matching

---

## 🚶 `.hum` - Humans (human.rs)

**Funkce**: `extract_human_dependencies()`

**Kódování**: Windows-1252

**Parsované sekce**:

### `[model]`

```rust
cfg_path = lines.next(); // např. "model/man_cheap.cfg"
full_cfg_path = {hum_folder}/{cfg_path};
dependencies.insert(full_cfg_path);

// Parsovat model config → extract_cfg_dependencies()
```

### Model .cfg parsing (`extract_cfg_dependencies`)

#### `[CTC]`

```rust
lines.next(); // skip "Colorscheme" nebo prázdný řádek
tex_base_path = lines.next(); // např. "Texture\woman01"

// Volá add_textures_from_ctc_folder()
// → Přidá VŠECHNY textury z {human_base}/texture/{subfolder}/
```

#### `[mesh]`

```rust
mesh_file = lines.next(); // např. "body.o3d"
full_mesh_path = {cfg_folder}/{mesh_file};
dependencies.insert(full_mesh_path);

// POZNÁMKA: Neextrahuje textury z .o3d!
```

#### `[CTCTexture]`

```rust
lines.next(); // skip farbschema
tex_file = lines.next(); // např. "skin.jpg"

// Hledání (v pořadí):
// 1. {human_base}/texture/{base_path}/{tex_file}
// 2. {human_base}/texture/{tex_file}
// 3. Texture/{base_path}/{tex_file}
```

### `add_textures_from_ctc_folder()` - speciální funkce

```rust
subfolder = base_path.replace("Texture\\", ""); // např. "woman01"
texture_folder = {human_base}/texture/{subfolder}/;

// Přidá VŠECHNY .jpg/.bmp/.dds/.png/.tga soubory z této složky
for entry in WalkDir::new(texture_folder).max_depth(1) {
    if is_texture_extension(entry) {
        dependencies.insert(entry.path());
    }
}
```

---

## 🎨 `.o3d` - 3D Mesh (sceneryobject.rs)

**Funkce**: `extract_o3d_textures()`

**Typ**: Binární soubor

**Metoda**: Byte-level scanning

```rust
buffer = read_binary_file(o3d_path);
texture_extensions = [".bmp", ".tga", ".dds", ".jpg", ".jpeg", ".png",
                      ".BMP", ".TGA", ".DDS", ".JPG", ".JPEG", ".PNG"];

for ext in texture_extensions {
    i = 0;
    while i + ext.len() <= buffer.len() {
        if buffer[i..i+ext.len()] == ext {
            // Nalezena přípona, jdi zpět a hledej začátek názvu
            start = i;
            found_valid = false;

            while start > 0 {
                c = buffer[start - 1];

                // Platné znaky: A-Z, a-z, 0-9, _, -, ., \, /, #
                if is_valid_filename_char(c) {
                    start -= 1;
                    found_valid = true;
                } else {
                    break; // neplatný znak → konec názvu
                }
            }

            if found_valid {
                filename = String::from_utf8(buffer[start..i+ext.len()]);

                // Vyčistit od začátečních neplatných znaků
                cleaned = filename.skip_while(|c| !c.is_alphanumeric() && c != '_');

                if cleaned.len() > ext.len() && first_char_is_valid(cleaned) {
                    textures.push(cleaned);
                }
            }
        }
        i += 1;
    }
}
```

**Příklad**:

```
Binární: ((ŔBlavecka_nova.ddsy  €?
         ^^^^^ neplatné
             ^^^^^^^^^^^^^^ platné
                          ^^^^ přípona
Výsledek: "lavecka_nova.dds"
```

---

## 🎯 `.x` - DirectX Mesh (sceneryobject.rs)

**Funkce**: `extract_x_textures()`

**Typ**: Text NEBO binární

### Textový formát:

```rust
content = read_text_file(x_path);

for line in content.lines() {
    if line.contains("TextureFilename") {
        // Formát: TextureFilename { "texture.bmp"; }
        start = line.find('"');
        end = line[start+1..].find('"');
        tex_name = line[start+1..start+1+end];

        // Odstranit cestu, nechat jen filename
        if tex_name.contains('\\') {
            tex_name = tex_name.split('\\').last();
        }

        textures.push(tex_name);
    }
}
```

### Binární formát:

```rust
// Pokud text read selže
buffer = read_binary_file(x_path);
return extract_textures_from_binary(buffer);
// → Stejná logika jako extract_o3d_textures()
```

---

## ⚙️ Sound Config `.cfg` (sceneryobject.rs)

**Funkce**: `extract_sound_config_dependencies()`

**Kódování**: Windows-1252

**Parsování**:

```rust
cfg_content = read_file(cfg_path);

for line in cfg_content.lines() {
    if line.ends_with(".wav") {
        // Hledání (v pořadí):
        // 1. {cfg_folder}/sound/{wav_file}
        // 2. {cfg_folder}/{wav_file}
        // 3. {wav_file} (as-is)

        // Přidá se pouze pokud existuje
    }
}
```

---

## 🖼️ Textury - Varianty (sceneryobject.rs, spline.rs)

### Funkce společné pro .sco a .sli:

**`add_texture_variants(base_name, folder, omsi_root)`**

**Podporované přípony**: `.jpg`, `.jpeg`, `.bmp`, `.dds`, `.png`, `.tga`

**Lokace hledání** (v pořadí):

1. `{folder}/texture/` (např. `Sceneryobjects\ABC\texture\`)
2. `{folder}/` (např. `Sceneryobjects\ABC\`)
3. `Texture/` (globální)

**Pro každou lokaci hledá**:

#### 1. V hlavní složce

```rust
for ext in texture_extensions {
    file_path = "{base_name}.{ext}";
    if exists(file_path) {
        dependencies.insert(file_path);

        // Přidat .cfg a .surf
        if exists("{file_path}.cfg") {
            dependencies.insert("{file_path}.cfg");
        }
        if exists("{file_path}.surf") {
            dependencies.insert("{file_path}.surf");
        }
    }

    // Case-insensitive check
    file_path_lower = "{base_name.lowercase()}.{ext}";
    if exists(file_path_lower) {
        // ... stejná logika
    }
}
```

#### 2. V seasonal složkách

```rust
seasonal_folders = ["night", "Night", "alpha", "Alpha", "winter", "Winter",
                    "WinterSnow", "wintersnow", "spring", "Spring", "fall", "Fall"];

for subfolder in seasonal_folders {
    for ext in texture_extensions {
        file_path = "{subfolder}/{base_name}.{ext}";
        // ... stejná logika jako výše
    }
}
```

#### 3. V OSTATNÍCH podsložkách

```rust
for entry in read_dir(search_path) {
    if entry.is_dir() && !seasonal_folders.contains(entry.name()) {
        for ext in texture_extensions {
            file_path = "{entry.name()}/{base_name}.{ext}";
            // ... stejná logika
        }
    }
}
```

### **`search_textures_by_prefix(prefix, folder, omsi_root)`**

**Používá se**: Pro hledání textur podle názvu .sco souboru

**Příklad**: `Dum_cetkovice4.sco` → hledá `Dum_cetkovice4*`

```rust
for search_path in [texture/, ./, Texture/] {
    // Hlavní složka
    for entry in read_dir(search_path) {
        filename = entry.name();
        if filename.starts_with_ignore_case(prefix) {
            if has_texture_extension(filename) {
                dependencies.insert(filename);
                // + .cfg a .surf varianty
            }
        }
    }

    // Seasonal složky
    for subfolder in seasonal_folders {
        for entry in read_dir(search_path/subfolder) {
            // ... stejná logika
        }
    }
}
```

---

## 🔄 Rekurzivní zpracování (dependencies/mod.rs)

**Funkce**: `extract_nested_dependencies()`

```rust
for asset_path in asset_paths {
    match asset_type {
        "human" => {
            deps = human::extract_human_dependencies(asset_path);
            all_dependencies.extend(deps);
        }

        "sceneryobject" => {
            if asset_path.ends_with(".ovh") {
                deps = sceneryobject::extract_ovh_dependencies(asset_path);
            } else {
                deps = sceneryobject::extract_sceneryobject_dependencies(asset_path);
            }
            all_dependencies.extend(deps);
        }

        "spline" => {
            deps = spline::extract_spline_dependencies(asset_path);
            all_dependencies.extend(deps);
        }

        "vehicle" => {
            if asset_path.ends_with(".zug") {
                deps = train::extract_train_dependencies(asset_path);
            } else if asset_path.ends_with(".bus|.ovh|.sco") {
                deps = vehicle::extract_vehicle_dependencies(asset_path);
            }
            all_dependencies.extend(deps);
        }
    }
}
```

**POZNÁMKA**: Žádná ochrana proti cyklům! Pokud A odkazuje na B a B na A → nekonečný loop.

---

## ⚠️ Problémy v současné implementaci

### 1. Duplikace kódu

- `add_texture_variants()` existuje v `sceneryobject.rs` i `spline.rs` - identický kód
- `extract_textures_from_binary()` - používá se pro .o3d i .x

### 2. Kontextové zpracování

- `.ovh` se zpracovává 2× různě podle umístění:
  - V `Sceneryobjects` → detailní parsing sekcí
  - V `Vehicles` → kopíruje celou složku
- Rozhodování je v `mod.rs`, ne v extrakčních funkcích

### 3. Nekonzistence ve vehicles/trains

- `.bus` a `.ovh` v `Vehicles` → kopíruje celou složku (neparsuje obsah)
- `.zug` → parsuje a kopíruje složku každého vagonu
- Ale `.ovh` v `Sceneryobjects` → parsuje detailně!

### 4. Chybějící ochrana

- Žádná kontrola cyklických závislostí
- Žádný cache pro již zpracované soubory
- Každý soubor může být zpracován vícekrát

### 5. Human textury

- `extract_cfg_dependencies()` **NEEXTRAHUJE** textury z .o3d meshů
- Musí se spoléhat na `add_textures_from_ctc_folder()` který jen hádá složku

### 6. Textury z meshů

- .o3d a .x textury se extrahují jen když jsou v `[mesh]` nebo `[collision_mesh]`
- Pokud je mesh odkazován odjinud (např. human .cfg) → textury se neextrahují

---

## 📊 Statistika současných funkcí

| Soubor           | Funkce                             | Řádky | Rekurzivní? |
| ---------------- | ---------------------------------- | ----- | ----------- |
| sceneryobject.rs | extract_sceneryobject_dependencies | ~400  | Ano         |
| sceneryobject.rs | extract_ovh_dependencies           | ~240  | Ano         |
| sceneryobject.rs | extract_o3d_textures               | ~70   | Ne          |
| sceneryobject.rs | extract_x_textures                 | ~75   | Ne          |
| sceneryobject.rs | add_texture_variants               | ~175  | Ne          |
| sceneryobject.rs | search_textures_by_prefix          | ~125  | Ne          |
| human.rs         | extract_human_dependencies         | ~110  | Ano         |
| human.rs         | extract_cfg_dependencies           | ~180  | Ne          |
| spline.rs        | extract_spline_dependencies        | ~75   | Ano         |
| spline.rs        | add_texture_variants               | ~175  | Ne          |
| vehicle.rs       | extract_vehicle_dependencies       | ~30   | Ne          |
| train.rs         | extract_train_dependencies         | ~80   | Ne          |

**Celkem**: ~1900 řádků kódu pro extrakci závislostí
**Duplikovaný kód**: ~350 řádků (add_texture_variants × 2, extract_textures_from_binary)

---

## 📝 Speciální markery

### `FOLDER:` prefix

Používá se v `vehicle.rs` a `train.rs`:

```rust
dependencies.insert("FOLDER:Vehicles\\MAN\\Lions_City");
```

Znamená: "Zkopíruj CELOU složku, ne jednotlivé soubory"

**Zpracování** v bundling kódu:

```rust
if dep.starts_with("FOLDER:") {
    folder = dep.strip_prefix("FOLDER:");
    copy_entire_folder(folder);
} else {
    copy_single_file(dep);
}
```

---

## 🎯 Co by se mělo změnit (návrh)

1. **Unified extraction** - jeden způsob zpracování pro každou koncovku
2. **Remove context** - `.ovh` se zpracovává stejně všude
3. **Deduplicate** - sdílené funkce (textury, binární parsing)
4. **Cycle protection** - sledovat processed files
5. **Consistent strategy** - buď všechno parsovat, nebo všechno kopírovat složky
6. **Extract from all meshes** - i když jsou v human .cfg

---

Tento dokument popisuje **SKUTEČNÝ stav kódu k 25.12.2025**.
