---
name: palworld-save-parser
description: Especificación técnica y guías de desarrollo para la lectura, descompresión y análisis de archivos de guardado de Palworld (Level.sav y Players/*.sav) directamente desde binario a JSON estructurado o reportes legibles usando Rust puro.
---

# Palworld Save Parser Skill

Esta Skill documenta la estructura, formatos y flujos binarios necesarios para leer y extraer información de las partidas guardadas de Palworld utilizando la herramienta de línea de comandos (CLI) escrita en Rust puro en este proyecto.

## 1. Arquitectura de Archivos de Guardado

El estado de la partida se compone de:

1. **Level.sav**:
   * Archivo global del mundo. Contiene los contenedores de inventario en `ItemContainerSaveData` y los personajes/Pals registrados en `CharacterSaveParameterMap`.
2. **Players/<PlayerUID>.sav**:
   * Archivo de metadatos del jugador individual. Almacena las referencias GUID de sus contenedores específicos (`CommonContainerId`, `WeaponLoadOutContainerId`, `PlayerEquipArmorContainerId`, `OtomoCharacterContainerId`).

---

## 2. Descompresión de Archivos GVAS (.sav)

Los archivos de guardado de Palworld utilizan el formato GVAS (Unreal Engine Save Game). Su cabecera indica el método de compresión:
* **Firma "PlZ"**: Comprimido usando Zlib (cabeceras simples o dobles). Se descomprime mediante `flate2`.
* **Firma "PlM"**: Comprimido usando Oodle. Requiere cargar dinámicamente la biblioteca de enlace dinámico `oo2core_9_win64.dll` y ejecutar el símbolo `OodleLZ_Decompress`.

---

## 3. Lógica de Lectura y Mapeo en Rust Puro (Sin Tauri / Sin Python)

El proyecto actual funciona como una herramienta CLI pura en [main.rs](file:///C:/workspace/palsync-ai-liveagent/src/main.rs). La lectura se realiza a nivel de análisis binario en memoria sin conversión previa a JSON:

### Paso 1: Extracción de Nicknames y Estadísticas de Personajes
El CLI escanea los bytes de `Level.sav` buscando el patrón `NickName\x00`. Cuando localiza un nombre, retrocede en el búfer de bytes para encontrar la propiedad `PlayerUId\x00` y asocia el UID (GUID) con su apodo. También extrae propiedades como `Level\x00`, `Exp\x00`, `Hp\x00`, `MaxHp\x00`, `FullStomach\x00`, `PhysicalHealth\x00` y datos estéticos en `SkinChange\x00` (como `BodyMeshName`, `HeadMeshName`, `HairMeshName`, `VoiceID`).

### Paso 2: Extracción de GUIDs de Contenedores del Jugador
Para saber qué contenedores corresponden a cada jugador, el CLI lee el archivo individual `Players/<PlayerUID>.sav` y busca las propiedades `CommonContainerId\x00`, `WeaponLoadOutContainerId\x00`, `PlayerEquipArmorContainerId\x00` y `OtomoCharacterContainerId\x00`. Localiza la firma `Guid` y copia los 16 bytes de payload correspondientes a cada contenedor.

### Paso 3: Decodificación de Ítems en Level.sav
Con el GUID del contenedor en formato binario, el CLI busca su posición correspondiente en `Level.sav` (verificando que esté seguido por la propiedad `BelongInfo\x00`). Una vez hallado, localiza la propiedad `Slots\x00` en la estructura de datos y procesa cada elemento `RawData` saltando correctamente los metadatos de tipo con `skip_property_at`:
* **Bytes 0-3**: Ranura de inventario (UInt32 Little-Endian).
* **Bytes 4-7**: Cantidad apilada del ítem (UInt32 Little-Endian).
* **Bytes 8-11**: Longitud de la cadena del ID del ítem (UInt32 Little-Endian).
* **Bytes 12+**: ID del ítem (UTF-8, omitiendo el byte nulo final).

### Paso 4: Extracción de Pals Activos (Equipo)
Los Pals del jugador se almacenan en `CharacterSaveParameterMap` en `Level.sav`. Para filtrar los Pals que están activos en el equipo del jugador:
1. Compara `instance_id` de la criatura para asegurarse de que no sea el propio personaje jugador.
2. Comprueba que el campo `OwnerPlayerUId\x00` coincida con el UID del jugador analizado.
3. Comprueba que el campo `ContainerId\x00` del Pal coincida con el `OtomoCharacterContainerId\x00` del jugador.
4. Si las condiciones se cumplen, extrae las estadísticas del Pal (ID interna, Nivel, Experiencia, Salud actual/máxima, Saciación, Amistad, Slot y Género).

---

## 4. Uso del CLI y Salida de Datos

### Ejecución de Comandos
El CLI cuenta con autodetección de la partida activa en Windows, pero también acepta una ruta de guardado personalizada como argumento:

```bash
# Ejecución estándar (genera un reporte detallado y amigable en terminal)
cargo run

# Generación de salida estructurada JSON (ideal para alimentar modelos de IA)
cargo run -- --json

# Ruta de guardado personalizada como argumento de entrada
cargo run -- "C:\ruta\al\mundo\guardado" --json
```

### Formato del Reporte de Texto (Consola)
La ejecución por defecto muestra un informe detallado con el siguiente diseño:

```text
================================================================================
                PALWORLD CHARACTER & SAVE FILE ANALYSIS REPORT                  
================================================================================
World Save Path: C:\Users\...

--------------------------------------------------------------------------------
  PLAYER PROFILE: SeWaNsX
--------------------------------------------------------------------------------
  * Player UID            : 00000000-0000-0000-0000-000000000001
  * Level                 : 4
  * Health (HP)           : 217.50
  ...
  [Active Team Pals (2)]
    1. Sheepball [Level 4] (Male)
       - HP               : 682.00 / 0.00
       - Satiety          : 150.00
    ...
```

### Formato de Salida JSON (`--json`)
La ejecución con el flag `--json` genera un JSON estructurado consumible por programas:

```json
{
  "status": "success",
  "world_path": "C:\\Users\\...",
  "players": [
    {
      "player_uid": "00000000-0000-0000-0000-000000000001",
      "nickname": "SeWaNsX",
      "level": 4,
      "customization": {
        "BodyMeshName": "TypeA",
        "HairMeshName": "Type7",
        "HeadMeshName": "type1",
        "VoiceID": 1
      },
      "common_inventory": [
        { "slot_index": 0, "item_id": "wood", "count": 999 }
      ],
      "weapons": [
        { "slot_index": 0, "item_id": "Bat", "count": 1 }
      ],
      "armor": [],
      "active_pals": [
        {
          "character_id": "Sheepball",
          "gender": "EPalGenderType::Male",
          "level": 4,
          "hp": 682.0,
          "slot_index": 0
        }
      ]
    }
  ]
}
```
