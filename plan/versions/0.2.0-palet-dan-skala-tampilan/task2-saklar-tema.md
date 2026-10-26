# task2 — Saklar tema dua skema

## Judul

Menambah tombol pilihan tema di bar alat, dengan default tetap ikut OS dan pilihan user disimpan.

## Deskripsi

Skema terang dan gelap sama-sama ada di `src/view.css` (blok `:root` baris 1-46 dan blok `@media (prefers-color-scheme: dark)` baris 47-90), tetapi tidak ada satu pun jalan untuk memakainya: `GELAP` ditetapkan satu kali dari OS di `src/render.js:88` (`bacaPalet`) dan hanya berubah kalau OS berubah (listener `src/render.js:2331` -> `gantiSkema`). Efeknya, seperti yang owner keluhkan, halaman "hanya mode gelap" di box yang OS-nya gelap.

Yang membuat ini bukan sekadar menukar CSS: warna graf dihitung di JavaScript dari `GELAP` (`hueTambahan()` `src/render.js:74-77` dan `statusColor`/`kindColor` yang dibangun `bacaPalet`). Kalau hanya variabel CSS yang berpindah, chrome halaman jadi terang sementara node graf tetap gelap.

## Tujuan Teknis

- Satu saklar di bar alat yang memindahkan **seluruh** halaman (CSS + warna graf) antar skema.
- Default tidak berubah: selama user belum memilih, perilaku persis seperti hari ini, termasuk ikut OS secara live.
- Pilihan user bertahan setelah reload dan tidak ikut hilang saat `prefers-color-scheme` OS berubah.

## Scope

- `src/render.js`: `bacaPalet()`, satu fungsi baca/tulis preferensi mengikuti pola `simpanTampilan()` (baris 1324-1340), satu kontrol di bar alat, dan pemanggilan ulang `gantiSkema()` yang sudah ada.
- `src/view.css`: pemilih atribut untuk memaksa skema (mis. `html[data-tema="gelap"]`) supaya CSS tidak lagi hanya bergantung pada `@media`.
- TIDAK: nilai warna baru (task3), tinggi kontrol (task4).

## Langkah Implementasi

1. Pindahkan blok `@media (prefers-color-scheme: dark)` ke selector ganda: tetap aktif saat OS gelap **dan** saat `html[data-tema="gelap"]`, dan nonaktif saat `html[data-tema="terang"]`.
2. `bacaPalet()` jadi `GELAP = pilihanTema() == "gelap" || (pilihanTema() == null && ikutOS())`.
3. Simpan preferensi di `localStorage` dengan kunci terpisah dari kunci tampilan, memakai helper simpan/baca yang sudah ada di berkas itu.
4. Tambah tombol di bar alat yang men-toggle `data-tema` lalu memanggil `gantiSkema()`. Labelnya teks; kalau owner memilih ikon (Catatan #3 plan), namanya harus ada di kamus `IKON` `src/render.js` supaya lolos `bentuk_terdaftar`.
5. Pastikan `mulaiPenjagaan()`/listener OS tidak menimpa pilihan user.
6. Ukur di Chrome: klik, reload, ganti `prefers-color-scheme` lewat emulation.

## Output yang Diharapkan

Halaman yang bisa berpindah tema tanpa build ulang, dengan warna graf yang ikut berpindah.

## Dependencies

Butuh task1 (token jarak/tinggi dipakai tombol baru). Dihaluskan oleh task5 (penjaga) dan task6 (penyegaran halaman).

## Acceptance Criteria

- Klik saklar -> `getComputedStyle(document.body).backgroundColor` berpindah dan `statusColor` node ikut berpindah (diukur, bukan dilihat).
- Reload -> pilihan bertahan.
- Tanpa pilihan user -> hasil identik dengan hari ini pada kedua emulation OS.
- `cargo test` hijau; `node --check src/render.js` bersih; `check`/`audit` exit 0.

## Estimasi

1,5-2 jam.

## Bukti (diisi saat eksekusi)

- Empat keadaan terukur: (OS gelap, belum memilih), (OS gelap, memilih terang), (OS terang, memilih gelap), (OS terang, belum memilih) -> nilai `--bg` dan warna satu node.
- Kunci `localStorage` yang dipakai dan isinya.
