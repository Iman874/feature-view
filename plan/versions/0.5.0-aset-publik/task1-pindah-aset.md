# Task 1 — Pindahkan aset tampilan ke `public/`

Judul: Memindahkan `src/view.css` dan `src/render.js` ke `public/`, mengganti pembaca aset menjadi baca-disk dengan galat yang menyebut path, lalu menghapus jalur "baking" `include_str!`.

Deskripsi: Dua berkas tampilan hari ini tinggal di `src/` bersama Rust dan dibakar ke binari lewat `include_str!`, sementara `aset()` membaca disk lebih dulu. Dua sumber kebenaran itu yang membuat menyunting tampilan terasa seperti menyunting kode program. Task ini memindahkannya apa adanya — tanpa satu pun perubahan isi — supaya bisa dibuktikan bahwa halaman hasilnya tidak berubah satu byte.

Tujuan Teknis: `src/` hanya Rust; aset dibaca dari `public/`; galat aset menyebut path dan keluar bukan nol; `index.html` hasil `generate` byte-identik dengan sebelum pemindahan.

Scope: `public/view.css`, `public/render.js` (pindah, isi sama), `src/aset.rs`, `src/halaman.rs`, `src/saji.rs`, `src/main.rs`. Tidak menyentuh isi CSS/JS, tidak menyentuh manifest, tidak menyentuh rilis mana pun di repo induk.

Langkah Implementasi:
1. Catat pembanding dulu, sebelum apa pun berubah: `sha256sum docs/view/index.html` dan simpan hasilnya.
2. `mkdir -p public` lalu `git mv src/view.css public/view.css` dan `git mv src/render.js public/render.js` (pakai `git mv`, bukan salin-lalu-hapus, supaya riwayat berkasnya ikut).
3. `src/aset.rs`: ganti `dir_aset()` menjadi `dir_src()` (`<crate>/src`) dan tambah `dir_publik()` (`<crate>/public`).
4. `src/aset.rs`: tambah `css() -> Result<String, String>` — baca semua `*.css` di `dir_publik()`, urut nama, gabung dengan pemisah `/* ==== <nama> ==== */`; galat menyebut direktori yang dibaca dan berkas yang tidak ditemukan. Pada task ini isinya satu berkas, jadi hasil gabungan = isi berkas itu.
5. `src/aset.rs`: tambah `js() -> Result<String, String>` — baca `public/render.js`; kalau ada `*.js` lain, tolak dengan pesan yang menyebut nama berkasnya dan alasan IIFE.
6. Hapus `aset()`, `aset_dari()`, `const CSS = include_str!`, `const JS = include_str!`, dan uji `aset_ambil_dari_disk_dan_jatuh_ke_baking`. Ganti dengan tiga uji baru: `css_digabung_urut_nama` (minimal: satu berkas tetap digabung utuh; dua berkas uji di folder sementara digabung urut nama), `js_hanya_satu_berkas` (folder sementara berisi dua `.js` ditolak), `aset_hilang_menyebut_path` (folder kosong menghasilkan galat yang memuat nama berkas dan jalurnya).
7. Ubah pemanggil `halaman.rs` (`bangun_html`), `saji.rs`, `main.rs`: pakai `css()`/`js()`, teruskan `Err` sebagai pesan keluar + exit bukan nol. Jangan `panic!` — di alat CLI, panic terbaca seperti bug, bukan seperti aset yang hilang.
8. Sebar uji yang membaca `dir_aset().join("render.js")`/`view.css` ke `dir_publik()`; yang membaca berkas Rust (`halaman.rs`, `uji.rs`, `template.rs`) tetap `dir_src()`.
9. `cargo run -q -- generate --root .` lalu bandingkan `sha256sum docs/view/index.html` dengan angka langkah 1. Wajib sama persis.

Output yang Diharapkan: `src/` bersih dari aset; `public/` berisi dua berkas; halaman hasil identik; aset hilang menghasilkan pesan seperti `GAGAL membaca aset tampilan: public/view.css tidak ada (folder public/ wajib ikut bersama alat ini)`.

Dependencies: tidak ada.

Acceptance Criteria (semua dijalankan dari akar crate `tools/feature-view`):

```
ls src/*.css src/*.js
```
harapan: tidak ada berkas yang cocok (`No such file or directory` untuk keduanya).

```
git status --short
```
harapan: dua baris berpola `R  src/render.js -> public/render.js` dan `R  src/view.css -> public/view.css` (pemindahan, bukan hapus+tambah).

```
sha256sum docs/view/index.html
```
harapan: nilai sama dengan catatan langkah 1 (pemindahan murni, nol byte berubah).

```
mv public/view.css /tmp/view.css && cargo run -q -- generate --root . ; echo "exit=$?" ; mv /tmp/view.css public/view.css
```
harapan: `exit=1` (atau bukan nol) dan pesan galat memuat `public/view.css`; setelah berkas dikembalikan, `generate` hijau lagi.

```
cargo test
```
harapan: seluruh suite lulus; jumlah tes dilaporkan (berkurang satu uji baking, bertambah tiga uji aset baru).

```
cargo clippy --all-targets -- -D warnings
cargo run -q -- check --root .
cargo run -q -- audit --root .
```
harapan: clippy selesai tanpa warning; `check` mencetak `sinkron`; `audit` exit 0 dengan 0 baris kuning.

Estimasi: 1,5 jam.

Status: selesai -- sembilan langkah dikerjakan. Bukti murni: `sha256sum docs/view/index.html` = `1901c96c…` sebelum dan sesudah pemindahan (dengan satu berkas CSS, `css()` tidak mencetak penanda). `git status` menunjukkan dua pemindahan (R), bukan hapus+tambah. Uji baking dihapus; empat penjaga aset baru masuk; suite 92 -> 95 pada titik ini. Aset hilang terbukti keluar bukan nol dengan pesan yang menyebut `public/view.css`.
