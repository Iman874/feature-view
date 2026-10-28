# Task 2 — Pecah CSS menjadi berkas berprefix dan jagalah urutannya

Judul: Memecah `public/view.css` menjadi tiga berkas berprefix angka (`10-token.css`, `20-komponen.css`, `30-layar.css`) yang digabung menurut urutan nama, dan menambah penjaga yang menolak berkas tanpa prefix serta urutan yang berubah.

Deskripsi: 1.017 baris CSS dalam satu berkas adalah alasan kedua owner meminta folder aset ("gampang edit/hapus/atur"). Setelah task 1 berkasnya ada di `public/`, tapi masih satu. Task ini memecahnya menurut lapisan yang sudah ada di dalam berkas itu (token, komponen, layar) — bukan menurut selera baru — dan mengikat urutannya dengan penjaga, karena cascade bergantung urutan sementara `check` hanya membandingkan hasil akhir.

Tujuan Teknis: tiga berkas dengan batas yang jelas, urutan gabungan terbaca dari nama, dan penjaga yang merah kalau muncul berkas tanpa prefix angka atau urutannya berubah.

Scope: `public/10-token.css`, `public/20-komponen.css`, `public/30-layar.css` (isi dipindah, bukan ditulis ulang), `src/aset.rs` (penjaga). Tidak mengubah satu deklarasi CSS pun.

Langkah Implementasi:
1. `generate` dulu dan catat beda byte halaman sebagai pembanding: `sha256sum docs/view/index.html`.
2. Potong `public/view.css` menjadi tiga menurut batas yang sudah ada: blok `:root` token dan skala -> `10-token.css`; reset, kontrol, tombol, menu, pencarian, rel, inspektur -> `20-komponen.css`; kanvas SVG, papan pekerjaan, media query -> `30-layar.css`. Setiap berkas dibuka komentar satu baris yang menyebut lapisannya.
3. Hapus `public/view.css` setelah ketiganya lengkap — jangan tinggalkan dua sumber.
4. `src/aset.rs`: perluas `css_digabung_urut_nama` supaya menuntut setiap nama berkas cocok `^\d{2}-[a-z0-9-]+\.css$` dan urutan gabungan sama dengan urutan nama; pesan galat menyebut berkas yang melanggar.
5. `cargo run -q -- generate --root .` lalu bandingkan isi halaman dengan pembanding langkah 1: yang boleh berbeda hanya tiga baris penanda `/* ==== <nama> ==== */`. Kalau ada beda lain, ada blok yang pindah urutan — perbaiki urutannya, jangan longgarkan pembandingnya.

Output yang Diharapkan: tiga berkas CSS berprefix, urutan gabungan sama dengan perilaku sebelum pemecahan, dan penjaga yang menjaganya tetap begitu.

Dependencies: task 1.

Acceptance Criteria (dijalankan dari akar crate):

```
ls public/*.css
```
harapan: tepat tiga berkas bernama `10-token.css`, `20-komponen.css`, `30-layar.css`; tidak ada `view.css`.

```
cargo test css_digabung_urut_nama
```
harapan: lulus. Bukti bergigi: ganti nama `20-komponen.css` menjadi `komponen.css`, jalankan tes yang sama, harapan MERAH dengan pesan menyebut berkas itu; kembalikan namanya dan jalankan lagi, harapan hijau.

```
cargo run -q -- generate --root . && git diff --numstat docs/view/index.html
```
harapan: baris yang berubah hanya bagian penanda berkas (tambahan tiga baris komentar gabungan); kalau ada baris CSS lain yang berpindah, urutan pemecahannya salah.

```
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -q -- check --root .
```
harapan: suite lulus, clippy tanpa warning, `check` mencetak `sinkron`.

Estimasi: 1 jam.

Status: selesai -- owner menyetujui pemecahan pada 2026-10-28. Tiga berkas: 10-token.css 176 baris, 20-komponen.css 650, 30-layar.css 195. Bukti tidak ada yang berpindah: isi `<style>` di halaman sama persis dengan `src/view.css` lama setelah tiga penanda dan empat baris komentar kepala dibuang (diff 0 baris); halaman tumbuh 208.298 -> 208.709 byte, seluruhnya penanda+kepala. `berkas_css_publik_berprefix_angka` dibuktikan bergigi (ganti nama jadi komponen.css -> merah).
