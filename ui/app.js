'use strict';

const invoke = window.__TAURI__.core.invoke;

const config = {
  windowTitle: 'About This PiForma',
  appTitle: 'PiForma OS',
  iconAlt: 'PiForma OS Finder-style icon',
  iconFallbackText: 'PF',
  errorMessage: 'The PiForma OS system information could not be read right now.',
  loadErrorMessage: 'Sorry, About This PiForma OS cannot read system information.',
  hardwareLabels: {},
  ...window.ABOUT_THIS_COMPUTER_CONFIG
};

const elements = {
  desktop: document.querySelector('.desktop'),
  window: document.querySelector('.window'),
  windowTitle: document.getElementById('window-title'),
  appTitle: document.getElementById('appTitle'),
  version: document.getElementById('version'),
  hardware: document.getElementById('hardware'),
  builtInMemory: document.getElementById('builtInMemory'),
  virtualMemory: document.getElementById('virtualMemory'),
  largestUnusedBlock: document.getElementById('largestUnusedBlock'),
  processList: document.getElementById('processList'),
  errorBox: document.getElementById('errorBox'),
  lastUpdated: document.getElementById('lastUpdated'),
  finderIcon: document.getElementById('finderIcon'),
  iconFallback: document.getElementById('iconFallback'),
  closeButton: document.getElementById('closeButton'),
  minimizeButton: document.getElementById('minimizeButton'),
  collapseButton: document.getElementById('collapseButton')
};

function applyConfig() {
  document.title = config.windowTitle;
  elements.desktop.setAttribute('aria-label', config.windowTitle);
  elements.windowTitle.textContent = config.windowTitle;
  elements.finderIcon.alt = config.iconAlt;
  elements.iconFallback.textContent = config.iconFallbackText;
  elements.errorBox.textContent = config.errorMessage;
  setText('appTitle', config.appTitle);
}

function setText(id, value) {
  elements[id].textContent = value || 'Unknown';
}

function formatHardware(value) {
  return (config.hardwareLabels || {})[value] || value;
}

function formatTime(date) {
  return date.toLocaleTimeString([], {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  });
}

function showError(message) {
  elements.errorBox.hidden = false;
  elements.errorBox.textContent = message || config.errorMessage;
  elements.lastUpdated.textContent = 'Update failed';
}

function clearError() {
  elements.errorBox.hidden = true;
}

function renderProcesses(processes) {
  if (!Array.isArray(processes) || processes.length === 0) {
    elements.processList.innerHTML = '<div class="empty-row">No process memory data is available.</div>';
    return;
  }

  const maxBytes = Math.max(...processes.map((process) => process.memoryBytes || 0), 1);
  const fragment = document.createDocumentFragment();

  for (const process of processes) {
    const row = document.createElement('div');
    row.className = 'process-row';

    const name = document.createElement('div');
    name.className = 'process-name';
    name.textContent = process.name || 'Unknown';
    name.title = process.name || 'Unknown';

    const memory = document.createElement('div');
    memory.className = 'process-memory';
    memory.textContent = process.memory || '0.0 MB';

    const track = document.createElement('div');
    track.className = 'usage-track';
    track.setAttribute('aria-hidden', 'true');

    const fill = document.createElement('div');
    fill.className = 'usage-fill';
    fill.style.width = `${Math.max(3, Math.round(((process.memoryBytes || 0) / maxBytes) * 100))}%`;

    track.appendChild(fill);
    row.append(name, memory, track);
    fragment.appendChild(row);
  }

  elements.processList.replaceChildren(fragment);
}

function toggleTitleCollapse() {
  elements.window.classList.toggle('is-collapsed');
}

function startWindowDrag(event) {
  if (event.button !== 0 || event.target.closest('.control-button')) return;

  invoke('start_window_drag');
}

async function refreshSystemInfo() {
  try {
    const data = await invoke('get_system_info');

    setText('appTitle', config.appTitle || data.title);
    setText('version', data.version);
    setText('hardware', formatHardware(data.hardware));
    setText('builtInMemory', data.builtInMemory);
    setText('virtualMemory', data.virtualMemory);
    setText('largestUnusedBlock', data.largestUnusedBlock);
    renderProcesses(data.processes);

    clearError();
    elements.lastUpdated.textContent = `Updated ${formatTime(new Date())}`;
  } catch (error) {
    showError(config.loadErrorMessage);
  }
}

elements.finderIcon.addEventListener('error', () => {
  elements.finderIcon.closest('.icon-frame').classList.add('missing');
});

elements.closeButton.addEventListener('click', () => {
  invoke('close_app');
});

elements.minimizeButton.addEventListener('click', () => {});

elements.collapseButton.addEventListener('click', toggleTitleCollapse);

const titlebar = elements.window.querySelector('.titlebar');

titlebar.addEventListener('mousedown', startWindowDrag);

titlebar.addEventListener('dblclick', (event) => {
  if (event.target.closest('.control-button')) return;

  toggleTitleCollapse();
});

applyConfig();
refreshSystemInfo();
setInterval(refreshSystemInfo, 5000);
