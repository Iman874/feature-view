# task1 — Token skala: jarak, radius, tipografi

## Judul

Ganti 16 nilai `font-size` dan 5 nilai `border-radius` yang ad-hoc dengan tiga keluarga token di `src/view.css`.

## Deskripsi

`src/view.css` (681 baris) hari ini menyetel tiap elemen dengan nilainya sendiri: 39 deklarasi `font-size:` memakai 16 nilai berbeda (`0.66 0.7 0.72 0.74 0.75 0.76 0.78 0.8 0.82 0.84 0.86 0.95 1.05rem` + `17px 19px 21px` + `inherit` + `100%`), 23 deklarasi `border-radius:` dengan 5 nilai berbeda (2/5/6/7/8 px), dan 39 `padding:` + 33 `gap:` + 21 `margin:` yang menghasilkan jarak pecahan (`2.56px 4.8px`, `4.8px 24px 4.8px 8px`, gap 2,4/4,8/5,6/6,4/7,2/14,4 px). Terukur di Chrome pada dataset contoh, 1440x900: 16 ukuran huruf ter-render antara 10,56 dan 19 px, 9 gap berbeda, 5 radius berbeda.

Task ini memperkenalkan skalanya dan memaksa seluruh deklarasi memakainya.

## Tujuan Teknis

- Enam langkah tipografi, semua >= 12 px, semua lewat `var(--fs-*)`.
- Enam langkah jarak kelipatan 4 px (2 px khusus celah rambut), semua lewat `var(--sp-*)`.
- Dua langkah radius (`--r-sm` 4 px, `--r-md` 6 px) + `50%` untuk yang bulat.
- Nilai yang tersisa di berkas hanya token, bukan angka telanjang, supaya task5 bisa menjaganya.

## Scope

- `src/view.css` blok `:root` (baris 1-46) dan seluruh deklarasi `font-size`/`padding`/`gap`/`margin`/`border-radius` di sisanya.
- TIDAK: warna (task3), tinggi kontrol dan letak kartu (task4), `render.js` (task2), dataset (task6).

## Langkah Implementasi

1. Tambah blok token di `:root`: `--sp-1..6` = 2/4/8/12/16/24 px; `--r-sm` 4px, `--r-md` 6px; `--fs-1..6` = 0.75/0.8125/0.875/1/1.25/1.75rem. Bobot: tetap 400/600/700.
2. Petakan tiap nilai lama ke langkah terdekat: `0.66..0.78rem` -> `--fs-1`; `0.8..0.86rem` -> `--fs-2`; `0.95rem` -> `--fs-3`; `1rem`/`17px` -> `--fs-4`; `19px`/`21px` -> `--fs-5` (judul) atau `--fs-6` (angka besar).
3. Ganti tiap padding/gap/margin ke `var(--sp-*)`; nilai dua komponen ditulis dua token (`var(--sp-2) var(--sp-3)`).
4. Ganti tiap `border-radius` ke `--r-md` (atau `--r-sm` untuk badge kecil, `50%` untuk avatar/ikon).
5. `cargo run --quiet -- generate --root contoh` lalu ukur di Chrome (skrip penghitung yang sama seperti di plan) pada 1440x900 dan 1280x800.
6. Catat angka sebelum/sesudah di baris "Bukti" di bawah.

## Output yang Diharapkan

`src/view.css` dengan tiga keluarga token dan nol angka telanjang di properti yang dijaga; halaman contoh tersusun ulang (disegarkan massal di task6, di sini hanya untuk mengukur).

## Dependencies

Tidak ada (task pertama). Menghalangi task3, task4, task5.

## Acceptance Criteria

- `grep -o "font-size: [^;]*" src/view.css | sort -u | wc -l` <= 6.
- `grep -o "border-radius: [^;]*" src/view.css | sort -u` hanya berisi `var(--r-sm)`, `var(--r-md)`, `50%`, `0`.
- `grep -cE "(padding|gap|margin): [^;]*[0-9]\.[0-9]+px" src/view.css` = 0.
- Terukur di browser: nol teks di bawah 12 px; jumlah ukuran font ter-render <= 6; kontras terendah tetap >= 4,5.
- `cargo test` tetap hijau; `cargo run -- check` exit 0.

## Estimasi

2-3 jam (39 + 23 + 93 deklarasi, mekanis tapi lebar; jangan diformat otomatis).

## Bukti (diisi saat eksekusi)

- Sebelum: 16 nilai font-size, 5 radius, 9 gap ter-render, teks terkecil 10,56 px.
- Sesudah: (diukur, bukan diperkirakan)
- Gigi tes: ditunjukkan di task5 lewat mutasi.
