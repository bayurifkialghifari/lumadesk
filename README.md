<picture>
  <img alt="LumaDesk — Your animation library, alive on the desktop." src="docs/assets/readme-banner.svg">
</picture>

<p align="center">
  Kelola GIF, APNG, WebP, dan gambar milikmu lalu tampilkan sebagai overlay ringan di desktop.
</p>

<p align="center">
  <a href="https://github.com/bayurifkialghifari/lumadesk/actions/workflows/ci.yml"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/bayurifkialghifari/lumadesk/ci.yml?branch=main&style=flat-square&label=checks"></a>
  <a href="https://github.com/bayurifkialghifari/lumadesk/releases"><img alt="Release" src="https://img.shields.io/github/v/release/bayurifkialghifari/lumadesk?style=flat-square&display_name=tag"></a>
  <a href="LICENSE"><img alt="MIT License" src="https://img.shields.io/badge/license-MIT-39a8ff?style=flat-square"></a>
</p>

## Yang bisa kamu lakukan

- Impor GIF, APNG, WebP, PNG, dan JPG melalui file picker atau drag-and-drop.
- Cari, beri tag, favoritkan, ganti nama, dan hapus item dari koleksi lokal.
- Buat beberapa overlay transparan dengan posisi, ukuran, opasitas, kecepatan, dan arah masing-masing.
- Kelola seluruh overlay aktif, gunakan Mode edit untuk menggeser, lalu putar, jeda, duplikat, atau tutup setiap instance.
- Pulihkan susunan overlay saat aplikasi dibuka kembali.
- Simpan semuanya secara lokal tanpa akun dan tanpa telemetri.

## Unduh

| Sistem | Paket |
| --- | --- |
| Windows 10/11 x64 | NSIS `.exe` |
| macOS Apple Silicon + Intel | Universal `.dmg` |
| Linux x86-64 | AppImage dan `.deb` |

Installer akan tersedia di [GitHub Releases](https://github.com/bayurifkialghifari/lumadesk/releases) mulai rilis publik pertama. Build pengembangan dapat dijalankan dari sumber.

## Mulai mengembangkan

Prasyarat native mengikuti [Tauri 2](https://v2.tauri.app/start/prerequisites/) untuk sistem operasi masing-masing. Setelah dependensi sistem tersedia:

```bash
pnpm install
pnpm tauri dev
```

Pemeriksaan utama:

```bash
pnpm check
pnpm test
cargo test --workspace
```

## Dokumentasi

- [Koleksi animasi](docs/koleksi.md)
- [Mengimpor media](docs/impor-media.md)
- [Overlay desktop](docs/overlay-desktop.md)
- [Tray dan lifecycle](docs/tray-dan-lifecycle.md)
- [Pengaturan dan data lokal](docs/pengaturan-dan-data.md)
- [Instalasi dan rilis](docs/instalasi-dan-rilis.md)
- [Product requirements](PRD.md)
- [Design system](DESIGN.md)
- [Architecture](architecture.md)
- [Changelog](CHANGELOG.md)

## Lisensi

LumaDesk tersedia di bawah [MIT License](LICENSE). Aset starter pihak ketiga selalu menyertakan catatan lisensinya sendiri.
