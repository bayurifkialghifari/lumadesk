# Instalasi dan rilis

## Paket resmi

- Windows 10/11 x64: installer NSIS `.exe`.
- macOS Apple Silicon dan Intel: universal `.dmg`.
- Linux x86-64: AppImage dan paket Debian `.deb`.

Linux menjalankan overlay langsung pada sesi X11. Pada sesi Wayland, LumaDesk menggunakan XWayland agar perilaku always-on-top dan click-through konsisten.

## Versi rilis

Rilis mengikuti Semantic Versioning. Tag Git `vMAJOR.MINOR.PATCH` memicu build seluruh platform. Release baru diterbitkan setelah pemeriksaan, test, dan packaging semua platform selesai. Setiap installer disertai checksum SHA-256.

Build macOS dan Windows memakai signing ketika secret proyek sudah tersedia. Build pengembangan yang belum ditandatangani tetap diberi keterangan jelas pada halaman rilis.
