# Task 4 — Bukti tanpa-build, CHANGELOG, dan papan yang tersegel

Judul: Membuktikan bahwa menyunting CSS tidak menuntut build Rust (dua hash binari sama, dua hash halaman berbeda), lalu menulis blok CHANGELOG, menutup baris papan pekerjaan, dan mencatat di memori proyek.

Deskripsi: Klaim inti rilis ini — "gampang diubah tanpa compile Rust" — hanya berarti kalau dibuktikan dengan angka. Task ini menjalankan suntingan CSS pada salinan yang bisa dikembalikan, menjalankan binary yang sudah ada (bukan `cargo run`), dan membandingkan hash sebelum/sesudah. Setelah buktinya ada, barulah CHANGELOG ditulis, karena angka di sana harus berasal dari perintah yang sungguh dijalankan.

Tujuan Teknis: satu bukti berangka yang bisa diulang; CHANGELOG, papan pekerjaan, dan memori proyek menyebut keadaan yang sama dengan disk.

Scope: `CHANGELOG.md`, `docs/view/tugas.tsv`, `.memori.txt`. Tidak menyentuh kode; kalau pengukuran menemukan cacat, cacat itu diperbaiki dulu di task yang bersangkutan, bukan ditutupi di sini.

Langkah Implementasi:
1. Catat `sha256sum target/debug/feature-view.exe` dan `sha256sum docs/view/index.html`.
2. Tambahkan satu komentar penanda di akhir berkas CSS (mis. `/* uji tanpa-build */`) — sengaja deklarasi mati supaya tidak mengubah tampilan.
3. Jalankan binary langsung, tanpa cargo: `./target/debug/feature-view.exe generate --root .`.
4. Catat kedua hash lagi. Harapan: hash binari **sama**, hash halaman **berbeda**.
5. Kembalikan berkas CSS (hapus penandanya) dan `generate` sekali lagi; pastikan hash halaman kembali ke angka langkah 1.
6. Tulis blok "Belum dirilis" di `CHANGELOG.md`: pemindahan, pembaca baru, penghapusan baking, pemecahan CSS (kalau jadi), dokumen, dan keempat angka hash di atas.
7. Tutup baris `docs/view/tugas.tsv` untuk 0.5.0: status dan progres disamakan dengan baris `Status:` tiap berkas task, dan catatan menyebut angka ukur yang relevan.
8. Tulis satu entri baru di `.memori.txt` paling atas (newest first): pemindahan, penghapusan jalur baking, bukti hash, penjaga baru, dan konsekuensi binari tanpa `public/`.
9. `generate` kedua root (proyek dan `contoh`), lalu jalankan gerbang terakhir.

Output yang Diharapkan: bukti hash tercatat, CHANGELOG dan papan pekerjaan sinkron, memori proyek tahu ke mana aset pindah dan apa konsekuensinya.

Dependencies: task 1, task 2 (bila dijalankan), task 3.

Acceptance Criteria (dijalankan dari akar crate):

```
sha256sum target/debug/feature-view.exe
```
harapan: nilai sebelum dan sesudah suntingan CSS sama (dua kali cetak, dibandingkan).

```
sha256sum docs/view/index.html
```
harapan: nilai setelah suntingan CSS berbeda dari sebelumnya, dan kembali sama setelah penanda dihapus + `generate`.

```
cargo test
```
harapan: suite lulus; jumlahnya sama dengan angka di README (gerbang CI `[0-9]+ tes`).

```
cargo clippy --all-targets -- -D warnings
cargo run -q -- check --root .
cargo run -q -- audit --root .
```
harapan: clippy tanpa warning; `check` mencetak `sinkron`; `audit` exit 0, 0 baris kuning, 0 merah.

```
cargo run -q -- generate --root contoh && cargo run -q -- check --root contoh
```
harapan: halaman contoh ditulis ulang dan langsung sinkron (CI menjalankan urutan yang sama).

```
git status --short
```
harapan: hanya berkas yang memang disentuh rilis ini (aset, Rust, dokumen); tidak ada berkas sementara.

Estimasi: 45 menit.

Status: selesai -- bukti tercatat: hash binari `3ac2cccc…` sama sebelum dan sesudah suntingan CSS, hash halaman `6de228ef…` -> `6a0e35b0…`, dan kembali ke `6de228ef…` setelah suntingan ditarik, tanpa satu pun `cargo build` di antaranya. CHANGELOG, empat baris `tugas.tsv`, dan `.memori.txt` sudah disegel. Suite 96 lulus, clippy nol, `check` sinkron untuk root proyek dan contoh, `audit` exit 0.
