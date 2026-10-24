# Overlay desktop

Pilih item di Koleksi lalu tekan **Tampilkan di desktop**. Satu item dapat memiliki beberapa instance dengan posisi dan pengaturan masing-masing.

## Yang sudah tersedia

- Overlay dibuat di tengah monitor utama dengan ukuran bawaan dari pengaturan.
- Jendela transparan selalu berada di atas jendela biasa, tidak muncul di taskbar, dan meneruskan klik ke aplikasi di bawahnya.
- GIF, APNG, dan WebP animasi diputar memakai timing frame asli. PNG dan JPEG statis dirender sekali tanpa scheduler animasi.
- Pengaturan instance yang sudah tersimpan seperti ukuran, opasitas, kecepatan, balik horizontal, dan putar/jeda diterapkan oleh renderer.
- Halaman **Aktif di Desktop** menampilkan seluruh instance aktif beserta media, monitor, ukuran, status, dan kontrolnya.
- **Mode edit** mengaktifkan pointer pada semua overlay. Setiap overlay menampilkan batas tipis dan toolbar untuk menggeser, putar/jeda, balik horizontal, duplikat, dan tutup.
- Posisi terakhir setelah overlay digeser disimpan secara lokal dalam koordinat logis.
- Aksi massal dapat menjeda semua overlay yang sedang diputar atau menutup semua overlay setelah konfirmasi.
- Overlay aktif dipulihkan saat LumaDesk dibuka kembali jika **Pulihkan overlay** diaktifkan.
- Menghapus media dari Koleksi menutup semua window overlay miliknya sebelum data lokal dihapus. Window yang sempat ditutup dipulihkan jika penghapusan gagal.
- Perubahan instance dipulihkan ke nilai sebelumnya jika window native gagal diperbarui. Penghapusan instance juga menutup window sebelum record lokal dihapus.

## Kontrol instance

- **Posisi:** seret overlay ketika Mode edit aktif.
- **Ukuran:** mengubah lebar sambil mempertahankan rasio media.
- **Opasitas:** 10% sampai 100%.
- **Kecepatan:** 0,25× sampai 4×.
- **Balik horizontal:** membalik visual tanpa mengubah media asli.
- **Putar/Jeda:** mengontrol satu instance.
- **Duplikat:** membuat instance baru dengan pengaturan serupa dan posisi sedikit bergeser.
- **Tutup:** menghapus instance dari desktop tanpa menghapus item Koleksi.

Kontrol rinci tersedia pada halaman **Aktif di Desktop**. Toolbar yang menempel pada overlay sengaja lebih ringkas agar tidak menutupi animasi. Saat antarmuka dibuka di browser untuk pengembangan, halaman tetap dapat dipratinjau tetapi kontrol window native dinonaktifkan.

## Banyak monitor

LumaDesk menyimpan posisi dalam koordinat logis beserta identitas monitor tempat overlay terakhir digeser. Overlay baru ditempatkan di monitor utama. Pemulihan otomatis ke area terlihat ketika monitor tersimpan sudah tidak tersedia masih menjadi pekerjaan lanjutan.
