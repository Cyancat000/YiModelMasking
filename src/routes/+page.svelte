<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { open } from '@tauri-apps/plugin-dialog';

  interface FileItem {
    path: string;
    name: string;
    status: 'pending' | 'processing' | 'success' | 'error';
    message?: string;
  }

  let files: FileItem[] = [];
  let processing = false;
  let unlisten: () => void;

  // Presets
const presets = [
  {
    id: 'pen-f',
    name: 'Olympus PEN-F',
    make: 'OLYMPUS CORPORATION',
    model: 'PEN-F',
  },
  {
    id: 'e-m1',
    name: 'Olympus E-M1',
    make: 'OLYMPUS CORPORATION',
    model: 'E-M1',
  },
  {
    id: 'yi-m1',
    name: 'YI M1',
    make: 'YI Technology',
    model: 'YI M1',
  },
  {
    id: 'gx8',
    name: 'Panasonic Lumix GX8',
    make: 'Panasonic',
    model: 'DMC-GX8',
  },
  {
    id: 'g9',
    name: 'Panasonic Lumix G9',
    make: 'Panasonic',
    model: 'DC-G9',
  },
  {
    id: 'g100',
    name: 'Panasonic Lumix G100',
    make: 'Panasonic',
    model: 'DC-G100',
  },
  {
    id: 'e-m5-iii',
    name: 'Olympus E-M5 Mark III',
    make: 'OLYMPUS CORPORATION',
    model: 'E-M5MarkIII',
  },
  {
    id: 'e-m10-iv',
    name: 'Olympus E-M10 Mark IV',
    make: 'OLYMPUS CORPORATION',
    model: 'E-M10MarkIV',
  },
  {
    id: 'e-p7',
    name: 'Olympus PEN E-P7',
    make: 'OLYMPUS CORPORATION',
    model: 'E-P7',
  }
];

  let selectedPreset = presets[0];

  let isDraggingOver = false;

  onMount(async () => {
    console.log('Mounting component, setting up listeners...');
    
    // Listen for file drop (Tauri v2)
    const unlistenDragDrop = await listen('tauri://drag-drop', (event) => {
      console.log('Tauri drag-drop event:', event);
      isDraggingOver = false;
      
      // Handle v2 payload structure: { paths: string[], position: { x, y } }
      // or fallback to v1 string[] if structure differs
      const payload = event.payload as any;
      const paths = payload.paths || (Array.isArray(payload) ? payload : []);
      
      if (Array.isArray(paths) && paths.length > 0) {
          addFiles(paths);
      } else {
          console.warn('Invalid drag-drop payload:', payload);
      }
    });

    // Listen for drag enter
    const unlistenDragEnter = await listen('tauri://drag-enter', () => {
        console.log('Tauri drag-enter');
        isDraggingOver = true;
    });

    // Listen for drag leave
    const unlistenDragLeave = await listen('tauri://drag-leave', () => {
        console.log('Tauri drag-leave');
        isDraggingOver = false;
    });
    
    // Keep legacy listener just in case
    const unlistenFileDrop = await listen('tauri://file-drop', (event) => {
      console.log('Legacy file-drop event:', event);
      const paths = event.payload as string[];
      addFiles(paths);
    });
    
    return () => {
        unlistenDragDrop();
        unlistenDragEnter();
        unlistenDragLeave();
        unlistenFileDrop();
    }
  });

  onDestroy(() => {
    if (unlisten) unlisten(); // Clean up the initial unlisten if set
  });

  function addFiles(paths: string[]) {
      console.log('Adding files:', paths);
      // Filter for DNG (case insensitive)
      const dngFiles = paths.filter(p => p.toLowerCase().endsWith('.dng'));
      
      if (dngFiles.length === 0) {
          console.warn('No DNG files found in selection');
      }

      const newFiles = dngFiles.map(path => ({
        path,
        name: path.split(/[\\/]/).pop() || path,
        status: 'pending' as const
      }));

      // Avoid duplicates
      files = [...files, ...newFiles.filter(nf => !files.some(f => f.path === nf.path))];
      console.log('Current file list:', files);
  }

  async function openFileDialog() {
      console.log('Opening file dialog...');
      try {
          const selected = await open({
              multiple: true,
              filters: [{
                  name: 'DNG Images',
                  extensions: ['dng']
              }]
          });
          
          console.log('Dialog selection:', selected);

          if (selected) {
               // Handle both single string (if multiple: false) and array
               const paths = Array.isArray(selected) ? selected : [selected];
               // In Tauri v2 dialog plugin, open returns string[] directly when multiple is true
               addFiles(paths); 
           }
      } catch (err) {
          console.error('Failed to open dialog:', err);
      }
  }

  async function processFiles() {
    console.log('Starting processing...');
    if (files.length === 0) return;
    processing = true;

    const pendingFiles = files.filter(f => f.status !== 'success');
    const filePaths = pendingFiles.map(f => f.path);
    
    if (filePaths.length === 0) {
        console.log('No pending files to process');
        processing = false;
        return;
    }

    // Prepare arguments
    // -m: Ignore minor errors
    // -overwrite_original: Overwrite original file
    // -Make="OLYMPUS CORPORATION"
    // -Model="PEN-F"
    // -ImageDescription="yi2olympus"
    const args = [
      '-m',
      '-overwrite_original',
      `-Make=${selectedPreset.make}`,
      `-Model=${selectedPreset.model}`,
      // Optionally set ImageDescription to mark processed files
      `-ImageDescription=Processed by yi2olympus`, 
      ...filePaths
    ];
    
    console.log('ExifTool args:', args);

    try {
        // Update UI to processing
        files = files.map(f => pendingFiles.some(pf => pf.path === f.path) ? { ...f, status: 'processing' } : f);

        const result = await invoke('run_exiftool', { args });
        console.log('ExifTool output:', result);

        // Assume success if no error thrown
        files = files.map(f => f.status === 'processing' ? { ...f, status: 'success' } : f);

    } catch (error) {
        console.error('ExifTool error:', error);
        files = files.map(f => f.status === 'processing' ? { ...f, status: 'error', message: String(error) } : f);
    } finally {
        processing = false;
    }
  }

  function removeFile(path: string) {
    console.log('Removing file:', path);
    files = files.filter(f => f.path !== path);
  }

  function clearFiles() {
      if (processing) return;
      files = [];
  }
</script>

<main class="h-screen w-screen bg-gray-900 text-white font-sans flex flex-col p-8 select-none">
  <div class="mb-8">
      <h1 class="text-3xl font-bold bg-gradient-to-r from-blue-400 to-purple-500 bg-clip-text text-transparent">
          Yi M1 to Olympus
      </h1>
      <p class="text-gray-400 mt-2">Drag and drop DNG files to disguise them as Olympus cameras.</p>
  </div>

  <!-- Preset Selector -->
  <div class="mb-6 bg-gray-800/50 p-4 rounded-xl border border-gray-700">
      <label class="block text-sm font-medium text-gray-300 mb-2">Target Camera Model</label>
      <div class="flex gap-4">
          {#each presets as preset}
              <button 
                  class="px-4 py-2 rounded-lg transition-all {selectedPreset.id === preset.id ? 'bg-blue-600 text-white shadow-lg shadow-blue-500/30' : 'bg-gray-700 text-gray-300 hover:bg-gray-600'}"
                  on:click={() => selectedPreset = preset}
              >
                  {preset.name}
              </button>
          {/each}
      </div>
  </div>

  <!-- Drop Zone / File List -->
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div 
    class="flex-1 bg-gray-800/30 rounded-2xl border-2 border-dashed flex flex-col overflow-hidden relative transition-colors cursor-pointer {isDraggingOver ? 'border-blue-500 bg-blue-500/10' : 'border-gray-700 hover:border-gray-500'}"
    on:click={openFileDialog}
    on:dragover|preventDefault={(e) => {
        // Prevent default to allow drop
        e.dataTransfer!.dropEffect = 'copy';
        console.log('HTML5 DragOver');
    }}
    on:drop|preventDefault={(e) => console.log('HTML5 Drop:', e)}
  >
      
      {#if files.length === 0}
          <div class="absolute inset-0 flex flex-col items-center justify-center text-gray-500 pointer-events-none">
              <svg class="w-16 h-16 mb-4 opacity-50 {isDraggingOver ? 'text-blue-500 animate-bounce' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"></path></svg>
              <p class="text-lg">{isDraggingOver ? 'Drop files now!' : 'Drag & Drop DNG files here or Click to Select'}</p>
          </div>
      {:else}
          <div class="flex-1 overflow-y-auto p-4 space-y-2" on:click|stopPropagation>
              {#each files as file (file.path)}
                  <div class="flex items-center justify-between bg-gray-800 p-3 rounded-lg border border-gray-700 group animate-in fade-in slide-in-from-bottom-2 duration-300">
                      <div class="flex items-center gap-3 overflow-hidden">
                          <div class="w-8 h-8 rounded bg-gray-700 flex items-center justify-center flex-shrink-0 text-xs text-gray-400">DNG</div>
                          <div class="truncate text-sm text-gray-200" title={file.path}>{file.name}</div>
                      </div>
                      
                      <div class="flex items-center gap-3">
                          {#if file.status === 'success'}
                              <span class="text-green-400 text-xs font-medium px-2 py-1 bg-green-400/10 rounded">Done</span>
                          {:else if file.status === 'error'}
                              <span class="text-red-400 text-xs font-medium px-2 py-1 bg-red-400/10 rounded" title={file.message}>Error</span>
                          {:else if file.status === 'processing'}
                              <span class="text-blue-400 text-xs font-medium px-2 py-1 bg-blue-400/10 rounded animate-pulse">Processing...</span>
                          {:else}
                              <button on:click={() => removeFile(file.path)} class="text-gray-500 hover:text-red-400 p-1 opacity-0 group-hover:opacity-100 transition-opacity">
                                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
                              </button>
                          {/if}
                      </div>
                  </div>
              {/each}
          </div>
      {/if}
  </div>

  <!-- Action Bar -->
  <div class="mt-6 flex justify-end items-center gap-4">
      {#if files.length > 0}
          <div class="text-sm text-gray-500">
              {files.length} files selected
          </div>
          <button 
              class="px-4 py-2 text-gray-400 hover:text-white hover:bg-gray-800 rounded-lg transition-colors disabled:opacity-50"
              disabled={processing}
              on:click={clearFiles}
          >
              Clear All
          </button>
      {/if}
      <button 
          class="px-6 py-3 bg-blue-600 hover:bg-blue-500 active:bg-blue-700 text-white font-semibold rounded-xl shadow-lg shadow-blue-500/20 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center gap-2"
          disabled={files.length === 0 || processing}
          on:click={processFiles}
      >
          {#if processing}
              <svg class="animate-spin h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              Processing...
          {:else}
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"></path></svg>
              Convert to {selectedPreset.model}
          {/if}
      </button>
  </div>
</main>
