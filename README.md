# feature-view

**Versi 0.2.0.** Rust tanpa satu pun dependency, keluarannya satu berkas HTML yang dibuka tanpa
server dan tanpa aset eksternal.

Peta fitur monorepo yang dibangun dari delapan manifest TSV: satu perintah memvalidasi, satu perintah
lagi membuktikan klaim barisnya terhadap kode yang sebenarnya, dan hasilnya satu halaman yang bisa
dibuka tanpa server, tanpa framework, dan tanpa satu pun dependency Rust.

Alat ini tidak mengenal repo tempat ia dipakai: tidak ada satu pun nama service, aplikasi, produk,
status, atau jenis relasi yang hidup di kodenya — semuanya datang dari manifest. Pindah repo = salin
folder ini, jalankan `init`. Yang dijaga oleh tes, bukan oleh janji — lihat bagian
[Penjaga anti-hardcode](#7-penjaga-anti-hardcode).

**Alat ini bukan pencari fitur.** Ia tidak membaca kode sumber untuk menemukan fitur, tidak menjalankan
aplikasi, tidak menyentuh database. Baris manifest adalah hasil kurasi orang (atau agen) yang membaca
kode dulu; alat ini yang menjaga supaya kurasi itu tidak basi.

## 1. Mulai di repo Anda

```
cargo run -- init          # cetak kerangka docs/view/ (berkas yang sudah ada tidak pernah ditimpa)
$EDITOR docs/view/*.tsv    # isi dari kode yang benar-benar Anda baca
cargo run -- generate      # validasi + tulis docs/view/index.html
cargo run -- check         # validasi + pastikan HTML masih sinkron (pakai di CI)
cargo run -- audit         # cocokkan klaim tiap baris terhadap isi kode
cargo run -- serve         # baca manifest ulang pada tiap permintaan, tanpa generate
```

Sisanya pindah ke **[how_to_run.md](how_to_run.md)**: prasyarat dan versi toolchain, bentuk biner tiap
sistem operasi, rute dan port `serve`, batas yang bisa dinaikkan lewat env, langkah verifikasi, dan
tujuh gejala error yang sudah punya sebab serta obat. Yang harus diingat dari halaman ini hanya satu:
alat ini mencari `docs/view/` relatif ke **direktori kerja**, jadi jalankan dari root repo atau sebut
`--root <path-ke-root-repo>`. Di repo besar yang menaruh alat di subfolder sementara `docs/view/`
tetap di root repo, menjalankan dari folder alat gagal membaca `docs/view/config.tsv` (exit 2).
Bentuk yang benar di susunan itu:

```
cargo run --manifest-path tools/feature-view/Cargo.toml -- check --root .
```

Ingin melihat bentuk jadi sebelum menyentuh repo Anda? Dataset [`contoh/`](contoh/README.md) adalah
repo imajiner lengkap (2 service, 2 aplikasi, 2 produk, 7 sub-fitur, kode palsunya) yang sudah
dibuktikan `generate` + `check` + `audit` lulus — jalankan ketiganya dengan `--root contoh`.

## 2. Perintah dan kode keluar

| Perintah | Kerja | Keluar 0 | Keluar 1 | Keluar 2 |
|---|---|---|---|---|
| `init` | tulis kerangka delapan manifest | selalu, termasuk saat tidak ada yang ditulis (`LEWATI`) | - | gagal I/O |
| `generate` | validasi manifest lalu tulis `docs/view/index.html` | halaman tertulis | manifest ditolak validasi | manifest tidak terbaca / argumen salah |
| `check` | validasi + bandingkan HTML dengan yang seharusnya | HTML sinkron | HTML tertinggal atau manifest ditolak | manifest tidak terbaca |
| `audit` | baca berkas bukti, cocokkan token klaim | tidak ada baris peran utama yang merah | ada baris peran utama `merah-hampa` / `merah-sempit` | manifest tidak terbaca |
| `serve` | HTTP kecil di `127.0.0.1`, manifest dibaca ulang tiap permintaan | - (menunggu) | - | port terpakai |
| `stop` | hentikan pendengar di port ini (khusus Windows) | port bebas dan tidak ada proses alat ini di port lain, atau minimal satu PID berhasil dihentikan | port yang diminta sepi tapi alat ini masih memegang port lain (dinamedakan di layar), atau ada pendengar dan tidak satu pun berhasil dihentikan | `netstat`/`tasklist`/`taskkill` tidak bisa dijalankan |

`check` tidak memeriksa kebenaran perilaku; ia memeriksa manifest konsisten, pemetaan lengkap dua arah
(setiap sub-fitur dipetakan, dan setiap baris pemetaan menunjuk sub-fitur yang benar-benar ada), dan
path bukti/dokumen benar-benar ada. `audit` selangkah lebih jauh: ia membaca isi berkas yang
ditunjuk `bukti` dan membuktikan bahwa identifier yang baris itu klaim memang disebut di sana.
**Keduanya tidak membuktikan fitur bekerja.**

## 3. Ketujuh manifest

Semuanya di `docs/view/`, format `|`-separated, baris `#` diabaikan. Kolom wajib persis seperti di
bawah ini (kelebihan kolom = ditolak dengan pesan yang menyebut nomor baris).

| Berkas | Kolom |
|---|---|
| `config.tsv` | `kunci\|nilai` — lihat tabel kunci di bawah |
| `services.tsv` | `key\|nama\|stack\|deskripsi\|folder\|dokumen\|ikon` |
| `produk.tsv` | `id\|nama\|aktor\|hasil\|ringkasan\|dokumen\|catatan` |
| `applications.tsv` | `id\|service\|nama\|jenis\|folder\|deskripsi\|dokumen\|ikon` |
| `features.tsv` | `id\|service\|fitur\|status\|ringkasan\|cara_kerja\|titik_masuk\|bukti\|dokumen\|catatan` |
| `pemetaan.tsv` | `subfitur\|produk\|aplikasi` |
| `edges.tsv` | `dari\|ke\|jenis\|label` |
| `tugas.tsv` | `id\|subfitur\|judul\|status\|progres\|pemegang\|berkas\|catatan` — **opsional**: tidak ada berkas = papan pekerjaan tidak dipakai, halaman tidak berubah |

`config.tsv` adalah tempat identitas halaman berada. Kunci yang dikenali parser:

| Kunci | Artinya |
|---|---|
| `merek`, `judul`, `bahasa` | label yang muncul di halaman |
| `status` | daftar status yang boleh ditulis, sekaligus urutan penyaring di halaman |
| `status_peran_utama` / `_separuh` / `_mati` | status mana yang dianggap "sudah hidup", "setengah", "dimatikan" |
| `jenis_relasi` | daftar jenis edge yang boleh dipakai |
| `aktor_ikon` | `Nama:ikon` untuk tiap aktor produk |
| `minimum_fitur_per_service` | ambang bawah jumlah sub-fitur per service |

Peran status bukan hiasan: peran **utama** yang klaimnya tidak terbukti membuat `audit` keluar 1;
peran **separuh** hanya kuning; peran **mati** dan status sisanya tidak diuji sama sekali.

**Dua pemisah yang berbeda di baris yang sama** — ini yang paling sering salah ketik:

- kolom daftar (`dokumen`, `bukti`) dipisah **koma**: `a.go,b.go`
- kolom klaim (`titik_masuk`, `ringkasan`, `cara_kerja`) boleh memakai **titik koma** karena `audit`
  memecahnya menjadi klaim-klaim kecil: `GET /hello; HelloController`

Menulis `a.go,b.go` di `titik_masuk` tidak salah secara validasi, tapi keduanya akan diperlakukan
sebagai satu frasa; menulis `a.go,b.go` dengan titik koma di `bukti` membuat satu path panjang yang
tidak ada, dan `check` langsung menolaknya.

## 4. Aturan yang ditegakkan `check`

- id unik dan hanya huruf kecil, angka, tanda hubung; nama fitur tidak boleh duplikat dalam service
- setiap sub-fitur dipetakan **tepat satu kali** di `pemetaan.tsv`, dan aplikasinya wajib satu service
- produk dan aplikasi tidak boleh kosong sub-fitur; satu aplikasi boleh melayani banyak produk
- setiap sub-fitur punya minimal satu edge; ujung edge harus sub-fitur yang dikenal
- folder service/aplikasi dan semua path `bukti`/`dokumen` harus ada (relatif `--root`)
- nilai `ikon` (service, aplikasi, aktor) harus anggota kamus bentuk di `render.js` — salah ketik
  ikon tertangkap di `check`, bukan tampil kosong di halaman
- status hanya di `features.tsv`; status produk dan aplikasi **dihitung** dari sub-fiturnya
- baris berstatus peran separuh/mati wajib punya `catatan`
- jumlah service dibatasi (graf terlalu lebar tidak lagi terbaca), dan tiap service wajib punya
  `minimum_fitur_per_service` sub-fitur

## 5. Apa yang `audit` membuktikan

Tiga kolom (`titik_masuk`, `ringkasan`, `cara_kerja`) dipecah menjadi token identitas: identifier
camelCase, `snake_case`, path berkas, route, dan akronim. Prosa biasa ("menampilkan", "lalu",
"dengan") dibuang, supaya alat tidak mengaku memeriksa kalimat.

Tiap token lalu dicari lewat tiga pintu, berurutan dari yang paling kuat: isi berkas yang ditunjuk
`bukti`, lalu nama berkas bukti, lalu keberadaan identifier itu di seluruh folder service barisnya.

| Vonis | Arti |
|---|---|
| `hijau-isi` | semua klaim terbukti dan minimal satu dari **isi** berkas |
| `hijau-nama` | semua klaim terbukti, tapi hanya karena **nama berkas** bukti mengandungnya |
| `kuning` | sebagian klaim terbukti |
| `kuning-hampa` | baris separuh yang klaimnya tidak ada di mana pun dalam service |
| `merah-sempit` | baris peran utama yang nol klaimnya terbukti dari berkas yang ditunjuk, padahal tokennya ada di service — kolom `bukti`-nya yang terlalu sempit |
| `merah-hampa` | baris peran utama dengan klaim yang tidak disebut berkas mana pun dalam service-nya |
| `tidak-diuji` | ketiga kolom murni prosa, atau statusnya memang tidak dituntut cocok |

Yang dilaporkan juga, supaya angkanya tidak dibaca naif: rasio cakupan, berkas bukti yang tidak
terbaca (hilang/terlalu besar), bukti yang menunjuk keluar folder service-nya sendiri, dan token yang
hanya ada di service **lain** (`KLAIM LINTAS SERVICE` — tidak bikin keluar 1; yang perlu diperbaiki
adalah kolom bukti baris itu). Kalau penyapuan folder service terpotong batas, angkanya disebut
sebagai batas bawah, bukan angka pasti.

## 6. Batas yang bisa dinaikkan tanpa build ulang

Empat env — `PETA_PORT`, `PETA_BATAS_BACA_BUKTI`, `PETA_INDEKS_BATAS_BERKAS`, `PETA_INDEKS_BATAS_BYTE`
— beserta default angkanya ada di [how_to_run.md](how_to_run.md) bagian 5. Yang penting untuk bagian
"apa yang alat ini boleh disetel": nilai tidak sah atau `0` selalu jatuh kembali ke default, dan
**tidak ada satu pun setelan yang bisa mematikan pemeriksaan**. Kalau sebuah env dipakai, laporan
`audit` menyebut namanya, jadi angka tidak dibaca sebagai fakta mentah. `-h` mencetak daftar ini
beserta kode keluar; `-V` mencetak versi.

## 7. Penjaga anti-hardcode

Beberapa tes membaca sumber alat (dan dokumen provenance) itu sendiri, jadi aturannya berlaku terus tanpa
ada yang ingat-ingat:

- `sumber_alat_tidak_memuat_merek_dari_config` — merek dan seluruh kosakata `config.tsv` (status dan
  jenis relasi) tidak boleh muncul sebagai literal di kode produksi modul mana pun; sekaligus menjaga
  supaya satu item top-level hanya hidup di satu berkas (menyalin `fn` ke dua modul akan merah).
- `render_js_tidak_mengenal_nama_data_repo` — tidak ada key service, stack, id aplikasi, id produk,
  nama aktor, atau kunci kamus bentuk yang sama dengan salah satu nama data repo.
- `kode_alat_tidak_menyebut_letaknya_di_repo_pemakai` — tidak ada pesan (Rust, `render.js`, `view.css`)
  yang menyebut path alat di repo ini; petunjuk cara jalan harus tetap benar setelah alat dipindah.
- `sumber_alat_tidak_menyebut_nama_entitas_repo` — nama entitas yang berbentuk identifier (`svc-satu`,
  `Node.js`, `TypeScript`) tidak boleh muncul di sumber alat mana pun, **termasuk di komentar**. Aktor
  Aktor dan id produk yang berupa kata Indonesia biasa sengaja tidak dituntut, dan
  menyuruh alat berkomentar tanpa mereka berarti melarang bahasanya. Yang dipakai sebagai saringan adalah
  `layak_token()` — definisi "identifier, bukan prosa" yang sama dengan mesin `audit`.
- `nama_contoh_tidak_muncul_di_render_js` — aturan yang sama dijalankan terhadap dataset contoh,
  sehingga alat yang berdiri sendiri tetap punya penjaga yang nyata.
- `bentuk_terdaftar_membaca_kamus_ikon_dari_sumber_js` — kamus bentuk harus benar-benar terbaca dari
  `render.js` (48 nama). Kalau parsing kamus rusak, validasi ikon di `check` berubah jadi no-op dan
  tes inilah yang menyalakannya.
- `penjaga_merek_membuang_isi_kerangka_bukan_kodenya` — memastikan potongan kerangka `init` benar-benar
  dibuang dari sapuan merek, dan bahwa literal yang ditanam di luar kerangka tetap tertangkap.
- `sitasi_baris_di_dokumen_provenance_masih_hidup` — kalau dokumen provenance repo induk ada, setiap
  kutipan `nama.rs:NNN` di dalamnya harus jatuh di baris yang memuat `assert` atau `fn`. Angka baris
  bergeser diam-diam setiap kali ada tes baru disisipkan; kelas kegagalan ini benar-benar terjadi dan
  tes inilah yang sekarang menahannya.
- `tiap_penjaga_anti_hardcode_didokumentasikan` — setiap tes yang namanya memakai awalan penjaga
  (`sumber_alat_`, `render_js_`, `kode_alat_`, `nama_contoh_`, `bentuk_terdaftar_`, `penjaga_`) harus
  disebut di dokumen provenance. Menjaga renamed/hilang, bukan hanya angka.

Dataset contoh sengaja memakai kosakata asing (`jalan`, `setengah`, `reda`, `rencana`, `kirim`,
`minta`) supaya kalau masih ada satu saja nama yang tertinggal di kode, ia tertangkap di sana.

## 8. Struktur kode

| Berkas | Tanggung jawab | Baris | Tes |
|---|---|---|---|
| `src/main.rs` | titik masuk: argumen, pemilihan perintah, bantuan, petunjuk, penjaga struktur | 739 | 9 |
| `src/data.rs` | bentuk data: entri manifest, kosakata config, peran status, papan pekerjaan | 166 | 0 |
| `src/manifest.rs` | baca +urai delapan TSV jadi `Model`, cap isi manifest | 834 | 10 |
| `src/aturan.rs` | aturan validasi yang membuat `check` gagal, termasuk papan pekerjaan | 788 | 13 |
| `src/halaman.rs` | perangkai HTML, JSON untuk JS, perhitungan status | 886 | 8 |
| `src/audit.rs` | mesin audit: token, tiga pintu bukti, vonis, laporan | 1158 | 20 |
| `src/saji.rs` | server HTTP kecil + `stop` | 483 | 8 |
| `src/aset.rs` | baca `view.css`/`render.js` dari folder sumber, kamus bentuk, penjaga skala tampilan | 368 | 11 |
| `src/setelan.rs` | batas dan default yang bisa di-override lewat env | 72 | 3 |
| `src/template.rs` | kerangka `init` + tes dataset contoh | 364 | 9 |
| `src/uji.rs` | pabrik objek tes + root uji (repo pemakai, jatuh ke `contoh/`) + pemotong wilayah produksi | 231 | 0 |
| `src/render.js` | bentuk grafik + interaksi halaman (48 bentuk ikon) | - | - |
| `src/view.css` | gaya halaman | - | - |

Alur satu kali jalan: `muat_model` → `validate` → (`tulis_index` | `audit` | `sajikan`).

Semua item memakai `pub(crate)`, bukan `pub`: ini biner, bukan library, jadi tidak ada API publik
yang harus dijaga stabil. Kalau alat ini nanti Anda jadikan library, `pub(crate)` adalah daftar
tempat yang harus dipertimbangkan ulang satu per satu.

**Mau menambah apa?** Kolom manifest baru → `data.rs` + parser di `manifest.rs` + validasi di
`aturan.rs`. Aturan validasi baru → `aturan.rs` saja, plus tesnya di situ. Bagian halaman baru →
`halaman.rs` (data) dan `render.js`/`view.css` (tampilan), lalu `generate`. Kata yang tampil di
halaman → hampir selalu berarti ia seharusnya jadi data di `config.tsv`, bukan teks di kode.

**Peta fungsi yang sering dicari.** Nomor di bawah ini dijaga oleh tes
`sitasi_baris_di_dokumen_masih_hidup`; kalau sebuah fungsi berpindah dan dokumen tidak ikut diperbarui,
suite yang menyalakan — bukan ingatan saya.

| Fungsi | Tempat | Yang perlu diketahui |
|---|---|---|
| `produksi_dari` | `src/uji.rs:186` | memotong kode produksi pada `#[cfg(test)]` yang diikuti `mod tests`, bukan yang pertama |
| `tanpa_kerangka` | `src/uji.rs:162` | membuang isi `const TEMPLATE_*` dari sapuan merek/kosakata |
| `root_uji` | `src/uji.rs:142` | hanya menerima manifest yang benar-benar terurai, lalu jatuh ke `contoh/` |
| `petunjuk` | `src/main.rs:191` | satu-satunya tempat teks cara menjalankan alat ditulis |
| `klaim_baris` | `src/audit.rs:177` | menggabungkan tiga kolom klaim beserta asal kolomnya |
| `sokong_lintas` | `src/audit.rs:327` | mengenali token yang hidup di service lain tanpa membuat exit 1 |
| `indeks_service` | `src/audit.rs:251` | menyapu folder service; mencatat target yang tidak bisa didekode |
| `bukti_keluar_service` | `src/audit.rs:377` | bukti di luar folder service, dibandingkan per komponen path |
| `path_klaim_dalam_repo` | `src/audit.rs:745` | menolak drive letter, UNC, dan `..` — termasuk yang bersembunyi di balik awalan `/` |

## 9. Tes

`cargo test` sekarang **91 tes**, tanpa jaringan dan tanpa satu pun berkas di luar folder alat ini
ditulis permanen (tes `init` memakai folder temporer). `cargo clippy --all-targets -- -D warnings`
harus nol: penjaga format kode ini adalah suite-nya sendiri. Perintah verifikasi lengkap — termasuk
gerbang md5 untuk refactor murni dan resep mengukur tampilan di browser — ada di
[how_to_run.md](how_to_run.md) bagian 6.

Sebelas tes di `src/aset.rs` membaca ulang `view.css`, `render.js`, dan kamus bentuk ikon sebagai
teks lalu menghitung angkanya — langkah huruf (maksimal enam, semuanya `var(--fs-`), tangga
radius, jarak pecahan, variabel yang dipakai tanpa definisi, kunci tema yang harus sama antara
cangkang HTML dan skrip, dua daftar `display: none` yang harus tetap memuat blok yang ikut hilang
saat rail diperkecil atau kartu dikecilkan jadi cip, `width: 0` pada judul papan yang menahan rail,
langit-langit tinggi daftar papan, dan setiap bentuk ikon yang diklaim manifest harus ada di
kamus. Alasannya persis
sama dengan penjaga dokumen: kerapihan yang hanya dibuktikan mata akan melonggar oleh satu baris
yang tidak dilihat siapa pun.

**Gerbang untuk refactor murni:** sebelum dan sesudah, jalankan `generate` pada dataset Anda lalu
bandingkan `md5 index.html` dan `data-cap`. Cap isi manifest tidak berubah kalau data tidak berubah;
kalau HTML berubah padahal Anda hanya memindahkan kode, refactor itu tidak murni. Perubahan yang
sengaja menyentuh tampilan (misalnya pesan bantuan di `render.js`) memang mengubah md5 — catat angka
barunya, jangan diam-diam.

`cargo fmt --check` belum bersih di repo ini dan formatter sengaja tidak dijalankan alat: lihat
`CHANGELOG.md` bagian "Belum".

## 10. Boleh diubah, jangan diubah

- `docs/view/index.html` hasil generate — jangan pernah diedit manual.
- `contoh/` adalah fixtur sekaligus demo; yang boleh diubah hanya TSV dan kode palsunya.
  `contoh/docs/view/index.html` **ikut di-commit** dan dijaga tetap segar oleh
  `halaman_contoh_di_repo_computed_dari_render_js_yang_sekarang`: tanpanya, clone baru tidak punya
  halaman untuk dibandingkan dan `check --root contoh` langsung exit 2. Jangan masukkan dia kembali
  ke `.gitignore` dengan alasan "artefak build" — itu persis alasan yang membuat halaman ter-commit
  basi satu putaran tanpa ada yang tahu.
- `.gitattributes` harus menjaga `eol=lf`. Alat ini membandingkan byte (`check` menjajarkan halaman
  di disk dengan hasil susun, `cap_manifest` menghash isi mentah TSV, `render.js`/`view.css`
  ter-bake apa adanya ke halaman). Dengan `* text=auto` saja, peng-clone ber-`core.autocrlf=true`
  menerima CRLF: terukur 3.095 byte CR di `index.html` dan `check --root contoh` exit 1 pada isi
  yang identik. `* text=auto eol=lf` membuat hasil clone tidak bergantung konfigurasi orang.
- Angka-angka presentasi (48 nama bentuk ikon, rampa posisi `--grup-1..12`, `--status-1..4`,
  `--relasi-1..6`) satu-satunya yang boleh berupa hardcode: itu keputusan gaya, bukan data repo.
  JS membaca rampa sampai token habis lalu melanjutkan dengan hue sudut emas, jadi jumlah service
  tidak dibatasi daftar tetap. Hue kurasi itu memakai S/L yang sama dengan rotasi sudut emas karena
  `render.js` membaca `--sat`/`--terang` dari CSS: menambah atau menggeser satu langkah warna cukup
  di satu tempat, tidak di dua daftar yang harus diingat sama.
- Skema terang/gelap dipilih lewat atribut `data-tema` pada `<html>`, bukan `@media
  (prefers-color-scheme)`. Alasannya bukan selera: warna graf dihitung JavaScript dari keadaan yang
  sama, dan dua sumber kebenaran menghasilkan chrome terang dengan node gelap. Cangkang HTML
  memasang atribut itu sebelum lukisan pertama memakai kunci `fv-tema` yang harus sama dengan
  `KUNCI_TEMA` di `render.js` — kalau salah satu berganti, tes
  `kunci_tema_sama_di_cangkang_dan_di_skrip` merah.
- Skala tampilan (jarak, radius, tipografi) tidak boleh dilewati nilai telanjang. Penjaganya di
  `src/aset.rs`; lihat bagian 9.

## 11. Ide yang dipinjam dari repo serupa

Standar yang diadopsi supaya alat ini layak jadi repo sendiri: `#[derive]` + `pub(crate)` ketat untuk
biner, modul `//!` di kepala tiap berkas, `clippy --all-targets` sebagai bagian gerbang, dataset
contoh yang ikut repo sebagai fixtur CI (pola yang sama dipakai alat-alat yang harus bisa dites
"di repo orang"), `CHANGELOG.md` bermotif Keep a Changelog, dan workflow CI yang hanya butuh
`cargo`. Yang belum diadopsi dan sengaja ditunda ada di `CHANGELOG.md`.

## 12. Lisensi

MIT — lihat [LICENSE](LICENSE). Boleh dipakai, diubah, digabung, dijual, dan dipasang di repo privat
tanpa perlu izin tambahan; yang wajib hanya menyimpan pesan hak cipta itu bersama salinannya. Tidak
ada jaminan apa pun: alat ini membuktikan bahwa sebuah klaim masih tertulis di berkas yang ditunjuk,
tidak pernah membuktikan bahwa sebuah fitur bekerja.

Berkas `LICENSE` memakai teks resmi MIT berbahasa Inggris. Teks lisensi ditulis apa adanya karena
itosnya yang punya makna hukum; menerjemahkannya berubah jadi membuat lisensi sendiri.

`Cargo.toml` memuat `license = "MIT"` sehingga `cargo publish` dan pemeriksa lisensi membaca nilai
yang sama dengan berkasnya.
