//! Bentuk data peta: satu per satu entri manifest, kosakata dari config, dan peran status.
//! Modul ini tidak membaca berkas dan tidak mengenal HTML.


/// Kosakata data yang dulu kerasukan di kode: daftar status, daftar jenis relasi, pemetaan aktor ke
/// nama bentuk ikon, dan merek halaman. Semuanya dibaca dari config.tsv.
#[derive(Clone, Debug)]
pub(crate) struct Config {
    pub(crate) merek: String,
    pub(crate) judul: String,
    pub(crate) bahasa: String,
    pub(crate) statuses: Vec<String>,
    pub(crate) kinds: Vec<String>,
    pub(crate) aktor_ikon: Vec<(String, String)>,
    pub(crate) minimum_fitur: usize,
    pub(crate) peran: [(&'static str, String); 3],
}

impl Config {
    pub(crate) fn status_peran(&self, peran: &str) -> &str {
        self.peran
            .iter()
            .find(|(k, _)| *k == peran)
            .map(|(_, v)| v.as_str())
            .unwrap_or("")
    }
}

/// Peran sebuah status bagi perhitungan jumlah: `utama` (berfungsi penuh), `separuh` (belum penuh),
/// `mati` (dimatikan). Status tanpa peran dianggap `belum` (belum dibuat).
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Peran {
    Utama,
    Separuh,
    Mati,
    Belum,
}

pub(crate) fn peran_status(status: &str, config: &Config) -> Peran {
    if status == config.status_peran("utama") {
        Peran::Utama
    } else if status == config.status_peran("separuh") {
        Peran::Separuh
    } else if status == config.status_peran("mati") {
        Peran::Mati
    } else {
        Peran::Belum
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Service {
    pub(crate) key: String,
    pub(crate) name: String,
    pub(crate) stack: String,
    pub(crate) description: String,
    pub(crate) folder: String,
    pub(crate) docs: Vec<String>,
    pub(crate) ikon: String,
}

#[derive(Clone, Debug)]
pub(crate) struct Product {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) actor: String,
    pub(crate) outcome: String,
    pub(crate) summary: String,
    pub(crate) docs: Vec<String>,
    pub(crate) notes: String,
    pub(crate) line: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct App {
    pub(crate) id: String,
    pub(crate) service: String,
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) folder: String,
    pub(crate) description: String,
    pub(crate) docs: Vec<String>,
    pub(crate) ikon: String,
    pub(crate) line: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct Mapping {
    pub(crate) subfeature: String,
    pub(crate) product: String,
    pub(crate) app: String,
    pub(crate) line: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct Feature {
    pub(crate) id: String,
    pub(crate) product: String,
    pub(crate) app: String,
    pub(crate) service: String,
    pub(crate) name: String,
    pub(crate) status: String,
    pub(crate) summary: String,
    pub(crate) how: String,
    pub(crate) entry: String,
    pub(crate) evidence: Vec<String>,
    pub(crate) docs: Vec<String>,
    pub(crate) notes: String,
    pub(crate) line: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct Edge {
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) kind: String,
    pub(crate) label: String,
    pub(crate) line: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct AggEdge {
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) kind: String,
    pub(crate) label: String,
}

pub(crate) struct Model {
    pub(crate) config: Config,
    pub(crate) services: Vec<Service>,
    pub(crate) products: Vec<Product>,
    pub(crate) apps: Vec<App>,
    pub(crate) mappings: Vec<Mapping>,
    pub(crate) features: Vec<Feature>,
    pub(crate) edges: Vec<Edge>,
}

#[derive(Debug)]
pub(crate) enum LoadError {
    Manifest(Vec<String>),
    Io(String),
}
