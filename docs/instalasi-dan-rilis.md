# Instalasi dan rilis

## Paket resmi

- Windows 10/11 x64: installer NSIS `.exe`.
- macOS Apple Silicon dan Intel: universal `.dmg`.
- Linux x86-64: AppImage dan paket Debian `.deb`.

Linux menjalankan overlay langsung pada sesi X11. Pada sesi Wayland, LumaDesk menggunakan XWayland agar perilaku always-on-top dan click-through konsisten.

## Versi rilis

Rilis mengikuti Semantic Versioning. Tag Git `vMAJOR.MINOR.PATCH` memicu build seluruh platform. Release baru diterbitkan setelah pemeriksaan, test, dan packaging semua platform selesai. Setiap installer disertai checksum SHA-256.

Versi pada tag, `package.json`, workspace Cargo, dan konfigurasi Tauri harus sama. Workflow menolak rilis ketika salah satunya berbeda. Artifact Linux, macOS, dan Windows dikumpulkan lebih dahulu, lalu GitHub Release beserta `SHA256SUMS.txt` diterbitkan hanya jika seluruh build berhasil.

Build publik awal belum ditandatangani. Code signing macOS dan Windows akan dikonfigurasi setelah certificate dan secret proyek tersedia.
