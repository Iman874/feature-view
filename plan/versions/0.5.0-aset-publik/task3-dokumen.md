# Task 3 — Dokumen petunjuk berhenti menunjuk `src/`

Judul: Memperbarui `how_to_run.md`, `README.md`, dan komentar `.gitattributes` supaya menunjuk `public/`, menambah tabel `public/` di README, dan menambah penjaga yang menghitung ulang angka tabel itu.

Deskripsi: Tiga dokumen masih menyuruh orang menyunting `src/render.js`/`src/view.css`, dan satu komentar `.gitattributes` menjelaskan `include_str!` yang sudah tidak ada. Membiarkannya berarti dokumen menunjuk berkas yang tidak pernah ada lagi — kelas cacat yang sudah pernah terjadi di repo ini. Tabel modul README juga dijaga tes, jadi tabel baru untuk `public/` butuh penjaga pasangannya.

Tujuan Teknis: nol rujukan path lama di dokumen petunjuk; alat bantu (nama berkas, jumlah baris, cara menyalin) tercatat benar; penjaga menagih angka tabel `public/`.

Scope: `how_to_run.md`, `README.md`, `.gitattributes` (komentar saja), `src/aset.rs` atau `src/main.rs` (satu penjaga baru `tabel_publik_mencocokkan_angka`). Tidak menyentuh kode produksi, tidak menulis CHANGELOG (itu task 4 karena angkanya lahir di sana).

Langkah Implementasi:
1. `how_to_run.md` baris 12 dan 164: `node --check src/render.js` -> `node --check public/render.js`.
2. `how_to_run.md` baris 220: kalimat "`src/view.css` dan `src/render.js`, lalu `cargo build` + `generate`" diganti "sunting `public/`, lalu `generate` — tanpa build".
3. `how_to_run.md`: tambah satu bagian pendek "Menyunting tampilan tanpa build": perintah binary langsung (`.\target\debug\feature-view.exe generate --root .` di Windows, `./target/debug/feature-view generate --root .` di shell POSIX), dan satu kalimat bahwa `public/` wajib ikut kalau alat disalin ke mesin lain.
4. `README.md`: pertahankan tabel modul `src/*.rs`; tambah tabel `public/` berisi nama berkas, isi, dan jumlah baris.
5. Penjaga `tabel_publik_mencocokkan_angka`: baca semua baris dokumen yang diawali `| \`public/`, ambil nama berkas dan angka baris, hitung ulang dari disk, tolak bila beda. Pola dan pesannya mengikuti `tabel_modul_di_dokumen_mencocokkan_angka_sahnya`; sebutkan di komentar bahwa penjaga lama sengaja tidak diperluas karena ia hanya membaca `src/*.rs`.
6. `README.md`: satu paragraf di bagian struktur yang menyebut `public/` sebagai tempat aset tampilan dan bahwa `src/` hanya Rust.
7. `.gitattributes`: perbarui komentar yang menyebut "`render.js`/`view.css` di-`include_str!` lalu tertanam apa adanya". Aturan `* text=auto eol=lf` **tidak** diubah — `check` masih membandingkan byte.

Output yang Diharapkan: dokumen yang menunjuk `public/`, tabel `public/` di README, dan penjaga yang membuat tabel itu tidak bisa basi.

Dependencies: task 1 (dan task 2 kalau CSS dipecah, karena nama berkasnya berubah).

Acceptance Criteria (dijalankan dari akar crate):

```
grep -rn "src/view.css\|src/render.js" how_to_run.md README.md
```
harapan: nol hasil. (Plan lama di `plan/versions/0.2.0-*` tidak diperiksa karena itu dokumen sejarah.)

```
grep -n "public/" how_to_run.md README.md | head
```
harapan: baris 12/164/220 di `how_to_run.md` memuat `public/render.js`, dan README memuat tabel `public/`.

```
cargo test tabel_publik_mencocokkan_angka
```
harapan: lulus. Bukti bergigi: ubah satu angka di tabel README, jalankan tes yang sama, harapan MERAH dengan pesan menyebut berkas dan dua angka; kembalikan dan jalankan lagi, harapan hijau.

```
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -q -- check --root .
```
harapan: suite lulus, clippy tanpa warning, `check` mencetak `sinkron`.

Estimasi: 1 jam.

Status: selesai -- `grep -rn "src/view.css\|src/render.js" how_to_run.md README.md` nol hasil; bagian 8a "Menyunting tampilan tanpa build" ditambahkan; tabel `public/` (empat berkas) masuk README; komentar `.gitattributes` tidak lagi menyebut `include_str!`; penjaga `tabel_publik_di_dokumen_mencocokkan_angka_sahnya` ditulis dan dibuktikan merah saat satu angka diubah satu, lalu hijau setelah dikembalikan.
