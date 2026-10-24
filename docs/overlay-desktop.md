# Overlay desktop

Pilih item di Koleksi lalu tekan **Tampilkan di desktop**. Satu item dapat memiliki beberapa instance dengan posisi dan pengaturan masing-masing.

## Yang sudah tersedia

- Overlay dibuat di tengah monitor utama dengan ukuran bawaan dari pengaturan.
- Jendela transparan selalu berada di atas jendela biasa, tidak muncul di taskbar, dan meneruskan klik ke aplikasi di bawahnya.
- GIF, APNG, dan WebP animasi diputar memakai timing frame asli. PNG dan JPEG statis dirender sekali tanpa scheduler animasi.
- Pengaturan instance yang sudah tersimpan seperti ukuran, opasitas, kecepatan, balik horizontal, dan putar/jeda diterapkan oleh renderer.
- Overlay aktif dipulihkan saat LumaDesk dibuka kembali jika **Pulihkan overlay** diaktifkan.
- Menghapus media dari Koleksi menutup semua window overlay miliknya sebelum data lokal dihapus. Window yang sempat ditutup dipulihkan jika penghapusan gagal.
- Perubahan instance dipulihkan ke nilai sebelumnya jika window native gagal diperbarui. Penghapusan instance juga menutup window sebelum record lokal dihapus.

## Kontrol yang sedang dikembangkan

Mode edit dan toolbar overlay belum tersedia pada antarmuka saat ini. Tahap berikutnya akan menambahkan pemilihan overlay, drag placement, serta sinkronisasi kontrol berikut dari jendela utama dan toolbar:

- **Posisi:** seret overlay ketika Mode edit aktif.
- **Ukuran:** mengubah lebar sambil mempertahankan rasio media.
- **Opasitas:** 10% sampai 100%.
- **Kecepatan:** 0,25× sampai 4×.
- **Balik horizontal:** membalik visual tanpa mengubah media asli.
- **Putar/Jeda:** mengontrol satu instance.
- **Duplikat:** membuat instance baru dengan pengaturan serupa dan posisi sedikit bergeser.
- **Tutup:** menghapus instance dari desktop tanpa menghapus item Koleksi.

Halaman **Aktif di Desktop** dan aksi massal untuk Mode edit, jeda, serta tutup juga masih dalam pengembangan.

## Banyak monitor

LumaDesk menyimpan posisi dalam koordinat logis beserta identitas monitor. Overlay baru ditempatkan di monitor utama. Pemulihan otomatis ke area terlihat ketika susunan monitor berubah akan ditambahkan bersama Mode edit.
