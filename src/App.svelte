<script lang="ts">
import { Images, Layers3, Settings2, Sparkles } from "@lucide/svelte";

const navigation = [
  { label: "Koleksi", icon: Images, active: true },
  { label: "Aktif di Desktop", icon: Layers3, active: false },
  { label: "Pengaturan", icon: Settings2, active: false },
] as const;
</script>

<svelte:head>
  <title>LumaDesk</title>
</svelte:head>

<div class="app-shell">
  <header class="titlebar" data-tauri-drag-region>
    <div class="brand" data-tauri-drag-region>
      <span class="brand-mark"><Sparkles size={17} strokeWidth={2.4} /></span>
      <span data-tauri-drag-region>LumaDesk</span>
    </div>
    <span class="version" data-tauri-drag-region>v0.1.0</span>
  </header>

  <aside class="sidebar">
    <nav aria-label="Navigasi utama">
      {#each navigation as item}
        <button class:active={item.active} type="button">
          <item.icon size={18} />
          <span>{item.label}</span>
        </button>
      {/each}
    </nav>
    <div class="privacy-note">
      <span class="status-dot"></span>
      <div><strong>Lokal sepenuhnya</strong><small>Tanpa akun atau telemetri</small></div>
    </div>
  </aside>

  <main>
    <div class="eyebrow">KOLEKSI ANIMASI</div>
    <h1>Bikin desktop terasa hidup.</h1>
    <p class="lede">Impor animasi milikmu, atur koleksi, lalu tempatkan di desktop.</p>

    <section class="empty-state">
      <div class="empty-mark"><Sparkles size={32} /></div>
      <div>
        <h2>Koleksimu siap diisi</h2>
        <p>Impor GIF, APNG, WebP, PNG, atau JPG untuk memulai.</p>
      </div>
      <button class="primary" type="button">Impor animasi</button>
    </section>
  </main>
</div>
