const { invoke } = window.__TAURI__.core;

// App State
let appState = {
  saveDir: "",
  pythonPath: "default",
  players: [],
  activePlayer: null,
  activeInventory: [],
  selectedSlotIndex: null,
};

// DOM Elements
let el = {};

function initDOMElements() {
  el.saveDirInput = document.querySelector("#save-dir-input");
  el.btnDetectSave = document.querySelector("#btn-detect-save");
  el.btnLoadSave = document.querySelector("#btn-load-save");
  el.playerListContainer = document.querySelector("#player-list-container");
  el.statusDot = document.querySelector("#status-dot");
  el.statusText = document.querySelector("#status-text");
  el.activePlayerName = document.querySelector("#active-player-name");
  el.activeContainerType = document.querySelector("#active-container-type");
  el.inventoryGridContainer = document.querySelector("#inventory-grid-container");
  el.consoleLogsContainer = document.querySelector("#console-logs-container");
  el.consoleClear = document.querySelector("#console-clear");
  el.editSlotEmpty = document.querySelector("#edit-slot-empty");
  el.editSlotForm = document.querySelector("#edit-slot-form");
  el.itemIdInput = document.querySelector("#item-id-input");
  el.itemCountInput = document.querySelector("#item-count-input");
  el.btnSaveSlot = document.querySelector("#btn-save-slot");
  el.btnDeleteSlot = document.querySelector("#btn-delete-slot");
  el.btnCleanSeeds = document.querySelector("#btn-clean-seeds");
  el.btnCleanMaterials = document.querySelector("#btn-clean-materials");
  el.btnApplySav = document.querySelector("#btn-apply-sav");
}

// Log utility
function log(message, type = "info") {
  const entry = document.createElement("div");
  entry.className = `log-entry ${type}`;
  const timestamp = new Date().toLocaleTimeString();
  entry.textContent = `[${timestamp}] ${message}`;
  el.consoleLogsContainer.appendChild(entry);
  el.consoleLogsContainer.scrollTop = el.consoleLogsContainer.scrollHeight;
}

// Set application status
function setStatus(status, text) {
  el.statusDot.className = "status-dot";
  if (status === "active") {
    el.statusDot.classList.add("active");
  } else if (status === "loading") {
    el.statusDot.classList.add("loading");
  }
  el.statusText.textContent = text;
}

// Get color for slot based on item category
function getItemColor(itemId) {
  const id = itemId.toLowerCase();
  if (id.includes("seed")) return "#10b981"; // Green for seeds
  if (id.includes("sphere")) return "#06b6d4"; // Cyan for spheres
  if (id.includes("meat") || id.includes("egg") || id.includes("berry") || id.includes("bread")) return "#f59e0b"; // Orange/Yellow for food
  if (id.includes("armor") || id.includes("shield") || id.includes("helmet")) return "#8b5cf6"; // Purple for armor
  if (id.includes("bat") || id.includes("pickaxe") || id.includes("axe") || id.includes("bow") || id.includes("spear")) return "#ef4444"; // Red for weapons/tools
  if (id === "wood" || id === "stone" || id.includes("ore") || id.includes("crystal") || id.includes("wool") || id.includes("fiber")) return "#9ca3af"; // Gray for raw materials
  return "#6366f1"; // Indigo default
}

// Detect save directory automatically on start
async function autoDetectSavePath() {
  try {
    setStatus("loading", "Detectando guardado...");
    const path = await invoke("get_default_save_dir");
    if (path) {
      el.saveDirInput.value = path;
      appState.saveDir = path;
      log(`Directorio de guardado detectado en: ${path}`, "success");
      setStatus("active", "Partida detectada");
    } else {
      log("No se pudo detectar automáticamente la ruta de guardados. Por favor, ingrésala manualmente.", "warning");
      setStatus("idle", "Listo");
    }
  } catch (err) {
    log(`Error al detectar ruta de guardados: ${err}`, "error");
    setStatus("idle", "Listo");
  }
}

// Load the save game file
async function loadSaveGame() {
  const dir = el.saveDirInput.value.trim();
  if (!dir) {
    log("Por favor, selecciona o ingresa una ruta de guardado válida.", "warning");
    return;
  }

  appState.saveDir = dir;
  setStatus("loading", "Cargando partida...");
  log("Iniciando carga de partida. Esto puede tardar unos segundos...", "info");

  try {
    // 1. Convert Level.sav and scan players
    const players = await invoke("load_save_file", {
      pythonPath: appState.pythonPath,
      saveDir: appState.saveDir,
    });

    appState.players = players;
    log(`¡Carga exitosa! Se detectaron ${players.length} perfiles de jugador.`, "success");
    setStatus("active", "Partida cargada");

    renderPlayerList();
    resetActivePlayerView();

  } catch (err) {
    log(`Error crítico al decodificar partida: ${err}`, "error");
    setStatus("idle", "Error de carga");
  }
}

// Render the player list in the sidebar
function renderPlayerList() {
  el.playerListContainer.innerHTML = "";
  if (appState.players.length === 0) {
    el.playerListContainer.innerHTML = `<div class="empty-state">No se encontraron jugadores</div>`;
    return;
  }

  appState.players.forEach((player) => {
    const card = document.createElement("div");
    card.className = "player-card";
    if (appState.activePlayer && appState.activePlayer.player_uid === player.player_uid) {
      card.classList.add("active");
    }

    const initials = player.nickname.substring(0, 2).toUpperCase();
    card.innerHTML = `
      <div class="player-avatar">${initials}</div>
      <div class="player-info">
        <span class="player-name">${player.nickname}</span>
        <span class="player-uid">${player.player_uid.substring(0, 8)}...</span>
      </div>
    `;

    card.addEventListener("click", () => selectPlayer(player));
    el.playerListContainer.appendChild(card);
  });
}

// Select player and load inventory
async function selectPlayer(player) {
  appState.activePlayer = player;
  document.querySelectorAll(".player-card").forEach((c, idx) => {
    c.classList.toggle("active", appState.players[idx].player_uid === player.player_uid);
  });

  el.activePlayerName.textContent = player.nickname;
  el.activeContainerType.textContent = "Mochila Común";
  log(`Cargando inventario de ${player.nickname}...`, "info");
  setStatus("loading", "Cargando mochila...");

  try {
    const levelJsonPath = `${appState.saveDir}/Level.tmp.json`;
    const items = await invoke("get_container_items", {
      levelJsonPath,
      containerGuid: player.common_container_id,
    });

    appState.activeInventory = items;
    log(`Mochila de ${player.nickname} cargada con ${items.length} objetos.`, "success");
    setStatus("active", "Partida cargada");

    renderInventoryGrid();
    closeEditPanel();

  } catch (err) {
    log(`Error al cargar mochila del jugador: ${err}`, "error");
    setStatus("active", "Partida cargada");
  }
}

// Render the inventory grid
function renderInventoryGrid() {
  el.inventoryGridContainer.innerHTML = "";
  const totalSlots = 40; // Palworld common container default size is usually 40 slots

  // Create an array mapping slot_index -> item
  const slotsMap = new Array(totalSlots).fill(null);
  appState.activeInventory.forEach((item) => {
    if (item.slot_index < totalSlots) {
      slotsMap[item.slot_index] = item;
    }
  });

  for (let idx = 0; idx < totalSlots; idx++) {
    const slot = document.createElement("div");
    slot.className = "inventory-slot";
    if (appState.selectedSlotIndex === idx) {
      slot.classList.add("active");
    }

    const item = slotsMap[idx];
    slot.innerHTML = `<span class="slot-index">${idx + 1}</span>`;

    if (item) {
      const initials = item.item_id.substring(0, 3).toUpperCase();
      const color = getItemColor(item.item_id);
      
      const itemVisual = document.createElement("div");
      itemVisual.style.display = "flex";
      itemVisual.style.flexDirection = "column";
      itemVisual.style.alignItems = "center";
      itemVisual.style.justifyContent = "center";
      itemVisual.style.width = "40px";
      itemVisual.style.height = "40px";
      itemVisual.style.borderRadius = "8px";
      itemVisual.style.border = `2px solid ${color}`;
      itemVisual.style.background = `${color}20`;
      itemVisual.style.fontSize = "10px";
      itemVisual.style.fontWeight = "700";
      itemVisual.style.color = color;
      itemVisual.textContent = initials;

      slot.appendChild(itemVisual);
      slot.innerHTML += `<span class="item-count">${item.count}</span>`;
    }

    slot.addEventListener("click", () => selectSlot(idx, item));
    el.inventoryGridContainer.appendChild(slot);
  }
}

// Select inventory slot
function selectSlot(idx, item) {
  appState.selectedSlotIndex = idx;
  document.querySelectorAll(".inventory-slot").forEach((s, sIdx) => {
    s.classList.toggle("active", sIdx === idx);
  });

  el.editSlotEmpty.style.display = "none";
  el.editSlotForm.style.display = "flex";

  if (item) {
    el.itemIdInput.value = item.item_id;
    el.itemCountInput.value = item.count;
  } else {
    el.itemIdInput.value = "";
    el.itemCountInput.value = "";
  }
}

// Save active slot modifications
async function saveSlotChanges() {
  if (appState.selectedSlotIndex === null || !appState.activePlayer) return;

  const itemId = el.itemIdInput.value.trim();
  const count = parseInt(el.itemCountInput.value) || 0;
  const levelJsonPath = `${appState.saveDir}/Level.tmp.json`;
  const containerGuid = appState.activePlayer.common_container_id;

  setStatus("loading", "Actualizando slot...");
  log(`Modificando slot ${appState.selectedSlotIndex + 1}...`, "info");

  try {
    await invoke("modify_container_item", {
      levelJsonPath,
      containerGuid,
      slotIndex: appState.selectedSlotIndex,
      itemId,
      count,
    });

    log(`¡Slot ${appState.selectedSlotIndex + 1} modificado con éxito!`, "success");
    
    // Refresh inventory data
    const items = await invoke("get_container_items", {
      levelJsonPath,
      containerGuid,
    });

    appState.activeInventory = items;
    renderInventoryGrid();
    setStatus("active", "Partida cargada");

  } catch (err) {
    log(`Error al modificar slot: ${err}`, "error");
    setStatus("active", "Partida cargada");
  }
}

// Delete item inside active slot
async function deleteActiveSlot() {
  el.itemIdInput.value = "";
  el.itemCountInput.value = "0";
  await saveSlotChanges();
}

// Bulk cleaning of seeds
async function cleanAllSeeds() {
  if (!appState.activePlayer) return;
  
  setStatus("loading", "Limpiando semillas...");
  log("Limpiando todas las semillas de la mochila del jugador...", "info");
  
  const levelJsonPath = `${appState.saveDir}/Level.tmp.json`;
  const containerGuid = appState.activePlayer.common_container_id;
  let clearedCount = 0;

  try {
    for (const item of appState.activeInventory) {
      const id = item.item_id.toLowerCase();
      if (id.includes("seed") || id.endsWith("seeds")) {
        await invoke("modify_container_item", {
          levelJsonPath,
          containerGuid,
          slotIndex: item.slot_index,
          itemId: "",
          count: 0,
        });
        clearedCount++;
      }
    }

    log(`Limpieza de semillas completada. Se vaciaron ${clearedCount} slots.`, "success");

    // Refresh inventory data
    const items = await invoke("get_container_items", {
      levelJsonPath,
      containerGuid,
    });

    appState.activeInventory = items;
    renderInventoryGrid();
    closeEditPanel();
    setStatus("active", "Partida cargada");

  } catch (err) {
    log(`Error al limpiar semillas: ${err}`, "error");
    setStatus("active", "Partida cargada");
  }
}

// Bulk cleaning of raw wood and stones
async function cleanWoodAndStones() {
  if (!appState.activePlayer) return;
  
  setStatus("loading", "Limpiando materiales...");
  log("Limpiando madera y piedras de la mochila...", "info");
  
  const levelJsonPath = `${appState.saveDir}/Level.tmp.json`;
  const containerGuid = appState.activePlayer.common_container_id;
  let clearedCount = 0;

  try {
    for (const item of appState.activeInventory) {
      const id = item.item_id.toLowerCase();
      if (id === "wood" || id === "stone") {
        await invoke("modify_container_item", {
          levelJsonPath,
          containerGuid,
          slotIndex: item.slot_index,
          itemId: "",
          count: 0,
        });
        clearedCount++;
      }
    }

    log(`Limpieza de madera y piedra completada. Se vaciaron ${clearedCount} slots.`, "success");

    // Refresh inventory data
    const items = await invoke("get_container_items", {
      levelJsonPath,
      containerGuid,
    });

    appState.activeInventory = items;
    renderInventoryGrid();
    closeEditPanel();
    setStatus("active", "Partida cargada");

  } catch (err) {
    log(`Error al limpiar materiales básicos: ${err}`, "error");
    setStatus("active", "Partida cargada");
  }
}

// Save all changes back to Level.sav binary
async function saveAllToSav() {
  if (!appState.saveDir) {
    log("No hay una partida cargada para guardar.", "warning");
    return;
  }

  setStatus("loading", "Guardando partida...");
  log("Escribiendo y comprimiendo cambios en Level.sav. Por favor espera...", "info");

  try {
    await invoke("apply_save_changes", {
      pythonPath: appState.pythonPath,
      saveDir: appState.saveDir,
    });

    log("¡Level.sav guardado con éxito! Se creó una copia de seguridad en Level.sav.bak.", "success");
    log("Ya puedes abrir el juego o iniciar el servidor.", "success");
    setStatus("active", "Cambios aplicados");
    
    // Reset view as temporary files are cleaned
    resetActivePlayerView();
    el.playerListContainer.innerHTML = `<div class="empty-state">Vuelve a cargar la partida</div>`;
    appState.players = [];

  } catch (err) {
    log(`Error al guardar Level.sav: ${err}`, "error");
    setStatus("active", "Partida cargada");
  }
}

// Helper UI Resets
function resetActivePlayerView() {
  el.activePlayerName.textContent = "Selecciona un Jugador";
  el.inventoryGridContainer.innerHTML = `<div class="empty-state">Selecciona un jugador en la barra lateral</div>`;
  closeEditPanel();
}

function closeEditPanel() {
  appState.selectedSlotIndex = null;
  el.editSlotEmpty.style.display = "flex";
  el.editSlotForm.style.display = "none";
}

// Initialization
window.addEventListener("DOMContentLoaded", () => {
  initDOMElements();

  // Event Listeners
  el.btnDetectSave.addEventListener("click", autoDetectSavePath);
  el.btnLoadSave.addEventListener("click", loadSaveGame);
  el.btnSaveSlot.addEventListener("click", saveSlotChanges);
  el.btnDeleteSlot.addEventListener("click", deleteActiveSlot);
  el.btnCleanSeeds.addEventListener("click", cleanAllSeeds);
  el.btnCleanMaterials.addEventListener("click", cleanWoodAndStones);
  el.btnApplySav.addEventListener("click", saveAllToSav);
  el.consoleClear.addEventListener("click", () => {
    el.consoleLogsContainer.innerHTML = "";
  });

  // Start path detection automatically
  autoDetectSavePath();
});
