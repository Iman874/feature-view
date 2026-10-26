# task6 — Segarkan halaman, sinkronkan dokumen, catat provenance

## Judul

Membuat halaman contoh dan halaman repo induk benar-benar hasil susun dari CSS/JS yang baru, dan membuat semua angka di dokumen ikut bergerak.

## Deskripsi

Dua hal pecah kalau task1-task5 selesai tanpa task ini:

1. `contoh/docs/view/index.html` yang terkommit dibandingkan **byte demi byte** dengan hasil susun hari ini oleh `halaman_contoh_di_repo_computed_dari_render_js_yang_sekarang` (`src/template.rs:246`). Mengubah `view.css`/`render.js` tanpa `generate --root contoh` membuat CI merah di tempat yang terlihat sepele.
2. Dokumen alat menulis angka tentang dirinya sendiri (jumlah baris per modul, jumlah tes, ukuran halaman). Penjaga tabel modul sudah menangkap sebagian; sisanya (md5, `data-cap`, angka README) harus dicatat tangan.

Selain itu task ini mencatat satu temuan yang **tidak** diperbaiki di sini karena bug CI diparkir: `root_uji()` (`src/uji.rs:139`) mencoba `.` lalu `../..`. Di laptop penyusun, `../..` adalah repo induk yang manifestnya sah, jadi sejumlah penjaga membaca data induk; di runner GitHub kedua kandidat gagal dan penjaga jatuh ke `contoh/`. Artinya "77 hijau lokal" dan "77 hijau CI" bukan objek yang sama. Ini harus ditulis di dokumen supaya tidak ada yang mengulang klaim keliru itu.

## Tujuan Teknis

- Kedua halaman adalah hasil `generate` dari CSS/JS akhir, bukan hasil edit manual.
- `data-cap` tidak berubah (cap dihitung dari byte ketujuh TSV, bukan dari tampilan); md5 lama dan baru tercatat berdampingan.
- Semua angka dokumen (README, CHANGELOG, tabel modul, `docs/view/README.md` milik repo induk) cocok dengan berkas yang sekarang.

## Scope

- Jalankan `cargo run --quiet -- generate --root contoh` dan `generate` di repo induk; commit halaman hasilnya.
- `README.md`, `CHANGELOG.md` milik alat; `docs/view/README.md` milik repo induk (bagian gaya dan angka).
- Catatan jujur soal `root_uji()` di README bagian verifikasi.
- TIDAK: memperbaiki `root_uji()`, memperbaiki bug CI ubuntu, menambah baris dataset.

## Langkah Implementasi

1. `cargo run --quiet -- generate --root contoh` lalu `cargo run --quiet -- check --root contoh` dan `-- audit --root contoh`; catat exit code ketiganya.
2. Dari root repo induk: `generate`, `check`, `audit`; catat exit code dan `md5sum docs/view/index.html` sebelum dan sesudah.
3. `cargo test` penuh dan `cargo clippy --all-targets -- -D warnings`; catat jumlah tes yang sekarang (naik karena task5).
4. Sinkronkan angka di README (jumlah tes, ukuran halaman, daftar modul) dan tulis satu bagian CHANGELOG untuk 0.2.0: apa yang berubah, angka sebelum/sesudah, dan apa yang belum diuji.
5. Tulis baris provenance: md5 lama -> md5 baru, `data-cap` (harus sama), dan perintah yang dipakai orang lain untuk membuktikannya sendiri.
6. Catat di CHANGELOG bahwa CI ubuntu masih merah dan apa yang dibutuhkan untuk melanjutkannya (nama tes dari anotasi run 40280461153).

## Output yang Diharapkan

Dua halaman tersusun ulang, dokumen sinkron, jejak provenance yang bisa diulang orang lain.

## Dependencies

Terakhir: butuh task1-task5. Task5 wajib lebih dulu supaya jumlah tes di README benar saat ditulis.

## Acceptance Criteria

- `check` dan `audit` exit 0 di kedua root; `generate` tidak mengubah berkas apa pun kalau dijalankan dua kali (idempoten, md5 sama).
- `data-cap` halaman lama dan baru identik; md5-nya berubah dan keduanya tercatat.
- Langkah CI "Jumlah tes di README sama dengan suite" lolos kalau dijalankan di cabang ini.
- Tidak ada satu pun nama entitas data repo induk yang masuk ke berkas repo publik (penjaga `sumber_alat_tidak_menyebut_nama_entitas_repo` hijau).

## Estimasi

1 jam.

## Bukti (diisi saat eksekusi)

- md5 lama/baru kedua halaman, `data-cap`, exit code enam perintah.
- Jumlah tes sebelum/sesudah dan baris README yang bergerak.
