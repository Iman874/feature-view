# Cara menjalankan `feature-view`

Bagian ini khusus perintah dan peralatannya. README menyimpan apa yang alat ini bisa dan tidak bisa
buktikan; kalau Anda hanya ingin tahu "jalankan apa", halaman ini yang dibaca.

## 1. Prasyarat dan versi

| Yang dibutuhkan | Versi | Catatan |
|---|---|---|
| Rust (`cargo`, `rustc`) | dibangun dan dites pada **rustc 1.99.1**, edition 2021 | `rust-version` sengaja **belum** diumumkan di `Cargo.toml`: angka minimum belum diuji pada toolchain lama, dan menuliskannya tanpa uji berarti berbohong ke CI orang |
| Dependency Rust | **nol** | hanya `std`; tidak ada satu pun crate di `[dependencies]`, jadi tidak ada `Cargo.lock` pihak ketiga yang bisa ikut terseret |
| Node.js | opsional | hanya dipakai saat mengembangkan: `node --check public/render.js` untuk memastikan skrip halaman masih valid sintaks |
| Browser | apa pun yang mendukung `color-mix()` dan CSS custom properties | halaman hasilnya satu berkas HTML, tanpa aset eksternal, tanpa jaringan |
| Sistem operasi | **Linux dan Windows** diuji di CI | macOS belum diuji. `serve` jalan di semua; `stop` khusus Windows (lihat bagian 5) |

Cek versi alat:

```
cargo run -- -V        # feature-view 0.2.0 (manifest peta fitur)
cargo run -- -h        # daftar perintah, flag, env, dan kode keluar
```

## 2. Bangun

```
cargo build --release
```

Lokasi biner hasil build:

| OS | Berkas |
|---|---|
| Linux / macOS | `target/release/feature-view` |
| Windows | `target\release\feature-view.exe` |

Biner ini statis terhadap crate lain dan tidak memuat DLL tambahan. Ia juga **membawa salinan
`render.js` dan `view.css` di dalamnya** (`include_str!`), tapi saat dijalankan dari checkout alat,
file itu dibaca ulang dari `src/` — jadi suntingan tampilan baru terlihat setelah `cargo build`
ulang, bukan setelah `generate` saja.

Mau dipasang sebagai perintah biasa:

```
cargo install --path .
feature-view check --root /path/ke/repo/anda
```

## 3. Direktori kerja — kesalahan yang paling sering terjadi

Alat ini mencari manifest di `docs/view/` relatif ke **direktori kerja**, kecuali Anda menyebut
`--root`. Karena itu:

```
# BENAR — dari root repo (yang menyimpan docs/view/)
cargo run --manifest-path tools/feature-view/Cargo.toml -- check

# BENAR — sebut root-nya secara eksplisit
cargo run --manifest-path tools/feature-view/Cargo.toml -- check --root .

# SALAH — dari folder alat di repo besar
cd tools/feature-view && cargo run -- check
# GAGAL membaca <crate>/docs/view/config.tsv: The system cannot find the path specified. (os error 3)
# exit 2
```

`--root` boleh menunjuk repo mana pun selama `docs/view/` ada di dalamnya. Untuk repo mandiri seperti
repo alat ini sendiri, root-nya memang folder alat.

## 4. Enam perintah

### `init` — cetak kerangka manifest

```
cargo run -- init
```

Keluaran pertama: `DIBUAT 7 kerangka manifest di docs/view:` diikuti daftar berkas. Keluaran kedua
(sudah ada isinya): `LEWATI: docs/view sudah berisi manifest, tidak ada yang ditulis (7 berkas
dipertahankan)`. Berkas yang sudah Anda isi **tidak pernah** ditimpa; kalau sebagian sudah ada,
pesannya `DIPERTAHANKAN (sudah ada): ...`. Kerangka kosong sengaja ditolak `check` (exit 1), jadi
`init` tidak bisa menyamar sebagai manifest yang jadi.

### `generate` — validasi lalu tulis halaman

```
cargo run -- generate
# OK: 17 produk, 17 aplikasi, 84 sub-fitur, 7 service, 108 relasi, 222944 byte ditulis ke docs/view/index.html
```

`docs/view/index.html` adalah hasil susun; jangan pernah diedit manual. Keluar 1 kalau manifest
ditolak, keluar 2 kalau manifest tidak terbaca.

### `check` — gerbang CI

```
cargo run -- check
# OK: docs/view/index.html sinkron (17 produk, 17 aplikasi, 84 sub-fitur, 7 service, 108 relasi)
```

`check` menjalankan `generate` tanpa menulis: validasi struktur, lalu menjajarkan halaman di disk
dengan hasil susun hari ini. Beda satu byte pun merah, dan pesannya menyebut nomor baris tempat
perbedaan pertama. Ini yang membuat "sudah di-generate" tetap benar setelah orang lain mengubah
`render.js`/`view.css`.

### `audit` — buktikan klaim tiap baris

```
cargo run -- audit
```

Membaca isi berkas yang ditunjuk kolom `bukti` dan menguji identifier yang baris itu klaim. Exit 0
kalau tidak ada baris berstatus peran **utama** yang merahnya nyata; exit 1 kalau ada
`merah-hampa` atau `merah-sempit`. Keluarannya menyebut cakupan, berkas bukti yang tidak terbaca,
bukti yang keluar folder service-nya, target yang tidak ikut tersapu, dan token yang hanya hidup di
service lain.

### `serve` — baca manifest ulang tanpa generate

```
cargo run -- serve --port 8787
# OK: serve di http://127.0.0.1:8787/
```

Rutenya:

| Rute | Tingkah |
|---|---|
| `GET /` dan `GET /index.html` | sajikan `docs/view/index.html` **dari disk**; kalau belum ada, halaman khusus "belum di-generate" dengan tombol yang memanggil `POST /regenerate` |
| `GET /stamp` | baca ulang manifest **pada tiap permintaan**, balas `{"cap":...,"valid":bool,"galat":[]}` — selalu 200 supaya manifest rusak dilaporkan, bukan dilempar |
| `POST /regenerate` / `POST /hapus` | tulis / hapus berkas hasil; hanya path konstanta internal, tidak pernah path dari permintaan |
| `GET /favicon.ico` | 204 (menjawab 404 menaruh error palsu di console) |

### `stop` — hentikan server (Windows)

```
cargo run -- stop --port 8787
```

Membaca `netstat -ano -p TCP`, menyaring PID pendengar pada port itu, memastikan namanya
`feature-view*` lewat `tasklist`, lalu `taskkill`. Kalau ada proses alat ini hidup di port **lain**,
ia menamedakannya dan keluar 1 — bukan mengaku port sudah bebas. Di Linux/macOS `stop` tidak berlaku:
hentikan dengan PID-nya sendiri (cari dari port yang mendengarkan, jangan bunuh per nama proses).

## 5. Setelan lewat environment

Nilai tidak sah atau `0` selalu jatuh ke default; tidak ada setelan yang bisa mematikan pemeriksaan.

| Env | Default | Mengatur |
|---|---|---|
| `PETA_PORT` | `8787` | port bawaan `serve`/`stop` |
| `PETA_BATAS_BACA_BUKTI` | `4000000` byte | ukuran maksimum satu berkas bukti |
| `PETA_INDEKS_BATAS_BERKAS` | `12000` | batas jumlah berkas saat menyapu folder service |
| `PETA_INDEKS_BATAS_BYTE` | `24000000` | batas total byte saat menyapu folder service |

Kalau salah satu dinaikkan, laporan `audit` menyebut nama env yang sedang berlaku, jadi angkanya
tidak terbaca sebagai fakta mentah. Di repo besar, batas sapuan itu membuat "N berkas menyebut
token ini" menjadi **batas bawah**, dan laporan mengatakannya.

## 6. Verifikasi sendiri

```
cargo test                                              # 96 tes, tanpa jaringan
cargo clippy --all-targets -- -D warnings               # harus nol
node --check public/render.js                           # hanya saat menyentuh skrip halaman
cargo run -- generate --root contoh && cargo run -- check --root contoh && cargo run -- audit --root contoh
```

Dataset [`contoh/`](contoh/README.md) adalah repo imajiner lengkap (2 service, 2 aplikasi, 2 produk,
7 sub-fitur, 6 relasi, kode palsunya) yang terbukti lulus tiga langkah itu. Ia ikut di-commit supaya
clone baru punya sesuatu untuk dibandingkan.

**Gerbang refactor murni.** `generate` bersifat deterministik: dua kali jalan menghasilkan md5 yang
sama. Sebelum dan sesudah memindahkan kode, catat `md5sum docs/view/index.html` dan `data-cap` di
chip halaman. Cap dihitung dari byte mentah manifest, jadi cap yang bergerak berarti datanya
yang berubah. Kalau Anda hanya memindah kode tapi md5 bergerak, refactor itu tidak murni.

**Verifikasi tampilan tidak bisa dilakukan dari tes.** Suite membaca `view.css`/`render.js` sebagai
teks dan menjaga skalanya (jumlah langkah huruf, tangga radius, jarak pecahan, variabel yatim, kunci
tema), tapi ia buta terhadap hasil render. Angka kerapihan harus diukur di browser. Resep yang saya
pakai, dan angka yang harus Anda dapatkan dari halaman hasil 0.2.0:

```
buka docs/view/index.html -> DevTools -> ukur:
- jumlah ukuran huruf berbeda di elemen terlihat  -> 5 (12/13/14/16/20 px), tidak ada < 12 px
- nilai border-radius yang muncul                -> 0, 4, 6, 10 px, 50% (semua token)
- gap dengan angka pecahan                        -> 0
- tinggi semua kontrol di satu baris alat         -> satu nilai (30 px)
- tinggi baris pohon                              -> satu nilai (24 px)
- rasio kontras teks terendah                     -> >= 4,5 di kedua skema
- mode Sub-fitur: kolom yang berisi lebar kartu campur -> 0
```

## 7. Masalah yang benar-benar pernah terjadi

| Gejala | Sebab | Obat |
|---|---|---|
| `GAGAL membaca .../docs/view/config.tsv ... (os error 3)`, exit 2 | dijalankan dari folder alat, bukan root repo | `--root .` atau `--manifest-path` seperti bagian 3 |
| `check` merah padahal manifest tidak disentuh | halaman basi: `render.js`/`view.css` berubah setelah `generate` terakhir | `cargo run -- generate` lalu `check` |
| `Access is denied. (os error 5)` / `failed to remove file` saat build | ada `serve` yang masih hidup memegang `target/debug/feature-view.exe` | hentikan servernya (atau build ke `--target-dir` lain + `--port` lain) |
| Sunting `view.css` tidak muncul di halaman | biner yang dipakai adalah biner lama; `cargo test` **tidak** menyegarkan `target/debug/feature-view.exe` | `cargo build` dulu, baru `generate`/`serve` |
| `check --root contoh` merah hanya di mesin Windows, hijau di Linux | `core.autocrlf=true` memberi CRLF di working tree sementara alat membandingkan byte | jangan hapus `* text=auto eol=lf` di `.gitattributes` |
| `audit` keluar 1 padahal fitur memang ada | kolom `bukti` terlalu sempit (`merah-sempit`): tokennya hidup di service itu, tapi tidak di berkas yang ditunjuk | perlebar `bukti`, jangan ubah klaimnya |
| `stop` bilang port bebas tapi server masih hidup | ia memeriksa port lain dan menamedakan pendengarnya | baca nama port yang dicetak, atau hentikan dengan PID |

## 8. Di CI

`.github/workflows/ci.yml` berjalan di `ubuntu-latest` dan `windows-latest` dengan aksi nol pihak
ketiga: pasang Rust + clippy, `cargo clippy --all-targets -- -D warnings`, `cargo test`, langkah
pencocok jumlah tes di README dengan suite, `generate`/`check`/`audit` pada `contoh/`, lalu langkah
`init` di repo kosong yang juga menuntut `check` **menolak** kerangka kosong.

## 8a. Menyunting tampilan tanpa build

Aset tampilan hidup di `public/`, di luar `src/`. Rust tidak membakar keduanya ke binari lagi, jadi
mengubah warna, tata letak, atau perilaku halaman **tidak menuntut `cargo build`**:

```
# 1. sunting public/10-token.css, public/20-komponen.css, public/30-layar.css, atau public/render.js
# 2. tulis ulang halaman memakai binari yang sudah ada
./target/debug/feature-view generate --root <root-repo>          # shell POSIX
.	arget\debugeature-view.exe generate --root <root-repo>    # PowerShell / cmd

# 3. atau, kalau serve sedang jalan, tekan tombol generate di halaman / POST /regenerate
```

Buktinya berupa angka, bukan janji: `sha256sum` binari sama sebelum dan sesudah suntingan CSS,
sementara `sha256sum` `docs/view/index.html` berubah.

Aturan yang dijaga tes, bukan ingatan:

- **CSS boleh dipecah, asal berprefix dua angka.** `css()` menggabung semua `public/*.css` menurut
  **urutan nama**, jadi `10-`, `20-`, `30-` itulah urutan cascade. Berkas tanpa prefix ditolak
  `berkas_css_publik_berprefix_angka`.
- **Skrip halaman tetap satu berkas.** `public/render.js` adalah satu IIFE; berkas `.js` kedua
  ditolak `js_hanya_satu_berkas` karena tidak akan bisa melihat fungsi di dalam IIFE itu.
- **`public/` wajib ikut kalau alat disalin** ke repo atau mesin lain. Kalau folder itu hilang,
  `generate` berhenti dengan pesan yang menyebut path — bukan halaman tanpa gaya.

## 9. Tempat yang layak dibuka kalau Anda mau mengubah perilaku

| Yang mau diubah | Mulai dari |
|---|---|
| teks bantuan `-h`, daftar perintah, petunjuk cara jalan | `petunjuk` (`src/main.rs:191`) — satu-satunya tempat teks ini ditulis, jadi pesan dan halaman tidak bisa berbeda |
| root yang dipakai tes saat alat berdiri sendiri | `root_uji` (`src/uji.rs:142`) — menerima kandidat hanya kalau manifestnya benar-benar terurai, lalu jatuh ke `contoh/` |
| aturan yang membuat `check` gagal | `src/aturan.rs` saja, plus tesnya di situ |
| kolom manifest baru | `src/data.rs` (bentuk) + parser di `src/manifest.rs` + validasi di `src/aturan.rs` |
| tampilan | `public/10-token.css` (skala dan palet), `20-komponen.css`, `30-layar.css`, dan `public/render.js` (bentuk graf) — sunting lalu `generate`, **tanpa** `cargo build` |

Kutipan `nama.rs:NNN` di halaman ini ikut diperiksa tes `sitasi_baris_di_dokumen_masih_hidup`: kalau
sebuah fungsi berpindah dan dokumen tidak ikut diperbarui, suite yang menyalakan — bukan ingatan
seseorang.

## 10. Riwayat versi

Angka tiap putaran ada di [CHANGELOG.md](CHANGELOG.md). Yang terbaru: **0.2.0** — skala tipografi,
jarak, dan radius; palet dua skema dari satu skala netral; dan saklar tema yang juga mengubah warna
graf.
