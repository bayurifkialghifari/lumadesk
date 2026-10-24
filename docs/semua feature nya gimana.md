# Semua fitur LumaDesk dan cara pakainya

Dokumen ini menjelaskan perilaku LumaDesk v0.1 dari sudut pandang pengguna. Nama menu mengikuti antarmuka bahasa Indonesia.

## Koleksi

Halaman **Koleksi** menampilkan semua animasi bawaan dan hasil impor. Gunakan kotak **Cari koleksi** untuk mencari nama atau tag. Filter di samping pencarian membatasi hasil ke favorit, format tertentu, atau tag. Pilihan tampilan grid dan daftar disimpan otomatis.

Klik satu item untuk membuka panel detail. Dari panel ini kamu bisa melihat ukuran, format, jumlah frame, durasi, tag, dan berapa salinan yang sedang aktif di desktop. Klik dua kali untuk langsung menampilkannya dengan pengaturan default.

## Mengimpor animasi sendiri

Tekan **Impor animasi** lalu pilih satu atau beberapa file, atau seret file ke area koleksi. Format yang diterima adalah GIF, APNG, WebP animasi maupun statis, PNG, dan JPG/JPEG.

LumaDesk menyalin file ke folder data aplikasinya. File asli tetap berada di tempat semula. Jika isi file yang sama pernah diimpor, LumaDesk tidak membuat salinan kedua dan akan memilih item yang sudah ada. File rusak atau format yang tidak didukung ditolak tanpa meninggalkan item kosong.

## Mengatur koleksi

- **Ganti nama** mengubah nama yang tampil di LumaDesk. Nama file internal tidak berubah.
- **Favorit** menambahkan item ke filter favorit.
- **Tag** membantu mengelompokkan koleksi. Satu item bisa memiliki beberapa tag.
- **Hapus dari koleksi** menutup seluruh overlay item tersebut lalu menghapus salinan dan cache yang dikelola LumaDesk. File asli tidak ikut dihapus.

## Menampilkan animasi di desktop

Pilih item lalu tekan **Tampilkan di desktop**. Setiap penekanan membuat satu instance baru, jadi satu animasi dapat tampil beberapa kali dengan ukuran dan posisi berbeda. Instance aktif tercatat di halaman **Aktif di Desktop**.

Dalam **Mode tampil**, overlay selalu berada di atas jendela biasa dan klik mouse diteruskan ke aplikasi di bawahnya. Aktifkan **Mode edit** dari LumaDesk atau tray untuk memilih dan memindahkan overlay.

## Kontrol overlay

- **Posisi:** seret overlay saat Mode edit aktif.
- **Ukuran:** atur lebar dari panel detail; rasio gambar dipertahankan.
- **Opasitas:** 10% sampai 100%.
- **Kecepatan:** 0,25× sampai 4×.
- **Balik horizontal:** membalik arah visual tanpa mengubah file.
- **Putar/Jeda:** mengontrol satu instance.
- **Duplikat:** membuat instance baru dengan pengaturan yang sama dan posisi sedikit bergeser.
- **Tutup:** menghapus instance dari desktop, tetapi item tetap ada di Koleksi.

LumaDesk menyimpan susunan instance. Saat aplikasi dibuka kembali, susunan dipulihkan jika opsi **Pulihkan susunan** aktif. Jika monitor yang sebelumnya dipakai tidak tersedia, instance dipindahkan ke area terlihat pada monitor utama.

## Aktif di Desktop

Halaman ini menampilkan seluruh instance yang sedang aktif. Kamu bisa memilih satu instance untuk mengubah kontrolnya, menjeda semua animasi, mengaktifkan Mode edit, atau menutup beberapa instance sekaligus.

## Tray

Saat jendela utama ditutup, LumaDesk dapat tetap berjalan di tray agar overlay tidak hilang. Menu tray menyediakan **Buka LumaDesk**, **Mode edit**, **Jeda semua**, dan **Keluar**. Perintah **Keluar** menutup overlay dan aplikasi; susunannya tetap tersimpan untuk pembukaan berikutnya.

## Pengaturan

- **Jalankan saat masuk:** nonaktif secara bawaan.
- **Pulihkan susunan:** menghidupkan kembali instance aktif saat LumaDesk dibuka.
- **Ukuran, opasitas, dan kecepatan default:** dipakai ketika membuat instance baru.
- **Batas cache:** membatasi ruang thumbnail dan frame hasil decode.
- **Bersihkan cache:** menghapus data yang dapat dibuat ulang tanpa menghapus koleksi.
- **Buka folder data:** membuka lokasi database, media terkelola, dan cache.

## Data dan privasi

Semua data disimpan lokal. LumaDesk tidak membutuhkan akun dan tidak mengirim telemetri. Database SQLite menyimpan metadata, tag, pengaturan, dan tata letak. File media dan cache disimpan sebagai file biasa di direktori data aplikasi agar database tetap kecil dan mudah dirawat.

## Dukungan sistem

- Windows 10/11 64-bit.
- macOS modern pada Apple Silicon dan Intel.
- Linux x86-64 melalui AppImage atau paket Debian. Sesi X11 didukung langsung; sesi Wayland memakai XWayland untuk perilaku overlay yang konsisten.
