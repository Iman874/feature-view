# task7 — `how_to_run.md`: dokumen operasional terpisah dari README

## Judul

Pindahkan seluruh bagian operasional alat ke `how_to_run.md` supaya README cukup berisi versi dan
kemampuan, lalu buat berkas baru itu ikut dijaga penjaga dokumen.

## Deskripsi

Task ini tidak ada di daftar awal plan 0.2.0. Ia lahir setelah owner membaca hasil task1–task6 dan
meminta: "tambahin `how_to_run.md` dong biar readme nya lebih lega ke version dan apa yang bisa
dilakukan app, jadi how to run itu lebih spesifik." Jadi isinya spesifik: versi toolchain, bentuk
biner tiap sistem operasi, perintah dengan keluaran aslinya, port dan rute, batas env, langkah
verifikasi, dan gejala yang sudah punya sebab.

Konvensi ini bukan temuan baru: tiap service di repo induk sudah punya `how_to_run.md`, dan
`AGENTS.md` memerintahkan berkas itu ikut diperbarui bila command/env/port berubah.

## Tujuan Teknis

- README hanya memegang "apa yang alat ini bisa dan tidak bisa buktikan"; "cara menjalankan" pindah.
- Setiap angka di `how_to_run.md` berasal dari kode atau dari keluaran yang dijalankan, bukan dari
  ingatan: default env, nomor port, panjang byte, dan kalimat keluaran dikutip dari sumbernya.
- Berkas baru tidak boleh jadi dokumen kelas dua: kutipan `nama.rs:NNN` dan tabel modul di dalamnya
  harus diperiksa penjaga yang sama seperti README dan CHANGELOG.
- Nomor bagian README tidak berubah, supaya anchor `#7-penjaga-anti-hardcode` yang sudah dirujuk
  orang tidak patah.

## Scope

- Dikerjakan: `how_to_run.md` (baru), README §1/§6/§9 dipangkas + baris versi di kepala,
  `src/main.rs` (dua loop daftar dokumen penjaga), `CHANGELOG.md`.
- TIDAK dikerjakan: mengubah perilaku alat, mengubah teks `petunjuk()` di `src/main.rs`, menambah
  perintah baru, atau mengklaim dukungan macOS/MSRV yang belum diuji.

## Langkah Implementasi

1. Ambil nilainya dari kode: `PORT_BAWAAN` di `src/saji.rs`, `BATAS_BACA_BUKTI` /
   `INDEKS_BATAS_BERKAS` / `INDEKS_BATAS_BYTE` di `src/audit.rs`, dan string keluaran `init`,
   `generate`, `check`, `serve` dari `src/template.rs`, `src/main.rs`, `src/saji.rs`.
2. Tulis sembilan bagian: prasyarat dan versi, bangun, direktori kerja, enam perintah, setelan env,
   verifikasi sendiri, tabel gejala-sebab-obat, isi CI, tempat mulai mengubah perilaku.
3. README: tambah baris versi di kepala; ganti prose §1 dengan quick-start + satu peringatan
   direktori kerja; ganti tabel env §6 dengan pointer; ganti blok perintah §9 dengan pointer yang
   tetap menyebut jumlah tes (langkah CI menuntut angka itu ada di README).
4. Perluas kedua penjaga dokumen ke `how_to_run.md`.
5. Buktikan giginya: geser satu kutipan `src/main.rs:191` menjadi `:190`, jalankan
   `cargo test sitasi`, catat merahnya, kembalikan.
6. Jalankan langkah CI "Jumlah tes di README sama dengan suite" secara manual.

## Output yang Diharapkan

`how_to_run.md` yang bisa dipakai orang tanpa membaca README, README yang lebih ringan, dan dua
penjaga yang sekarang menyapu tiga berkas dokumen.

## Dependencies

Setelah task1–task6 (isinya menyebut versi 0.2.0, angka hasil ukur tampilan, dan saklar tema).

## Acceptance Criteria

- `cargo test` hijau; `clippy --all-targets -- -D warnings` nol; `check` dan `audit` exit 0 di kedua root.
- Langkah CI jumlah tes direproduksi lokal: angka suite sama dengan angka pertama di README.
- Penjaga kutipan **merah** ketika sebuah `nama.rs:NNN` di `how_to_run.md` digeser, dan pesan galatnya
  menyebut isi baris yang sebenarnya.
- Tidak ada satu pun angka di `how_to_run.md` yang tidak bisa ditunjuk sumbernya di kode.

## Estimasi

45 menit.

## Hasil (terukur, 2026-10-26)

- `how_to_run.md` **230 baris**; README **301 baris** (sebelumnya 311). Yang keluar dari README
  adalah detail cara jalan, bukan isi: blok perintah §1, tabel env §6, blok perintah §9.
- Nilai yang dikutip dari kode, bukan dari ingatan: `PETA_PORT` 8787 (`src/saji.rs`),
  `PETA_BATAS_BACA_BUKTI` 4.000.000, `PETA_INDEKS_BATAS_BERKAS` 12.000,
  `PETA_INDEKS_BATAS_BYTE` 24.000.000 (`src/audit.rs`); kalimat keluaran `DIBUAT 7 kerangka
  manifest di ...`, `LEWATI: ... (7 berkas dipertahankan)`, `DIPERTAHANKAN (sudah ada): ...`,
  `OK: ... sinkron (...)`, `OK: serve di http://127.0.0.1:8787/` dikutip apa adanya dari
  `src/template.rs`, `src/main.rs`, `src/saji.rs`.
- Gigi penjaga dibuktikan: kutipan digeser ke `src/main.rs:190` ->
  `test tests::sitasi_baris_di_dokumen_masih_hidup ... FAILED`, pesan:
  `kutipan baris di dokumen alat busuk: ["how_to_run.md: main.rs:190 — bukan lagi assert atau
  definisi fn: `/// Yang menjaga aturan ini: kode_alat_tidak_menyebut_letaknya_di_repo_pemakai`.`"]`.
  Dikembalikan -> hijau.
- Langkah CI dijalankan manual di luar GitHub: `suite=81 readme=81 sama=ya`.
- Gerbang akhir: **81/81 tes**, clippy nol warning, `check` exit 0 dan `audit` exit 0 di root induk,
  `check --root contoh` exit 0.
- Satu kekhawatiran saya sendiri tidak terjadi dan saya ukur, bukan saya tenangkan: paragraf
  CHANGELOG baru menyebut `:190` dan saya mengira penjaga akan menyalahkannya; ternyata polanya
  menuntut nama berkas di depan angka, jadi hanya `src/main.rs:191` yang diperiksa.
- Perbaikan klaim task4 yang saya tulis di laporan sebelumnya: perataan tepi kartu hanya berlaku di
  mode bertingkat. Terukur pada halaman repo induk mode Sub-fitur: 84 kartu pada 42 sumbu x, **0
  kolom** berisi lebar kartu campur. Mode Produk memakai layout gaya sehingga 17 kartunya memang
  tidak sekolom — klaim "tepi kartu sejajar" tidak berlaku di sana.
- Tidak dilakukan: commit `docs/view/index.html` dan `docs/view/README.md` (root `Monoframe/` bukan
  repo git), push, klaim dukungan macOS atau `rust-version`.
