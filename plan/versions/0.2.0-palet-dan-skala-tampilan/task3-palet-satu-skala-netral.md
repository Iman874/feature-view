# task3 — Palet dari satu skala netral dan tiga peran status

## Judul

Menurunkan kedua skema dari satu skala netral 12 langkah, dua langkah teks, border gelap transparan, dan rampa status tiga peran satu hue.

## Deskripsi

Palet sekarang dikumpulkan per berkas, bukan diturunkan dari aturan: `src/view.css:3-45` menulis 12 warna grup dan 6 warna relasi sebagai hex terpisah, lalu blok gelap di baris 49-89 menulis ulang semuanya sebagai hex lain. Hasilnya dua palet yang tidak punya hubungan struktural, dan mata membacanya sebagai "belum selesai" walaupun kontrasnya lolos (terukur 5,06 terang / 5,75 gelap).

Acuan yang dipakai (semuanya sistem shipped, nilainya diambil dari dokumentasi/sumbernya):

| sumber | nilai yang dipinjam |
|---|---|
| Radix Colors, skala slate | 12 langkah netral: terang `#fcfcfd #f9f9fb #f0f0f3 #e8e8ec #e0e1e6 #d9d9e0 #cdced6 #b9bbc6 #8b8d98 #80838d #60646c #1c2024`; gelap `#111113 #18191b #212225 #272a2d #2e3135 #363a3f #43484e #5a6169 #696e77 #777b84 #b0b4ba #edeef0`. Peran: 1-2 latar aplikasi, 3-5 permukaan, 6-8 garis, 11 teks lembut, 12 teks keras |
| Radix Colors, solid vs teks | satu hue tiga peran: teks di langkah 11, solid di 9, latar bernuansa di 2-5 (green `#218358`/`#30a46c`, amber `#ab6400`/`#ffc53d`, indigo `#3a5bc7`/`#3e63dd`) |
| shadcn/ui | border skema gelap = putih transparan `oklch(1 0 0 / 10%)`, bukan hex baru; radius `0.625rem` |
| GitHub Primer | pasangan `*-fg` vs `*-emphasis` untuk status; netral gelap `#0d1117`/`#151b23`, border `#3d444d` |
| Tailwind v4 / Geist | satu keluarga netral dengan kroma rendah untuk permukaan gelap |

## Tujuan Teknis

- Kedua skema jadi dua pemotongan dari satu skala yang sama, bukan dua daftar hex yang berbeda.
- Teks hanya dua langkah (keras dan lembut) di seluruh halaman; tidak ada "teks setengah lembut".
- Rampa status: tiap status punya teks, solid, dan latar bernuansa dari hue yang sama, jadi badge, chip, dan garis node terasa satu keluarga.
- Grup dibatasi ke 8 hue dengan pasangan S/L tetap per skema; rotasi sudut emas tetap ada untuk grup ke-9 dst tetapi dengan S/L yang sama.

## Scope

- `src/view.css`: blok `:root` terang dan blok gelap (termasuk `--grup-*`, `--relasi-*`, `--status-*`).
- `src/render.js`: hanya kalau pemetaan `rampa('--grup')` perlu mengikuti bentuk token baru; tidak mengubah logika sudut emas.
- TIDAK: ukuran dan jarak (task1), saklar (task2), geometri (task4).

## Langkah Implementasi

1. Petakan tiap peran lama ke langkah skala baru; tulis sebagai token `--netral-1..12` lalu `--bg: var(--netral-2)` dst, supaya skema gelap tinggal mengganti pemetaan langkahnya, bukan daftarnya.
2. Ganti border gelap dengan lapisan transparan (`rgba(255,255,255,.10)` untuk garis halus, `.16` untuk garis keras) dan hapus hex border gelap yang sekarang berdiri sendiri.
3. Bangun tiga peran per status dari satu hue: `--status-1-teks`, `--status-1-solid`, `--status-1-latar` (empat status = empat hue: hijau, amber, slate, biru). Alias posisional `--aktif/--sebagian/--mati/--rencana` tetap ada karena dipakai badge dan chip.
4. Batasi `--grup-1..8` pada delapan hue dengan S/L tetap; `--relasi-*` diturunkan dari grup yang sama, bukan daftar terpisah.
5. Ukur ulang kontras tiap pasangan teks/latar di kedua skema; tidak ada yang boleh turun di bawah 4,5.
6. Segarkan halaman contoh dan ukur di browser dua skema.

## Output yang Diharapkan

Dua blok palet yang strukturnya sama, dengan tiga peran per status dan delapan hue grup terkuras.

## Dependencies

Butuh task1 (token) dan dikerjakan sebelum task2 selesai diuji visual (saklar membuat kedua skema terlihat berdampingan).

## Acceptance Criteria

- Jumlah hex berbeda di `src/view.css` turun dibanding baseline (catat dua-duanya).
- Tidak ada lagi satu peran yang ditulis dua kali (mis. `--status-1` dan `--grup-1` bernilai sama tapi ditulis terpisah).
- Semua pasangan teks/latar >= 4,5 di kedua skema (terukur, bukan dihitung tangan).
- `cargo test` hijau; `bentuk_terdaftar` dan penjaga `render_js_*` tetap hijau.

## Estimasi

2 jam.

## Bukti (diisi saat eksekusi)

- Jumlah hex unik sebelum/sesudah; tabel kontras per status per skema.
- Cuplikan pemetaan langkah -> peran supaya pemakai repo lain bisa menyalanya.
