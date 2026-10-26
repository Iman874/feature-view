# task4 — Geometri: satu tinggi kontrol, satu tinggi baris, kartu sekolom sejajar

## Judul

Merapikan yang paling cepat dibaca mata: tinggi kontrol di baris yang sama, tinggi baris di rail, dan tepi kiri kartu graf.

## Deskripsi

Terukur di Chrome pada halaman contoh, 1440x900 (angka yang sama di kedua skema):

- Bar alat duduk di y=8..41 dan menaruh tiga tinggi berbeda dalam satu baris: `.tombol` 27 px, `select` 30 px, `.tombol-ikon-saja` 28 px.
- Rail kiri punya tiga tinggi baris: `.pohon-grup-tombol` 19 px, `.pohon-sub` 24 px, `.pohon-pilih` 27 px.
- Kartu graf tidak berbagi tepi: sumbu x 298 dan 312 untuk kartu pada baris yang sama, lebar 240 / 263 / 269 / 292 px karena lebarnya mengikuti panjang label.
- Pembagian wilayah hari ini: rail 250 px, kanvas peta 830 px, panel detail 360 px.

## Tujuan Teknis

- Semua kontrol pada satu baris alat punya tinggi yang sama (30 px), termasuk `select` dan `input`.
- Baris rail memakai satu tinggi dasar dengan satu tingkat penanda hierarki (grup vs butir), bukan tiga.
- Kartu sekolom punya tepi kiri yang sama; lebar tetap mengikuti isi tetapi tidak lagi menggeser tepi.

## Scope

- `src/view.css` (blok bar alat, rail, badge) dan `src/render.js` (perhitungan letak/lebar kartu graf).
- TIDAK: warna (task3), saklar (task2), isi dataset (task6).

## Langkah Implementasi

1. Tambah `--kontrol-tinggi: 30px` (hasil keputusan task1 soal lantai huruf; kalau owner memilih lantai 11 px, tinggi ini ikut dihitung ulang, bukan ditebak).
2. Terapkan ke `button`, `select`, `input`, `.tombol`, `.tombol-ikon-saja` lewat `min-height` + `box-sizing` yang sudah ada; jangan menambah padding per kelas.
3. Rail: tetapkan tinggi baris dasar dari token jarak (`--sp-*`) dan bedakan grup vs butir lewat bobot/warna, bukan lewat tinggi.
4. Kartu graf: bulatkan `x` tiap kartu ke grid kolom yang sama; tinggi baris kartu dibiarkan mengikuti isi.
5. Ukur ulang di 1440x900 dan 1280x800: daftar tinggi kontrol, daftar sumbu x kartu, dan pastikan tidak ada horizontal overflow.
6. Pastikan mode tanpa entri (LOD `simpl`) dan batas zoom `SKALA_MIN` (`src/render.js:28`) tidak berubah perilakunya.

## Output yang Diharapkan

Satu angka tinggi per baris kontrol, satu tinggi baris rail, tepi kartu sekolom sejajar.

## Dependencies

Butuh task1 (token) dan task3 (warna badge sudah tetap, supaya tinggi baris tidak perlu dikompensasi dua kali).

## Acceptance Criteria

- Terukur: semua kontrol di bar alat punya `height` sama; semua baris rail punya tinggi dari satu pasangan nilai.
- Tidak ada teks terpotong dan tidak ada scrollbar horizontal pada 1280x800.
- Jumlah kartu yang terpotong LOD tidak berubah pada dataset yang sama (sebelum/sesudah).
- `cargo test` hijau; `check` exit 0.

## Estimasi

1,5 jam.

## Bukti (diisi saat eksekusi)

- Tabel tinggi kontrol sebelum/sesudah; daftar sumbu x kartu sebelum/sesudah; dua viewport.
