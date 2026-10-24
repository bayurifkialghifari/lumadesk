# Pengaturan dan data lokal

## Pengaturan umum

- **Jalankan saat masuk:** memulai LumaDesk setelah pengguna login; nonaktif secara bawaan.
- **Pulihkan susunan:** menghidupkan kembali overlay aktif ketika aplikasi dimulai.
- **Tampilan koleksi:** grid atau daftar.
- **Default overlay:** ukuran, opasitas, dan kecepatan untuk instance baru.
- **Batas cache:** membatasi ruang thumbnail dan frame hasil decode.

## Cache

**Bersihkan cache** menghapus thumbnail dan frame hasil decode yang dapat dibuat ulang. Item Koleksi, tag, pengaturan, media terkelola, dan tata letak overlay tidak ikut terhapus.

## Lokasi dan isi data

Menu **Buka folder data** membuka direktori aplikasi yang berisi:

- database SQLite untuk metadata, tag, pengaturan, dan tata letak;
- folder `media` untuk salinan file yang diimpor;
- folder `thumbnails` dan `frames` untuk cache.

LumaDesk tidak membutuhkan akun, penyimpanan cloud, analitik, atau telemetri untuk fitur utamanya.
