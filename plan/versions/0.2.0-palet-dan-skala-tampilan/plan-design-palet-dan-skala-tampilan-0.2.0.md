# Plan Design — Palet, skala tipografi, dan kerapihan tampilan (0.2.0)

Target: `tools/feature-view` (crate mandiri, Rust std-only + `src/view.css` + `src/render.js`). Tidak menyentuh service mana pun di repo induk. Berkas plan ini hidup di repo publik alat, jadi tidak menyebut satu pun nama host.

## Latar Belakang

Alat ini sudah benar di sisi data (tujuh manifest, `check`, `audit`), tetapi halaman hasilnya belum layak dijual. Owner menilai dua hal: paletnya belum selesai ("hanya mode gelap saja") dan tataannya "belum begitu teratur". Keduanya saya ukur, bukan ditebak, di Chrome pada dataset contoh, viewport 1440x900, 282 elemen terlihat, skema terang dan gelap menghasilkan angka yang identik.

Temuan terukur:

1. **Tidak ada skala tipografi.** `src/view.css` punya 39 deklarasi `font-size:` dengan **16 nilai berbeda**: `0.66 0.7 0.72 0.74 0.75 0.76 0.78 0.8 0.82 0.84 0.86 0.95 1.05rem` plus `17px 19px 21px` plus `inherit` dan `100%`. Nilainya digeser per elemen dengan langkah 0.02rem — itu sebabnya halaman terasa "tidak teratur": tiap blok punya ukuran hurufnya sendiri. Ter-render: 10,56 / 11,2 / 11,52 / 12 / 12,16 / 12,48 / 12,8 / 13,12 / 13,44 / 16 / 17 / 19 px. Yang paling kecil, 10,56 px, terlalu kecil untuk teks chrome.
2. **Tidak ada skala jarak.** 39 deklarasi `padding:`, 33 `gap:`, 21 `margin:`. Ter-render ada 9 nilai gap berbeda (2,4 / 4 / 4,8 / 5,6 / 6,4 / 7,2 / 8 / 12 / 14,4 px) dan padding pecahan seperti `2.56px 4.8px`, `3.52px 4.8px`, `4.8px 24px 4.8px 8px`.
3. **Tidak ada token radius.** 23 deklarasi `border-radius:`; ter-render 5 nilai berbeda (2 / 5 / 6 / 7 / 8 px) di samping 237 elemen `0px`.
4. **Satu baris alat, tiga tinggi kontrol.** Terukur: `.tombol` 27 px, `select` 30 px, `.tombol-ikon-saja` 28 px — ketiganya duduk di baris yang sama (y=8..41). Di rail kiri barisnya 19 / 24 / 27 px. Ini yang paling cepat dibaca mata sebagai "berantakan".
5. **Tidak ada saklar tema.** `GELAP` hanya diambil dari OS: `src/render.js:88` (`bacaPalet`) dan listener `src/render.js:2331`. Jadi orang yang OS-nya gelap hanya pernah melihat gelap dan tidak bisa memilih — inilah yang owner rasakan sebagai "hanya mode gelap saja". Skema terang sebenarnya ada (`src/view.css:1-46`), hanya tidak pernah terlihat di box itu.
6. **Kontras bukan masalahnya.** Rasio terendah 5,06 (terang) dan 5,75 (gelap); nol elemen di bawah 4,5. Jadi kerjanya menurunkan palet dari satu aturan, bukan menaikkan kontras.
7. **Koreksi klaim saya sendiri:** `SKALA_MIN = 0.62` di `src/render.js:28` adalah clamp skala zoom SVG, bukan lantai ukuran huruf. Tidak ada satu pun aturan yang menahan teks chrome di 13 px selama ini.

Acuan nilai diambil dari sistem yang sudah shipped (lihat task3 untuk tabelnya): GitHub Primer (type 12/14/16/20/32, space 2/4/8/12/16/24, radius 6 px), Radix Colors (satu skala netral 12 langkah, teks hanya di langkah 11 dan 12, status = teks + solid + latar bernuansa satu hue), shadcn/ui (dark border = putih transparan 10 %, radius 0,625rem), Tailwind v4 (`--spacing: 0.25rem`, `--radius-md: 0.375rem`), Vercel Geist (netral 10 langkah, satu keluarga).

## Tujuan (terukur)

1. Nilai `font-size:` yang berbeda di `src/view.css` turun dari **16 menjadi maksimal 6**, semuanya token, semuanya >= 12 px.
2. `border-radius:` yang berbeda turun dari **5 menjadi satu tangga token** (`--r-sm` 4, `--r-md` 6, `--r-lg` 10, `--r-pill` 999) + `50%` untuk yang bulat; **nol** nilai telanjang. (Deviasi dari angka "maksimal 2" yang saya tulis sebelum melihat isinya: kartu panel, minimap, dan chip pil memang butuh tiga tingkat berbeda; yang dihapus adalah nilai tengah 2/5/7/8 px, bukan tingkatan yang dipakai.)
3. `padding`/`gap`/`margin` hanya memakai token jarak kelipatan 4 px (2 px diizinkan untuk celah rambut dalam badge); **nol** nilai pecahan seperti `4.8px` atau `2.56px` yang tersisa di berkas.
4. Semua kontrol pada satu baris alat punya tinggi yang sama: **30 px** (terukur di browser, bukan dari kode).
5. Ada **saklar tema** di bar alat: default ikut OS, pilihan user disimpan dan dipulihkan tanpa build ulang, dan warna graf (yang dihitung di JS dari `GELAP`) ikut berpindah — bukan hanya CSS-nya.
6. Kedua skema diturunkan dari **satu skala netral 12 langkah** dan **dua langkah teks saja**; border gelap jadi lapisan transparan, bukan hex baru; rampa status empat-nama memakai tiga peran dari satu hue (teks / solid / latar bernuansa).
7. Tidak ada satu pun nama entitas data repo pemakai yang masuk ke kode atau ke berkas plan ini (penjaga yang sudah ada harus tetap hijau).
8. Nol regresi fungsi: 77 tes tetap hijau, `clippy --all-targets -D warnings` bersih, `check` dan `audit` exit 0, halaman contoh dan halaman repo induk disegarkan ulang lewat `generate` (bukan diedit manual).

## Scope

Dikerjakan:

- `src/view.css` — token skala (jarak, radius, tipografi), palet netral dan rampa status/grup untuk kedua skema, tinggi kontrol.
- `src/render.js` — saklar tema (override `GELAP` + penyimpanan pilihan), penyesuaian `bacaPalet()` dan `gantiSkema()`.
- Penjaga baru di `src/` yang membaca `view.css`/`render.js` sebagai teks, supaya skala tidak bisa diam-diam melonggar lagi.
- Penyegaran `contoh/docs/view/index.html` dan `docs/view/index.html` (repo induk) lewat `cargo run -- generate`.
- Sinkronisasi angka yang dokumen klaim tentang CSS/JS: `README.md`, `CHANGELOG.md`, tabel modul, dan `docs/view/README.md` milik repo induk.

TIDAK dikerjakan:

- **Bug CI ubuntu (run 40280461153, step "Tes") diparkir lebih dulu atas keputusan owner.** Nama tes yang merah belum tersedia; log job butuh autentikasi GitHub. Enam hipotesis sudah diuji dan dibuang (line-ending, nama berkas beda huruf, kolom `folder`, urutan `read_dir`, tabrakan direktori temp, env/socket di dalam tes). Yang ditemukan justru cacat lain dan dicatat di task6: `root_uji()` (`src/uji.rs:139`) memilih `../..` di laptop penyusun tetapi jatuh ke `contoh/` di runner, jadi "hijau lokal" dan "hijau CI" bukan objek yang sama.
- Menambah jumlah baris dataset `contoh/` supaya halaman jualannya tampak padat — keputusan terpisah, masih menunggu pilihan owner (dataset fiktif vs repo nyata).
- Screenshot/GIF untuk README — butuh alat tangkap layar, dikerjakan setelah paletnya disetujui.
- Perubahan struktur data, kolom manifest, aturan `check`/`audit`, atau kontrak HTML apa pun. Halaman tetap satu berkas tanpa dependency.
- `rust-version` dan `cargo fmt --check` (121 blok) — masih keputusan owner, tidak ikut di sini.

## Breakdown Task

| task | isi | estimasi |
|---|---|---|
| task1 | Token skala: jarak, radius, tipografi; ganti 16 nilai huruf + 9 gap + 5 radius di `view.css` | 2-3 jam |
| task2 | Saklar tema dua skema di bar alat + penyimpanan pilihan + graf ikut warna | 1,5-2 jam |
| task3 | Palet: satu skala netral 12 langkah, dua langkah teks, border gelap transparan, rampa status 3 peran, grup 8 hue | 2 jam |
| task4 | Geometri: tinggi kontrol satu baris jadi 30 px, baris rail satu tinggi, kartu graf sejajar | 1,5 jam |
| task5 | Penjaga anti-regresi-skala (tes yang menghitung ulang nilai di `view.css`/`render.js`) + bukti taring lewat mutasi | 1-1,5 jam |
| task6 | Segarkan halaman contoh + halaman repo induk, sinkronkan semua angka dokumen, catat provenance md5/cap | 1 jam |
| task7 | **Ditambahkan setelah eksekusi** atas permintaan owner: pindahkan bagian operasional ke `how_to_run.md` dan perluas penjaga dokumen ke berkas baru itu | 45 menit |

Urutannya sengaja **sekuensial, bukan paralel**: task1, task3, dan task4 menulis ke `src/view.css` yang sama, dan task2 menyentuh `render.js` yang sama dengan task3. Membaginya ke beberapa agen sekaligus menghasilkan konflik pada baris yang sama.

## Design Teknis

**task1 — `src/view.css`.** Blok `:root` (baris 1-46) ditambah tiga keluarga token:

```
--sp-1: 2px; --sp-2: 4px; --sp-3: 8px; --sp-4: 12px; --sp-5: 16px; --sp-6: 24px;
--r-sm: 4px; --r-md: 6px;
--fs-1: 0.75rem;  /* 12px label, badge, chip  */
--fs-2: 0.8125rem;/* 13px teks chrome         */
--fs-3: 0.875rem; /* 14px teks utama panel    */
--fs-4: 1rem;     /* 16px judul blok          */
--fs-5: 1.25rem;  /* 20px judul halaman       */
--fs-6: 1.75rem;  /* 28px angka besar         */
```

Bobot tetap tiga yang sudah ada (400 / 600 / 700), tidak menambah 500. 39 deklarasi `font-size:` dipetakan ke enam token itu; yang 10,56-11,52 px naik ke `--fs-1`. Konsekuensi yang harus diterima: baris rail dan badge jadi lebih tinggi, jadi task4 menyesuaikan tingginya di tempat yang sama, bukan di dua tempat.

**task2 — `src/render.js`.** `bacaPalet()` (baris 87-88) adalah satu-satunya tempat `GELAP` ditetapkan. Perubahannya kecil dan terlokalisasi: `GELAP = pilihanTema() ?? ikutOS()`, dengan `pilihanTema()` membaca `localStorage` memakai kunci dan pola yang sama dengan `simpanTampilan()` (baris 1324-1340, kunci `rangkiang-simple`), lalu tombol baru di bar alat memanggil `gantiSkema()` yang **sudah ada** (dipakai listener `src/render.js:2331`) — jadi tidak ada jalur render baru, hanya sumber `GELAP` yang bisa ditimpa. Kontrak yang harus dijaga: selama user belum memilih, perilaku persis seperti hari ini (ikut OS, termasuk perubahan live).

**task3 — `src/view.css` + kamus `IKON` tidak tersentuh.** Palet netral diambil dari Radix slate (12 langkah, terang dan gelap) lalu dipetakan ke peran: langkah 1-2 = latar aplikasi/kanvas, 3-5 = permukaan panel dan hover, 6-8 = garis, 11 = teks lembut, 12 = teks keras. Border skema gelap ditulis sebagai lapisan putih transparan (pola shadcn `1 0 0 / 10 %`) supaya satu nilai cocok di atas panel mana pun. Rampa status jadi tiga peran satu hue per status (teks / solid / latar bernuansa) mengikuti Primer `*-fg` vs `*-emphasis` dan Radix langkah 11/9/2-5. Grup dibatasi ke 8 hue dengan pasangan S/L tetap per skema; rotasi sudut emas di `hueTambahan()` (baris 74-77) tetap ada untuk grup ke-9 dan seterusnya, tetapi S/L-nya disamakan dengan delapan pertama supaya tidak melompat.

**task4 — `src/view.css` (bar alat, rail) + `src/render.js` (letak kartu).** Tinggi kontrol ditetapkan lewat satu token tinggi (`--kontrol-tinggi: 30px`) yang dipakai `button`, `select`, `input`, dan `.tombol-ikon-saja`. Untuk kartu graf: lebarnya hari ini ikut panjang label (terukur 240/263/269/292 px, sumbu x 298 vs 312), jadi yang dirapikan adalah **tepi kirinya** — kartu sekolom dibulatkan ke grid kolom, bukan dilebarkan seragam sampai teksnya terpotong.

**task5 — penjaga baru di `src/`.** Mengikuti pola penjaga yang sudah ada (sumber dibaca sebagai teks, angka dihitung ulang): jumlah nilai `font-size:` berbeda di `view.css` harus <= 6 dan semuanya `var(--fs-`; tidak boleh ada `border-radius:` di luar `{var(--r-sm), var(--r-md), 50%, 0}`; tidak boleh ada `px` pecahan di `padding`/`gap`/`margin`. Penjaga ini yang membuat "sudah rapi" tetap benar enam bulan lagi, bukan hanya hari ini.

**task6 — dokumen dan artefak.** Setiap perubahan `view.css`/`render.js` wajib diakhiri `cargo run -- generate --root contoh` pada commit yang sama, karena tes `halaman_contoh_di_repo_computed_dari_render_js_yang_sekarang` (`src/template.rs:246`) membandingkan byte demi byte dan akan merah kalau halaman tertinggal satu putaran. Halaman repo induk (`docs/view/index.html`) juga disegarkan; md5 dan `data-cap` lama dicatat di laporan supaya perubahannya bisa dibuktikan, bukan diklaim.

## Dampak ke Sistem

- **Kosmetik, tidak menyentuh kebenaran data.** `check`, `audit`, `cap_manifest`, dan isi manifest tidak berubah. Yang berubah cuma dua aset yang di-`include_str!` ke dalam setiap halaman yang dihasilkan.
- **Semua halaman hasil generate berubah byte.** Ini konsekuensi yang diharapkan, tapi berarti md5 `docs/view/index.html` repo induk bergerak dari `d48446cba09a570d152c7451666db2de` dan halaman contoh dari `9fc5f92d1406fc20f0cf6f9ae59235a4`. Angka baru dicatat di task6; `data-cap` tidak boleh berubah (cap dihitung dari TSV, bukan dari CSS).
- **Risiko regresi nyata #1 — ukuran.** Menaikkan lantai huruf dari 10,56 px ke 12 px membuat rail dan badge lebih tinggi. Kalau tinggi baris ikut naik, daftar panjang bisa mulai menggulir. Diukur di browser pada 1280x800 dan 1440x900, bukan diasumsikan.
- **Risiko regresi nyata #2 — warna graf dihitung di JS.** `hueTambahan()` dan `statusColor` dibangun dari `GELAP` saat boot. Kalau saklar tema hanya mengganti CSS var, halaman jadi separuh terang separuh gelap. Karena itu task2 wajib memakai jalur `gantiSkema()` yang sudah ada.
- **Risiko regresi nyata #3 — penjaga byte-exact.** Lupa `generate --root contoh` membuat CI merah di langkah pertama. Sudah dicatat di task6 sebagai perintah, bukan sebagai peringatan lisan.
- **Risiko regresi #4 — tes berbasis teks buta terhadap tampilan.** Suite hijau tidak membuktikan halaman rapi; verdict kerapiannya wajib angka browser. Ini pelajaran yang sudah dua kali terbayar di repo induk.
- **Tidak ada dependency baru.** Tetap Rust std-only, tetap satu HTML tanpa aset eksternal; penyimpanan tema hanya `localStorage`.
- **Tidak menyentuh service lain.** Tidak ada endpoint, tidak ada skema DB, tidak ada kontrak lintas service.

## Definition of Done

- `grep -o "font-size: [^;]*" src/view.css | sort -u | wc -l` <= 6 dan tidak ada nilai di bawah 12 px.
- `grep -o "border-radius: [^;]*" src/view.css | sort -u` hanya berisi token radius, `50%`, dan `0`. Terukur di browser: `{0px, 4px, 6px, 10px, 50%}`.
- `grep -cE "(padding|gap|margin): [^;]*[0-9]\.[0-9]+px" src/view.css` = 0.
- Terukur di Chrome 1440x900 dan 1280x800, kedua skema: satu tinggi kontrol per baris, nol teks < 12 px, kontras tetap >= 4,5, dan jumlah ukuran font ter-render <= 6.
- Saklar tema: klik -> kedua skema berpindah termasuk warna graf; reload mempertahankan pilihan; tanpa pilihan user, perilaku persis seperti hari ini.
- `cargo test` hijau (jumlahnya naik, dan README ikut), `cargo clippy --all-targets -- -D warnings` bersih.
- `cargo run -- check` dan `-- audit` exit 0 di repo induk; `generate`/`check`/`audit --root contoh` exit 0.
- `data-cap` halaman tidak berubah; md5 lama dan baru tercatat.
- Semua perubahan direview owner sebelum dianggap selesai.

## Catatan untuk owner (masih keputusan)

1. Warna aksen: tetap hijau tua seperti sekarang, atau diambil dari data lewat kunci baru di `config.tsv` (mis. `aksen|#hex`) supaya tiap repo bisa punya warna merek sendiri? Yang kedua mengubah kontrak manifest, jadi bukan keputusan kecil.
2. Lantai 12 px menaikkan tinggi baris rail. Setuju, atau lantai tetap 11 px khusus badge?
3. Saklar temanya pakai teks ("Terang"/"Gelap") atau ikon? Kalau ikon, namanya wajib ada di kamus `IKON` `src/render.js` supaya lolos `bentuk_terdaftar`.
4. Versi crate 0.1.0 -> 0.2.0 dinaikkan pada commit ini, atau dititip ke rilis berikutnya?
5. Folder `plan/` ikut terpublikasi di repo publik alat. Dipertahankan di sini, atau plan diletakkan di luar repo?

## Status eksekusi (2026-10-26)

SELESAI task1-task6. Owner tidak menjawab lima "Catatan untuk owner", lalu memerintahkan
"selesaikan sampai selesai", jadi kelima keputusan itu saya ambil sebagai default konservatif dan
dicatat di sini supaya bisa dibatalkan: aksen tetap hex di CSS (kontrak manifest tidak disentuh),
lantai huruf 12 px, saklar tema berlabel teks, versi crate naik ke 0.2.0, folder `plan/` tetap di
repo publik.

Angka sesudah, terukur di Chrome dengan skrip yang sama seperti di Latar Belakang — halaman contoh
(282 elemen) dan halaman repo induk (1.058 elemen, 17 node) pada 1440x900 dan 1280x800, kedua skema:

| sasaran | sebelum | sesudah |
|---|---|---|
| ukuran huruf ter-render | 16 (terkecil 10,56 px) | 5: 12 / 13 / 14 / 16 / 20 px |
| `border-radius` | 2, 5, 6, 7, 8 px + 0 + 50% | 4, 6, 10, 999 px, 0, 50% — semuanya token |
| `gap` pecahan | 2,4 / 4,8 / 5,6 / 6,4 / 7,2 / 14,4 px | nol |
| tinggi kontrol satu baris | 27 / 28 / 30 px | satu nilai: 30 px |
| tinggi baris rail | 19 / 24 / 27 px | satu nilai: 24 px |
| kontras terendah | 5,06 / 5,75 | 5,03 (terang) / 7,53 (gelap), nol elemen < 4,5 |
| saklar tema | tidak ada | ada; warna graf ikut; pilihan bertahan; nol console error |

Yang berubah di luar rencana: `--terang` langkah terang sempat 38% dan **gagal terukur** — label
aktor jatuh ke kontras 3,95 di skema terang (ditemukan dengan mendaftar rasio tiap elemen teks,
bukan dengan mata). Diturunkan ke 33%. Penjaga variabel yatim juga menangkap dua nama yang memang
suntikan JS (`--grup-warna`, `--jenis-warna`) dan aturannya dipertajam: hanya boleh tak terdefinisi
dalam bentuk bercadangan `var(--x, cadangan)`.

Taring empat penjaga baru dibuktikan dengan tiga mutasi sekaligus dalam satu kali jalan:
`font-size: 0.79rem` di `.bar h1` -> `ukuran huruf di luar skala: 0.79rem`; definisi `--sp-6`
dihapus -> `variabel dipakai tanpa definisi dan tanpa cadangan: ["sp-6", "sp-6", "sp-6", "sp-6"]`;
kunci tema di cangkang diganti -> `cangkang HTML memakai kunci tema lain`; penjaga halaman basi ikut
merah (4 gagal / 77 lulus). Ketiganya dikembalikan.

Gerbang akhir: **81 tes hijau**, `clippy --all-targets -- -D warnings` nol, `check` dan `audit` exit 0
di kedua root (audit induk 0 kuning / 3 tidak teruji, sama seperti sebelumnya), `-V` mencetak
`feature-view 0.2.0`. Provenance: `data-cap 5b3d521ee425ad20` TIDAK berubah sementara md5 induk
`d48446cb…` (214282 B) -> `5a3345c1…` (222944 B) dan contoh `9fc5f92d…` (126980 B) -> `190c960f…`
(135642 B) — bukti yang berubah hanya lapisan tampilan yang ter-bake, bukan manifest.

**Belum:** commit dan push (menunggu review owner); bug CI ubuntu tetap diparkir; screenshot README;
dataset `contoh/` yang lebih padat; `root_uji()` yang membaca manifest repo pemakai di mesin penyusun
tetapi jatuh ke `contoh/` di runner.

**task7 menyusul setelah laporan.** Owner membaca hasilnya lalu meminta bagian operasional dipisah ke
`how_to_run.md` supaya README lega untuk versi dan kemampuan. Task itu tidak ada di daftar awal, jadi
ia dicatat sebagai task7 dengan laporannya sendiri di
`task7-how-to-run.md`. Dua hal yang keluar dari situ dan memperbaiki laporan di atas: penjaga dokumen
sekarang menyapu tiga berkas (bukti tarangnya terukur), dan klaim perataan kartu saya persempit —
terukur hanya berlaku di mode Sub-fitur (84 kartu, 42 sumbu x, 0 kolom dengan lebar campur), tidak di
mode Produk yang memakai layout gaya.
