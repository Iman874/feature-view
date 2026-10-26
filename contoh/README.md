# Repo Contoh

Dataset contoh selengkap mungkin untuk `feature-view`: satu repo imajiner dengan dua service, dua
aplikasi, dua produk, dan tujuh sub-fitur. Fungsinya ada tiga:

1. **Bukti bahwa alat ini tidak mengenal repo induknya.** Semua label, status, dan jenis relasi di sini
   sengaja memakai kata yang berbeda dari repo mana pun (`jalan`, `setengah`, `reda`, `rencana`, `kirim`).
   Kalau masih ada nama yang tertinggal di kode Rust atau `render.js`, dataset ini yang akan menolaknya.
2. **Fixtur untuk tes dan CI.** `cargo test` berjalan cukup dari folder alat ini, tanpa membutuhkan repo
   induk yang punya `docs/view/`.
3. **Resep mulai pakai.** Untuk mencobanya dari folder alat:

   ```
   cargo run -- generate --root contoh
   cargo run -- check    --root contoh
   cargo run -- audit    --root contoh
   cargo run -- serve    --root contoh
   ```

   Hasil generate ditulis ke `contoh/docs/view/index.html`; berkas itu artefak build, bukan sumber.
   Untuk repo Anda sendiri, jangan menyalin isi folder ini — jalankan `init` lalu isi manifestnya
   dari kode yang benar-benar dibaca.

Isinya dibuat agar `audit` menghasilkan campuran vonis, bukan semua hijau: baris `setengah`
memang punya klaim yang tidak ada di berkas service-nya, supaya perbedaan `hijau-isi`, `kuning`,
dan `merah` terlihat nyata pada satu dataset kecil.
