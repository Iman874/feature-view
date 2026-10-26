# task5 — Penjaga skala: dokumen dan kode tidak boleh longgar lagi

## Judul

Tes yang menghitung ulang skala dari `src/view.css` dan `src/render.js`, supaya "sudah rapi" tetap benar setelah orang lain menyentuh berkasnya.

## Deskripsi

Kode ini sudah punya tradisi penjaga yang mengukur dirinya sendiri: `sumber_alat()` (`src/uji.rs:207`) menyapu seluruh modul, `tabel_modul_di_dokumen_mencocokkan_angka_sahnya` (`src/main.rs:655`) menghitung ulang angka di tabel dokumen, `sitasi_baris_di_dokumen_masih_hidup` (`src/main.rs:423`) membuktikan tiap kutipan baris masih jatuh di tempatnya. Tanpa padanannya di sisi tampilan, hasil task1-task4 bisa dilonggarkan satu baris CSS tanpa satu pun tes yang tahu — persis pola yang membuat tabel modul basi dua kali dalam sehari.

## Tujuan Teknis

- Jumlah nilai `font-size:` berbeda di `view.css` dijaga <= 6 dan semuanya `var(--fs-`.
- `border-radius:` hanya boleh `var(--r-sm)`, `var(--r-md)`, `50%`, atau `0`.
- Nol `px` pecahan pada `padding`/`gap`/`margin`.
- Setiap token yang dipanggil di CSS punya definisinya di blok `:root` (variabel yatim tertangkap, bukan jadi `unset` diam-diam).
- Penjaga ini ikut terhitung di CI yang sudah ada (`cargo test`), tanpa aksi pihak ketiga.

## Scope

- Satu modul tes (di `src/` mengikuti tempat penjaga lain berada) + pembacaan `aset("view.css", CSS)` yang sudah tersedia.
- TIDAK: mengubah `view.css` (itu objek yang dijaga), mengubah CI.

## Langkah Implementasi

1. Ambil isi `view.css` lewat `aset()` yang sudah dipakai penjaga lain; jangan `include_str!` dua kali.
2. Parse deklarasi per properti dengan pemisah baris/titik-koma yang sama seperti `check` memakai teks, bukan regex pihak ketiga.
3. Kumpulkan himpunan nilai yang berbeda, bandingkan dengan batas di atas, dan sebutkan nilai pelanggarnya di pesan galat (bukan hanya jumlahnya) supaya orang tahu apa yang harus dibetulkan.
4. Silang-check: setiap `var(--x)` yang terpakai harus punya `--x:` di `:root` salah satu skema.
5. Buktikan giginya dengan mutasi sementara: tambah satu `font-size: 0.79rem` di `view.css`, jalankan tes, catat merahnya, kembalikan; lalu hapus satu definisi token dan pastikan penjaga variabel yatim ikut merah.
6. Naikkan jumlah tes yang tercatat di README (CI punya langkah "Jumlah tes di README sama dengan suite").

## Output yang Diharapkan

Satu kelompok tes baru yang merah kalau skala dilanggar, hijau pada hasil task1-task4.

## Dependencies

Butuh task1, task3, task4 selesai (kalau tidak, penjaganya merah oleh pekerjaan yang belum selesai).

## Acceptance Criteria

- Tes baru hijau pada keadaan akhir, dan **merah terukur** pada dua mutasi yang disebutkan di langkah 5.
- `cargo clippy --all-targets -- -D warnings` bersih.
- README/CHANGELOG ikut menyebut penjaga baru ini (aturan `tiap_penjaga_anti_hardcode_didokumentasikan`).

## Estimasi

1-1,5 jam.

## Bukti (diisi saat eksekusi)

- Nama tes baru + jumlah kasus.
- Output merah dari tiap mutasi (disalin apa adanya), dan angka suite sesudahnya.
