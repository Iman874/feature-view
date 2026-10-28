# Changelog

Seluruh catatan perubahan untuk `feature-view`. Format mengikuti pola
[Keep a Changelog](https://keepachangelog.com/id/1.1.0/) dan versi mengikuti
[Semantic Versioning](https://semver.org/lang/id/).

Konvensi yang berlaku di repo ini: setiap angka di bawah ini adalah hasil ukur, bukan rencana. Kalau
sesuatu belum terukur, ia masuk bagian **Belum** — tidak ditulis sebagai pencapaian.

## [Changelog]

### Belum dirilis — aset tampilan pindah ke `public/`, tanpa build untuk menyuntingnya (2026-10-28)

**Tujuan rilis ini satu: menyunting tampilan berhenti terasa seperti menyunting kode program.**
`src/` sekarang hanya Rust; CSS dan JS hidup di `public/`.

1. **Dua berkas pindah, nol byte berubah.** `src/view.css` -> `public/view.css` dan
   `src/render.js` -> `public/render.js` dipindah dengan `git mv`. Bukti murni: `sha256sum`
   `docs/view/index.html` **identik** sebelum dan sesudah pemindahan (`1901c96c…`), karena dengan
   satu berkas CSS `css()` tidak mencetak penanda apa pun.
2. **Pembaca aset baru.** `css()` menggabung semua `public/*.css` menurut **urutan nama** dengan
   penanda `/* ==== <berkas> ==== */` per berkas (penanda hanya muncul kalau berkasnya lebih dari
   satu); `js()` membaca `public/render.js` dan **menolak** berkas `.js` kedua, karena `render.js`
   satu IIFE. `aset()`, `aset_dari()`, dan kedua `include_str!` dihapus — tidak ada lagi salinan
   terbaking, jadi `public/` adalah satu-satunya sumber.
3. **Galat, bukan halaman tanpa gaya.** Aset hilang -> `generate` keluar bukan nol dengan pesan
   yang menyebut path (`public/… wajib ikut bersama alat ini`), dijaga `aset_hilang_menyebut_path`.
   Konsekuensi yang diterima: binari `cargo install` tanpa `public/` tidak lagi bisa jalan sendiri.
4. **CSS dipecah tiga berkas berprefix angka.** `10-token.css` (176 baris), `20-komponen.css` (650),
   `30-layar.css` (195); `public/render.js` tetap 2751. Urutan cascade terbaca dari nama, dan
   `berkas_css_publik_berprefix_angka` menolak nama tanpa prefix (dibuktikan: mengganti nama satu
   berkas jadi `komponen.css` membuat tes itu merah, lalu hijau lagi setelah dikembalikan).
5. **Bukti tanpa build, bukan janji.** `sha256sum target/debug/feature-view.exe` = `3ac2cccc…`
   **sebelum dan sesudah** menyunting CSS, sementara `sha256sum docs/view/index.html` berpindah
   `6de228ef…` -> `6a0e35b0…`; suntingan ditarik dan `generate` mengembalikan halaman tepat ke
   `6de228ef…`. Tidak ada `cargo build` di antara keduanya. (Angka hash itu milik binari sesi itu;
   build bersih menghasilkan hash binari lain — yang dibuktikan adalah hash yang **sama** sebelum
   dan sesudah suntingan, bukan binari yang reproducible.)
6. **Pemecahan CSS terbukti tidak mengubah apa pun.** Isi `<style>` di halaman sama persis dengan
   `src/view.css` versi lama setelah tiga penanda dan empat baris komentar kepala lapis dibuang
   (diff nol baris). Halaman tumbuh 208.298 -> 208.709 byte, seluruhnya penanda dan komentar itu.
7. **Dokumen ikut pindah.** `how_to_run.md` tidak lagi menyuruh `node --check src/render.js` (dua
   tempat) dan bagian "tampilan" tidak lagi menjanjikan `cargo build`; ada bagian baru **8a.
   Menyunting tampilan tanpa build**. README mendapat tabel `public/`, dan penjaga baru
   `tabel_publik_di_dokumen_mencocokkan_angka_sahnya` menghitung ulang angkanya (dibuktikan merah
   saat satu angka diubah satu).

**Angka akhir.** Suite 92 -> **96 tes** (satu uji baking dihapus, lima penjaga aset ditambah),
clippy nol, `check` dan `audit` exit 0 untuk root proyek dan dataset contoh.


### Belum dirilis — rel kiri bertab dan alat header jadi satu menu (2026-10-28)

**Enam perubahan, semuanya terukur di Chrome pada halaman dataset pemakai (1280/820/420px, kedua skema).**

1. **Rel kiri punya satu baris tab.** Daftar struktur dan papan pekerjaan tidak lagi bertumpuk
   vertikal: `#panel-struktur` dan `#papan` bergantian lewat tiga tab (`Produk|Pekerjaan 4|
   Selesai 17`). Tab pertama memakai nama sumbu aktif, dua berikutnya status papan dari
   `config.tsv`. Perawatannya sama dengan tab yang disetujui 0.4.3: rambut 1px di bawah baris,
   garis bawah 2px untuk terpilih. Terukur: baris 54px, kolom tetap 286px, `overflowX` false.
2. **Lima pil jumlah turun dari header ke bawah tab.** Angka service/aplikasi/produk/sub-fitur/
   relasi adalah properti rel, bukan properti merek. Terukur: `header.bar` nol pil, lima pil +
   lima ikon di `#sisi-info` pada y 110..153, tepat di bawah baris tab (y 56..110).
3. **Satu pemilih panel menggantikan dua.** `pilihSisi(nama)` memilih panel; `aturDaftarPapan(nama)`
   hanya memilih daftar di dalamnya, supaya papan yang baru dibangun tidak mencuri panel dari
   struktur. Kunci simpanan sesi berganti `papan` -> `sisi`. Target yang tabnya tidak tersedia
   jatuh ke struktur, bukan ke tab berikutnya.
4. **Lima tombol alat kanan atas jadi satu titik tiga.** Popup `#menu-alat-isi` (`role=menu`)
   berisi tema, generate ulang, hapus hasil, muat data, setel ulang — masing-masing 30px,
   berikon berlabel, rata kanan ke pemicunya, dan **membalik ke rata kiri** kalau ruang di
   kanannya tidak cukup (terukur di 820px: tanpa flip kotaknya duduk di x = -131, keluar jendela).
   Escape menutup dan mengembalikan fokus ke pemicu; ArrowDown/Up/Home/End berpindah item; klik
   di luar menutup. Segment sumbu dan dua penyaring sengaja tinggal di header.
5. **Ikon dipilih ulang dari bentuk yang dirender.** `muat-data` pindah dari `folder` ke `unduh`,
   `reset` dari `perbarui` ke `balik` (cermin `perbarui`, supaya dua aksi berbeda tidak memakai
   satu bentuk), tema dapat `rembulan`, pemicunya `menu` (tiga titik terisi). Kamus bentuk
   49 -> 52.
6. **Dua kontras di skema terang dinaikkan ke atas lantai 4,5.** Teks tab tidak terpilih 4,28 ->
   5,47 dan ikon dalam pil 4,02 -> 5,15 setelah keduanya pindah ke `--teks-baris` — aturan yang
   sama yang sudah dipakai baris pohon sejak 0.4.3, dan yang sama pula yang sempat terlewat di
   tab papan lama. Gelap: 8,77 dan 7,58.

**Kontrak tampilan yang ditulis ulang (bukan dilonggarkan).** `papan_tidak_menambah_tinggi_saat_daftar_penuh`
dulu menuntut `flex: 0 0 auto` + `max-height` angka; sekarang menuntut `flex: 1 1 auto` +
`min-height: 0` di `.papan`, `.sisi-panel`, dan `overflow: auto` di `.papan-isi` — janji yang sama
("tinggi panel tidak pernah ikut isi") dengan mekanisme yang berbeda, karena yang membatasi kini
kolomnya (`.app { height: 100dvh }`), bukan angka 168px. Terbukti di layar: tab Selesai dengan 17
baris (isi 1.618px) tetap di dalam rel 804px dan daftarnya yang menggulir. Penjaga baru
`info_jumlah_ikut_disembunyikan_saat_rail_diperkecil` menuntut `.sisi-info` dan `.sisi-tab-tombol`
ikut daftar sembunyi rail; `teks_baris_tidak_menahan_lebar_rail` kini mengikat empat selector
termasuk `.sisi-tab-tombol span`.

**Yang ditemukan pengukuran, belum diperbaiki.** Tombol `#kecil-pohon` tetap terlihat di dalam laci
rel pada lebar <= 1180px, padahal mode kecil tidak berarti di sana. Ini cacat lama dari urutan
spesifisitas (`.pohon-kecil { display: none }` kalah oleh `.tombol { display: inline-flex }` yang
ditulis belakangan), bukan bagian rilis ini, dan tidak disentuh.

**Angka akhir.** Suite alat 92 lulus (91 -> 92, satu penjaga baru), clippy nol, `check --root .`
exit 0, `audit --root .` 33 hijau-isi / 0 kuning / 0 merah. Rail kecil 286px <-> 60px, laci 320px
terbuka dan tertutup lewat Escape, `overflowX` false di 1280/820/420. Halaman root 206.595 byte,
halaman contoh 167.076 byte.


### Belum dirilis — papan pekerjaan dan lajur yang menahan isinya (2026-10-27)

**Enam perubahan, semuanya terukur di halaman dataset pemakai.**

1. **Lajur grup mengikuti isinya.** Di mode Sub-fitur lajur service digambar pada `x: 26` tetap,
   sementara `pisahkanKlaster()` boleh mendorong isi ke kiri nilai itu dan sentralisasi hanya jalan
   kalau muatannya `<= W - 60`. Terukur pada 33 sub-fitur: kartu ter-kiri duduk **-16,1 px** di luar
   border lajurnya sendiri (lajur lain `+235,6` dan `+8,7`). Sekarang tepi kiri lajur dihitung dari
   kartu paling kiri, dan ketiga lajur menjadi `+261,6`, `+9,9`, `+34,7`.
2. **`tugas.tsv`, manifest kedelapan dan opsional.** Satu baris = satu berkas task nyata:
   `id|subfitur|judul|status|progres|pemegang|berkas|catatan`. `check` menolak tugas tanpa sub-fitur
   induk, tanpa berkas, dengan progres di luar 0..=100, dengan status di luar kosakata `status_tugas`
   baru di `config.tsv`, dan dengan status yang berbeda dari kata pertama pada baris `Status:`
   berkasnya. Papan memakai tab `Berjalan`/`Selesai` (`role="tablist"`), jumlah baris di kanan,
   tab kosong disembunyikan, dan tab aktif disimpan di `sessionStorage`. Tab tersembunyi tidak bisa
   dipilih lewat klik maupun panah: pilihan jatuh ke tab yang terlihat, dan panah diabaikan bila
   hanya satu tab tampil. Tinggi papan dibatasi dan isi tab bergulir sendiri supaya halaman tidak
   memanjang.
3. **Detail per baris papan.** Setiap baris di kedua tab punya tombol `Detail` yang membuka
   pekerjaan terkait; membuka detail memilih tab sesuai status, dan pilihan detail bertahan
   setelah reload/ganti tema. Batang progres tetap beranimasi dan persen muncul pada kartu yang
   punya pekerjaan berjalan; animasi mati di `prefers-reduced-motion`.
4. **Enam cacat tampilan papan ditutup.** Inset papan disamakan dengan legenda, mode kecil
   benar-benar mengecil, judul papan menjadi 12 px, badge progres tidak lagi tertimpa cat,
   badge disembunyikan saat mode cip, dan kontras isi batang terhadap jalurnya 4,05:1 (gelap)
   / 3,73:1 (terang) setelah jalur dinaikkan ke 55%. Angka spacing/radius tetap hanya token.

5. **Tinggi papan tidak boleh ikut isi.** Panel sempat memakai `max-height: 45vh` dengan daftar
   yang tumbuh bebas: tab `Selesai` dengan 13 baris menuntut 1.048px dan mendorong blok lain keluar
   layar, sementara satu baris hanya memakai ~150px — jadi berpindah tab mengubah seluruh sidebar.
   Sekarang panel `flex: 0 0 auto` dan daftarnya yang dipatok (`max-height: calc(var(--sp-6) * 7)`
   = 168px, menggulir di dalam dirinya). Terukur pada 1.440x828: panel 248px dengan 13 baris,
   `scrollHeight` 1.048px di dalam `clientHeight` 168px, sidebar tidak meluber
   (`scrollHeight == clientHeight == 781`), dan lebarnya identik di kedua tab (287,7px). Dua hal
   lain ditutup sekaligus: judul papan yang panjang tidak lagi menahan rail (`width: 0`, dijaga tes),
   dan jejak panel detail memakai kata yang sama dengan tab (`NAMA_BERJALAN`, bukan teks tempelan).

6. **Arah tampilan Jalan A: hangat, tenang, dan legenda pindah ke tombol `i`.** Rampa `--netral-1..12`
   dihangatkan pada kedua skema tanpa mengubah satu nama token pun, dan saturasi skema terang turun
   45 -> 36. Ambang lama (warna solid tetap >= 4,5 saat dipakai sebagai teks di atas panel) diukur
   ulang dan tetap lolos: 11,27:1 gelap / 5,59:1 terang. Header 56px, sidebar 286px, panel kanan
   340px. Kartu terpilih tidak lagi menyala — `drop-shadow(0 0 9px var(--aksen))` diganti tepi
   `--teks` 1,5px + bayangan `0 2px 6px` hitam 20%. Legenda Relasi/Status keluar dari sidebar dan
   jadi popup di kanan atas kanvas (262x138px, 9 item), ditutup oleh klik tombol, klik di luar, dan
   Escape yang mengembalikan fokus ke tombol; 145px yang ia tinggalkan jatuh ke daftar struktur
   (`#pohon-isi` 440px dari 906px isi). Satu cacat ikut ketutup di jalan: baris pohon nowrap menahan
   lebar kolom, jadi sidebar menolak menyusut ke basisnya (321px terhadap 286px) sampai `width: 0`
   dipasang — penjaga `teks_baris_tidak_menahan_lebar_rail` sekarang mengikat tiga selector.
   Yang **tidak** diubah: renderer tetap SVG, font tetap lokal tanpa jaringan, tema tetap atribut
   `data-tema`, tidak ada `backdrop-filter`, dan tidak ada tes skala yang dilonggarkan.
7. **Rel daftar dirapikan mengikuti acuan yang pemilik tempel.** Item 6 baru mengubah warna; bentuk
   barisnya masih 24px dan terpilih masih sapuan hijau. Sekarang: rel memakai token peran baru
   `--rail` dan baris terpilih `--rel`, yang sengaja **berlawanan urutan di kedua skema**
   (terang netral-2 di belakang netral-1, gelap netral-1 di belakang netral-2) karena menunjuk
   `--panel` saja membuat kartu terpilih justru terlihat cekung di skema gelap. Baris pohon naik ke
   `--kontrol-tinggi` 30px, judul grup ke 34px dan jadi `position: sticky` berlatar rel, titik status
   0,45rem -> 6px, judul rel kehilangan garis bawahnya, dan kolom teks baris memakai `--teks-baris`
   (`color-mix` 80% `--teks-lembut` + 20% `--teks`) karena `--teks-lembut` murni hanya 4,28 terhadap
   rel baru — di bawah lantai 4,5. Terpilih jadi `--rel` + rambut `--garis` 1px + `--teks`, tanpa
   warna aksen. Kolom pencarian ikut naik ke `--rel`, radius `--r-lg`, tinggi 34px, dan kaca pembesarnya
   ternyata menempel di tepi atas (pusat ikon 130 terhadap pusat kolom 139) — sekarang diturunkan dari
   tinggi kolom, bukan angka telanjang. Baris disclosure yang tadinya cuma chevron dan angka telanjang
   ("3") sekarang "3 sub-fitur" lewat span `.pohon-satuan`; angkanya tetap punya sendiri supaya
   penghitungan ulang saat menyaring tidak ikut berubah. Di rail kecil penanda terpilih tetap cincin
   aksen 2px, dan rambut baris penuh sengaja dimatikan di sana supaya tidak jadi dua bingkai.
   Terukur setelahnya, lantai 4,5 lolos di semua teks rel: terang judul rel 14,69 / ringkasan 5,47 /
   baris 5,47 / judul grup min 5,21 / disclosure 5,47; gelap 15,37 / 8,77 / 8,77 / 7,33 / 8,77.
   Latar terpilih terhadap rel 1,07 (terang) dan 1,05 (gelap) — itu langkah permukaan, bukan teks, dan
   memang begitu acuan aslinya. Dua kesalahan ukur saya sendiri ikut dicatat: `getComputedStyle`
   mengembalikan `color-mix` sebagai `color(srgb 0..1)` sehingga pembaca 0..255 memuntahkan 19,37 dan
   1,14 yang tidak masuk akal, dan memindah atribut `data-tema` langsung **tidak** menjalankan kode yang
   menghitung ulang hue grup dari `--sat`/`--terang`, jadi sempat muncul kontras palsu 3,85. Yang
   pertama ditutup dengan komposit lewat canvas, yang kedua dengan menekan tombol tema sungguhan.
8. **Empat blok lain ikut disamakan, dan satu percobaan ditarik kembali.** Putaran 7 baru menyentuh
   rel; pemilik membalas dengan "tampilannya gak mirip sama sekali". Karena berkas CSS acuan tidak ada
   di disk, acuan itu **dirender sendiri** (HTML kerangka + `reset.css` supaya bingkai tombol bawaan
   browser tidak menyamar jadi gaya acuan) lalu dibandingkan kepala-kepala dengan halaman alat pada
   1280x860 dan 1536x860. Empat selisih ditutup: kotak merek 26x26 `--teks` dengan huruf pertama nama
   dari `config.merek` (dilewat sebagai `data-inisial`, dihitung dari nilai mentah sebelum `esc` supaya
   huruf pertamanya tidak bisa berupa `&`), angka header jadi pil berlatar `--panel-lembut` 26px,
   tab papan turun dari kotak berbingkai ke garis bawah 2px, dan panel inspektur jadi datar -- rambut
   di atas saja, tanpa bingkai dan tanpa radius. Satu selisih **dibiarkan**: acuan masih menyimpan
   kartu legenda Relasi/Status di sidebar, dan memindahkannya ke popup `i` adalah perintah tertulis
   owner pada putaran 6, jadi yang dimenangkan adalah perintah itu. Satu percobaan ditarik di tengah
   jalan: label sumbu ("3 produk", "33 sub-fitur", ...) ikut dipasang di pil, lalu diukur -- header
   melengkung jadi dua baris bahkan di 1536px, melanggar syarat tinggi 56px, jadi kata itu kembali
   ke atribut `title` saja. Terukur setelahnya: tinggi `.bar` 56px pada satu baris di 1536px,
   `data-inisial="M"`, kotak 26x26 radius 6, pil 26px, garis bawah tab 2px, panel `border-top` saja.
   Suite tetap 91 lulus; dua penjaga menyala di jalan dan keduanya benar -- satu radius 2px di luar
   tangga dan satu hitungan baris `halaman.rs` di README yang tertinggal 893 -> 907.

**Kosakata tetap data.** Tidak ada nama status tugas yang masuk ke kode: `status_tugas` dan
`judul_papan` dibaca dari `config.tsv` dan keduanya opsional, jadi repo yang belum memakai papan
berperilaku persis seperti sebelum perubahan ini (terukur: `check --root contoh` tetap exit 0 tanpa
`tugas.tsv`). Angka: `cargo test` 81 -> **91 lulus**, clippy nol, `node --check` lulus.

### 0.2.0 — palet, skala, dan saklar tema (2026-10-26)

**Masalahnya diukur lebih dulu**, di Chrome pada halaman contoh 1440x900, 282 elemen terlihat, angka
identik di kedua skema: 16 ukuran huruf berbeda dengan yang terkecil 10,56 px, 5 nilai
`border-radius`, 9 nilai `gap`, tiga tinggi kontrol dalam satu baris alat (27/28/30 px), tiga tinggi
baris di rail (19/24/27 px), dan tidak ada jalan memilih tema karena `GELAP` hanya dibaca dari OS.
Kontras bukan penyebabnya: terendah 5,06 (terang) dan 5,75 (gelap), nol elemen di bawah 4,5.

**Skala.** `view.css` sekarang punya tiga keluarga token: jarak 2/4/8/12/16/24 px, radius
4/6/10/999 px, tipografi enam langkah 12/13/14/16/20 px dengan lantai 12 px. Sesudah, terukur di
halaman yang sama dan di halaman repo induk (1.058 elemen terlihat, 17 node, 1280x800): 5 ukuran
huruf berbeda, radius hanya token, nol jarak pecahan, satu tinggi kontrol (30 px), satu tinggi baris
pohon (24 px), nol overflow horizontal.

**Palet.** Satu skala netral 12 langkah per skema dan hanya dua langkah teks (pola Radix Colors);
border skema gelap menjadi lapisan putih transparan (pola shadcn/ui), bukan hex baru. Status dan grup
mengikuti satu aturan hue — S/L tetap per skema dengan tiga peran (solid / teks / latar bernuansa) —
dan `render.js` membaca `--sat`/`--terang` dari CSS, jadi rotasi sudut emas untuk grup ke-9 dst tidak
lagi memakai pasangan S/L sendiri. Langkah terang sempat 38% dan diukur gagal: label aktor jatuh ke
kontras 3,95 di skema terang. Diturunkan ke 33%; sesudah: terendah 5,03 (terang) / 7,53 (gelap),
nol elemen di bawah 4,5.

**Saklar tema.** Tombol di bar alat memindahkan kedua skema termasuk warna graf (yang dihitung
JavaScript, bukan CSS), pilihannya bertahan di `localStorage`, dan dipasang oleh cangkang HTML sebelum
lukisan pertama supaya tidak berkedip. Tanpa pilihan user, perilakunya persis seperti sebelumnya: ikut
preferensi OS, termasuk saat OS berubah.

**Penjaga.** Empat tes baru di `src/aset.rs` menghitung ulang isi `view.css`/`render.js` sebagai teks:
langkah huruf, tangga radius, jarak pecahan, variabel yatim, dan kesamaan kunci tema antara cangkang
dan skrip. Giginya dibuktikan dengan tiga mutasi sekaligus — `font-size: 0.79rem` di `.bar h1` ->
merah `ukuran huruf di luar skala: 0.79rem`; definisi `--sp-6` dihapus -> merah `variabel dipakai
tanpa definisi dan tanpa cadangan: ["sp-6", "sp-6", "sp-6", "sp-6"]`; kunci tema di cangkang diganti
-> merah `cangkang HTML memakai kunci tema lain`; penjaga halaman basi ikut merah (4 gagal / 77
lulus). Ketiganya dikembalikan: 81/81 hijau, `clippy --all-targets -D warnings` nol, `check` dan
`audit` exit 0 di repo pemakai maupun `--root contoh`.

**Geometri graf.** Lebar kartu dibulatkan ke tangga 24px dan kartu sekolom ditepi-kirikan ke kolomnya
(sebelumnya keduanya berpusat sama tetapi tepi kirinya berbeda sepanjang selisih lebarnya).

**Provenance.** Halaman repo induk `d48446cb…` (214282 byte) -> `5a3345c1…` (222944 byte) dengan
`data-cap 5b3d521ee425ad20` TIDAK berubah, dan halaman contoh `9fc5f92d…` (126980) -> `190c960f…`
(135642). Cap tidak berubah karena tidak satu byte manifest pun diubah: yang berubah hanya lapisan
tampilan yang ter-bake ke halaman.

**Dokumen dipecah.** `how_to_run.md` (230 baris) sekarang memegang bagian operasional: prasyarat dan
versi toolchain, bentuk biner tiap sistem operasi, enam perintah dengan keluaran aslinya, rute dan
port `serve`, default empat env, langkah verifikasi (termasuk resep mengukur tampilan di browser),
tabel gejala-sebab-obat untuk tujuh kegagalan yang benar-benar terjadi, dan isi workflow CI. README
tinggal kemampuan dan batasnya (301 baris, dari 311 — yang hilang adalah detail cara jalan, bukan
isi). Dua penjaga dokumen (`sitasi_baris_di_dokumen_masih_hidup` dan
`tabel_modul_di_dokumen_mencocokkan_angka_sahnya`) ikut membaca berkas baru itu, jadi kutipan
`nama.rs:NNN` di `how_to_run.md` punya hak yang sama dengan yang di README: giginya dibuktikan dengan
menggeser `src/main.rs:191` menjadi `:190` -> merah dengan isi baris yang sebenarnya
(`bukan lagi assert atau definisi fn: /// Yang menjaga aturan ini: ...`), lalu hijau lagi setelah
dikembalikan.

**Belum di putaran ini:** bug CI ubuntu (run 40280461153, step "Tes") diparkir atas keputusan owner
karena nama tes yang merah hanya bisa dibaca dari anotasi GitHub dan log job butuh autentikasi;
screenshot halaman baru untuk README; dataset contoh yang lebih padat; dan `root_uji()` yang memilih
manifest repo pemakai di mesin penyusun tetapi jatuh ke `contoh/` di runner, sehingga "hijau lokal"
dan "hijau CI" bukan objek yang sama.

### Belum dirilis — putaran perburuan bug (2026-10-25)

**Dua belas defek tampilan ditutup** (`render.js` + `view.css`), semuanya diukur di Chrome lewat DOM,
bukan dibaca dari kode; angka di bawah diverifikasi ulang pada halaman hasil generate repo ini:

- "Tampilkan semua node" tadinya meninggalkan 81 dari 84 kartu ber-opacity 0,14 dengan filter status
  masih terpasang; sekarang 0 kartu redup dan `select` kembali ke "semua". Handler-nya ikut melepas
  filter service dan kotak cari, karena tanpa itu janjinya tetap bohong (kueri "printer" menyisakan
  15 dari 17 kartu redup).
- Kartu terpilih tidak lagi ikut diredup: `node terpilih` opacity 1 (sebelumnya `node redup terpilih`
  0,14, jadi panel menjelaskan kartu yang nyaris tidak terlihat).
- Edge memakai warna jenisnya sejak keadaan default: 108 garis sub-fitur menghasilkan 6 hue dan 6
  marker, 0 abu-abu (sebelumnya 38 dari 38 `rgb(93,111,128)` dan semua `panah-netral`). Atribut
  `stroke=` yang selalu dikalahkan CSS dihapus, dan jenis ke-7 di luar slot rampa jatuh ke sudut emas
  (ter probe), bukan mengulang warna.
- Token `--aktif`, `--sebagian`, `--mati`, `--rencana` akhirnya didefinisikan (`:root` + blok gelap).
  Badge status kini `rgb(61,216,157)` untuk `aktif`, bukan lagi sama dengan warna teks.
- Chip "Menampilkan N sub-fitur" menghitung baris yang cocok: `Menampilkan 3 dari 84 sub-fitur dan 7
  operasi` dengan filter `dimatikan` (sebelumnya mengklaim 27).
- Judul klaster dan grup rail dihitung ulang saat filter aktif: rail 20/17/16/11/5/8/7 menjadi
  0/0/0/1/1/1/0 untuk `dimatikan`.
- Angka pada opsi `<select>` disegerakan dari data halaman, jadi dua filter yang digabung tidak lagi
  memakai marginal yang dibaking saat generate (`Semua service · 3`, per service 0/0/0/1/1/1/0).
- Label "Muat data" sekarang jujur per keadaan: di `file://` `aria-label`-nya "Muat ulang halaman", dan
  kembali penuh hanya kalau server hidup. Dialog konfirmasi tidak lagi menyebut "Enam manifest TSV".
- Tidak ada kartu yang keluar tepi (0 dari 84; sebelumnya 5) dan 0 pill menimpa teks; 22 label dipotong
  dengan elipsis; `LEBAR_MAX` tetap 344 dan teks tetap di atas ambang 13 px (13,02 / 14,27 / 13,72).
- Legenda status menampilkan tiga yang ada datanya, bukan empat.
- Tingkat redup disamakan lewat satu variabel `--redup`, dan `.pohon-grup-tombol` serta dua `summary`
  penulis sekarang punya gaya `:focus-visible`.

**Kelas cacat yang belum tersentuh penjaga apa pun: angka di dokumen.** Dua gerbang baru menahannya —
`tabel_modul_di_dokumen_mencocokkan_angka_sahnya` (kolom Baris/Tes pada tabel modul dibaca ulang dari
berkas; saat pertama dinyalakan ia langsung menemukan tiga angka basi di dua dokumen) dan langkah CI
"Jumlah tes di README sama dengan suite" (terukur menangkap `suite=70 readme=68`, lalu lolos).

**Sebelas perbaikan kode Rust.** Tiga auditor baca-saja (kode Rust, kode tampilan, isi manifest) + satu pemeriksaan langsung ke proses
yang sedang berjalan. Semua butir di bawah punya tes baru dan merah yang terukur saat kodenya dirusak
sengaja; total tes 58 -> 71.

- **`stop` melaporkan sukses palsu untuk server di port lain.** `serve --port 8795` lalu `stop` (tanpa
  port) menjawab "tidak ada yang mendengarkan 127.0.0.1:8787, port sudah bebas" dengan exit 0, padahal
  `/stamp` masih menjawab 200. Sekarang: kalau port yang diminta sepi, alat mencari pendengar lain
  milik proses `feature-view` dan melaporkannya dengan exit 1. Terukur di mesin: `stop --port 8796`
  -> `AWAS: feature-view masih mendengarkan 8795 (PID ...)`, exit 1, curl tetap 200; `stop --port 8795`
  -> exit 0 dan curl berikutnya 000; dijalankan lagi saat benar-benar sepi -> exit 0 dengan pesan bebas.
- **Klaim berbentuk path absolut bisa membuat baris hijau tanpa bukti apa pun** — `C:/Windows/...`
  sungguh ada di mesin, jadi satu kalimat prosa yang menyebutnya cukup untuk vonis `hijau-isi` dan alat
  membaca di luar root. Sekarang path dengan drive letter, UNC, atau yang menyelinap lewat `..`
  ditolak sebagai bukti. Kontrol negatifnya justru berkas sistem Windows itu sendiri.
- **Sebaliknya, path repo yang ditulis dengan awalan `/` jadi merah palsu**: `root.join("/src/x.go")`
  di Rust memaknainya absolut sehingga berkas di dalam repo tidak pernah ditemukan. Awalan dipotong
  sebelum digabung.
- **Kalimat penutup `audit` lebih luas dari yang diperiksa.** Keluarannya `OK: tidak ada baris 'aktif'
  dengan klaim yang tidak terbukti` padahal pada run yang sama daftar KUNING menyebut baris aktif
  yang klaimnya belum penuh. Sekarang exit 0 disertai jumlah baris kuning dan tidak-teruji.
- **Baris `Cakupan:` menghitung salah.** Ia memakai jumlah `hijau-isi` semua status lalu menempelkan
  label "berstatus `aktif`"; baris berstatus separuh ikut terhitung. Sekarang dihitung per status
  (terukur: 53 baris `aktif` dari 58 hijau-isi seluruh status).
- **Laporan tidak bisa direproduksi: service pendukung lintas layanan terpilih menurut urutan acak
  HashMap.** Tiga puluh pembacaan menghasilkan dua nama service berbeda untuk token yang sama.
  Key diurutkan lebih dulu. Gigi: `kunci.sort()` dilepas -> tes merah 3 dari 3 proses, di putaran
  pertama.
- **Validasi pemetaan satu arah.** Baris `pemetaan.tsv` yang menunjuk sub-fitur yang tidak ada lolos
  `check` dan `generate` (exit 0, ringkasan tetap bilang jumlah yang benar) karena barisnya tidak pernah
  dibaca halaman. Sekarang ditolak, dengan pesan yang menyebut berkas dan nomor baris.
- **Kolom `hampa` pada tabel per-service menghitung token dari baris yang sudah dibebaskan**
  (`tidak-diuji`), menghasilkan `merah 0, hampa 1` yang membingungkan. Sekarang hanya baris yang diuji.
- **Batas byte tidak bisa dibedakan**: `PETA_BATAS_BACA_BUKTI=1000` tercetak "0 MB" dan `1500000`
  tercetak "1 MB". Sekarang memakai satu sebutan satuan yang membedakan nilai.
- **Kanvas utama berperan `img` padahal berisi kartu yang bisa diklik dan difokus** — 17 dari 84
  anak `role="button"` jadi presentational bagi pembaca layar. Peran diganti `group`; peta mini tetap
  `img` karena memang gambar tanpa anak interaktif (ada tes yang menahan kedua arah).
- **Agregat tanpa sub-fitur kehilangan labelnya sendiri.** `hitung_status` mengembalikan string kosong
  untuk produk atau aplikasi yang belum punya satu pun sub-fitur — dan validator tidak mewajibkan setiap
  produk punya sub-fitur, jadi badge-nya bisa hilang tanpa penjelasan. Sekarang memakai label peran `mati`.
  Gigi: perbaikan dikembalikan ke `String::new()` -> tepat satu tes merah
  (`agregat_tanpa_subfitur_punya_label_peran_mati`), tanpa menyinggung 67 tes lain.
- **Halaman "belum di-generate" dan pesan `check` sama-sama menyuruh perintah yang gagal di susunan
  repo induk.** Keduanya mencetak `cargo run -- generate` dengan keterangan "dari folder alat ini". Di
  repo yang menyimpan alat di subfolder, perintah itu mencari manifest di dalam folder alat sendiri lalu
  mati — terukur sendiri ketika putaran ini berjalan. Sekarang `petunjuk(perintah, root)` adalah satu
  pintu untuk kedua pesan dan ia mengulang root yang sungguh dipakai proses:
  `Jalankan: cargo run -- generate --root <folder yang memuat docs/view>`. Terukur dua kali: pada
  serve sungguhan (port 8798, halaman disembunyikan lalu dipulihkan lewat `POST /regenerate`) dan pada
  `check` yang sedang tertinggal. Penjaga `kode_alat_tidak_menyebut_letaknya_di_repo_pemakai` langsung
  menangkap komentar baru saya sendiri yang menyebut path alat, jadi teksnya diulang tanpa path.
- **Angka di dokumen tidak dikunci oleh apa pun.** README menyebut jumlah tes dan jumlah baris per modul,
  tapi tidak ada tes yang bisa menghitung jumlah tes (menghitung dari dalam tes berarti memanggil cargo
  dari dalam cargo). Sekarang CI punya langkah `Jumlah tes di README sama dengan suite`; gerbang yang
  sama dijalankan lokal dan terukur menangkap tulisannya sendiri: `suite=70 readme=68` -> gagal, lalu
  lolos setelah README disamakan. Penjaga kutipan lama terlalu longgar (ia hanya menuntut baris NNN
  memuat `assert` atau `fn`, jadi nomor yang bergeser ke `fn` LAIN lolos): tabel "Peta fungsi" kini punya
  penjaga sendiri yang mencocokkan NAMA fungsi dengan nomornya. Gigi terukur: nomor `petunjuk` dikembalikan
  ke baris lamanya -> merah dengan pesan yang menyebut berkas, nomor, dan nama fungsi yang tidak cocok.
- **Permulaan di README alat menyarankan perintah yang salah untuk susunan repo besar.** Tulisannya
  "Jalankan dari folder alat ini"; di repo induk yang menyimpan alat di subfolder, bentuk itu mencari
  `docs/view/config.tsv` di dalam folder alat dan keluar dengan exit 2. Sekarang README menyebut folder
  yang benar (yang memuat `docs/view/`) dan memberi bentuk `--manifest-path ... --root .` yang terukur
  exit 1 karena HTML tertinggal, bukan exit 2 karena path salah.
- Satu kalimat jumlah ("Tujuh tes membaca sumber alat") dibuang jadi tanpa angka: angka itu sudah salah
  saat dibaca (realitasnya sembilan, lalu sepuluh) dan tidak ada yang menjaganya.
- Vonis dataset contoh hari ini terukur ulang: `4 hijau-isi, 0 hijau-nama, 1 kuning, 0 merah, 2 tidak
  teruji` dari 7 baris (catatan v9 yang lama menyebut 3/2/2 dan tidak ikut berubah ketika `contoh/`
  diedit; angka lama dibiarkan di tempatnya sebagai riwayat, yang baru dicatat di sini).
- Teks bantuan `stop` disesuaikan dengan perilaku barunya.

### Belum dirilis — lanjutan perburuan bug (2026-10-25, sesudah empat ronde ditutup)

Empat ronde sebelumnya sudah dinyatakan selesai. Atas pertanyaan "apakah masih ada?", permukaan yang
belum pernah disentuh sendiri di ronde itu diuji ulang. Tiga cacat nyata keluar; yang lain saya catat
bersih beserta apa yang diuji.

- **Laporan `audit` tidak reproduktibel untuk satu barisnya.** Catatan "penyapuan folder service
  terpotong untuk: A, B" diambil dari iterasi `HashMap`, jadi dataset yang sama tercetak dengan dua
  urutan berbeda. Terukur delapan kali jalan pada satu dataset: empat `svc-inti, svc-muka`, empat
  `svc-muka, svc-inti`. Sekarang lewat `senarai_terbatas` yang menyortir. Gigi: `hasil.sort()`
  dihapus -> tes `senarai_terbatas_menyebut_service_terpotong_secara_terurut` merah pada lima dari
  lima panggilan (peta dibangun ulang dua belas kali per tes dengan urutan sisip dibalik).
- **Penjaga angka modul menghitung jarumnya sendiri.** `tabel_modul_di_dokumen_mencocokkan_angka_sahnya`
  menghitung tes per berkas dengan `isi.matches("#[test]").count()`, dan teks `"#[test]"` itu sendiri
  ada di dalam baris perhitungan tersebut — jadi baris `main.rs` di tabel boleh tertulis 10 padahal
  suite menjalankan 9, dan penjaganya tidak pernah bisa melihatnya. Penetapan sekarang per baris
  (baris yang `trim_start`-nya diawali `#[test]`). Kontrol negatif: angka lama 10 dipasang kembali ->
  merah dengan pesan "src/main.rs tertulis 10 tes, nyata 9". Ini kelas bug yang sama dengan jarum yang
  cocok pada dirinya sendiri; ia berumur satu putaran penuh sebelum ketahuan.
- **Halaman contoh yang ter-commit masih hasil susun `render.js` putaran lama.** Tujuh puluh satu baris
  berbeda (122.975 byte di disk terhadap 125.495 byte hasil susun hari ini), termasuk dua perbaikan yang
  sudah dinyatakan selesai: kartu yang belum menempel di dokumen tidak bisa diukur lebarnya, dan label
  tombol muat data. Efek yang terlihat orang: pada clone baru, `check --root contoh` keluar dengan exit 1
  dan janji "dataset contoh terbukti lulus generate + check + audit" menjadi salah. Halaman disegarkan.
  Lobangnya sendiri menjelaskan kenapa tidak ada yang menangkap: `contoh_lulus_validasi_dan_terbangun_jadi_halaman`
  membangun HTML di memori dan hanya memeriksa merek serta cap muncul di dalamnya — ia tidak pernah
  membandingkannya dengan halaman yang ter-commit. Tes baru
  `halaman_contoh_di_repo_computed_dari_render_js_yang_sekarang` membandingkan halaman di disk dengan
  hasil susun hari ini memakai kesamaan string yang sama dengan yang dipakai `check`. Gigi: halaman lama
  dipasang kembali -> merah menunjuk baris 1684, lalu hijau lagi sesudah disegarkan.
- **Yang diuji dan hasilnya bersih** (dicatat supaya tidak diuji ulang sebagai "masih ada"): koma di
  akhir kolom daftar dan `;;` di kolom klaim — nol efek yang terlihat, terukur dari `diff index.html`
  (hanya `data-cap` yang berubah) dan `diff` laporan `audit` yang identik, karena `split_daftar`,
  `operasi_dari`, dan `layak_token` sudah membuang segmen kosong; manifest ber-CRLF menghasilkan halaman
  identik dengan nol byte CR; `--root` yang menunjuk berkas, folder tanpa `docs/view/`, path tidak ada,
  path kosong, path ber-spasi dan non-ASCII — semuanya pesan benar dan exit 2 kecuali yang terakhir
  benar-benar jalan; halaman basi tertangkap `check` (exit 1, nomor baris, petunjuk yang bisa dijalankan);
  produk/aplikasi/service tanpa sub-fitur dan edge yang ujungnya id produk ditolak `check` dengan alasan
  yang tepat, jadi `render.js` tidak pernah menerima agregat kosong; seluruh situs `field[..]` di wilayah
  produksi didahului pemeriksaan panjang. Batas lewat env: nilai tidak sah atau 0 jatuh ke default dan
  laporan tetap mencetak batas yang berlaku, bukan yang diminta.
- Angka hari ini: tujuh puluh tiga tes hijau (naik dari tujuh puluh satu karena dua penjaga baru),
  `cargo fmt --check` 118 blok diff (naik dari 113; tetap keputusan pemilik, tidak saya jalankan),
  `cargo clippy --all-targets` nol warning, `md5 docs/view/index.html` tetap
  `2e0558c02b45b7b98aadbcd91b78fa34` dan `data-cap` tetap `5b3d521ee425ad20` — bukti tidak ada satu bait
  pun teks halaman yang berubah di putaran ini.

### Belum dirilis — deidentifikasi untuk repo publik (2026-10-25)

Folder alat ini akan jadi repo yang dilihat orang lain, jadi seluruh jejak workspace tempat alat ini
dilahirkan dikeluarkan dari isinya. Terukur sebelum-sesudah dengan saringan otomatis memakai daftar
nama yang dibaca dari manifest repo induk (57 nama: key service, stack, id aplikasi, id produk, nama
aktor): **33 baris cocok di enam berkas -> 0 baris**.

- Fixture di blok tes memakai nama generik milik dataset contoh: `svc-muka`, `svc-inti`,
  `BayarV2Controller`, `LAYAR05`, `svc-intiLain/x.go`. Yang diuji tidak berubah bentuknya — tetap
  satu service menyebut controller service lain, tetap satu bukti berawalan sama dengan folder
  service sendiri.
- Contoh nama di komentar penjaga dan di kerangka `init` diganti (`Pengguna` untuk aktor); kalimat
  yang menjelaskan KENAPA nama biasa tidak dituntut ditulis ulang tanpa mencontoh nama apa pun,
  sehingga alasannya tetap utuh dan namanya hilang.
- `README.md`: satu paragraf "lahir di workspace ..." dihapus, diganti pernyataan bahwa alat tidak
  mengenal repo tempat ia dipakai. `CHANGELOG.md`: riwayat yang menamai folder service, id produk,
  dan satu path absolut laptop pemilik repo dihapus atau dinetralkan.
- Sisa yang masih tertangkap grep dan itu memang bukan identitas: kata `cetak` (kata kerja Indonesia
  untuk "print", dipakai alat untuk mencetak laporan dan kerangka) dan satu contoh drive letter
  `C:\` di catatan bug. Keduanya dibiarkan.
- Tidak ada perilaku yang berubah: `md5 docs/view/index.html` tetap `d48446cba09a570d152c7451666db2de`,
  halaman contoh tetap `9fc5f92d1406fc20f0cf6f9ae59235a4`, `check` dan `audit` tetap exit 0 dengan
  vonis yang sama, dan crate yang disalin ke folder tanpa repo induk tetap hijau 77/77.
- **Batas penjaga yang perlu diketahui:** sapuan anti-hardcode membaca wilayah PRODUKSI dan
  menyaring lewat `layak_token()` (identifier, bukan prosa). Nama di dalam blok tes, di komentar tes,
  dan di dokumen riwayat karena itu TIDAK dijaga mesin — putaran ini dibersihkan dengan tangan dan
  grep. Kalau "nol nama host di seluruh isi crate" mau dijadikan aturan yang ditegakkan otomatis,
  itu penjaga baru dan masih keputusan pemilik.

### Belum dirilis — lisensi dan poles `.gitignore` (2026-10-26)

- **Lisensi dipasang: MIT.** Berkas `LICENSE` baru, `Cargo.toml` kini memuat `license = "MIT"`
  (kunci itu sebelumnya sengaja kosong menunggu keputusan pemilik), dan README mendapat bagian
  "Lisensi". Teks `LICENSE` dipakai apa adanya dalam bahasa Inggris karena itu teks bakunya; yang
  dijelaskan dengan kata-kata sendiri hanya di README, termasuk satu batasan yang perlu terbaca di
  depan: alat ini membuktikan klaim masih tertulis di berkas, tidak pernah membuktikan fitur bekerja.
- **Satu baris `.gitignore` ternyata mengubur janji repo ini.** Baris `/contoh/docs/view/index.html`
  membuat halaman contoh tidak ikut ter-commit. Terukur sebelum diubah: `git check-ignore -v`
  menunjuk `.gitignore:3` untuk berkas itu dan `git ls-files contoh/docs/view/` kosong. Akibatnya
  pada clone pertama: `check --root contoh` keluar dengan exit 2 ("belum di-generate") dan penjaga
  `halaman_contoh_di_repo_computed_dari_render_js_yang_sekarang` gagal karena berkas yang seharusnya
  ada di repo tidak ada. Alasan baris itu dulu ditulis — "artefak build, CI yang membangkitkan
  ulang" — persis pola yang membuat halaman ter-commit basi satu putaran penuh tanpa ada yang tahu.
  Barisnya dihapus dan alasannya sekarang tertulis di `.gitignore` supaya tidak dipasang lagi.
- `Cargo.lock` tetap dilacak, dan itu disebut eksplisit di `.gitignore`: ini biner, bukan library,
  jadi lockfile bagian dari hasil yang teruji.
- Sampah editor dan sistem operasi ditambahkan ke pengabaian (`.vscode/`, `.idea/`, `*.swp`,
  `.DS_Store`, `Thumbs.db`, `desktop.ini`); `/target` tetap.
- Setelah perubahan, `git check-ignore` tidak lagi mengenali halaman contoh sebagai terabaikan dan
  `contoh/` muncul sebagai siap dilacak. **Tidak ada `git add`, `commit`, atau `push` yang saya
  jalankan** — repo masih branch `main` tanpa commit, dan itu keputusan pemilik.
- **`.gitattributes` yang lama merusak hasil clone di Windows.** Isinya cuma `* text=auto`, dan
  simulasi clone nyata (repo uji di `%TEMP%`, `core.autocrlf=true`, commit lalu `git clone`)
  menghasilkan `contoh/docs/view/index.html` dengan **3.095 byte CR** sehingga
  `check --root contoh` keluar exit 1 pada isi yang sebenarnya identik — alat ini membandingkan
  byte, dan `cap_manifest` menghash isi mentah TSV, jadi CRLF mengubah keduanya. `.gitattributes`
  sekarang `* text=auto eol=lf` plus daftar biner. Diukur ulang dengan cara yang sama: 0 byte CR di
  `index.html`, `render.js`, dan `features.tsv`; `check --root contoh` exit 0; dan `cargo test` di
  dalam hasil clone itu hijau 77/77. README bagian "Boleh diubah, jangan diubah" kini melarang
  menghapus `eol=lf`, karena ini bukan selera melainkan syarat supaya clone orang lain bisa jalan.
- Satu klaim dokumen ikut dicabut: README lama menulis halaman contoh "artefak build (tidak ikut
  commit)" — kalimat yang sama yang membenarkan baris `.gitignore` di atas. Sekarang keduanya
  menyatakan hal yang benar dan yang benar itu dijaga tes.

### Belum (butuh keputusan pemilik repo)

- **`rust-version` belum diisi.** Alat ini terukur dibangun dan dites pada rustc 1.99.1 dengan
  edition 2021, tapi angka minimum belum diuji pada toolchain lama, jadi menuliskannya akan jadi
  klaim yang belum terbukti.
- **`cargo fmt --check` belum bersih** (terukur 113 blok beda pada 2026-10-25; 88 sesudah pekerjaan v9,
  80 sebelum — angkanya tumbuh bersama kode, jadi angka ini bukan sasaran yang tetap).
  Formatter sengaja tidak dijalankan oleh alat ini karena akan mengacak diff pekerjaan orang lain;
  jalankan sendiri: `cargo fmt -- --check` lalu `cargo fmt`.
- `stop` hanya bekerja di Windows (memakai `netstat` + `taskkill`); di luar itu ia menolak dengan
  pesan, bukan berpura-pura berhasil.
- `serve` tidak memvalidasi manifest lebih dulu (sengaja), tapi tautan bukti di panel inspektor
  mengarah ke path repo yang tidak selalu bisa dibaca server kecil ini.
- **`POST /regenerate` dan `POST /hapus` tidak memeriksa asal permintaan** (tidak ada pengecekan
  `Origin`/token). Server ini hanya mendengarkan 127.0.0.1 dan sasaran tulisnya sebuah konstanta
  (`docs/view/index.html`, selalu bisa dibangun ulang), jadi risikonya kecil — tapi `curl -X POST
  /hapus` tanpa header mana pun benar-benar menghapus halaman itu. Pemilik repo yang memutuskan
  apakah alat ini perlu penjaga asal sebelum dipakai di mesin bersama.

### Belum dirilis — putaran keenam, auditor buta (2026-10-25)

Kelimanya datang dari satu auditor yang tidak diberi tahu apa yang sudah saya perbaiki, jadi ia
membaca kode tanpa terpengaruh laporan saya. Lima temuannya saya ukur sendiri sebelum menyentuh
kode; tiga terbukti bug, satu terbukti bug dengan angka yang saya koreksi, satu saya tolak.

- **`audit` bisa membuktikan klaim dari luar repo hanya dengan satu garis miring depan.**
  `path_klaim_dalam_repo` menilai bentuk MENTAH sementara `join` memakai bentuk HASIL PEMOTONGAN
  awalan, jadi `/C:/Windows/win.ini` lolos (byte pertamanya `/`) lalu berubah menjadi
  `C:/Windows/win.ini` dan `Path::join` di Windows memaknainya absolut. Terukur pada dataset
  contoh: baris `jalan` menjadi **hijau-nama** karena berkas sistem itu ada, sementara `check`
  dan `generate` keluar dengan exit 0. Sekarang satu fungsi `potong_awalan` dipakai penjaga dan
  penggabungnya, dan `\\C:\...` ikut ditolak. Gigi: pemotongan dikembalikan ke bentuk mentah ->
  tes baru merah; hidup -> `Ringkas` berubah dari `1 hijau-nama, 0 merah` menjadi
  `0 hijau-nama, 1 merah (1 hampa)` dengan exit 1.
- **Penyapuan folder service bolong tanpa jejak.** Berkas berekstensi indeks yang gagal didekode
  dan folder yang gagal dibuka tidak menambah apa pun: bukan `berkas`, bukan `terbatas`. Klaim
  yang cuma hidup di sana divonis `merah-hampa` dengan teks "tidak disebut berkas mana pun di
  service" — tuduhan kepada baris, padahal yang tidak sampai adalah alatnya. Sekarang tercatat di
  `Indeks.bolong` dan dilaporkan satu baris CATATAN baru. Angkanya saya koreksi sendiri:
  pemindai independen saya melaporkan 13 berkas non-UTF-8, tetapi itu cakupan SELURUH folder
  service; pada root yang sungguh disapu (folder `service` tiap baris manifest, yang untuk dua
  service pertama menunjuk ke subfolder, bukan ke folder service-nya sendiri) terukur **3 dari 8.794**,
  sama persis dengan
  yang alat laporkan. Gigi: `Err(_) => indeks.bolong += 1` diubah jadi `Err(_) => {}` -> merah
  `left: 0, right: 1`.
- **Tiga teks laporan tidak cocok dengan hitungannya.** Kolom `hampa` menghitung KLAIM sedangkan
  kolom serumitnya menghitung BARIS, tanpa keterangan satuan (kini satu baris "Keterangan kolom");
  `Cakupan:` menyebut `hijau-isi` "terverifikasi penuh dari ISI berkas" padahal vonis itu cukup
  SATU token lewat isi (kini: "terbukti seluruh klaimnya dan minimal satu dari ISI berkas"); dan
  caption `hijau-nama` menulis "isi berkas tidak dibaca" padahal isinya dibaca dan tokennya memang
  tidak ada di sana. Ditambah `bukti_keluar_service`: perbandingan awalan-Teks membuat bukti
  `svc-satuLain/x.go` disangka milik folder `svc-satu`, jadi bukti yang sungguh keluar service tidak
  pernah dilaporkan. Gigi: `Path::starts_with` dikembalikan ke `str::starts_with` -> merah dengan
  `left: [], right: ["svc-intiLain/x.go"]`.
- **Satu kartu bisa menampilkan angka yang tidak ada di layar.** Kepala klaster dan angka di rail
  sudah saya ikutkan penyaring pada putaran sebelumnya, tetapi pill di kartu ditinggal: dengan
  filter `dimatikan` satu layar menunjukkan klaster "3" berisi kartu "20", dan chip menjanjikan 3.
  Pill kini memakai `subCocok` yang sama dengan chip; total tidak hilang, ia pindah ke tooltip
  kartu ("Sub-fitur: 2 (dari 2, 0 terlihat saat ini)"). Terukur di Chrome pada halaman hasil
  generate: sebelum kartu besar berpil "12" dan kartu kecil "2"; sesudah filter status pil kedua
  kartu itu "0" dan "1" dengan chip `3/17 · 3`; `filter=semua` mengembalikan "12". Console nol pesan.
- **Service bernama `constructor` mematikan halaman.** `id_aman`/`kunci_service_aman` mengizinkan
  nama itu (huruf kecil semua), dan peta grup berupa `{}` biasa: `peta["constructor"]` sudah terisi
  sebelum pernah ditulis, lalu `.push` melempar TypeError. Empat puluh peta data-keyed di `render.js`
  sekarang memakai `Object.create(null)` lewat satu helper `peta()`. Ini BUG yang saya verifikasi
  dengan fixture nyata: repo asing dengan service `constructor` saya jalankan di Chrome — sebelum
  perbaikan console berisi `Uncaught TypeError: perGrupTabrakan[n.group].push is not a function`
  dan tombol mode mati; sesudahnya tiga mode jalan (implementasi `constructor=1`, sub-fitur
  `constructor=4`) dengan console kosong. Auditor memperkirakan temuan ini "LATEN, butuh nama
  eksotis" — benar, tapi tetap satu klik dari halaman putih.
- **Satu tuduhan auditor saya tolak** dan ini perlu tercatat supaya tidak dikerjakan: ia menyebut
  `status`/`jenis_relasi` longgar karena nama ber-spasi membuat panah SVG hilang. Tidak diukur pada
  halaman nyata, dan memperketatnya berarti menolak kosakata repo lain yang hari ini diterima
  `check` — itu perubahan kontrak, bukan perbaikan bug. Tetap di daftar keputusan pemilik.
- Penjaga anti-hardcode bekerja seperti seharusnya pada putaran ini: ia menangkap komentar BARU
  saya sendiri yang menamai folder service repo induk (`audit.rs:memuat:`svc-satu``) dan komentar
  yang menyebut satu berkas `.dart` nyata, padahal komentar itu tidak perlu menamai apa pun.
  Penjaga kesegaran halaman contoh juga menangkap dirinya sendiri dua kali: setelah `render.js`
  berubah, `contoh/docs/view/index.html` kembali basi dan tesnya merah sampai di-`generate` ulang.
- Angka hari ini: tujuh puluh tujuh tes hijau (naik dari tujuh puluh tiga: empat penjaga baru),
  `cargo clippy --all-targets -- -D warnings` nol, `generate` idempoten pada
  `md5 d48446cba09a570d152c7451666db2de` (214282 byte; bergerak karena `render.js` ter-bake, cap
  manifest tetap `5b3d521ee425ad20` karena tidak satu bait manifest pun diubah), `check` + `audit`
  exit 0, crate yang disalin ke folder tanpa repo induk hijau 77/77 dan menghasilkan halaman contoh
  `9fc5f92d1406fc20f0cf6f9ae59235a4` — nilai yang sama dengan yang ter-commit.
  `cargo fmt --check` 121 blok (naik dari 118; tetap tidak saya jalankan).

## [0.1.0] - 2026-10-25

Rupa pertama yang layak dipindah ke repo sendiri: tidak ada lagi path repo induk di kode, alat bisa
dimulai dari nol di repo asing, dan seluruh tesnya jalan tanpa repo induk.

### Ditambahkan

- Perintah `init`: cetak kerangka tujuh manifest `docs/view/`. Berkas yang sudah ada tidak pernah
  ditimpa; dijalankan dua kali menghasilkan `LEWATI` (terukur: 7 berkas dibuat, lalu 0).
- Dataset contoh di `contoh/`: repo imajiner lengkap (2 service, 2 aplikasi, 2 produk, 7 sub-fitur,
  6 relasi, plus kode palsunya) dengan kosakata yang tidak ada di repo mana pun
  (`jalan`, `setengah`, `reda`, `rencana`, `kirim`, `minta`, `kelola`, `kunci`, `menuju`, `catat`).
  Ia lulus `generate` + `check` + `audit` dan menjadi fixtur tes saat repo alat berdiri sendiri.
- `-V` / `--version`, dan bantuan `-h` yang kini memuat daftar perintah, opsi, kode keluar, dan env.
- Batas yang bisa dinaikkan lewat env tanpa build ulang: `PETA_PORT`, `PETA_BATAS_BACA_BUKTI`,
  `PETA_INDEKS_BATAS_BERKAS`, `PETA_INDEKS_BATAS_BYTE`. Nilai tidak sah atau `0` jatuh ke default.
- Dokumen `README.md` (cara pakai di repo lain), `CHANGELOG.md` ini, `.github/workflows/ci.yml`
  (clippy `-D warnings`, tes, tiga perintah pada dataset contoh, dan uji `init` di repo kosong di
  ubuntu + windows), kunci metadata di `Cargo.toml`, dan komentar modul `//!` di kepala tiap berkas.
- Penjaga baru `kode_alat_tidak_menyebut_letaknya_di_repo_pemakai`,
  `sumber_alat_tidak_menyebut_nama_entitas_repo`, dan `nama_contoh_tidak_muncul_di_render_js`, plus
  `penjaga_merek_membuang_isi_kerangka_bukan_kodenya`.

### Diubah

- Penjaga tidak lagi mengutip `merek` atau kosakata sebagai literal: ia membaca daftar `statuses` dan
  `kinds` dari manifest, sehingga aturan anti-hardcode tidak melanggar dirinya sendiri.
- Aset tampilan (`view.css`, `render.js`) dibaca dari folder crate (`dir_aset()`), bukan dari
  `tools/feature-view/` milik repo induk.
- Petunjuk cara menjalankan alat tidak lagi menyebut path repo: `cargo run -- generate` + catatan
  "dari folder alat ini", karena pesan itu ter-bake ke `index.html`.
- Penemuan root untuk tes hanya menerima kandidat yang manifestnya benar-benar terurai, lalu jatuh
  ke `contoh/`. Sebelum ini, repo yang baru `init` membuat `cargo test` merah dengan tuduhan palsu.
- `bangun_html` tidak lagi menerima `root` (ia hanya butuh model + cap), dan `audit` memakai
  `Indeks::kosong()` alih-alih `expect` untuk service tanpa folder.

### Diperbaiki

- **Penjaga anti-hardcode buta terhadap `main.rs`.** Batas "kode produksi" dihitung dari
  `#[cfg(test)]` pertama, padahal `main.rs` punya `#[cfg(test)] mod uji;` di deretan deklarasi modul,
  jadi seluruh isi `main.rs` (baris 14-233, 220 dari 381 baris) tidak ikut tersapu dan penjaga tetap
  hijau. Ketahuan lewat
  mutasi: path repo yang ditanam di `petunjuk()` tidak membuat suite merah. Sekarang batasnya dicari
  dari `#[cfg(test)]` yang diikuti `mod tests` (`produksi_dari()`), dan mutasi yang sama merah di
  `kode_alat_tidak_menyebut_letaknya_di_repo_pemakai`.
- `template.rs` memuat kontrak kolom yang salah: tertulis "pisahkan banyak nilai dengan `;`" padahal
  kolom daftar (`bukti`, `dokumen`) dipisah **koma**. Diperbaiki, dan contoh di kerangka ikut dibetulkan.
- Tiga pesan masih menyebut "enam manifest" padahal `config.tsv` sudah menjadi yang ketujuh.
- `cap_manifest_stabil_dan_berupa_hex` punya penemuan root sendiri yang `panic` di repo mandiri;
  sekarang memakai `root_uji()` yang sama dengan penjaga lain.
- Satu komentar di `audit.rs` masih mencontoh path service repo induk (`svc-satu/cmd/...`) dan tidak
  ada tes yang menangkapnya — ditemukan lewat sapuan manual, lalu dijadikan penjaga
  `sumber_alat_tidak_menyebut_nama_entitas_repo`. Mutasi M7 (komentar itu dipasang kembali) merah dengan
  `audit.rs:memuat:`svc-satu``; komentar itu lalu dipakai sebagai contoh nama generik. Sengaja hanya
  nama berbentuk identifier yang dituntut: kata Indonesia biasa — termasuk yang kebetulan dipakai
  sebagai nama aktor atau id produk — tidak, dan
  alat ini boleh berkomentar dalam bahasa itu.
- **Kutipan baris di dokumen provenance busuk dan tidak ada yang tahu.** Menyisipkan satu tes baru di
  `main.rs` menggeser tiga kutipan `main.rs:NNN` yang dipakai ledger mutasi; dokumen tetap mengklaim
  angka yang salah dan suite tetap hijau. Diperbaiki dua arah: ledger beralih ke **nama tes** (nomor
  baris hanya untuk definisi fungsi), dan dua penjaga baru menahannya —
  `sitasi_baris_di_dokumen_masih_hidup` (setiap `nama.rs:NNN` di README/CHANGELOG alat dan dokumen
  provenance harus jatuh di baris `assert` atau `fn`) dan
  `tiap_penjaga_anti_hardcode_didokumentasikan` (setiap penjaga harus disebut di README alat, jadi
  tes yang diganti nama tidak bisa hilang diam-diam dari dokumen). Taring yang pertama terukur:
  kutipan ke baris tempat `fn audit` didefinisikan digeser ke nomor lain yang bukan `assert` maupun
  definisi `fn` -> penjaga merah dengan pesan yang menyebut berkas dan nomor palsu itu, lalu hijau
  lagi setelah kutipan dikembalikan.
- **Penjaga dokumen yang baru lahir sendiri adalah no-op di repo mandiri.** Versi pertama hanya membaca
  `docs/view/README.md` repo induk, jadi di repo alat yang berdiri sendiri ia lolos tanpa memeriksa apa
  pun. Sekarang README dan CHANGELOG milik alat ikut dibaca (dan wajib ada) sebelum dokumen induk
  dipakai.
- Satu warning clippy (`needless_borrows_for_generic_args`) di tes `init`.

### Angka yang diukur pada hari itu (riwayat v9)

- `cargo test`: 58 passed, 0 failed (repo induk DAN salinan mandiri). `cargo clippy --all-targets -- -D warnings`: nol warning.
- Dataset contoh: `2 produk, 2 aplikasi, 7 sub-fitur, 2 service, 6 relasi`; vonis
  `3 hijau-isi, 0 hijau-nama, 2 kuning, 0 merah, 2 tidak-diuji`; `audit` keluar 0.
- Peta fitur di repo induk tidak berubah isinya: cap manifest tetap `871468511f006f4c`, `84 sub-fitur`,
  `108 relasi`; `md5 index.html` bergerak `6697682a06de0073d680ab9c14087576` →
  `5f48427326ceac9a138069a7a35ee5fa` karena teks petunjuk di `render.js` berubah (200790 → 200814
  byte), bukan karena data berubah.
- Alat diuji berdiri sendiri: folder crate disalin ke direktori tanpa `docs/view/` induk, lalu
  `cargo test` hijau pada tiga keadaan (tanpa manifest, sesudah `init`, dan sesudah `init` dengan
  manifest diisi tetapi `merek` masih placeholder).

### Catatan pengembangan sebelum 0.1.0

Alat ini tumbuh di satu workspace monorepo dalam tiga gelombang: `v7` memindahkan merek, kosakata, dan
ikon ke data (`config.tsv` + kolom `ikon` divalidasi terhadap kamus bentuk), `v8` memecah satu berkas
3.659 baris menjadi sembilan modul dengan output byte-identik sebagai gerbangnya, dan `v9`
(pekerjaan di atas) yang membuatnya benar-benar bisa pindah repo. Rincian provenance tiap angka di
halaman ada di `docs/view/README.md` repo induk.
