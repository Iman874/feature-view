//! Pabrik objek untuk tes: Model, Service, Feature, Edge dan root uji. Hanya ikut dikompilasi saat `cargo test`.

use std::fs;
use std::path::PathBuf;
use crate::aset::*;
use crate::data::*;
use crate::manifest::*;

pub(crate) fn config_uji() -> Config {
    Config {
        merek: "Repo Uji".to_string(),
        judul: "Peta Repo Uji".to_string(),
        bahasa: "id".to_string(),
        statuses: vec![
            "aktif".to_string(),
            "sebagian".to_string(),
            "dimatikan".to_string(),
            "direncanakan".to_string(),
        ],
        kinds: vec!["alur".to_string(), "panggil".to_string()],
        aktor_ikon: vec![("Pengguna".to_string(), "pengguna".to_string())],
        minimum_fitur: 2,
        peran: [
            ("utama", "aktif".to_string()),
            ("separuh", "sebagian".to_string()),
            ("mati", "dimatikan".to_string()),
        ],
    }
}

pub(crate) fn service(kunci: &str) -> Service {
    Service {
        key: kunci.to_string(),
        name: format!("Nama {kunci}"),
        stack: "Test".to_string(),
        description: "Deskripsi".to_string(),
        folder: "docs".to_string(),
        docs: vec![],
        ikon: "layanan".to_string(),
    }
}

pub(crate) fn produk() -> Product {
    Product {
        id: "p".to_string(),
        name: "Produk".to_string(),
        actor: "Pengguna".to_string(),
        outcome: "Hasil".to_string(),
        summary: "Ringkasan".to_string(),
        docs: vec![],
        notes: String::new(),
        line: 2,
    }
}

pub(crate) fn aplikasi() -> App {
    App {
        id: "a".to_string(),
        service: "app".to_string(),
        name: "Aplikasi".to_string(),
        kind: "aplikasi".to_string(),
        folder: "docs".to_string(),
        description: "Deskripsi".to_string(),
        docs: vec![],
        ikon: "aplikasi".to_string(),
        line: 2,
    }
}

pub(crate) fn model_uji(features: Vec<Feature>, edges: Vec<Edge>) -> Model {
    let mappings = features
        .iter()
        .map(|f| Mapping {
            subfeature: f.id.clone(),
            product: "p".to_string(),
            app: "a".to_string(),
            line: 2,
        })
        .collect();
    Model {
        config: config_uji(),
        services: vec![service("app")],
        products: vec![produk()],
        apps: vec![aplikasi()],
        mappings,
        features,
        edges,
    }
}

pub(crate) fn fitur(id: &str, service: &str, status: &str, bukti: &str) -> Feature {
    Feature {
        id: id.to_string(),
        product: "p".to_string(),
        app: "a".to_string(),
        service: service.to_string(),
        name: format!("Fitur {id}"),
        status: status.to_string(),
        summary: "Ringkasan".to_string(),
        how: "Cara kerja".to_string(),
        entry: "Titik masuk".to_string(),
        evidence: vec![bukti.to_string()],
        docs: vec![],
        notes: if status == "aktif" {
            String::new()
        } else {
            "Catatan".to_string()
        },
        line: 2,
    }
}

pub(crate) fn edge(dari: &str, ke: &str) -> Edge {
    Edge {
        from: dari.to_string(),
        to: ke.to_string(),
        kind: "alur".to_string(),
        label: "label".to_string(),
        line: 2,
    }
}

/// Dataset contoh milik alat ini, di dalam folder crate: `contoh/docs/view/*.tsv` beserta kode palsunya.
/// Karena letaknya di samping sumber, ia ikut terbawa saat alat ini menjadi repo mandiri.
pub(crate) fn root_contoh() -> PathBuf {
    let crate_dir = dir_aset()
        .parent()
        .expect("sumber alat harus hidup di dalam folder crate")
        .to_path_buf();
    crate_dir.join("contoh")
}

/// Root repo untuk tes: `cargo test` berjalan dari direktori paket, jadi manifest induk dicari dua
/// kandidat. Yang diterima hanya kandidat yang manifestnya benar-benar terurai: repo yang baru
/// menjalankan `init` punya `docs/view/` berisi kerangka kosong, dan kalau itu yang dipilih, penjaga
/// di bawahnya akan menuduh kerangka milik alat ini sebagai data orang lain. Kalau tidak ada kandidat
/// yang sah — keadaan repo mandiri yang belum punya manifest — tes jatuh ke dataset contoh milik alat,
/// sehingga penjaga tetap memeriksa sesuatu yang nyata alih-alih mati diam-diam.
pub(crate) fn root_uji() -> PathBuf {
    for kandidat in [PathBuf::from("."), PathBuf::from("../..")] {
        if kandidat.join(CONFIG_FILE).is_file() && muat_model(&kandidat).is_ok() {
            return kandidat;
        }
    }
    let contoh = root_contoh();
    assert!(
        muat_model(&contoh).is_ok(),
        "tidak ada manifest sah pada kandidat ./ dan ../.., dan dataset contoh {} pun tidak bisa \
         diurai — penjaga tidak boleh lolos diam-diam",
        contoh.display()
    );
    contoh
}

/// Buang isi konstanta `TEMPLATE_*` dari sebuah sumber. Isi kerangka `init` memang memuat nama
/// placeholder (`merek|Repo Saya`); itu data yang sengaja dicetak alat, bukan alat yang mengenal nama
/// di repo orang. Tanpa potongan ini, orang yang mengisi manifest tapi meninggalkan placeholder merek
/// akan mendapat tuduhan palsu dari penjaga merek.
pub(crate) fn tanpa_kerangka(isi: &str) -> String {
    let mut out = String::new();
    let mut membuang = false;
    for baris in isi.lines() {
        if baris.starts_with("pub(crate) const TEMPLATE_") {
            membuang = true;
        }
        if membuang {
            if baris.trim_end().ends_with("\";") {
                membuang = false;
            }
            continue;
        }
        out.push_str(baris);
        out.push('\n');
    }
    out
}

/// Kode produksi sebuah modul: isinya sampai blok tesnya dimulai. Titik potongnya dicari dari pola
/// `#[cfg(test)]` yang diikuti `mod tests`, BUKAN dari `#[cfg(test)]` pertama. Di `main.rs` ada
/// `#[cfg(test)] mod uji;` di deretan deklarasi modul, dan memotong di sana membuat seluruh isi
/// `main.rs` lolos dari sapuan penjaga — terukur 2026-10-25: mutasi yang menanam
/// `tools/feature-view` di `petunjuk()` tetap hijau.
pub(crate) fn produksi_dari(isi: &str) -> &str {
    const PENANDA: &str = "#[cfg(test)]";
    let mut dari = 0usize;
    while let Some(relatif) = isi[dari..].find(PENANDA) {
        let posisi = dari + relatif;
        let baris = isi[posisi + PENANDA.len()..]
            .trim_start()
            .lines()
            .next()
            .unwrap_or("")
            .trim();
        if baris.starts_with("mod tests") {
            return &isi[..posisi];
        }
        dari = posisi + PENANDA.len();
    }
    isi
}

/// Semua berkas sumber alat yang ikut dijaga, terurut nama, sebagai (nama berkas, isi). `uji.rs` sengaja
/// dikecualikan: ia berisi fixture berbahasa Indonesia, jadi menyapunya akan membuat penjaga merek gagal
/// oleh datanya sendiri. Sumber dibaca dari folder crate (`dir_aset`), bukan dari bentuk repo pemakai.
/// Dipakai tes penjaga yang harus menyapu SELURUH modul — kalau penjaga hanya membaca satu berkas, memecah
/// kode diam-diam melonggarkan aturannya.
pub(crate) fn sumber_alat() -> Vec<(String, String)> {
    let folder = dir_aset();
    let mut daftar: Vec<String> = fs::read_dir(&folder)
        .expect("folder sumber alat harus ada")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.ends_with(".rs") && n != "uji.rs")
        .collect();
    assert!(
        daftar.len() >= 8,
        "sumber alat harus terdiri dari modul-modulnya, dapat {} berkas: {daftar:?}",
        daftar.len()
    );
    daftar.sort();
    daftar
        .into_iter()
        .map(|n| {
            let isi = fs::read_to_string(folder.join(&n)).expect("sumber modul terbaca");
            (n, isi)
        })
        .collect()
}
