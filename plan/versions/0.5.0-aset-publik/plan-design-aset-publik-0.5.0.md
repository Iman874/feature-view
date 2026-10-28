# Plan Design — Aset tampilan pindah ke `public/` (0.5.0)

Area: alat `tools/feature-view` — klon permukaan ini sendiri (`src/aset.rs` pembaca aset, `src/halaman.rs`/`src/saji.rs`/`src/main.rs` pemanggil, folder baru `public/`, penjaga di `src/aset.rs`) dan dokumennya. Lapis Partnership dan lapis Monobox tidak tersentuh: nol migrasi, nol endpoint, nol layar produk.

Catatan lokasi: plan dan task rilis ini hidup di klon alat (`tools/feature-view/plan/versions/`), bukan di `backend/plan/` — pekerjaannya melekat pada permukaan ini, dan folder `plan/` di klon ini terlacak git sedangkan `backend/plan/` justru diabaikan.

Catatan riwayat: plan `0.5.0-palet-berwarna` (warna kartu/rel mengikuti acuan layar) dibatalkan owner pada 2026-10-28 sebelum satu baris kode ditulis. Nomor 0.5.0 dipakai ulang karena rilis itu tidak pernah terjadi.

## Latar Belakang

Dua berkas tampilan hidup di dalam `src/` bersama 9.764 baris Rust: `src/view.css` (1.017 baris) dan `src/render.js` (2.751 baris). Keduanya dibakar ke binari lewat `include_str!`, lalu `aset()` membaca disk lebih dulu dan jatuh ke salinan terbaking kalau berkasnya tidak ada. Susunan itu bekerja, tapi menyimpan empat ongkos:

1. **Menyunting tampilan berarti masuk ke folder kode.** `src/` seharusnya hanya Rust; orang yang hanya mau mengubah warna harus tahu bahwa `view.css` bukan kode program.
2. **Dua sumber kebenaran.** Isi yang tertanam di binari dan isi di disk bisa berbeda tanpa ada yang tahu. Yang menang adalah disk, jadi salinan basi di binari tidak pernah terlihat sampai alat dipasang di mesin tanpa folder sumber.
3. **Satu berkas besar tanpa cara memecahnya.** 1.017 baris CSS memaksa setiap perubahan membaca satu berkas penuh.
4. **Menambah berkas tampilan menuntut perubahan Rust.** Berkas baru tidak akan pernah ikut kalau tidak ada yang menambah `include_str!`, dan tidak ada tes yang mengingatkan.

Owner meminta aset tampilan jadi bagian aplikasi di `public/`: gampang disunting, dihapus, dan diatur, **tanpa compile Rust lagi**.

## Tujuan (terukur)

- `src/` tidak memuat satu pun berkas tampilan; `public/view.css` dan `public/render.js` jadi satu-satunya sumber.
- Menyunting CSS tidak menuntut build Rust: hash `target/debug/feature-view.exe` **sama** sebelum dan sesudah suntingan, sementara `docs/view/index.html` hasil `generate` **berubah**.
- Pemindahan murni: `docs/view/index.html` setelah `generate` **byte-identik** dengan sebelum pemindahan.
- Aset hilang menghasilkan galat yang menyebut path dan exit bukan nol — bukan halaman tanpa gaya dan bukan panic.
- CSS boleh dipecah menjadi beberapa `public/*.css` yang digabung menurut urutan nama; `render.js` tetap satu berkas, dan penjaga menolak `.js` kedua.
- Gerbang lama tetap hijau: `cargo test` (baseline 92), clippy nol, `check`/`audit` exit 0.

## Scope

**Dikerjakan**
- `public/view.css`, `public/render.js` — dipindah dengan `git mv`, isi tidak diubah pada task 1.
- `src/aset.rs` — `dir_src()` dan `dir_publik()`; pembaca `css()` (gabung semua `*.css` urut nama) dan `js()` (tunggal); `aset()`/`aset_dari()`/`include_str!` dihapus; galat menyebut path.
- `src/halaman.rs`, `src/saji.rs`, `src/main.rs` — memakai pembaca baru; galat aset diteruskan sebagai pesan keluar + exit bukan nol.
- Penjaga baru di `src/aset.rs`: `css_digabung_urut_nama`, `js_hanya_satu_berkas`, `aset_hilang_menyebut_path`, `tabel_publik_mencocokkan_angka`.
- Dokumen: `README.md`, `how_to_run.md`, `CHANGELOG.md`, `.gitattributes`, `docs/view/tugas.tsv`, `.memori.txt`.

**Tidak dikerjakan**
- Tidak mengubah rupa, tata letak, atau perilaku halaman. Buktinya byte-identik.
- Tidak memindahkan CSS halaman "belum di-generate" (`halaman_belum_generate`): halaman darurat itu harus hidup justru ketika aset hilang.
- Tidak mengubah bentuk keluaran: CSS dan JS tetap **ditanam** ke `index.html`, bukan berkas terpisah (alasan di Design Teknis).
- Tidak menyentuh plan lama di `plan/versions/0.2.0-*` (dokumen sejarah).

## Breakdown Task

| Task | Hasil (1 hasil, 1-4 jam) | Perkiraan | Berkas |
|---|---|---|---|
| 1 | Dua berkas pindah; aset dibaca dari `public/`; galat menyebut path; halaman byte-identik | 1,5 jam | `task1-pindah-aset.md` |
| 2 | CSS dipecah tiga berkas berprefix angka + penjaga urutan | 1 jam | `task2-pecah-css.md` |
| 3 | Dokumen petunjuk tidak lagi menunjuk `src/`, plus tabel `public/` di README | 1 jam | `task3-dokumen.md` |
| 4 | Bukti tanpa-build (dua hash) + papan dan memori tersegel | 45 menit | `task4-bukti-dan-segel.md` |

Urutan: 1 -> 2 -> 3 -> 4. Task 2 hanya dijalankan kalau owner menyetujui CSS dipecah (keputusan 1 di bawah); kalau tidak, task 3 dan 4 tetap jalan.

## Design Teknis

**`public/` di akar crate, bukan `docs/view/`.** Folder ini milik alat, bukan milik repo pemakai. `docs/view/` adalah data hasil kurasi pemakai; menaruh CSS/JS di sana berarti menyalin aset ke setiap repo yang memakai alat ini dan `check` ikut menjaganya. `public/` ikut ter-commit di repo alat, jadi setiap checkout — termasuk CI dan repo Monoframe — selalu punya sumbernya.

**Pembaca aset: satu fungsi per jenis, galat menyebut path.**

```
dir_publik()                     -> <crate>/public
css() -> Result<String, String>  -> semua *.css, urut nama, digabung dengan penanda
js()  -> Result<String, String>  -> public/render.js saja
```

Penggabungan CSS memakai pemisah `/* ==== <nama berkas> ==== */` supaya pembaca `index.html` tahu dari berkas mana satu aturan datang. Urutannya **nama berkas**, bukan waktu ubah: hasil `generate` harus sama di dua mesin, dan `check` membandingkan byte.

`js()` menolak `public/*.js` kedua. Bukan selera: `render.js` satu IIFE, dan dua berkas yang digabung berbagi satu `<script>` sehingga berkas kedua tidak bisa melihat satu pun fungsi di dalam IIFE berkas pertama. Menolak lebih jujur daripada membiarkan kode mati.

**`include_str!` dihapus beserta jalur "baking".** Aset sekarang bagian aplikasi, jadi tidak ada salinan kedua di binari. Konsekuensi yang diterima dan ditulis di `how_to_run.md`: binari `cargo install` **wajib** punya `public/` di sebelah crate; kalau tidak, alat berhenti dengan pesan yang menyebut path. Pertukaran ini disengaja: satu sumber kebenaran lebih berharga daripada bisa jalan tanpa folder aset, karena alat ini selalu dipakai dari checkout (CI membuktikannya lewat `generate --root contoh`).

**Keluaran tetap satu berkas.** `index.html` tetap memuat `<style>`/`<script>` hasil gabungan, bukan `<link>`/`<script src>`. Tiga alasan: (a) `check` membandingkan satu berkas byte-per-byte, sedangkan aset terpisah menuntut langkah salin dan penjaga kebasian baru; (b) halaman harus bisa dibuka dari `file://` dengan satu berkas; (c) `saji.rs` hanya menyajikan `index.html` dari disk, jadi aset terpisah menuntut rute statis baru. Yang diminta owner adalah **sumbernya** yang gampang diatur, bukan cara peramban mengambilnya.

**Pemecahan nama untuk uji.** `dir_aset()` sekarang berarti dua hal: sumber Rust (`src/`) dan aset tampilan (`public/`). Dipecah menjadi `dir_src()` dan `dir_publik()`; setiap pemanggil menyebut yang benar. Uji `aset_ambil_dari_disk_dan_jatuh_ke_baking` dihapus (perilaku baking tidak ada) dan diganti empat penjaga baru, jadi jumlah tes tidak turun tanpa jejak.

**Urutan kerja task 1 yang aman.** Pindahkan berkas (`git mv`, isi apa adanya) -> ubah pembaca -> `cargo test` -> `generate --root .` -> bandingkan hash `index.html` dengan salinan sebelum pemindahan. Satu byte berbeda berarti pemindahannya belum murni.

## Ketersediaan Data

Rilis ini tidak menambah, membaca, atau menampilkan satu field pun yang baru. Daftar periksa `PRD.md` bagian 8 tetap berlaku dan jawabannya nol untuk rilis ini:

| Kelompok `PRD.md` 8 | Apakah rilis ini menambah? |
|---|---|
| A. Diinput | Tidak. Manifest TSV yang sudah ada tidak diubah satu baris pun. |
| B. Diimpor | Tidak. |
| C. Diturunkan | Tidak. Tidak ada agregat baru; `render_json` tidak disentuh. |
| D. Butuh penghubung | Tidak ada, dan tidak ada tile baru yang menuntutnya. |

Yang berpindah hanya lokasi dua berkas statis (CSS dan JS) di disk. Isi halaman, angka, status, dan label tetap datang dari `docs/view/*.tsv` lewat jalur yang sudah ada.

## Dampak ke Sistem

- **Risiko utama: binari tanpa `public/` berhenti bekerja.** Dulu salinan terbaking menyelamatkannya. Mitigasi: galat menyebut path, `README.md` dan `how_to_run.md` menyebut `public/` wajib ikut, dan `aset_hilang_menyebut_path` menegaskan pesannya.
- **Risiko kedua: urutan gabungan CSS mengubah hasil tanpa disadari.** Cascade bergantung urutan; prefix angka (`10-token.css`, `20-komponen.css`, `30-layar.css`) membuat urutan terbaca dari nama. `check` menangkap perubahan byte halaman, `css_digabung_urut_nama` menangkap berkas tanpa prefix dan urutan yang berubah.
- **Risiko ketiga: dokumen menyebut path lama.** `how_to_run.md` baris 12, 164, dan 220 menyebut `src/render.js`/`src/view.css`; semuanya diperbarui di task 3. Plan lama di `plan/versions/0.2.0-*` **tidak** disentuh karena itu dokumen sejarah.
- **CI:** tidak ada langkah baru. `cargo test`, `generate --root contoh`, `check`, `audit` tetap jalan di checkout bersih karena `public/` ikut ter-commit. Gerbang "jumlah tes di README sama dengan suite" tetap menagih angka baru.
- **Monoframe:** nol. Tidak ada berkas, database, atau API Monoframe yang dibaca atau dituju; alat ini tetap bisa disalin ke repo mana pun asalkan `public/` ikut.
- **Angka dokumen bergerak:** jumlah baris `aset.rs`, jumlah tes `aset.rs`, tabel modul README. Dihitung ulang paling akhir (task 4).

## Keputusan owner (sudah diambil 2026-10-28)

1. **CSS dipecah tiga berkas berprefix angka** — disetujui; dijalankan sebagai task 2 (`10-token.css`, `20-komponen.css`, `30-layar.css`).
2. **Nama folder `public/`** — dipakai, sesuai kata owner.

## Definition of Done

- `src/` nol berkas tampilan; `public/` memuat minimal `view.css` dan `render.js`.
- `docs/view/index.html` byte-identik dengan sebelum pemindahan; kedua hash disebut di laporan.
- Bukti tanpa-build: dua hash binari sama, dua hash halaman berbeda, keduanya disebut.
- Aset hilang -> `generate` keluar bukan nol dengan pesan menyebut path (diuji, bukan diklaim).
- `cargo test` hijau (jumlah disebut), clippy nol, `check` dan `audit` exit 0.
- `README.md`, `how_to_run.md`, `CHANGELOG.md`, `docs/view/tugas.tsv`, `.memori.txt` sinkron; nol rujukan `src/view.css`/`src/render.js` di dokumen petunjuk.
- Perubahan kode direview owner sebelum dianggap selesai.
