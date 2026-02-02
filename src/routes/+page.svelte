<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";

  interface FileItem {
    path: string;
    name: string;
    status: "pending" | "processing" | "success" | "error";
    message?: string;
  }

  let files: FileItem[] = [];
  let processing = false;
  let unlisten: () => void;

  // Presets
  const presets = [
    {
      id: "pen-f",
      name: "Olympus PEN-F",
      make: "OLYMPUS CORPORATION",
      model: "PEN-F",
    },
    {
      id: "e-p7",
      name: "Olympus PEN E-P7",
      make: "OLYMPUS CORPORATION",
      model: "E-P7",
    },
    {
      id: "e-m1",
      name: "Olympus E-M1",
      make: "OLYMPUS CORPORATION",
      model: "E-M1",
    },
    {
      id: "e-m5-iii",
      name: "Olympus E-M5 Mark III",
      make: "OLYMPUS CORPORATION",
      model: "E-M5MarkIII",
    },
    {
      id: "e-m10-iv",
      name: "Olympus E-M10 Mark IV",
      make: "OLYMPUS CORPORATION",
      model: "E-M10MarkIV",
    },
    {
      id: "gx8",
      name: "Panasonic Lumix GX8",
      make: "Panasonic",
      model: "DMC-GX8",
    },
    {
      id: "g9",
      name: "Panasonic Lumix G9",
      make: "Panasonic",
      model: "DC-G9",
    },
    {
      id: "g100",
      name: "Panasonic Lumix G100",
      make: "Panasonic",
      model: "DC-G100",
    },

    {
      id: "yi-m1",
      name: "YI M1",
      make: "YI Technology",
      model: "YI M1",
    },
  ];

  let selectedPreset = presets[0];

  let isDraggingOver = false;

  onMount(async () => {
    console.log("Mounting component, setting up listeners...");

    // Listen for file drop (Tauri v2)
    const unlistenDragDrop = await listen("tauri://drag-drop", (event) => {
      console.log("Tauri drag-drop event:", event);
      isDraggingOver = false;

      // Handle v2 payload structure: { paths: string[], position: { x, y } }
      // or fallback to v1 string[] if structure differs
      const payload = event.payload as any;
      const paths = payload.paths || (Array.isArray(payload) ? payload : []);

      if (Array.isArray(paths) && paths.length > 0) {
        addFiles(paths);
      } else {
        console.warn("Invalid drag-drop payload:", payload);
      }
    });

    // Listen for drag enter
    const unlistenDragEnter = await listen("tauri://drag-enter", () => {
      console.log("Tauri drag-enter");
      isDraggingOver = true;
    });

    // Listen for drag leave
    const unlistenDragLeave = await listen("tauri://drag-leave", () => {
      console.log("Tauri drag-leave");
      isDraggingOver = false;
    });

    // Keep legacy listener just in case
    const unlistenFileDrop = await listen("tauri://file-drop", (event) => {
      console.log("Legacy file-drop event:", event);
      const paths = event.payload as string[];
      addFiles(paths);
    });

    return () => {
      unlistenDragDrop();
      unlistenDragEnter();
      unlistenDragLeave();
      unlistenFileDrop();
    };
  });

  onDestroy(() => {
    if (unlisten) unlisten(); // Clean up the initial unlisten if set
  });

  function addFiles(paths: string[]) {
    console.log("Adding files:", paths);
    // Filter for DNG (case insensitive)
    const dngFiles = paths.filter((p) => p.toLowerCase().endsWith(".dng"));

    if (dngFiles.length === 0) {
      console.warn("No DNG files found in selection");
    }

    const newFiles = dngFiles.map((path) => ({
      path,
      name: path.split(/[\\/]/).pop() || path,
      status: "pending" as const,
    }));

    // Avoid duplicates
    files = [
      ...files,
      ...newFiles.filter((nf) => !files.some((f) => f.path === nf.path)),
    ];
    console.log("Current file list:", files);
  }

  async function openFileDialog() {
    console.log("Opening file dialog...");
    try {
      const selected = await open({
        multiple: true,
        filters: [
          {
            name: "DNG Images",
            extensions: ["dng"],
          },
        ],
      });

      console.log("Dialog selection:", selected);

      if (selected) {
        // Handle both single string (if multiple: false) and array
        const paths = Array.isArray(selected) ? selected : [selected];
        // In Tauri v2 dialog plugin, open returns string[] directly when multiple is true
        addFiles(paths);
      }
    } catch (err) {
      console.error("Failed to open dialog:", err);
    }
  }

  async function processFiles() {
    console.log("Starting processing...");
    if (files.length === 0) return;
    processing = true;

    const pendingFiles = files.filter((f) => f.status !== "success");
    const filePaths = pendingFiles.map((f) => f.path);

    if (filePaths.length === 0) {
      console.log("No pending files to process");
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
      "-m",
      "-overwrite_original",
      `-Make=${selectedPreset.make}`,
      `-Model=${selectedPreset.model}`,
      // Optionally set ImageDescription to mark processed files
      `-ImageDescription=Processed by yi2olympus`,
      ...filePaths,
    ];

    console.log("ExifTool args:", args);

    try {
      // Update UI to processing
      files = files.map((f) =>
        pendingFiles.some((pf) => pf.path === f.path)
          ? { ...f, status: "processing" }
          : f,
      );

      const result = await invoke("run_exiftool", { args });
      console.log("ExifTool output:", result);

      // Assume success if no error thrown
      files = files.map((f) =>
        f.status === "processing" ? { ...f, status: "success" } : f,
      );
    } catch (error) {
      console.error("ExifTool error:", error);
      files = files.map((f) =>
        f.status === "processing"
          ? { ...f, status: "error", message: String(error) }
          : f,
      );
    } finally {
      processing = false;
    }
  }

  function removeFile(path: string) {
    console.log("Removing file:", path);
    files = files.filter((f) => f.path !== path);
  }

  function clearFiles() {
    if (processing) return;
    files = [];
  }
</script>

<main
  class="h-screen w-screen bg-[#fff] text-[#1b1b1b] font-sans flex flex-col p-8 select-none"
  on:contextmenu|preventDefault
>
  <div class="mb-8">
    <h1 class="text-2xl font-semibold text-[#1b1b1b]">DNG Masker</h1>
    <p class="text-[#616161] mt-1 text-sm">
      Drag and drop DNG files to disguise them as Olympus cameras.
    </p>
  </div>

  <div class="mb-6 bg-white border border-[#e5e5e5] p-5 rounded-lg shadow-sm">
    <label
      for="preset-select"
      class="block text-sm font-semibold text-[#1b1b1b] mb-3"
    >
      Target Camera Model
    </label>

    <div class="relative">
      <select
        id="preset-select"
        bind:value={selectedPreset}
        class="w-full px-3 py-2 bg-[#ffffff] text-[#1b1b1b] border border-[#d1d1d1] rounded-md
               hover:bg-[#f9f9f9] focus:outline-none
               transition-all cursor-default text-sm appearance-none"
      >
        {#each presets as preset}
          <option value={preset}>{preset.name}</option>
        {/each}
      </select>

      <div
        class="pointer-events-none absolute inset-y-0 right-0 flex items-center px-3 text-[#616161]"
      >
        <svg
          class="w-3.5 h-3.5"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2.5"
            d="M19 9l-7 7-7-7"
          />
        </svg>
      </div>
    </div>
  </div>

  <div
    class="flex-1 bg-white/40 rounded-lg border border-[#e5e5e5] flex flex-col overflow-hidden relative transition-all shadow-[0_2px_4px_rgba(0,0,0,0.04)]
    {isDraggingOver
      ? 'bg-[#cfe4fa]/50 border-[#0067c0] ring-1 ring-[#0067c0]'
      : ''}"
    on:click={openFileDialog}
    on:dragover|preventDefault={(e) => {
      e.dataTransfer.dropEffect = "copy";
      isDraggingOver = true;
    }}
    on:dragleave={() => (isDraggingOver = false)}
    on:drop|preventDefault={(e) => {
      isDraggingOver = false;
    }}
  >
    {#if files.length === 0}
      <div
        class="absolute inset-0 flex flex-col items-center justify-center text-[#616161] pointer-events-none"
      >
        <svg
          class="w-12 h-12 mb-4 text-[#0067c0] opacity-80"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="1.2"
            d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
          />
        </svg>
        <p class="text-sm font-medium">
          {isDraggingOver
            ? "Drop to add files"
            : "Drag & Drop DNG files here or Click to Select"}
        </p>
      </div>
    {:else}
      <div
        class="flex-1 overflow-y-auto p-2 space-y-1 bg-[#f9f9f9]/50"
        on:click|stopPropagation
      >
        {#each files as file (file.path)}
          <div
            class="flex items-center justify-between bg-white p-3 rounded border border-[#efefef] hover:border-[#d1d1d1] transition-colors shadow-sm"
          >
            <div class="flex items-center gap-3 overflow-hidden">
              <div
                class="w-8 h-8 rounded bg-[#f3f3f3] flex items-center justify-center flex-shrink-0 text-[10px] font-bold text-[#616161] border border-[#e5e5e5]"
              >
                DNG
              </div>
              <div class="truncate text-sm text-[#1b1b1b]" title={file.path}>
                {file.name}
              </div>
            </div>

            <div class="flex items-center gap-3">
              {#if file.status === "success"}
                <span
                  class="text-[#0f7b0f] text-xs font-medium px-2 py-0.5 bg-[#dff6dd] rounded-full"
                  >Done</span
                >
              {:else if file.status === "error"}
                <span
                  class="text-[#c42b1c] text-xs font-medium px-2 py-0.5 bg-[#fde7e9] rounded-full"
                  >Error</span
                >
              {:else if file.status === "processing"}
                <span
                  class="text-[#0067c0] text-xs font-medium px-2 py-0.5 bg-[#cfe4fa] rounded-full animate-pulse"
                  >Processing</span
                >
              {:else}
                <button
                  on:click={() => removeFile(file.path)}
                  class="text-[#616161] hover:bg-[#e5e5e5] p-1.5 rounded-md transition-colors"
                >
                  <svg
                    class="w-4 h-4"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M6 18L18 6M6 6l12 12"
                    />
                  </svg>
                </button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <div class="mt-6 flex justify-end items-center gap-3">
    {#if files.length > 0}
      <div class="text-xs text-[#616161] mr-2">
        {files.length} items selected
      </div>
      <button
        class="px-4 py-2 text-sm text-[#1b1b1b] bg-[#ffffff] border border-[#d1d1d1] hover:bg-[#f5f5f5] active:bg-[#eeeeee] rounded-md transition-all disabled:opacity-50"
        disabled={processing}
        on:click={clearFiles}
      >
        Clear All
      </button>
    {/if}

    <button
      class="px-8 py-2 bg-[#0067c0] hover:bg-[#005fb8] active:bg-[#0052a1] text-white text-sm font-medium rounded-md shadow-[0_2px_4px_rgba(0,0,0,0.1)] active:shadow-none disabled:bg-[#cccccc] disabled:shadow-none transition-all flex items-center gap-2"
      disabled={files.length === 0 || processing}
      on:click={processFiles}
    >
      {#if processing}
        <div
          class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"
        ></div>
        Processing...
      {:else}
        Convert to {selectedPreset.model}
      {/if}
    </button>
  </div>
</main>

<style>
  /* 隐藏 Webkit 滚动条以保持原生应用感 */
  ::-webkit-scrollbar {
    width: 6px;
  }
  ::-webkit-scrollbar-thumb {
    background: #cdcdcd;
    border-radius: 10px;
  }
  ::-webkit-scrollbar-thumb:hover {
    background: #a6a6a6;
  }
</style>
