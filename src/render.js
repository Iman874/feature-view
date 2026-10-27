(function () {
  var data = JSON.parse(document.getElementById('data').textContent);
  // Hanya halaman yang disajikan lewat HTTP yang boleh membaca manifest ulang.
  // Dari file:// browser melarang fetch ke TSV tetangga, jadi jalur ini mati sengaja.
  var dilayani = location.protocol === 'http:' || location.protocol === 'https:';
  var KUNCI_SIMPAN = 'fv-tampilan';
  // Kunci terpisah dari tampilan: tema harus bertahan lintas sesi dan lintas tab,
  // sedangkan posisi kamera sengaja hanya hidup di tab yang sedang dipakai.
  var KUNCI_TEMA = 'fv-tema';
  var MODES = ['produk', 'subfitur', 'implementasi'];
  var svg = document.getElementById('jejaring');
  var panel = document.getElementById('detail');
  var cari = document.getElementById('cari');
  var hasil = document.getElementById('hasil');
  var pohon = document.getElementById('pohon');
  var tombolPohon = document.getElementById('tombol-pohon');
  var tutupPohon = document.getElementById('tutup-pohon');
  var filterStatus = document.getElementById('filter-status');
  var filterService = document.getElementById('filter-service');
  var pohonIsi = document.getElementById('pohon-isi');
  var pohonJudul = document.getElementById('pohon-judul');
  var minimap = document.getElementById('minimap');
  var zoomTingkat = document.getElementById('zoom-tingkat');
  var NS = 'http://www.w3.org/2000/svg';
  var W = 1500;
  var H = 1050;
  var DUNIA = { produk: [1500, 1050], subfitur: [2600, 1600], implementasi: [1500, 1050] };
  var TINGGI_NODE = 58;
  var LEBAR_MIN = 196;
  var LEBAR_MAX = 344;
  // --- papan pekerjaan (tugas.tsv, semuanya opsional) ---
  // Status TERAKHIR kosakata dianggap selesai; yang lain dianggap berjalan dan dapat tempat di
  // papan. Repo tanpa `tugas.tsv` tidak merender apa pun: daftarnya kosong dan judulnya tidak
  // ada, jadi halaman lama tetap persis seperti sebelumnya.
  var STATUS_SELESAI = (data.statusTugas && data.statusTugas.length)
    ? data.statusTugas[data.statusTugas.length - 1]
    : '';
  var PROGRES_AKTIF = {};
  (data.tugas || []).forEach(function (t) {
    if (t.status === STATUS_SELESAI) return;
    if (typeof PROGRES_AKTIF[t.subfitur] === 'undefined' || t.progres > PROGRES_AKTIF[t.subfitur]) {
      PROGRES_AKTIF[t.subfitur] = t.progres;
    }
  });
  var papan = document.getElementById('papan');
  var papanJudul = document.getElementById('papan-judul');
  var papanIsi = document.getElementById('papan-isi');

  function bangunPapan() {
    if (!papan || !data.tugas || !data.tugas.length || !data.papanJudul) return;
    papanJudul.textContent = data.papanJudul;
    papanIsi.textContent = '';
    var selesai = 0;
    data.tugas.forEach(function (t) {
      if (t.status === STATUS_SELESAI) {
        selesai += 1;
        return;
      }
      var baris = document.createElement('div');
      baris.className = 'papan-baris';
      baris.setAttribute('data-status', t.status);
      var kepala = document.createElement('div');
      kepala.className = 'papan-kepala';
      var nama = document.createElement('span');
      nama.className = 'papan-nama';
      nama.textContent = t.judul;
      var angka = document.createElement('b');
      angka.className = 'papan-angka';
      angka.textContent = t.progres + '%';
      kepala.appendChild(nama);
      kepala.appendChild(angka);
      var rel = document.createElement('div');
      rel.className = 'papan-rel';
      rel.textContent = t.status + ' - ' + t.pemegang;
      var jalur = document.createElement('div');
      jalur.className = 'papan-jalur';
      var batang = document.createElement('div');
      batang.className = 'papan-batang';
      batang.style.width = t.progres + '%';
      jalur.appendChild(batang);
      var rumah = document.createElement('div');
      rumah.className = 'papan-rumah';
      rumah.textContent = t.subfitur;
      baris.appendChild(kepala);
      baris.appendChild(rel);
      baris.appendChild(jalur);
      baris.appendChild(rumah);
      papanIsi.appendChild(baris);
    });
    if (selesai) {
      var hitung = document.createElement('p');
      hitung.className = 'papan-hitung';
      hitung.textContent = selesai + ' ' + STATUS_SELESAI;
      papanIsi.appendChild(hitung);
    }
    papan.hidden = false;
  }
  var SKALA_MIN = 0.62;
  // Di bawah skala ini teks kartu toh tidak terbaca (21 px x 0,45 = 9,4 px), jadi
  // kartu ditampilkan sebagai cip saja.
  var SKALA_SEDERHANA = 0.45;
  // Batas zoom-keluar manual per mode, dinyatakan dalam persen terhadap tampilan pas.
  // Mode tanpa entri di sini berhenti di SKALA_MIN karena batas bacanya tidak boleh dilewati.
  var ZOOM_KELUAR_MIN = { subfitur: 0.4 };
  // Peta yang dikunci NAMA dari manifest wajib tanpa prototipe. Dengan `{}` biasa, service
  // bernama `constructor` (id itu sah menurut `id_aman`: huruf kecil, angka, tanda hubung)
  // membuat `peta[nama]` sudah terisi sebelum pernah ditulis, dan `.push` di atasnya melempar
  // TypeError yang mematikan halaman. Nama seperti `__proto__` lebih buruk lagi: penugasan ke
  // kunci itu mengganti prototipe objeknya.
  function peta() { return Object.create(null); }
  var byFeature = peta();
  var byProduct = peta();
  var byApp = peta();
  var serviceOf = peta();
  var colorOf = peta();
  var actorColor = peta();
  var ikonAktor = peta();
  var statusColor = peta();
  var kindColor = peta();
  var AKAR = document.documentElement;

  function token(nama, cadangan) {
    var v = getComputedStyle(AKAR).getPropertyValue(nama).trim();
    return v || cadangan || '#8a8a86';
  }

  // Rampa dibaca berapa pun panjangnya: tidak ada lagi asumsi "7 grup".
  function rampa(nama) {
    var out = [], i = 1;
    while (true) {
      var v = getComputedStyle(AKAR).getPropertyValue(nama + '-' + i).trim();
      if (!v) { break; }
      out.push(v);
      i += 1;
    }
    return out;
  }

  var RAMPA = { grup: [], status: [], relasi: [] };
  var GELAP = false;
  // S/L kurasi dibaca dari token CSS, bukan dipaku di sini. Delapan hue pertama
  // dikurasi di view.css; grup ke-9 dan seterusnya dirotasi sudut emas memakai
  // S/L yang sama, jadi kurasi dan hasil generate tidak pernah beda keluarga.
  var SAT = '45%';
  var TERANG = '38%';

  // Di luar rampa kurasi hue dirotasi sudut emas, jadi grup ke-N tidak pernah memakai
  // warna grup ke-M. S/L ikut skema supaya tetap terasa satu keluarga.
  function hueTambahan(i) {
    var hue = Math.round(i * 137.508) % 360;
    return 'hsl(' + hue + ', ' + SAT + ', ' + TERANG + ')';
  }

  function paletRampa(r, i, cadangan) {
    if (r.length && i < r.length) { return r[i]; }
    if (r.length) { return hueTambahan(i); }
    return cadangan || '#8a8a86';
  }

  function paletGrup(i) { return paletRampa(RAMPA.grup, i, '#8a8a86'); }

  // ---------- tema ----------
  // Skema dipilih lewat atribut data-tema pada <html>, bukan lewat @media di CSS.
  // Alasannya: warna graf dihitung di JavaScript dari GELAP, jadi harus ada SATU
  // sumber kebenaran yang bisa dibaca CSS dan JS sekaligus. Tanpa pilihan user,
  // hasilnya persis seperti sebelumnya: ikut preferensi OS, termasuk saat OS berubah.
  function temaOS() {
    return !!(window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches);
  }

  function bacaTema() {
    try {
      var v = localStorage.getItem(KUNCI_TEMA);
      return v === 'gelap' || v === 'terang' ? v : null;
    } catch (e) { return null; }
  }

  function temaAktif() {
    return bacaTema() || (temaOS() ? 'gelap' : 'terang');
  }

  function pasangTema(t) {
    AKAR.setAttribute('data-tema', t);
    try { localStorage.setItem(KUNCI_TEMA, t); } catch (e) { /* mode privat: tema tetap berlaku sesi ini */ }
  }

  function labelTema() {
    var tombol = document.getElementById('tombol-tema');
    if (!tombol) { return; }
    var ke = GELAP ? 'terang' : 'gelap';
    var span = tombol.querySelector('span');
    if (span) { span.textContent = ke === 'gelap' ? 'Gelap' : 'Terang'; }
    tombol.setAttribute('title', 'Terapkan tema ' + ke);
    tombol.setAttribute('aria-label', 'Terapkan tema ' + ke);
  }

  function bacaPalet() {
    GELAP = temaAktif() === 'gelap';
    var gaya = getComputedStyle(AKAR);
    SAT = gaya.getPropertyValue('--sat').trim() || '45%';
    TERANG = gaya.getPropertyValue('--terang').trim() || '38%';
    labelTema();
    RAMPA.grup = rampa('--grup');
    RAMPA.status = rampa('--status');
    RAMPA.relasi = rampa('--relasi');
    statusColor = peta();
    (data.statuses || []).forEach(function (s, i) {
      statusColor[s] = paletRampa(RAMPA.status, i, paletGrup(i));
    });
    kindColor = peta();
    (data.kinds || []).forEach(function (k, i) {
      kindColor[k] = paletRampa(RAMPA.relasi, i, paletGrup(i));
    });
    colorOf = peta();
    (data.services || []).forEach(function (sv, i) { colorOf[sv.key] = paletGrup(i); });
    actorColor = peta();
    (data.products || []).forEach(function (pr) {
      if (!(pr.actor in actorColor)) { actorColor[pr.actor] = paletGrup(Object.keys(actorColor).length); }
    });
    ikonAktor = peta();
    Object.keys(data.aktorIkon || {}).forEach(function (a) { ikonAktor[a] = data.aktorIkon[a]; });
  }
  var state = { mode: 'produk', query: '', status: 'semua', service: 'semua' };
  var IKON = {
    cari: '<circle cx="10.5" cy="10.5" r="6.5"/><path d="M15.4 15.4 20 20"/>',
    tambah: '<path d="M12 5v14M5 12h14"/>',
    kurang: '<path d="M5 12h14"/>',
    sesuaikan: '<path d="M4 9V4h5M20 9V4h-5M4 15v5h5M20 15v5h-5"/>',
    semua: '<circle cx="6" cy="6" r="1.6"/><circle cx="18" cy="6" r="1.6"/><circle cx="12" cy="12" r="1.9"/><circle cx="6" cy="18" r="1.6"/><circle cx="18" cy="18" r="1.6"/>',
    struktur: '<rect x="3.5" y="4.5" width="17" height="15" rx="2.5"/><path d="M9.5 4.5v15"/>',
    tutup: '<path d="M6 6l12 12M18 6L6 18"/>',
    produk: '<path d="M12 3.5 4.5 7v10L12 20.5l7.5-3.5V7z"/><path d="M4.5 7 12 10.5 19.5 7M12 10.5v10"/>',
    implementasi: '<rect x="3.5" y="4.5" width="17" height="6" rx="2"/><rect x="3.5" y="13.5" width="17" height="6" rx="2"/><path d="M7 7.5h.01M7 16.5h.01"/>',
    layanan: '<rect x="3.5" y="4.5" width="17" height="15" rx="2.5"/><path d="M3.5 9.5h17M7 7h.01"/>',
    aplikasi: '<rect x="6" y="3.5" width="12" height="17" rx="2.5"/><path d="M10.5 17.5h3"/>',
    subfitur: '<path d="M5 6.5h14M5 12h14M5 17.5h9"/>',
    relasi: '<circle cx="6.5" cy="12" r="2.8"/><circle cx="17.5" cy="12" r="2.8"/><path d="M9.3 12h5.4"/>',
    orang: '<circle cx="12" cy="8.5" r="3.6"/><path d="M5 20c0-3.6 3.1-5.8 7-5.8s7 2.2 7 5.8"/>',
    penggeser: '<path d="M4 8h10M18 8h2M4 16h4M12 16h8"/><circle cx="16" cy="8" r="2"/><circle cx="10" cy="16" r="2"/>',
    perisai: '<path d="M12 3.5 19 6v6c0 4.2-2.9 6.8-7 8.5-4.1-1.7-7-4.3-7-8.5V6z"/><path d="M9 12l2.2 2.2L15.5 10"/>',
    pancar: '<circle cx="12" cy="12" r="3.2"/><path d="M12 3.5v3M12 17.5v3M3.5 12h3M17.5 12h3M6 6l2.1 2.1M15.9 15.9 18 18M18 6l-2.1 2.1M8.1 15.9 6 18"/>',
    lipat: '<path d="M13 3.5 5 11.5l3 3 8-8zM13 10.5l-5 5 3 3 7-7z"/>',
    lengkung: '<path d="M4 6.5h9.5a2.5 2.5 0 0 1 0 5H8a2.5 2.5 0 0 0 0 5h11"/>',
    orbit: '<circle cx="12" cy="12" r="2.2"/><ellipse cx="12" cy="12" rx="9.5" ry="4"/><ellipse cx="12" cy="12" rx="9.5" ry="4" transform="rotate(60 12 12)"/><ellipse cx="12" cy="12" rx="9.5" ry="4" transform="rotate(120 12 12)"/>',
    gerbang: '<path d="M4 9.5h6M4 14.5h6M14 9.5h6M14 14.5h6"/><path d="M12 4.5v15"/>',
    heksagon: '<path d="M12 3.5 19 7.5v9L12 20.5 5 16.5v-9z"/><path d="M9.5 14.5v-5l5 5v-5"/>',
    monitor: '<rect x="4" y="5" width="16" height="12" rx="2"/><path d="M9 20h6M12 17v3"/>',
    mesin: '<path d="M7 9V4.5h10V9"/><rect x="4" y="9" width="16" height="7" rx="2"/><path d="M7 14h10v5.5H7z"/>',
    kamera: '<rect x="3" y="7" width="18" height="12" rx="2.5"/><circle cx="12" cy="13" r="3.4"/><path d="M8.5 7l1.5-2.5h4L15.5 7"/>',
    video: '<rect x="3" y="6.5" width="12" height="11" rx="2.5"/><path d="M15 11l6-3.5v9L15 13.5z"/>',
    berkas: '<path d="M6 3.5h7l5 5v12H6z"/><path d="M13 3.5V9h5"/>',
    kode: '<path d="M9 7.5 4.5 12 9 16.5M15 7.5 19.5 12 15 16.5"/>',
    folder: '<path d="M3.5 6.5h6l2 2.5h9v11h-17z"/>',
    panah: '<path d="M9 5.5 16 12l-7 6.5"/>',
    hasil: '<path d="M5 12.5 9.5 17 19 7"/>',
    catatan: '<circle cx="12" cy="12" r="8.5"/><path d="M12 11v5.5M12 7.8h.01"/>',
    operasi: '<path d="M8 5.5v13l10-6.5z"/>',
    unggah: '<path d="M12 16.5V5.5M7.5 10 12 5.5 16.5 10"/><path d="M4.5 19.5h15"/>',
    unduh: '<path d="M12 5.5v11M7.5 12 12 16.5 16.5 12"/><path d="M4.5 19.5h15"/>',
    perbarui: '<path d="M19.5 12a7.5 7.5 0 1 1-2.6-5.7"/><path d="M19.5 4.5V9H15"/>',
    buang: '<path d="M4.5 7h15M9 4.5h6M6.5 7l.9 12.5h9.2L17.5 7"/><path d="M10 10.5v6M14 10.5v6"/>',
    jam: '<circle cx="12" cy="12" r="8.5"/><path d="M12 7.5V12l3.5 2"/>',
    ringkas: '<path d="M9 5.5v3.5H5.5M15 5.5v3.5h3.5M9 18.5V15H5.5M15 18.5V15h3.5"/><path d="M5.5 12h13"/>',
    pengaturan: '<circle cx="12" cy="12" r="3.2"/><path d="M12 3.5v3M12 17.5v3M3.5 12h3M17.5 12h3M6 6l2.1 2.1M15.9 15.9 18 18M18 6l-2.1 2.1M8.1 15.9 6 18"/>',
    terminal: '<rect x="4" y="4.5" width="16" height="11" rx="2"/><path d="M9 19.5h6M12 15.5v4"/>',
    web: '<rect x="3.5" y="5" width="17" height="14" rx="2.5"/><path d="M3.5 9.5h17"/><path d="M6.6 7.2h.01M9.2 7.2h.01"/>',
    kilau: '<path d="M11 4l1.7 4L17 9.7 12.7 11.4 11 15.4 9.3 11.4 5 9.7 9.3 8z"/><path d="M17.5 14.5l.9 2.1 2.1.9-2.1.9-.9 2.1-.9-2.1-2.1-.9 2.1-.9z"/>',
    papan: '<rect x="6" y="4.5" width="12" height="16" rx="2"/><path d="M9.5 4.5V3h5v1.5"/><path d="M9 10.5h6M9 14.5h4"/>',
    panel: '<rect x="3.5" y="4.5" width="17" height="15" rx="2.5"/><path d="M8 4.5v15M11.5 9.5h5M11.5 13.5h5"/>',
    masuk: '<path d="M14 4.5h5.5v15H14"/><path d="M9.5 8 5.5 12l4 4M5.5 12h9"/>',
    status: '<circle cx="12" cy="12" r="8"/><circle cx="12" cy="12" r="3.2" fill="currentColor" stroke="none"/>',
    saring: '<path d="M4 5.5h16l-6.2 7v6.5l-3.6-2v-4.5z"/>'
  };
        var nodes = [];
  var edges = [];
  var posisi = peta();
  var groupsCache = [];
  var centersCache = peta();
  var selectedId = null;
  var detailTerakhir = null;
  var view = { x: 0, y: 0, w: W, h: H };
  var zoomDasar = W;
  var miniViewport = null;

  data.features.forEach(function (f) { byFeature[f.id] = f; });
  data.products.forEach(function (p) { byProduct[p.id] = p; });
  data.apps.forEach(function (a) { byApp[a.id] = a; });
  data.services.forEach(function (s, i) {
    serviceOf[s.key] = s;
  });

  function el(name, attrs) {
    var node = document.createElementNS(NS, name);
    if (attrs) {
      Object.keys(attrs).forEach(function (k) { node.setAttribute(k, attrs[k]); });
    }
    return node;
  }

  function ikonHtml(nama, kelas) {
    if (!IKON[nama]) { return ''; }
    return '<svg class="ikon' + (kelas ? ' ' + kelas : '') + '" width="16" height="16" ' +
      'viewBox="0 0 24 24" aria-hidden="true" focusable="false">' + IKON[nama] + '</svg>';
  }

  function isiIkon(root) {
    (root || document).querySelectorAll('[data-ikon]').forEach(function (e) {
      var nama = e.dataset.ikon;
      if (!IKON[nama]) { return; }
      e.innerHTML = ikonHtml(nama);
      e.removeAttribute('data-ikon');
    });
  }

  function gambarIkon(induk, nama, x, y, ukuran) {
    if (!IKON[nama]) { return null; }
    var svgEl = el('svg', {
      x: x.toFixed(1),
      y: y.toFixed(1),
      width: String(ukuran),
      height: String(ukuran),
      viewBox: '0 0 24 24',
      'class': 'ikon',
      'aria-hidden': 'true'
    });
    svgEl.innerHTML = IKON[nama];
    induk.appendChild(svgEl);
    return svgEl;
  }

  function ikonProduk(p) {
    return ikonAktor[p.actor] || 'produk';
  }

  function ikonAplikasi(a) {
    if (a.ikon) { return a.ikon; }
    var s = serviceOf[a.service];
    return (s && s.ikon) || 'aplikasi';
  }

  function hash(text) {
    var h = 2166136261 >>> 0;
    for (var i = 0; i < text.length; i++) {
      h ^= text.charCodeAt(i);
      h = Math.imul(h, 16777619);
    }
    return h >>> 0;
  }

  function seeded(seed) {
    var x = seed || 1;
    return function () {
      x ^= x << 13;
      x ^= x >>> 17;
      x ^= x << 5;
      return (x >>> 0) / 4294967296;
    };
  }

  function esc(text) {
    return String(text == null ? '' : text)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');
  }

  function buildGraph() {
    var ukuran = DUNIA[state.mode] || DUNIA.produk;
    W = ukuran[0];
    H = ukuran[1];
    if (state.mode === 'subfitur') {
      nodes = data.features.map(function (f) {
        var a = byApp[f.app];
        return {
          id: f.id,
          kind: 'subfitur',
          obj: f,
          label: f.name,
          group: f.service,
          ikon: a ? ikonAplikasi(a) : 'subfitur',
          color: colorOf[f.service] || '#6b6b6b'
        };
      });
      edges = data.edges.map(function (e) {
        return { from: e.from, to: e.to, kind: e.kind, label: e.label };
      });
    } else if (state.mode === 'produk') {
      nodes = data.products.map(function (p) {
        return {
          id: p.id,
          kind: 'produk',
          obj: p,
          label: p.name,
          group: p.actor,
          ikon: ikonProduk(p),
          color: actorColor[p.actor] || '#6b6b6b'
        };
      });
      edges = data.productEdges.map(function (e) {
        return { from: e.from, to: e.to, kind: e.kind, label: e.label };
      });
    } else {
      nodes = data.apps.map(function (a) {
        return {
          id: a.id,
          kind: 'app',
          obj: a,
          label: a.name,
          group: a.service,
          ikon: ikonAplikasi(a),
          color: colorOf[a.service] || '#6b6b6b'
        };
      });
      edges = data.appEdges.map(function (e) {
        return { from: e.from, to: e.to, kind: e.kind, label: e.label };
      });
    }

    posisi = peta();
    nodes.forEach(function (n) {
      posisi[n.id] = n;
      n.x = 0;
      n.y = 0;
      n.vx = 0;
      n.vy = 0;
      n.h = TINGGI_NODE;
      // Lebar kartu dibulatkan ke tangga 24px. Sebelumnya tiap kartu punya lebar
      // hasil rumus panjang label, jadi dua kartu yang bersebelahan berbeda 9px dan
      // tepi graf terlihat bergerigi; dengan tangga, kartu seukuran punya tepi yang
      // sama persis sementara kartu panjang tetap tidak kepotong (pembulatan ke atas).
      n.w = Math.max(LEBAR_MIN, Math.min(LEBAR_MAX, Math.ceil((128 + n.label.length * 11.4) / 24) * 24));
    });

    groupsCache = [];
    nodes.forEach(function (n) {
      if (groupsCache.indexOf(n.group) === -1) { groupsCache.push(n.group); }
    });

    if (state.mode === 'subfitur') {
      letakkanBertingkat();
      pisahkanKlaster();
      return;
    }
    var cols = Math.max(1, Math.ceil(Math.sqrt(groupsCache.length)));
    var rows = Math.max(1, Math.ceil(groupsCache.length / cols));
    var cellW = W / cols;
    var cellH = H / rows;
    // Jari-jari klaster harus cukup untuk menampung kartunya sendiri, tidak cukup
    // cuma sisa sel grid: 16 kartu bisa lebih lebar dari selnya.
    var tinggiBaris = TINGGI_NODE + 20;
    var perGrup = peta();
    nodes.forEach(function (n) { perGrup[n.group] = (perGrup[n.group] || 0) + 1; });
    var jarakMinimal = 140;
    groupsCache.forEach(function (g) {
      var n = perGrup[g] || 1;
      var kolom = Math.max(1, Math.round(Math.sqrt(n)));
      var baris = Math.ceil(n / kolom);
      jarakMinimal = Math.max(jarakMinimal,
        Math.max(kolom * 190, baris * tinggiBaris) / 2 + 30);
    });
    var jarakPusat = Math.max(90, Math.min(cellW, cellH) / 2 - 70, jarakMinimal);
    // Sumbu yang ruangnya sempit tidak boleh memaksa kartu menumpuk: klaster dibiarkan
    // melebar ke arah sumbu yang masih longgar, kotak klaster dipisah di akhir buildGraph.
    var jarakX = Math.max(90, (cellW - 120) / 2, jarakPusat);
    var jarakY = Math.max(90, (cellH - 120) / 2, jarakPusat);
    centersCache = peta();
    groupsCache.forEach(function (g, i) {
      centersCache[g] = {
        x: (i % cols + 0.5) * cellW,
        y: (Math.floor(i / cols) + 0.5) * cellH
      };
    });
    nodes.forEach(function (n) {
      var c = centersCache[n.group];
      var rnd = seeded(hash(n.id));
      var sudut = rnd() * Math.PI * 2;
      var jarak = 30 + rnd() * Math.max(40, jarakPusat * 0.7);
      n.x = c.x + Math.cos(sudut) * jarak * (jarakX / jarakPusat);
      n.y = c.y + Math.sin(sudut) * jarak * (jarakY / jarakPusat);
    });

    var iterasi = 430;
    for (var t = 0; t < iterasi; t++) {
      var dingin = 1 - t / iterasi;
      for (var i = 0; i < nodes.length; i++) {
        for (var j = i + 1; j < nodes.length; j++) {
          var dx = nodes[j].x - nodes[i].x;
          var dy = nodes[j].y - nodes[i].y;
          var d2 = dx * dx + dy * dy;
          if (d2 < 1) {
            dx = (i - j) || 1;
            dy = 1;
            d2 = dx * dx + dy * dy;
          }
          var d = Math.sqrt(d2);
          var gaya = (state.mode === 'produk' ? 3200 : state.mode === 'subfitur' ? 9000 : 4200) / d2;
          var fx = (dx / d) * gaya * dingin;
          var fy = (dy / d) * gaya * dingin;
          nodes[i].vx -= fx;
          nodes[i].vy -= fy;
          nodes[j].vx += fx;
          nodes[j].vy += fy;
        }
      }
      edges.forEach(function (e) {
        var a = posisi[e.from];
        var b = posisi[e.to];
        if (!a || !b) { return; }
        var dx = b.x - a.x;
        var dy = b.y - a.y;
        var d = Math.hypot(dx, dy) || 1;
        var jangkar = (a.w + b.w) / 2 + 70;
        var f = (d - jangkar) * 0.018 * dingin;
        var fx = (dx / d) * f;
        var fy = (dy / d) * f;
        a.vx += fx;
        a.vy += fy;
        b.vx -= fx;
        b.vy -= fy;
      });
      nodes.forEach(function (n) {
        var c = centersCache[n.group];
        n.vx += (c.x - n.x) * (state.mode === 'produk' ? 0.03 : 0.022) * dingin;
        n.vy += (c.y - n.y) * (state.mode === 'produk' ? 0.03 : 0.022) * dingin;
        n.vx += (W / 2 - n.x) * 0.001 * dingin;
        n.vy += (H / 2 - n.y) * 0.001 * dingin;
        n.vx *= 0.8;
        n.vy *= 0.8;
        var maks = 7;
        n.vx = Math.max(-maks, Math.min(maks, n.vx));
        n.vy = Math.max(-maks, Math.min(maks, n.vy));
        n.x += n.vx;
        n.y += n.vy;
        var dxc = n.x - c.x;
        var dyc = n.y - c.y;
        var dc = Math.hypot(dxc / jarakX, dyc / jarakY);
        if (dc > 1) {
          n.x = c.x + dxc / dc;
          n.y = c.y + dyc / dc;
        }
      });
    }

    // Susun ulang agar kartu tidak bertumpuk: dorong sepanjang sumbu tersingkat.
    var sela = 20;
    var iterasiPisah = nodes.length > 40 ? 900 : 200;
    for (var lapis = 0; lapis < iterasiPisah; lapis++) {
      var geser = false;
      for (var p = 0; p < nodes.length; p++) {
        for (var q = p + 1; q < nodes.length; q++) {
          var na = nodes[p];
          var nb = nodes[q];
          var ox = (na.w + nb.w) / 2 + sela - Math.abs(nb.x - na.x);
          var oy = (na.h + nb.h) / 2 + sela - Math.abs(nb.y - na.y);
          if (ox <= 0 || oy <= 0) { continue; }
          geser = true;
          if (ox < oy) {
            var arahx = nb.x >= na.x ? 1 : -1;
            na.x -= (arahx * ox) / 2;
            nb.x += (arahx * ox) / 2;
          } else {
            var arahy = nb.y >= na.y ? 1 : -1;
            na.y -= (arahy * oy) / 2;
            nb.y += (arahy * oy) / 2;
          }
        }
      }
      if (!geser) { break; }
    }

    // Sisa tumpukan didorong keluar lewat translasi per klaster: translasi tidak bisa
    // menciptakan tumpukan baru di dalam klaster, dan kotak klaster dipisah setelahnya.
    var perGrupTabrakan = peta();
    nodes.forEach(function (n) {
      if (!perGrupTabrakan[n.group]) { perGrupTabrakan[n.group] = []; }
      perGrupTabrakan[n.group].push(n);
    });
    var daftarGrupTabrakan = groupsCache.filter(function (g) { return !!perGrupTabrakan[g]; });
    for (var dorong = 0; dorong < 60; dorong++) {
      var sisaTabrakan = 0;
      for (var gi = 0; gi < daftarGrupTabrakan.length; gi++) {
        var anggota = perGrupTabrakan[daftarGrupTabrakan[gi]];
        var kotakAnggota = anggota.map(function (n) { return { x: n.x, y: n.y, w: n.w, h: n.h }; });
        for (var a = 0; a < kotakAnggota.length; a++) {
          for (var b = a + 1; b < kotakAnggota.length; b++) {
            var pa = kotakAnggota[a];
            var pb = kotakAnggota[b];
            var tX = Math.min(pa.x + pa.w, pb.x + pb.w) - Math.max(pa.x, pb.x);
            var tY = Math.min(pa.y + pa.h, pb.y + pb.h) - Math.max(pa.y, pb.y);
            if (tX <= 1 || tY <= 1) { continue; }
            sisaTabrakan++;
            var pusatA = { x: pa.x + pa.w / 2, y: pa.y + pa.h / 2 };
            var pusatB = { x: pb.x + pb.w / 2, y: pb.y + pb.h / 2 };
            var dx = pusatB.x - pusatA.x;
            var dy = pusatB.y - pusatA.y;
            var panjang = Math.hypot(dx, dy);
            if (panjang < 1) { dx = 1; dy = 0; panjang = 1; }
            var jarakDorong = (tX < tY ? tX + 6 : tY + 6) / 2;
            var ux = dx / panjang * jarakDorong;
            var uy = dy / panjang * jarakDorong;
            anggota[a].x -= ux;
            anggota[a].y -= uy;
            anggota[b].x += ux;
            anggota[b].y += uy;
          }
        }
      }
      if (!sisaTabrakan) { break; }
    }

    nodes.forEach(function (n) {
      n.x = Math.max(n.w / 2 + 30, Math.min(W - n.w / 2 - 30, n.x));
      n.y = Math.max(n.h / 2 + 70, Math.min(H - n.h / 2 - 30, n.y));
      n.x -= n.w / 2;
      n.y -= n.h / 2;
    });
    pisahkanKlaster();
  }

  // Penempatan mode Sub-fitur: tidak pakai simulasi gaya sama sekali. Tingkat (kolom)
  // dihitung dari lintasan terpanjang pada DAG, lajur (baris) dari service, jadi
  // tumpang-tindih kartu tidak bisa terjadi dan edge tetap mengalir kiri ke kanan.
  function letakkanBertingkat() {
    var adj = peta();
    nodes.forEach(function (n) { adj[n.id] = []; });
    edges.forEach(function (e) {
      if (adj[e.from] && posisi[e.to] && e.from !== e.to) { adj[e.from].push(e.to); }
    });

    // Buang sisi balik lewat DFS supaya siklus tidak membuat tingkat tak berhingga.
    var tanda = peta();
    var sisiDag = [];
    nodes.forEach(function (n) { tanda[n.id] = 0; });
    function telusuri(id) {
      tanda[id] = 1;
      adj[id].forEach(function (ke) {
        if (tanda[ke] === 1) { return; }
        sisiDag.push([id, ke]);
        if (tanda[ke] === 0) { telusuri(ke); }
      });
      tanda[id] = 2;
    }
    nodes.forEach(function (n) { if (tanda[n.id] === 0) { telusuri(n.id); } });

    var keluar = peta();
    var masuk = peta();
    nodes.forEach(function (n) { keluar[n.id] = []; masuk[n.id] = 0; });
    sisiDag.forEach(function (p) {
      keluar[p[0]].push(p[1]);
      masuk[p[1]] += 1;
    });
    var tingkat = peta();
    var antrean = [];
    nodes.forEach(function (n) {
      if (masuk[n.id] === 0) { tingkat[n.id] = 0; antrean.push(n.id); }
    });
    for (var q = 0; q < antrean.length; q++) {
      var id = antrean[q];
      keluar[id].forEach(function (ke) {
        tingkat[ke] = Math.max(tingkat[ke] || 0, tingkat[id] + 1);
        masuk[ke] -= 1;
        if (masuk[ke] === 0) { antrean.push(ke); }
      });
    }
    var tingkatTertinggi = 0;
    nodes.forEach(function (n) {
      if (tingkat[n.id] === undefined) { tingkat[n.id] = 0; }
      tingkatTertinggi = Math.max(tingkatTertinggi, tingkat[n.id]);
    });

    // Kompaksi lubang. Jangkauan sah tingkat sebuah kartu adalah
    // [lintasanTerpanjangDariSumber, tingkatTertinggi - lintasanTerpanjangKeMuara];
    // di dalam jangkauan itu semua garis tetap maju, jadi kartunya boleh digeser ke
    // kolom yang masih longgar. Tanpa langkah ini, lajur yang penuh kartu sumber menumpuk di
    // kolom 0 sementara seluruh kanan lajurnya kosong.
    var pred = peta();
    nodes.forEach(function (n) { pred[n.id] = []; });
    sisiDag.forEach(function (p) { pred[p[1]].push(p[0]); });
    var keMuara = peta();
    nodes.forEach(function (n) { keMuara[n.id] = 0; });
    for (var s = antrean.length - 1; s >= 0; s--) {
      var idMuara = antrean[s];
      keluar[idMuara].forEach(function (ke) {
        keMuara[idMuara] = Math.max(keMuara[idMuara], keMuara[ke] + 1);
      });
    }
    var isiSel = peta();
    antrean.forEach(function (id) {
      var n = posisi[id];
      var bawah = tingkat[id];
      pred[id].forEach(function (p) { bawah = Math.max(bawah, tingkat[p] + 1); });
      var atas = tingkatTertinggi - keMuara[id];
      if (bawah > atas) { return; }
      var pilih = bawah;
      var palingPenuh = Infinity;
      for (var rr = bawah; rr <= atas; rr++) {
        var kunci = n.group + '@' + rr;
        var penuh = isiSel[kunci] || 0;
        if (penuh < palingPenuh) { palingPenuh = penuh; pilih = rr; }
      }
      tingkat[id] = pilih;
      var terpilih = n.group + '@' + pilih;
      isiSel[terpilih] = (isiSel[terpilih] || 0) + 1;
    });

    // Rantai panjang memekarkan kolom: dua kolom bertetangga paling kosong digabung
    // selama jumlah kolom masih di atas MAKS_KOLOM. Menggeser kartu ke kolom kirinya
    // hanya bisa membuat edge berhenti di kolom yang sama, tidak pernah ke belakang.
    // Diukur tiga nilai: 12 kolom -> 8 kartu terbaca, 9 -> 11, 6 -> 11 tapi dunianya
    // menjulang (2600x3918, aspek 0,66 melawan kanvas 1,35). Sembilan paling pas.
    var MAKS_KOLOM = 9;
    while (tingkatTertinggi + 1 > MAKS_KOLOM) {
      var isi = [];
      for (var r0 = 0; r0 <= tingkatTertinggi; r0++) { isi[r0] = 0; }
      nodes.forEach(function (n) { isi[tingkat[n.id]] += 1; });
      var pilih = 1;
      var palingKecil = Infinity;
      for (var k = 1; k <= tingkatTertinggi; k++) {
        if (isi[k] + isi[k - 1] < palingKecil) { palingKecil = isi[k] + isi[k - 1]; pilih = k; }
      }
      nodes.forEach(function (n) {
        if (tingkat[n.id] === pilih) { tingkat[n.id] = pilih - 1; }
        else if (tingkat[n.id] > pilih) { tingkat[n.id] -= 1; }
      });
      tingkatTertinggi -= 1;
    }

    // Lajur mengikuti urutan services.tsv, kolom mengikuti tingkat.
    var urutanLajur = data.services.map(function (s) { return s.key; });
    groupsCache.sort(function (a, b) {
      return urutanLajur.indexOf(a) - urutanLajur.indexOf(b);
    });
    var sel = peta();
    nodes.forEach(function (n) {
      var kunci = n.group + '@' + tingkat[n.id];
      if (!sel[kunci]) { sel[kunci] = []; }
      sel[kunci].push(n);
    });
    var barisLajur = peta();
    groupsCache.forEach(function (g) { barisLajur[g] = 1; });
    Object.keys(sel).forEach(function (kunci) {
      var g = kunci.split('@')[0];
      sel[kunci].sort(function (a, b) { return a.label < b.label ? -1 : a.label > b.label ? 1 : 0; });
      barisLajur[g] = Math.max(barisLajur[g], sel[kunci].length);
    });

    var lebarKolom = [];
    for (var r = 0; r <= tingkatTertinggi; r++) { lebarKolom[r] = LEBAR_MIN; }
    nodes.forEach(function (n) {
      lebarKolom[tingkat[n.id]] = Math.max(lebarKolom[tingkat[n.id]], n.w);
    });
    var selaX = 22;
    var selaY = TINGGI_NODE + 14;
    var kepala = 46;
    var jarakLajur = 46;
    var awalX = 40;
    var awalY = 40;
    var posisiKolom = [];
    var x = awalX;
    for (var c = 0; c <= tingkatTertinggi; c++) {
      posisiKolom[c] = x;
      x += lebarKolom[c] + selaX;
    }

    var y = awalY;
    groupsCache.forEach(function (g) {
      var atas = y + kepala;
      for (var r = 0; r <= tingkatTertinggi; r++) {
        var isi = sel[g + '@' + r];
        if (!isi) { continue; }
        // Tumpukan sel dipusatkan dalam lajurnya, bukan ditempelkan ke atas, supaya
        // lajur tidak terlihat sebagai tangga kosong di bagian bawah.
        var mulai = atas + (barisLajur[g] - isi.length) * selaY / 2;
        isi.forEach(function (n, i) {
          // Tepi KIRI kartu dalam satu kolom disamakan. posisiKolom adalah titik pusat
          // kolom, jadi selama ini kartu berpusat sama tetapi tepi kirinya berbeda sepanjang
          // selisih lebarnya — kolom terlihat tidak lurus.
          n.x = posisiKolom[r] - lebarKolom[r] / 2 + n.w / 2;
          n.y = mulai + i * selaY;
        });
      }
      y = atas + barisLajur[g] * selaY + jarakLajur;
    });

    W = Math.max(W, posisiKolom[tingkatTertinggi] + lebarKolom[tingkatTertinggi] + awalX);
    H = Math.max(H, y - jarakLajur + 40);
  }

  // Klaster boleh berdempetan, tapi kotak dan labelnya tidak boleh saling menimpa:
  // dorong seluruh isi klaster sepanjang sumbu tersingkat, lalu pusatkan ulang isinya.
  function pisahkanKlaster() {
    var perGrup = peta();
    nodes.forEach(function (n) {
      if (!perGrup[n.group]) { perGrup[n.group] = []; }
      perGrup[n.group].push(n);
    });
    var daftar = groupsCache.filter(function (g) { return !!perGrup[g]; });
    for (var ronde = 0; ronde < 40; ronde++) {
      var gerak = false;
      for (var p = 0; p < daftar.length; p++) {
        for (var q = p + 1; q < daftar.length; q++) {
          var ba = batasKlaster(perGrup[daftar[p]]);
          var bb = batasKlaster(perGrup[daftar[q]]);
          var ox = Math.min(ba.x + ba.w, bb.x + bb.w) - Math.max(ba.x, bb.x);
          var oy = Math.min(ba.y + ba.h, bb.y + bb.h) - Math.max(ba.y, bb.y);
          if (ox <= 0 || oy <= 0) { continue; }
          gerak = true;
          var gx = 0;
          var gy = 0;
          if (ox < oy) {
            gx = (ba.x <= bb.x ? -1 : 1) * (ox / 2 + 6);
          } else {
            gy = (ba.y <= bb.y ? -1 : 1) * (oy / 2 + 6);
          }
          perGrup[daftar[p]].forEach(function (n) { n.x += gx; n.y += gy; });
          perGrup[daftar[q]].forEach(function (n) { n.x -= gx; n.y -= gy; });
        }
      }
      if (!gerak) { break; }
    }
    var x1 = Infinity;
    var y1 = Infinity;
    var x2 = -Infinity;
    var y2 = -Infinity;
    nodes.forEach(function (n) {
      x1 = Math.min(x1, n.x);
      y1 = Math.min(y1, n.y);
      x2 = Math.max(x2, n.x + n.w);
      y2 = Math.max(y2, n.y + n.h);
    });
    if (x2 - x1 <= W - 60) {
      nodes.forEach(function (n) { n.x += (W - (x1 + x2)) / 2; });
    }
    if (y2 - y1 <= H - 100) {
      nodes.forEach(function (n) { n.y += (H - (y1 + y2)) / 2; });
    }
  }

  function tcx(n) {
    return n.x + n.w / 2;
  }

  function tcy(n) {
    return n.y + n.h / 2;
  }

  function titikJoran(n, tx, ty) {
    var cx = tcx(n);
    var cy = tcy(n);
    var dx = tx - cx;
    var dy = ty - cy;
    if (!dx && !dy) { return { x: cx, y: cy }; }
    var sx = dx === 0 ? Infinity : (n.w / 2 + 6) / Math.abs(dx);
    var sy = dy === 0 ? Infinity : (n.h / 2 + 6) / Math.abs(dy);
    var s = Math.min(sx, sy);
    return { x: cx + dx * s, y: cy + dy * s };
  }

  function batasKlaster(pts) {
    var x1 = Infinity;
    var y1 = Infinity;
    var x2 = -Infinity;
    var y2 = -Infinity;
    pts.forEach(function (n) {
      x1 = Math.min(x1, n.x);
      y1 = Math.min(y1, n.y);
      x2 = Math.max(x2, n.x + n.w);
      y2 = Math.max(y2, n.y + n.h);
    });
    var kepala = 42;
    var sisi = 14;
    return {
      x: x1 - sisi,
      y: y1 - kepala,
      w: (x2 - x1) + sisi * 2,
      h: (y2 - y1) + kepala + sisi
    };
  }

  function groupLabel(group) {
    if (state.mode === 'produk') { return group; }
    var s = serviceOf[group];
    return s ? s.name : group;
  }

  function warnaGrup(g) {
    if (state.mode === 'produk') { return actorColor[g] || '#6b6b6b'; }
    return colorOf[g] || '#6b6b6b';
  }

  function ikonGrup(g) {
    if (state.mode === 'produk') { return ikonAktor[g] || 'produk'; }
    var s = serviceOf[g];
    return (s && s.ikon) || 'layanan';
  }

  // Lebar teks label kartu. getComputedTextLength hanya mengembalikan angka kalau
  // elemennya sedang ter-render, jadi zoom semantik (cip) dilawan dulu di awal draw().
  function lebarTeks(teksEl, isi) {
    teksEl.textContent = isi;
    if (typeof teksEl.getComputedTextLength === 'function') {
      var terukur = teksEl.getComputedTextLength();
      if (terukur > 0) { return terukur; }
    }
    return String(isi).length * 11.4;
  }

  // Label yang lebih lebar dari kartunya tidak boleh keluar tepi kartu atau tertutup pill:
  // dipotong dengan elipsis. Nama lengkapnya tetap utuh di aria-label, <title>, rail, dan panel.
  function potongLabel(teksEl, label, ruang) {
    var penuh = String(label);
    if (ruang <= 0) { return penuh; }
    if (lebarTeks(teksEl, penuh) <= ruang) { return penuh; }
    var bawah = 0;
    var atas = penuh.length;
    var punya = 0;
    while (bawah <= atas) {
      var tengah = (bawah + atas) >> 1;
      var coba = penuh.slice(0, tengah) + '\u2026';
      if (lebarTeks(teksEl, coba) <= ruang) {
        punya = tengah;
        bawah = tengah + 1;
      } else {
        atas = tengah - 1;
      }
    }
    return penuh.slice(0, punya) + '\u2026';
  }

  // Ujung panah mengikuti hue jenis relasinya, sama seperti garisnya. Dua tingkap per jenis:
  // satu untuk garis yang sedang dibaca, satu untuk garis yang diredup atau diburamkan,
  // supaya kepala panah tidak tetap menyala sementara garisnya sudah padam.
  var PUDAR_MARKA = 0.12;

  function markerPanah(defs, id, warna, alpha) {
    var marker = el('marker', {
      id: id,
      viewBox: '0 0 10 10',
      refX: '9',
      refY: '5',
      markerWidth: '5',
      markerHeight: '5',
      orient: 'auto-start-reverse'
    });
    var bentuk = el('path', { d: 'M 0 0 L 10 5 L 0 10 z', fill: warna });
    if (alpha !== 1) { bentuk.setAttribute('fill-opacity', String(alpha)); }
    marker.appendChild(bentuk);
    defs.appendChild(marker);
  }

  function draw() {
    // Kelas ini dihitung ulang oleh applyView() di akhir draw(); dibuang di sini supaya
    // pengukuran teks di atas punya angka yang benar.
    svg.classList.remove('simpl');
    while (svg.firstChild) { svg.removeChild(svg.firstChild); }
    var defs = el('defs');
    var warnaNetral = token('--jaringan', '#8a8a86');
    markerPanah(defs, 'panah-netral', warnaNetral, 1);
    markerPanah(defs, 'panah-netral-pudar', warnaNetral, PUDAR_MARKA);
    data.kinds.forEach(function (kind) {
      var warna = kindColor[kind] || '#8a8a86';
      markerPanah(defs, 'panah-' + kind, warna, 1);
      markerPanah(defs, 'panah-' + kind + '-pudar', warna, PUDAR_MARKA);
    });
    var pola = el('pattern', {
      id: 'titik-kanvas',
      width: '44',
      height: '44',
      patternUnits: 'userSpaceOnUse'
    });
    pola.appendChild(el('circle', { cx: '1.2', cy: '1.2', r: '1.2', 'class': 'titik-kanvas' }));
    defs.appendChild(pola);
    svg.appendChild(defs);

    svg.appendChild(el('rect', {
      x: '-400',
      y: '-400',
      width: String(W + 800),
      height: String(H + 800),
      fill: 'url(#titik-kanvas)',
      'pointer-events': 'none'
    }));

    var groups = peta();
    nodes.forEach(function (n) {
      if (!groups[n.group]) { groups[n.group] = []; }
      groups[n.group].push(n);
    });

    var klasterLayer = el('g', { 'class': 'klaster-layer' });
    groupsCache.forEach(function (g) {
      var pts = groups[g] || [];
      if (!pts.length) { return; }
      var b = batasKlaster(pts);
      if (state.mode === 'subfitur') {
        // Lajur service membentang selebar dunia: kotak yang diukur dari isi saja membuat
        // peta terlihat sebagai kotak mengambang dengan lubang besar di kanan.
        // Tepi kirinya tidak boleh konstanta: pisahkanKlaster bisa mendorong isi ke kiri
        // batas dunia, dan sentralisasi hanya jalan kalau muatannya masih sempit. Lajur yang
        // lebih sempit dari isinya membuat kartu keluar kotaknya sendiri, bahkan ke kiri
        // label grupnya. Jadi tepi kiri mengikuti kartu paling kiri yang sungguh ada.
        var kiri = 26;
        nodes.forEach(function (n) { kiri = Math.min(kiri, n.x - 16); });
        b = { x: kiri, y: b.y, w: W - kiri - 26, h: b.h };
      }
      var w = warnaGrup(g);
      var kotak = el('rect', {
        x: b.x.toFixed(1),
        y: b.y.toFixed(1),
        width: b.w.toFixed(1),
        height: b.h.toFixed(1),
        rx: '16',
        'class': 'klaster'
      });
      kotak.style.stroke = w;
      kotak.style.fill = w;
      klasterLayer.appendChild(kotak);
      var kepala = el('g', { 'class': 'klaster-kepala' });
      var ikon = gambarIkon(kepala, ikonGrup(g), b.x + 15, b.y + 13, 22);
      if (ikon) { ikon.style.stroke = w; }
      var label = el('text', {
        x: (b.x + 46).toFixed(1),
        y: (b.y + 29).toFixed(1),
        'class': 'klaster-label'
      });
      label.style.fill = w;
      label.textContent = groupLabel(g);
      var jumlah = el('tspan', { 'class': 'klaster-jumlah', 'data-klaster': g });
      jumlah.textContent = ' ' + pts.length;
      label.appendChild(jumlah);
      kepala.appendChild(label);
      klasterLayer.appendChild(kepala);
    });
    svg.appendChild(klasterLayer);

    var edgeLayer = el('g', { 'class': 'edges' });
    edges.forEach(function (e) {
      var a = posisi[e.from];
      var b = posisi[e.to];
      if (!a || !b) { return; }
      var p1 = titikJoran(a, tcx(b), tcy(b));
      var p2 = titikJoran(b, tcx(a), tcy(a));
      var dx = p2.x - p1.x;
      var dy = p2.y - p1.y;
      var d = Math.hypot(dx, dy) || 1;
      var arah = hash(e.from + e.to + e.label) % 2 === 0 ? 1 : -1;
      var tekuk = Math.min(22, 6 + d * 0.035) * arah;
      var mx = (p1.x + p2.x) / 2 - (dy / d) * tekuk;
      var my = (p1.y + p2.y) / 2 + (dx / d) * tekuk;
      var path = el('path', {
        d: 'M ' + p1.x.toFixed(1) + ' ' + p1.y.toFixed(1) + ' Q ' + mx.toFixed(1) + ' ' + my.toFixed(1) + ' ' + p2.x.toFixed(1) + ' ' + p2.y.toFixed(1),
        'class': 'edge',
        'marker-end': 'url(#panah-' + (kindColor[e.kind] ? e.kind : 'netral') + ')',
        'data-from': e.from,
        'data-to': e.to,
        'data-jenis': e.kind
      });
      // Hue jenis relasi masuk lewat properti kustom: aturan CSS menang atas atribut
      // presentasi, jadi menulis `stroke=` di sini hanya ilusi bahwa garis sudah berwarna.
      path.style.setProperty('--jenis-warna', kindColor[e.kind] || paletGrup(0));
      var title = el('title');
      title.textContent = e.label + ' (' + e.kind + ')';
      path.appendChild(title);
      e.el = path;
      edgeLayer.appendChild(path);
    });
    svg.appendChild(edgeLayer);

    var nodeLayer = el('g', { 'class': 'nodes' });
    // Layer kartu dipasang lebih dulu supaya kartu yang sedang disusun bisa diukur
    // lebarnya (getComputedTextLength hanya bekerja pada elemen yang menempel di dokumen).
    svg.appendChild(nodeLayer);
    nodes.forEach(function (n) {
      var obj = n.obj;
      var jumlah = state.mode === 'subfitur'
        ? (obj.operations || []).length
        : (obj.subfeatures || []).length;
      var g = el('g', {
        'class': 'node',
        'data-id': n.id,
        'data-kind': n.kind,
        'data-status': obj.status,
        tabindex: '0',
        role: 'button',
        'aria-label': n.label + ', ' + groupLabel(n.group) + ', status ' + obj.status +
          ', ' + jumlah + (state.mode === 'subfitur' ? ' operasi' : ' sub-fitur')
      });
      nodeLayer.appendChild(g);
      // Kartu sub-fitur yang punya pekerjaan berjalan memakai persen terbesarnya sebagai tanda.
      // Hanya mode sub-fitur: di mode lain `n.id` adalah produk atau aplikasi, bukan rumah tugas.
      if (state.mode === 'subfitur' && typeof PROGRES_AKTIF[n.id] !== 'undefined') {
        var tanda = el('text', {
          x: (n.x + n.w - 8).toFixed(1),
          y: (n.y + 16).toFixed(1),
          'text-anchor': 'end',
          'class': 'node-progres'
        });
        tanda.textContent = PROGRES_AKTIF[n.id] + '%';
        g.appendChild(tanda);
      }
      g.appendChild(el('rect', {
        x: n.x.toFixed(1),
        y: n.y.toFixed(1),
        width: n.w.toFixed(1),
        height: String(n.h),
        rx: '10',
        'class': 'node-kotak'
      }));
      g.appendChild(el('rect', {
        x: n.x.toFixed(1),
        y: (n.y + 10).toFixed(1),
        width: '4',
        height: String(n.h - 20),
        rx: '2',
        'class': 'node-aksen',
        fill: statusColor[obj.status] || '#8a8a86'
      }));
      g.appendChild(el('rect', {
        x: (n.x + 11).toFixed(1),
        y: (n.y + (n.h - 34) / 2).toFixed(1),
        width: '34',
        height: '34',
        rx: '10',
        'class': 'node-tile',
        fill: n.color
      }));
      var ikonNode = gambarIkon(g, n.ikon, n.x + 16, n.y + (n.h - 24) / 2, 24);
      if (ikonNode) { ikonNode.style.stroke = n.color; }
      var teksX = n.x + 56;
      var pillLebar = 16 + String(jumlah).length * 11;
      // Ruang teks menyisakan tepi kanan kartu dan pill-nya, jadi keduanya tidak pernah
      // berebut tempat: label dipotong, bukan ditutupi atau didorong keluar kartu.
      var ruangTeks = n.w - 56 - 12 - pillLebar - 10;
      var teks = el('text', {
        x: teksX.toFixed(1),
        y: (n.y + n.h / 2).toFixed(1),
        'class': 'node-teks'
      });
      teks.textContent = n.label;
      g.appendChild(teks);
      var labelTampil = potongLabel(teks, n.label, ruangTeks);
      var teksLebar = Math.min(lebarTeks(teks, labelTampil), ruangTeks);
      var pillX = Math.min(teksX + teksLebar + 10, n.x + n.w - 12 - pillLebar);
      var kotakPill = el('rect', {
        x: pillX.toFixed(1),
        y: (n.y + (n.h - 24) / 2).toFixed(1),
        width: String(pillLebar),
        height: '24',
        rx: '12',
        'class': 'node-pill'
      });
      var angka = el('text', {
        x: (pillX + pillLebar / 2).toFixed(1),
        y: (n.y + n.h / 2).toFixed(1),
        'text-anchor': 'middle',
        'class': 'node-jumlah'
      });
      angka.textContent = String(jumlah);
      g.insertBefore(kotakPill, teks);
      g.insertBefore(angka, teks);
      var title = el('title');
      title.textContent = n.label + '\n' + groupLabel(n.group) +
        '\nStatus: ' + obj.status + '\n' +
        (state.mode === 'subfitur' ? 'Operasi: ' : 'Sub-fitur: ') + jumlah;
      g.appendChild(title);
      n.el = g;
      n.jumlahTotal = jumlah;
      n.judulAwal = title.textContent;
    });
    applyView();
  }

  function applyView() {
    svg.setAttribute('viewBox', view.x.toFixed(1) + ' ' + view.y.toFixed(1) + ' ' + view.w.toFixed(1) + ' ' + view.h.toFixed(1));
    if (zoomTingkat) {
      zoomTingkat.textContent = Math.round((zoomDasar / view.w) * 100) + '%';
    }
    // Zoom semantik. Saat lebar kanvas belum terbaca (laci tertutup, display:none)
    // skala tidak boleh dihitung: hasilnya nol dan semua kartu akan ikut menyusut.
    var lebarKanvas = svg.getBoundingClientRect().width;
    svg.classList.toggle('simpl', lebarKanvas > 0 && lebarKanvas / view.w < SKALA_SEDERHANA);
    perbaruiMinimap();
    simpanTampilan();
  }

  function perbaruiMinimap() {
    if (!miniViewport) { return; }
    var x1 = Math.max(0, view.x);
    var y1 = Math.max(0, view.y);
    var x2 = Math.min(W, view.x + view.w);
    var y2 = Math.min(H, view.y + view.h);
    miniViewport.setAttribute('x', Math.max(0, x1).toFixed(1));
    miniViewport.setAttribute('y', Math.max(0, y1).toFixed(1));
    miniViewport.setAttribute('width', Math.max(0, x2 - x1).toFixed(1));
    miniViewport.setAttribute('height', Math.max(0, y2 - y1).toFixed(1));
  }

  function gambarMinimap() {
    if (!minimap) { return; }
    minimap.setAttribute('viewBox', '0 0 ' + W + ' ' + H);
    while (minimap.firstChild) { minimap.removeChild(minimap.firstChild); }
    var gEdge = el('g');
    edges.forEach(function (e) {
      var a = posisi[e.from];
      var b = posisi[e.to];
      if (!a || !b) { return; }
      gEdge.appendChild(el('line', {
        x1: tcx(a).toFixed(1),
        y1: tcy(a).toFixed(1),
        x2: tcx(b).toFixed(1),
        y2: tcy(b).toFixed(1),
        'class': 'mini-edge',
        stroke: kindColor[e.kind] || '#8a8a86',
        'stroke-width': '4'
      }));
    });
    minimap.appendChild(gEdge);
    var gNode = el('g');
    nodes.forEach(function (n) {
      gNode.appendChild(el('rect', {
        x: n.x.toFixed(1),
        y: n.y.toFixed(1),
        width: n.w.toFixed(1),
        height: String(n.h),
        rx: '8',
        'class': 'mini-node',
        fill: n.color
      }));
    });
    minimap.appendChild(gNode);
    miniViewport = el('rect', {
      'class': 'mini-viewport',
      x: '0',
      y: '0',
      width: String(W),
      height: String(H),
      rx: '10'
    });
    minimap.appendChild(miniViewport);
    perbaruiMinimap();
  }

  function pusatDariMini(event) {
    if (!minimap) { return; }
    var rect = minimap.getBoundingClientRect();
    if (rect.width < 2 || rect.height < 2) { return; }
    var gx = ((event.clientX - rect.left) / rect.width) * W;
    var gy = ((event.clientY - rect.top) / rect.height) * H;
    view.x = gx - view.w / 2;
    view.y = gy - view.h / 2;
    applyView();
  }

  if (minimap) {
    var miniSeret = false;
    minimap.addEventListener('pointerdown', function (event) {
      event.preventDefault();
      miniSeret = true;
      try { minimap.setPointerCapture(event.pointerId); } catch (e) {}
      pusatDariMini(event);
    });
    minimap.addEventListener('pointermove', function (event) {
      if (miniSeret) { pusatDariMini(event); }
    });
    function selesaiMini(event) {
      miniSeret = false;
      try { minimap.releasePointerCapture(event.pointerId); } catch (e) {}
    }
    minimap.addEventListener('pointerup', selesaiMini);
    minimap.addEventListener('pointercancel', selesaiMini);
  }

  function resetView() {
    view = { x: 0, y: 0, w: W, h: H };
    applyView();
  }

  function rasio() {
    var rect = svg.getBoundingClientRect();
    return rect.width > 0 && rect.height > 0 ? rect.width / rect.height : W / H;
  }

  function batasLebar() {
    var rect = svg.getBoundingClientRect();
    if (rect.width <= 0) { return W * 1.35; }
    // Skala layar = lebar kanvas / lebar viewBox. Batasi supaya teks kartu tidak menyusut
    // di bawah batas baca; sebagian grafik boleh keluar layar dan digeser lewat minimap.
    return Math.min(W * 1.35, rect.width / SKALA_MIN);
  }

  // Batas untuk zoom manual, bukan untuk tampilan pas. Tanpa ini, graf yang lebih lebar
  // dari batasLebar() langsung mentok saat fit sehingga roda mouse dan tombol perkecil
  // menjadi no-op dan angka persen berhenti di 100%.
  function batasLebarJelajah() {
    var batas = batasLebar();
    var persen = ZOOM_KELUAR_MIN[state.mode];
    if (!persen) { return batas; }
    return Math.max(batas, zoomDasar / persen);
  }

  function sesuaikanTampilan() {
    if (!nodes.length) { resetView(); return; }
    var minX = Infinity;
    var maxX = -Infinity;
    var minY = Infinity;
    var maxY = -Infinity;
    nodes.forEach(function (n) {
      minX = Math.min(minX, n.x);
      maxX = Math.max(maxX, n.x + n.w);
      minY = Math.min(minY, n.y);
      maxY = Math.max(maxY, n.y + n.h);
    });
    var w = Math.max(1, maxX - minX) + 120;
    var h = Math.max(1, maxY - minY) + 120;
    var r = rasio();
    if (w / h < r) { w = h * r; } else { h = w / r; }
    w = Math.min(w, batasLebar());
    h = w / r;
    // Kalau jendela lebih kecil dari kontennya, titik tengah bbox bisa jatuh ke lubang
    // (peta bertingkat menumpuk kartunya di kolom kiri). Kandidat pusatnya adalah
    // pusat setiap kartu, dan yang dipilih adalah yang memuat kartu utuh terbanyak.
    function jepitPusat(pusat, a, b, jendela) {
      if (jendela >= b - a) { return (a + b) / 2; }
      return Math.max(a + jendela / 2, Math.min(b - jendela / 2, pusat));
    }
    var pusatX = (minX + maxX) / 2;
    var pusatY = (minY + maxY) / 2;
    if (w < maxX - minX || h < maxY - minY) {
      var skorTerbaik = -1;
      nodes.forEach(function (calon) {
        var vx = jepitPusat(calon.x + calon.w / 2, minX, maxX, w) - w / 2;
        var vy = jepitPusat(calon.y + calon.h / 2, minY, maxY, h) - h / 2;
        var s = 0;
        nodes.forEach(function (n) {
          if (n.x >= vx && n.x + n.w <= vx + w && n.y >= vy && n.y + n.h <= vy + h) { s += 1; }
        });
        if (s > skorTerbaik) {
          skorTerbaik = s;
          pusatX = vx + w / 2;
          pusatY = vy + h / 2;
        }
      });
    }
    view = {
      x: pusatX - w / 2,
      y: pusatY - h / 2,
      w: w,
      h: h
    };
    zoomDasar = w;
    applyView();
  }

  function wheelZoom(event) {
    event.preventDefault();
    var rect = svg.getBoundingClientRect();
    var px = view.x + ((event.clientX - rect.left) / rect.width) * view.w;
    var py = view.y + ((event.clientY - rect.top) / rect.height) * view.h;
    var faktor = event.deltaY < 0 ? 0.86 : 1.16;
    var baruW = Math.max(W / 8, Math.min(batasLebarJelajah(), view.w * faktor));
    var baruH = baruW / rasio();
    view.x = px - (px - view.x) * (baruW / view.w);
    view.y = py - (py - view.y) * (baruH / view.h);
    view.w = baruW;
    view.h = baruH;
    applyView();
  }

  var seret = null;
  svg.addEventListener('pointerdown', function (event) {
    if (event.target.closest && event.target.closest('.node')) { return; }
    seret = {
      x: event.clientX,
      y: event.clientY,
      vx: view.x,
      vy: view.y,
      w: view.w,
      h: view.h
    };
    svg.classList.add('menyeret');
    try { svg.setPointerCapture(event.pointerId); } catch (e) {}
  });
  svg.addEventListener('pointermove', function (event) {
    if (!seret) { return; }
    var rect = svg.getBoundingClientRect();
    view.x = seret.vx - ((event.clientX - seret.x) / rect.width) * seret.w;
    view.y = seret.vy - ((event.clientY - seret.y) / rect.height) * seret.h;
    applyView();
  });
  function selesaiSeret(event) {
    if (!seret) { return; }
    seret = null;
    svg.classList.remove('menyeret');
    try { svg.releasePointerCapture(event.pointerId); } catch (e) {}
  }
  svg.addEventListener('pointerup', selesaiSeret);
  svg.addEventListener('pointercancel', selesaiSeret);
  svg.addEventListener('wheel', wheelZoom, { passive: false });

  document.querySelectorAll('[data-zoom]').forEach(function (tombol) {
    tombol.addEventListener('click', function () {
      if (tombol.dataset.zoom === 'reset') { sesuaikanTampilan(); return; }
      var faktor = tombol.dataset.zoom === 'in' ? 0.8 : 1.25;
      var cx = view.x + view.w / 2;
      var cy = view.y + view.h / 2;
      var baruW = Math.max(W / 8, Math.min(batasLebarJelajah(), view.w * faktor));
      var baruH = baruW / rasio();
      view.x = cx - baruW / 2;
      view.y = cy - baruH / 2;
      view.w = baruW;
      view.h = baruH;
      applyView();
    });
  });
  // Lepas semua penyaring yang membuat kartu diredup, dan sinkronkan kontrolnya.
  function lepasPenyaring() {
    if (filterStatus) { filterStatus.value = 'semua'; }
    if (filterService) { filterService.value = 'semua'; }
    if (cari) { cari.value = ''; }
    state.status = 'semua';
    state.service = 'semua';
    state.query = '';
  }

  document.querySelectorAll('[data-aksi="semua"]').forEach(function (tombol) {
    tombol.addEventListener('click', function () {
      // Janji tombolnya "tampilkan semua node", jadi penyaring yang meredupkan kartu harus
      // dilepas juga -- bukan fokusnya saja, dan namanya tidak diubah.
      lepasPenyaring();
      bersihkanFokus();
      terapkanFilter();
      sesuaikanTampilan();
      simpanTampilan();
    });
  });
  document.querySelectorAll('[data-aksi="reset"]').forEach(function (tombol) {
    tombol.addEventListener('click', function () {
      lepasPenyaring();
      document.querySelectorAll('#pohon-isi .pohon-grup').forEach(function (g) {
        g.classList.remove('tertutup');
        var kepala = g.querySelector('.pohon-grup-tombol');
        if (kepala) { kepala.setAttribute('aria-expanded', 'true'); }
      });
      aturKecil(false);
      bersihkanFokus();
      tampilkanAwal();
      terapkanFilter();
      sesuaikanTampilan();
    });
  });
  window.addEventListener('resize', ikutRasio);

  // ---------------------------------------------------------------------------
  // Muat data: reload lewat server yang membaca manifest ulang, lalu kembali ke
  // tempat kerja yang sama. Angka jumlah di header dan label penyaring dibaking
  // Rust (main.rs), jadi memuat data berarti memuat halaman, bukan tambal JS.
  // ---------------------------------------------------------------------------
  var kap = document.getElementById('kap');
  var capMuat = kap ? kap.getAttribute('data-cap') : null;

  function simpanTampilanSekarang() {
    var butir = {
      mode: state.mode,
      status: state.status,
      service: state.service,
      query: state.query,
      pilih: detailTerakhir ? { kind: detailTerakhir.kind, id: detailTerakhir.id } : null,
      view: { x: view.x, y: view.y, w: view.w, h: view.h },
      pohon: pohonTerbuka(),
      kecil: !!(pohon && pohon.classList.contains('kecil'))
    };
    try { sessionStorage.setItem(KUNCI_SIMPAN, JSON.stringify(butir)); }
    catch (e) { /* mode privat browser: simpanan boleh hilang, halaman tidak */ }
  }

  var jedaSimpan = null;
  function simpanTampilan() {
    if (!dilayani) { return; }
    if (jedaSimpan) { clearTimeout(jedaSimpan); }
    // Geser dan zoom menembak puluhan kali per detik; tulis sekali setelah diam.
    jedaSimpan = setTimeout(function () { jedaSimpan = null; simpanTampilanSekarang(); }, 250);
  }

  function bacaSimpanan() {
    if (!dilayani) { return null; }
    var teks = null;
    try { teks = sessionStorage.getItem(KUNCI_SIMPAN); } catch (e) { return null; }
    if (!teks) { return null; }
    var butir = null;
    try { butir = JSON.parse(teks); } catch (e) { return null; }
    return butir && typeof butir === 'object' ? butir : null;
  }

  function adaOpsi(pilih, nilai) {
    if (!pilih || !nilai) { return false; }
    return Array.prototype.some.call(pilih.options, function (o) { return o.value === nilai; });
  }

  function pulihkanTampilan() {
    var butir = bacaSimpanan();
    if (!butir) { aktifkanMode('produk'); return; }
    // Nilai yang sudah tidak ada di manifest (service diganti, status dihapus) dibuang,
    // kalau tidak halaman akan menyaring ke nol dan terlihat rusak.
    state.status = adaOpsi(filterStatus, butir.status) ? butir.status : 'semua';
    state.service = adaOpsi(filterService, butir.service) ? butir.service : 'semua';
    state.query = typeof butir.query === 'string' ? butir.query : '';
    if (filterStatus) { filterStatus.value = state.status; }
    if (filterService) { filterService.value = state.service; }
    cari.value = state.query;
    aktifkanMode(MODES.indexOf(butir.mode) >= 0 ? butir.mode : 'produk');
    if (butir.pohon) { aturPohon(true); }
    if (butir.kecil) { aturKecil(true); }
    if (butir.pilih && butir.pilih.id) { tampilkanDetail(butir.pilih.kind, butir.pilih.id); }
    var v = butir.view;
    if (v && typeof v.w === 'number' && v.w > 0) {
      view.x = v.x; view.y = v.y; view.w = v.w; view.h = v.h;
      applyView();
    }
  }

  function tampilkanKap(teks, judul, bahaya) {
    if (!kap) { return; }
    kap.textContent = teks;
    kap.title = judul || teks;
    kap.hidden = false;
    kap.classList.toggle('bahaya', !!bahaya);
  }

  function sembunyikanKap() {
    if (!kap) { return; }
    kap.hidden = true;
    kap.textContent = '';
    kap.removeAttribute('title');
    kap.classList.remove('bahaya');
  }

  function muatData() {
    simpanTampilanSekarang();
    location.reload();
  }

  // Nama aksi harus menggambarkan apa yang sungguh terjadi. Manifest hanya bisa dibaca ulang
  // oleh proses alat ini; selama probe /stamp tidak menjawab, yang dilakukan tombol ini
  // semata-mata memuat ulang halaman.
  function labelMuatData(manifestTerbaca) {
    document.querySelectorAll('[data-aksi="muat-data"]').forEach(function (tombol) {
      if (tombol.classList.contains('kap')) { return; }
      tombol.setAttribute('aria-label', manifestTerbaca ? 'Muat ulang data dari manifest' : 'Muat ulang halaman');
      if (!dilayani) {
        tombol.title = 'Hanya bekerja saat halaman disajikan: cargo run -- serve';
        return;
      }
      tombol.title = manifestTerbaca ? 'Muat data' : 'Muat ulang halaman';
    });
  }

  document.querySelectorAll('[data-aksi="muat-data"]').forEach(function (tombol) {
    tombol.addEventListener('click', function () {
      if (!dilayani) {
        tampilkanKap(
          'Buka lewat feature-view serve',
          'Halaman dari disk tidak boleh membaca file. Jalankan dari folder alat ini:\n' +
          'cargo run -- serve\nlalu buka http://127.0.0.1:8787/ (port bawaan, ubah dengan PETA_PORT atau --port)'
        );
        setTimeout(sembunyikanKap, 6000);
        return;
      }
      muatData();
    });
  });

  // Chip perubahan tidak cukup dimuat ulang di bawah opsi B: halamannya adalah isi file,
  // jadi yang harus dilakukan adalah menulis file itu dulu.
  document.querySelectorAll('[data-aksi="muat-ubah"]').forEach(function (tombol) {
    tombol.addEventListener('click', function () {
      jalanGenerate();
    });
  });

  function jalanGenerate() {
    kirimAksi('/regenerate', function (hasil) {
      tahanKap(20000);
      if (!hasil.ok) {
        tampilkanKap('Generate gagal', hasil.teks, true);
        return;
      }
      tampilkanKap('Selesai generate', hasil.teks);
      simpanTampilanSekarang();
      setTimeout(function () { location.reload(); }, 500);
    });
  }

  var jagaBeruntunGagal = 0;
  var jagaJeda = null;
  // Laporan aksi alat (hapus/generate) harus sempat dibaca: polling 4 detik akan
  // menimpa chip dengan hasil cap-nya sendiri kalau tidak ditahan.
  var kapTahan = 0;

  function tahanKap(miliDetik) {
    kapTahan = Date.now() + miliDetik;
  }

  function kirimAksi(path, selesai) {
    fetch(path, { method: 'POST' }).then(function (r) {
      return r.text().then(function (teks) {
        return { ok: r.ok, kode: r.status, teks: teks.trim() };
      });
    }).then(selesai).catch(function (e) {
      tahanKap(20000);
      tampilkanKap('Alat tidak menjawab', String(e), true);
    });
  }

  var hapusHasil = document.getElementById('hapus-hasil');
  var generateUlang = document.getElementById('generate-ulang');
  if (dilayani) {
    if (hapusHasil) { hapusHasil.hidden = false; }
    if (generateUlang) { generateUlang.hidden = false; }
  }
  if (hapusHasil) {
    hapusHasil.addEventListener('click', function () {
      var boleh = window.confirm(
        'Hapus docs/view/index.html?\n\n' +
        'Ini hanya hasil generate. Seluruh manifest TSV di docs/view tidak disentuh, dan tombol ' +
        'Generate ulang mengembalikannya byte per byte.'
      );
      if (!boleh) { return; }
      kirimAksi('/hapus', function (hasil) {
        if (!hasil.ok) {
          tahanKap(20000);
          tampilkanKap('GAGAL menghapus', hasil.teks, true);
          return;
        }
        // Reload supaya hilangnya file benar-benar kelihatan: halaman kini adalah isi disk,
        // jadi yang muncul berikutnya adalah halaman "belum di-generate".
        location.reload();
      });
    });
  }
  if (generateUlang) {
    generateUlang.addEventListener('click', function () {
      jalanGenerate();
    });
  }

  function mulaiPenjagaan() {
    if (!dilayani || !kap) { return; }
    jagaJeda = setInterval(function () {
      fetch('/stamp', { cache: 'no-store' }).then(function (r) {
        if (!r.ok) { throw new Error('stamp ' + r.status); }
        return r.json();
      }).then(function (b) {
        jagaBeruntunGagal = 0;
        labelMuatData(true);
        if (Date.now() < kapTahan) { return; }
        if (!b || typeof b.cap !== 'string') { return; }
        if (b.cap === capMuat) { if (!kap.classList.contains('bahaya')) { sembunyikanKap(); } return; }
        if (!b.valid) {
          tampilkanKap('Manifest gagal validasi', (b.galat || []).join('\n') || 'manifest ditolak', true);
          return;
        }
        tampilkanKap('Data berubah · generate ulang', 'cap halaman ' + capMuat + '\ncap manifest ' + b.cap);
      }).catch(function (e) {
        jagaBeruntunGagal += 1;
        if (jagaBeruntunGagal < 2) { return; }
        // Server mati bukan alasan untuk mengulang permintaan tiap empat detik.
        labelMuatData(false);
        clearInterval(jagaJeda);
        console.warn('Penjagaan manifest dihentikan: ' + e);
      });
    }, 4000);
  }

  function ikutRasio() {
    var r = rasio();
    view.h = view.w / r;
    applyView();
  }

  // Ujung panah memakai jenis relasi garisnya sendiri; garis yang diredup atau diburamkan
  // pindah ke tingkap pudar supaya kepalanya tidak tetap menyala sendirian.
  function jenisMarker(e) {
    var jenis = e.dataset.jenis && kindColor[e.dataset.jenis] ? e.dataset.jenis : 'netral';
    var pudar = (e.classList.contains('redup') || e.classList.contains('buram')) ? '-pudar' : '';
    return 'url(#panah-' + jenis + pudar + ')';
  }

  // Redup adalah akibat penyaring, bukan nasib kartu terpilih: kartu yang sedang dibuka di
  // panel inspektur dan garis yang terkait dengannya tidak boleh ikut diredup.
  function sinkronRedup() {
    document.querySelectorAll('#jejaring .node').forEach(function (n) {
      var node = posisi[n.dataset.id];
      var lolos = !node || node._ok !== false;
      n.classList.toggle('redup', !lolos && !n.classList.contains('terpilih'));
    });
    document.querySelectorAll('#jejaring .edge').forEach(function (e) {
      var a = posisi[e.dataset.from];
      var b = posisi[e.dataset.to];
      var lolos = (!a || a._ok !== false) && (!b || b._ok !== false);
      var terkait = e.classList.contains('terkait');
      e.classList.toggle('redup', !lolos && !terkait);
      e.setAttribute('marker-end', jenisMarker(e));
    });
  }

  function bersihkanFokus() {
    selectedId = null;
    document.querySelectorAll('#jejaring .node').forEach(function (n) {
      n.classList.remove('terpilih');
      n.classList.remove('buram');
      n.classList.remove('fokus-terkait');
    });
    document.querySelectorAll('#jejaring .edge').forEach(function (e) {
      e.classList.remove('terkait');
      e.classList.remove('buram');
    });
    sinkronRedup();
  }

  function fokusNode(id) {
    selectedId = id;
    var tetangga = peta();
    edges.forEach(function (e) {
      if (e.from === id) { tetangga[e.to] = true; }
      if (e.to === id) { tetangga[e.from] = true; }
    });
    document.querySelectorAll('#jejaring .node').forEach(function (n) {
      var terpilih = n.dataset.id === id;
      var terkait = !!tetangga[n.dataset.id];
      n.classList.toggle('terpilih', terpilih);
      n.classList.toggle('fokus-terkait', !terpilih && terkait);
      n.classList.toggle('buram', !terpilih && !terkait);
    });
    document.querySelectorAll('#jejaring .edge').forEach(function (e) {
      var terkait = e.dataset.from === id || e.dataset.to === id;
      e.classList.toggle('terkait', terkait);
      e.classList.toggle('buram', !terkait);
    });
    sinkronRedup();
    pusatkanNode(id);
  }

  function badgeStatus(status) {
    return '<span class="badge-status badge-' + esc(status) + '">' + esc(status) + '</span>';
  }

  function pilih(kind, id, label) {
    return '<button type="button" class="tautan-node tautan-pilih" data-pilih-kind="' +
      esc(kind) + '" data-pilih-id="' + esc(id) + '">' + esc(label) + '</button>';
  }

  function dokumenList(paths) {
    if (!paths || !paths.length) { return '<li>Belum ada dokumen khusus.</li>'; }
    return paths.map(function (p) {
      return '<li><a href="../../' + encodeURI(p).replace(/#/g, '%23') + '"><code>' + esc(p) + '</code></a></li>';
    }).join('');
  }

  function buktiList(paths) {
    return paths.map(function (p) {
      return '<li><a href="../../' + encodeURI(p).replace(/#/g, '%23') + '"><code>' + esc(p) + '</code></a></li>';
    }).join('');
  }

  function daftarRelasiSubfitur(id) {
    var keluar = data.edges.filter(function (e) { return e.from === id; });
    var masuk = data.edges.filter(function (e) { return e.to === id; });
    function baris(daftar, arah) {
      if (!daftar.length) { return '<p class="detail-kecil">Tidak ada relasi ' + arah + '.</p>'; }
      return '<ul class="relasi-list">' + daftar.map(function (e) {
        var lain = arah === 'keluar' ? byFeature[e.to] : byFeature[e.from];
        return '<li><span class="badge-service">' + esc(e.kind) + '</span> ' +
          pilih('subfitur', lain.id, lain.name) +
          '<span class="relasi-label">' + esc(e.label) + '</span></li>';
      }).join('') + '</ul>';
    }
    return '<p class="bagian-sub">Keluar</p>' + baris(keluar, 'keluar') +
      '<p class="bagian-sub">Masuk</p>' + baris(masuk, 'masuk');
  }

  function bagian(icon, judul, isi, buka) {
    return '<details class="bagian"' + (buka ? ' open' : '') + '>' +
      '<summary>' + ikonHtml(icon) + '<span>' + esc(judul) + '</span>' + ikonHtml('panah', 'chevron') + '</summary>' +
      '<div class="bagian-isi">' + isi + '</div></details>';
  }

  function kepala(icon, judul, badge, warna) {
    return '<div class="detail-kepala"' + (warna ? ' style="--grup-warna:' + warna + '"' : '') + '>' +
      '<span class="tile-kepala">' + ikonHtml(icon) + '</span>' +
      '<div><h2>' + esc(judul) + '</h2><div class="detail-badge">' + badge + '</div></div></div>';
  }

  function jejak(parts) {
    return '<div class="jejak">' + parts.join(ikonHtml('panah', 'pemisah')) + '</div>';
  }

  function tampilkanDetail(kind, id) {
    detailTerakhir = { kind: kind, id: id };
    simpanTampilan();
    if (kind === 'produk') {
      var p = byProduct[id];
      if (!p) { return; }
      var subs = p.subfeatures.map(function (fid) {
        var f = byFeature[fid];
        var a = byApp[f.app];
        return '<li>' + pilih('subfitur', fid, f.name) + ' ' + badgeStatus(f.status) +
          ' <span class="badge-service">' + esc(a ? a.name : f.app) + '</span></li>';
      }).join('');
      var apps = p.apps.map(function (aid) {
        var a = byApp[aid];
        return '<li>' + pilih('app', aid, a ? a.name : aid) +
          ' <span class="badge-service">' + esc(a ? a.service : '') + '</span></li>';
      }).join('');
      panel.innerHTML =
        jejak([ikonHtml('produk') + '<span>Produk</span>', '<span class="jejak-aktif">' + esc(p.name) + '</span>']) +
        kepala(ikonProduk(p), p.name, '<span class="badge-service">' + esc(p.actor) + '</span>' + badgeStatus(p.status),
          actorColor[p.actor]) +
        '<p class="ringkas">' + esc(p.summary) + '</p>' +
        '<p class="kunci">' + ikonHtml('hasil') + '<span>' + esc(p.outcome) + '</span></p>' +
        bagian('subfitur', 'Sub-fitur', '<ul class="dot-list">' + subs + '</ul>', true) +
        bagian('aplikasi', 'Aplikasi pelaksana', '<ul class="dot-list">' + apps + '</ul>', false) +
        bagian('berkas', 'Dokumentasi', '<ul class="tautan-list">' + dokumenList(p.docs) + '</ul>', false) +
        (p.notes ? bagian('catatan', 'Catatan', '<p>' + esc(p.notes) + '</p>', false) : '');
      tandaiPohon('produk', id);
      if (posisi[id]) { fokusNode(id); } else { bersihkanFokus(); }
      return;
    }

    if (kind === 'app') {
      var a = byApp[id];
      if (!a) { return; }
      var s = serviceOf[a.service];
      var fiturApp = a.subfeatures.map(function (fid) {
        var f = byFeature[fid];
        var operasi = f.operations.length
          ? '<ul class="operasi-list">' + f.operations.map(function (o) {
            return '<li><code>' + esc(o) + '</code></li>';
          }).join('') + '</ul>'
          : '';
        return '<li>' + pilih('subfitur', fid, f.name) + ' ' + badgeStatus(f.status) + operasi + '</li>';
      }).join('');
      var produkApp = a.products.map(function (pid) {
        return '<li>' + pilih('produk', pid, byProduct[pid] ? byProduct[pid].name : pid) + '</li>';
      }).join('');
      var namaService = s ? s.name : a.service;
      var rantai = [ikonHtml('implementasi') + '<span>Implementasi</span>', '<span>' + esc(namaService) + '</span>'];
      if (a.name !== namaService) {
        rantai.push('<span class="jejak-aktif">' + esc(a.name) + '</span>');
      }
      panel.innerHTML =
        jejak(rantai) +
        kepala(ikonAplikasi(a), a.name, '<span class="badge-service">' + esc(a.kind) + '</span>' + badgeStatus(a.status),
          colorOf[a.service]) +
        '<p class="ringkas">' + esc(a.description) + '</p>' +
        bagian('subfitur', 'Sub-fitur dan operasi', '<ul class="daftar-sub">' + fiturApp + '</ul>', true) +
        bagian('produk', 'Produk dilayani', '<ul class="dot-list">' + produkApp + '</ul>', false) +
        bagian('folder', 'Folder', '<p><code>' + esc(a.folder) + '</code></p>', false) +
        bagian('berkas', 'Dokumentasi', '<ul class="tautan-list">' + dokumenList(a.docs) + '</ul>', false);
      tandaiPohon('app', id);
      if (posisi[id]) { fokusNode(id); } else { bersihkanFokus(); }
      return;
    }

    var f = byFeature[id];
    if (!f) { return; }
    var produk = byProduct[f.product];
    var aplikasi = byApp[f.app];
    var operasiSub = f.operations.length
      ? '<ul class="operasi-list">' + f.operations.map(function (o) {
        return '<li><code>' + esc(o) + '</code></li>';
      }).join('') + '</ul>'
      : '<p class="detail-kecil">Tidak ada operasi tercatat.</p>';
    panel.innerHTML =
      jejak([
        pilih('produk', f.product, produk ? produk.name : f.product),
        pilih('app', f.app, aplikasi ? aplikasi.name : f.app),
        '<span class="jejak-aktif">' + esc(f.name) + '</span>'
      ]) +
      kepala('subfitur', f.name, badgeStatus(f.status), colorOf[f.service]) +
      '<p class="ringkas">' + esc(f.summary) + '</p>' +
      bagian('kode', 'Bukti kode', '<ul class="tautan-list">' + buktiList(f.evidence) + '</ul>', true) +
      bagian('relasi', 'Relasi', daftarRelasiSubfitur(id), true) +
      bagian('operasi', 'Operasi', operasiSub, false) +
      bagian('masuk', 'Titik masuk', '<p><code>' + esc(f.entry) + '</code></p>', false) +
      bagian('panel', 'Cara kerja', '<p>' + esc(f.how) + '</p>', false) +
      bagian('berkas', 'Dokumentasi', '<ul class="tautan-list">' + dokumenList((produk ? produk.docs : []).concat(f.docs || [])) + '</ul>', false) +
      (f.notes ? bagian('catatan', 'Catatan', '<p>' + esc(f.notes) + '</p>', false) : '');
    tandaiPohon('subfitur', id);
    // Yang disorot di kanvas adalah simpul yang benar-benar ada di mode itu: di mode
    // Sub-fitur sub-fiturnya sendiri, di mode lain induknya (produk atau aplikasi).
    var sorot = state.mode === 'subfitur' ? id : (state.mode === 'produk' ? f.product : f.app);
    if (posisi[sorot]) { fokusNode(sorot); } else { bersihkanFokus(); }
  }

  function tampilkanAwal() {
    detailTerakhir = null;
    simpanTampilan();
    tandaiPohon('', '');
    var pesan = state.mode === 'subfitur' ? 'Pilih sub-fitur' : state.mode === 'produk' ? 'Pilih produk' : 'Pilih aplikasi';
    var ikon = state.mode === 'subfitur' ? 'subfitur' : state.mode === 'produk' ? 'produk' : 'implementasi';
    panel.innerHTML = '<div class="detail-kosong">' +
      ikonHtml(ikon, 'ikon-besar') +
      '<p>' + pesan + '</p></div>';
  }

  panel.addEventListener('click', function (event) {
    var tombol = event.target.closest ? event.target.closest('.tautan-pilih') : null;
    if (!tombol) { return; }
    var kind = tombol.dataset.pilihKind;
    var id = tombol.dataset.pilihId;
    if (kind === 'produk') {
      aktifkanMode('produk');
      tampilkanDetail('produk', id);
    } else if (kind === 'app') {
      aktifkanMode('implementasi');
      tampilkanDetail('app', id);
    } else {
      tampilkanDetail('subfitur', id);
    }
  });

  if (pohonIsi) {
    pohonIsi.addEventListener('click', function (event) {
      var tombol = event.target.closest ? event.target.closest('.pohon-pilih,.pohon-sub,.pohon-grup-tombol') : null;
      if (!tombol) { return; }
      if (tombol.classList.contains('pohon-grup-tombol')) {
        var tertutup = tombol.closest('.pohon-grup').classList.toggle('tertutup');
        tombol.setAttribute('aria-expanded', tertutup ? 'false' : 'true');
        return;
      }
      tampilkanDetail(tombol.dataset.pilihKind, tombol.dataset.pilihId);
      aturPohon(false);
    });
  }

  function cocok(n) {
    var o = n.obj;
    var q = state.query.trim().toLowerCase();
    var teks = (o.name || '') + ' ' + (o.actor || '') + ' ' + (o.outcome || '') + ' ' +
      (o.summary || '') + ' ' + (o.kind || '') + ' ' + (o.description || '');
    if (state.mode === 'implementasi' && o.service && serviceOf[o.service]) {
      teks += ' ' + serviceOf[o.service].name;
    }
    (o.subfeatures || []).forEach(function (fid) {
      var f = byFeature[fid];
      if (f) { teks += ' ' + f.name + ' ' + f.how + ' ' + f.entry; }
    });
    if (n.kind === 'subfitur') {
      var induk = byProduct[o.product];
      var a = byApp[o.app];
      teks += ' ' + o.how + ' ' + o.entry + ' ' + (induk ? induk.name : '') + ' ' + (a ? a.name : '');
    }
    var okStatus = state.status === 'semua' || (n.kind === 'subfitur'
      ? o.status === state.status
      : (o.subfeatures || []).some(function (fid) {
        return byFeature[fid] && byFeature[fid].status === state.status;
      }));
    var okService = state.service === 'semua' || (n.kind === 'subfitur'
      ? o.service === state.service
      : state.mode === 'produk'
        ? (o.subfeatures || []).some(function (fid) {
          var f = byFeature[fid];
          return f && byApp[f.app] && byApp[f.app].service === state.service;
        })
        : o.service === state.service);
    var okCari = q === '' || teks.toLowerCase().indexOf(q) !== -1;
    return okStatus && okService && okCari;
  }

  // Sub-fitur (atau operasi) yang benar-benar ikut tersaring, bukan seluruh isi kartu yang
  // lolos: chip tidak boleh menjanjikan angka yang tidak ada di layar.
  function subCocok(node) {
    if (state.mode === 'subfitur') { return (node.obj.operations || []).length; }
    var ada = 0;
    (node.obj.subfeatures || []).forEach(function (fid) {
      var f = byFeature[fid];
      if (f && cocokFitur(f)) { ada += 1; }
    });
    return ada;
  }

  // Kepala klaster DAN pill kartu dihitung ulang atas penyaring yang aktif; kotak klasternya
  // sendiri tidak digeser, hanya angkanya yang tidak boleh lagi menampilkan total hasil
  // generate. Pill kartu ikut di sini karena kalau tidak, satu layar menunjukkan dua angka
  // untuk hal yang sama: klaster "3" berisi kartu "20". Total tetap tidak hilang — ia tinggal
  // di tooltip kartu.
  function angkaKlaster() {
    var hitung = peta();
    nodes.forEach(function (n) {
      if (n._ok !== false) { hitung[n.group] = (hitung[n.group] || 0) + 1; }
    });
    document.querySelectorAll('#jejaring .klaster-jumlah').forEach(function (t) {
      var g = t.getAttribute('data-klaster');
      t.textContent = ' ' + (hitung[g] || 0);
    });
    document.querySelectorAll('#jejaring .node').forEach(function (elNode) {
      var n = posisi[elNode.getAttribute('data-id')];
      if (!n || n.jumlahTotal === undefined) { return; }
      var ada = subCocok(n);
      var angka = elNode.querySelector('.node-jumlah');
      if (angka) { angka.textContent = String(ada); }
      var judul = elNode.querySelector('title');
      if (judul) {
        judul.textContent = ada === n.jumlahTotal
          ? n.judulAwal
          : n.judulAwal + ' (dari ' + n.jumlahTotal + ', ' + ada + ' terlihat saat ini)';
      }
    });
  }

  function terapkanFilter() {
    var tampil = 0;
    var subTampil = 0;
    document.querySelectorAll('#jejaring .node').forEach(function (n) {
      var node = posisi[n.dataset.id];
      if (!node) { return; }
      var ok = cocok(node);
      node._ok = ok;
      n.classList.toggle('cari-cocok', state.query.trim() !== '' && ok);
      if (ok) {
        tampil += 1;
        subTampil += subCocok(node);
      }
    });
    sinkronRedup();
    var jenis = state.mode === 'subfitur' ? 'sub-fitur' : state.mode === 'produk' ? 'produk' : 'aplikasi';
    var ikonJenis = state.mode === 'subfitur' ? 'subfitur' : jenis;
    var bagianJenis = state.mode === 'subfitur' ? 'operasi' : 'sub-fitur';
    var ikonBagian = state.mode === 'subfitur' ? 'operasi' : 'subfitur';
    hasil.innerHTML =
      ikonHtml(ikonJenis) + '<b>' + tampil + '</b><span>/' + nodes.length + '</span>' +
      ikonHtml(ikonBagian) + '<b>' + subTampil + '</b>';
    hasil.title = 'Menampilkan ' + tampil + ' dari ' + nodes.length + ' ' + jenis +
      ' dan ' + subTampil + ' ' + bagianJenis + '.';
    sinkronAngkaOpsi();
    angkaKlaster();
    filterPohon();
  }

  function aktifkanMode(mode) {
    state.mode = mode;
    selectedId = null;
    document.querySelectorAll('[data-mode]').forEach(function (t) {
      var pilihMode = t.dataset.mode === mode;
      t.classList.toggle('terpilih', pilihMode);
      t.setAttribute('aria-pressed', pilihMode ? 'true' : 'false');
    });
    buildGraph();
    draw();
    gambarMinimap();
    // Rail harus terisi sebelum menyesuaikan tampilan: sesuaikanTampilan() membaca lebar
    // kanvas, dan kanvas baru menyempit setelah rail punya isinya. Kalau fit jalan lebih
    // dulu, dasar persen zoom ikut lebar 29 px dan angka persen salah setelah reload.
    bangunPohon();
    sesuaikanTampilan();
    terapkanFilter();
    tampilkanAwal();
    simpanTampilan();
  }

  document.querySelectorAll('[data-mode]').forEach(function (tombol) {
    tombol.addEventListener('click', function () { aktifkanMode(tombol.dataset.mode); });
  });
  if (filterStatus) {
    state.status = filterStatus.value || 'semua';
    filterStatus.addEventListener('change', function () {
      state.status = filterStatus.value;
      terapkanFilter();
      simpanTampilan();
    });
  }
  if (filterService) {
    state.service = filterService.value || 'semua';
    filterService.addEventListener('change', function () {
      state.service = filterService.value;
      terapkanFilter();
      simpanTampilan();
    });
  }
  cari.addEventListener('input', function () {
    state.query = cari.value || '';
    terapkanFilter();
    simpanTampilan();
  });

  function aturPohon(buka) {
    if (!pohon) { return; }
    if (buka) { aturKecil(false); }
    pohon.classList.toggle('terbuka', buka);
    if (tombolPohon) { tombolPohon.setAttribute('aria-expanded', buka ? 'true' : 'false'); }
  }

  function pohonTerbuka() {
    return !!pohon && pohon.classList.contains('terbuka');
  }

  if (tombolPohon) {
    tombolPohon.addEventListener('click', function () { aturPohon(!pohonTerbuka()); });
  }
  if (tutupPohon) {
    tutupPohon.addEventListener('click', function () { aturPohon(false); });
  }

  function grupTertutup() {
    var tertutup = peta();
    document.querySelectorAll('#pohon-isi .pohon-grup').forEach(function (g) {
      var tombol = g.querySelector('.pohon-grup-tombol');
      if (g.classList.contains('tertutup') && tombol) {
        tertutup[tombol.querySelector('span').textContent] = true;
      }
    });
    return tertutup;
  }

  function pulihkanGrup(tertutup) {
    document.querySelectorAll('#pohon-isi .pohon-grup').forEach(function (g) {
      var tombol = g.querySelector('.pohon-grup-tombol');
      if (!tombol) { return; }
      if (!tertutup[tombol.querySelector('span').textContent]) { return; }
      g.classList.add('tertutup');
      tombol.setAttribute('aria-expanded', 'false');
    });
  }

  var kecilPohon = document.getElementById('kecil-pohon');
  function aturKecil(kecil) {
    if (!pohon) { return; }
    pohon.classList.toggle('kecil', kecil);
    if (kecilPohon) {
      var label = kecil ? 'Perlebar struktur' : 'Perkecil struktur';
      kecilPohon.setAttribute('aria-label', label);
      kecilPohon.setAttribute('aria-pressed', kecil ? 'true' : 'false');
      kecilPohon.title = kecil ? 'Perlebar' : 'Perkecil';
    }
    ikutRasio();
    simpanTampilan();
  }
  if (kecilPohon) {
    kecilPohon.addEventListener('click', function () {
      aturKecil(!pohon.classList.contains('kecil'));
    });
  }

  function pusatkanNode(id) {
    var n = posisi[id];
    if (!n) { return; }
    var x = tcx(n);
    var y = tcy(n);
    var utuh = n.x >= view.x && n.x + n.w <= view.x + view.w &&
      n.y >= view.y && n.y + n.h <= view.y + view.h;
    if (utuh) { return; }
    view.x = x - view.w / 2;
    view.y = y - view.h / 2;
    applyView();
  }

  svg.addEventListener('click', function (event) {
    var node = event.target.closest ? event.target.closest('.node') : null;
    if (!node) { return; }
    event.stopPropagation();
    var kind = node.dataset.kind;
    var id = node.dataset.id;
    tampilkanDetail(kind, id);
  });
  svg.addEventListener('keydown', function (event) {
    if (event.key !== 'Enter' && event.key !== ' ') { return; }
    var node = event.target.closest ? event.target.closest('.node') : null;
    if (!node) { return; }
    event.preventDefault();
    tampilkanDetail(node.dataset.kind, node.dataset.id);
  });
  document.addEventListener('keydown', function (event) {
    if (event.key !== 'Escape') { return; }
    if (pohonTerbuka()) { aturPohon(false); return; }
    bersihkanFokus();
  });

  function cocokFitur(f) {
    var q = state.query.trim().toLowerCase();
    return lewatStatusFitur(f, state.status) &&
      lewatServiceFitur(f, state.service) &&
      lewatCariFitur(f, q);
  }

  function lewatStatusFitur(f, status) { return status === 'semua' || f.status === status; }

  function lewatServiceFitur(f, service) { return service === 'semua' || f.service === service; }

  function lewatCariFitur(f, q) {
    var teks = (f.name + ' ' + f.how + ' ' + f.entry).toLowerCase();
    return q === '' || teks.indexOf(q) !== -1;
  }

  // Angka pada pilihan penyaring dibaking saat generate, jadi ia salah begitu dua penyaring
  // digabung. Disegerakan dari data yang sudah ada di halaman, dihitung atas penyaring lain
  // yang sedang aktif (bukan atas dirinya sendiri).
  function tulisOpsi(pilih, jumlah, total) {
    if (!pilih) { return; }
    Array.prototype.forEach.call(pilih.options, function (o) {
      var bagian = o.textContent.split('\u00b7');
      if (bagian.length < 2) { return; }
      var nama = bagian[0].trim();
      var ada = o.value === 'semua' ? total : (jumlah[o.value] || 0);
      o.textContent = nama + ' \u00b7 ' + ada;
    });
  }

  function sinkronAngkaOpsi() {
    var q = state.query.trim().toLowerCase();
    var perStatus = peta();
    var perService = peta();
    var totalStatus = 0;
    var totalService = 0;
    (data.features || []).forEach(function (f) {
      var cariOk = lewatCariFitur(f, q);
      if (cariOk && lewatServiceFitur(f, state.service)) {
        totalStatus += 1;
        perStatus[f.status] = (perStatus[f.status] || 0) + 1;
      }
      if (cariOk && lewatStatusFitur(f, state.status)) {
        totalService += 1;
        perService[f.service] = (perService[f.service] || 0) + 1;
      }
    });
    tulisOpsi(filterStatus, perStatus, totalStatus);
    tulisOpsi(filterService, perService, totalService);
  }

  function buatPohonItem(kind, id, label, status, subfeatures, ikon) {
    var item = document.createElement('div');
    item.className = 'pohon-item';
    var tombol = document.createElement('button');
    tombol.type = 'button';
    tombol.className = 'pohon-pilih';
    tombol.dataset.pilihKind = kind;
    tombol.dataset.pilihId = id;
    tombol.title = label + ' - ' + status;
    var ikonSpan = document.createElement('span');
    ikonSpan.className = 'ikon-bungkus';
    ikonSpan.innerHTML = ikonHtml(ikon || (kind === 'produk' ? 'produk' : 'aplikasi'));
    tombol.appendChild(ikonSpan);
    var teks = document.createElement('span');
    teks.className = 'pohon-nama';
    teks.textContent = label;
    tombol.appendChild(teks);
    var titik = document.createElement('span');
    titik.className = 'status-titik';
    titik.style.background = statusColor[status] || '#8a8a86';
    titik.title = status;
    tombol.appendChild(titik);
    item.appendChild(tombol);
    if (subfeatures && subfeatures.length) {
      var det = document.createElement('details');
      det.className = 'pohon-detail';
      var ringkas = document.createElement('summary');
      ringkas.innerHTML = ikonHtml('panah', 'chevron') +
        '<span class="pohon-angka">' + subfeatures.length + '</span>';
      ringkas.title = subfeatures.length + ' sub-fitur';
      det.appendChild(ringkas);
      var ul = document.createElement('ul');
      subfeatures.forEach(function (fid) {
        var f = byFeature[fid];
        if (!f) { return; }
        var li = document.createElement('li');
        var sub = document.createElement('button');
        sub.type = 'button';
        sub.className = 'pohon-sub';
        sub.dataset.pilihKind = 'subfitur';
        sub.dataset.pilihId = fid;
        sub.title = f.name + ' - ' + f.status;
        var nama = document.createElement('span');
        nama.className = 'pohon-nama';
        nama.textContent = f.name;
        sub.appendChild(nama);
        var titikSub = document.createElement('span');
        titikSub.className = 'status-titik';
        titikSub.style.background = statusColor[f.status] || '#8a8a86';
        sub.appendChild(titikSub);
        li.appendChild(sub);
        ul.appendChild(li);
      });
      det.appendChild(ul);
      item.appendChild(det);
    }
    return item;
  }

  function judulGrup(ikon, nama, jumlah) {
    var judul = document.createElement('h3');
    judul.innerHTML = '<button type="button" class="pohon-grup-tombol" aria-expanded="true" title="Buka atau tutup grup">' +
      ikonHtml('panah', 'chevron') + ikonHtml(ikon) +
      '<span>' + esc(nama) + '</span><b>' + jumlah + '</b></button>';
    return judul;
  }

  function bangunPohon() {
    if (!pohonIsi) { return; }
    var tertutupSebelum = grupTertutup();
    pohonIsi.innerHTML = '';
    if (state.mode === 'subfitur') {
      pohonJudul.textContent = 'Sub-fitur';
      data.services.forEach(function (s) {
        var subs = data.features.filter(function (f) { return f.service === s.key; });
        if (!subs.length) { return; }
        var grup = document.createElement('div');
        grup.className = 'pohon-grup';
        grup.style.setProperty('--grup-warna', colorOf[s.key] || paletGrup(0));
        grup.appendChild(judulGrup(s.ikon || 'layanan', s.name, subs.length));
        subs.forEach(function (f) {
          var item = document.createElement('div');
          item.className = 'pohon-item';
          var tombol = document.createElement('button');
          tombol.type = 'button';
          tombol.className = 'pohon-pilih';
          tombol.dataset.pilihKind = 'subfitur';
          tombol.dataset.pilihId = f.id;
          tombol.title = f.name + ' - ' + f.status;
          tombol.innerHTML = '<span class="ikon-bungkus">' +
            ikonHtml((byApp[f.app] && ikonAplikasi(byApp[f.app])) || 'subfitur') + '</span>' +
            '<span class="pohon-nama">' + esc(f.name) + '</span>' +
            '<span class="status-titik" style="background:' + (statusColor[f.status] || '#8a8a86') + '"></span>';
          item.appendChild(tombol);
          grup.appendChild(item);
        });
        pohonIsi.appendChild(grup);
      });
      pulihkanGrup(tertutupSebelum);
      return;
    }
    if (state.mode === 'produk') {
      pohonJudul.textContent = 'Produk';
      var grupProduk = peta();
      var urutanAktor = [];
      data.products.forEach(function (p) {
        if (!grupProduk[p.actor]) {
          grupProduk[p.actor] = [];
          urutanAktor.push(p.actor);
        }
        grupProduk[p.actor].push(p);
      });
      urutanAktor.forEach(function (aktor) {
        var grup = document.createElement('div');
        grup.className = 'pohon-grup';
        grup.style.setProperty('--grup-warna', actorColor[aktor] || paletGrup(0));
        grup.appendChild(judulGrup(ikonAktor[aktor] || 'produk', aktor, grupProduk[aktor].length));
        grupProduk[aktor].forEach(function (p) {
          grup.appendChild(buatPohonItem('produk', p.id, p.name, p.status, p.subfeatures, ikonProduk(p)));
        });
        pohonIsi.appendChild(grup);
      });
    } else {
      pohonJudul.textContent = 'Implementasi';
      data.services.forEach(function (s) {
        var apps = data.apps.filter(function (a) { return a.service === s.key; });
        if (!apps.length) { return; }
        var grup = document.createElement('div');
        grup.className = 'pohon-grup';
        grup.style.setProperty('--grup-warna', colorOf[s.key] || paletGrup(0));
        grup.appendChild(judulGrup(s.ikon || 'layanan', s.name, apps.length));
        apps.forEach(function (a) {
          grup.appendChild(buatPohonItem('app', a.id, a.name, a.status, a.subfeatures, ikonAplikasi(a)));
        });
        pohonIsi.appendChild(grup);
      });
    }
    pulihkanGrup(tertutupSebelum);
  }

  function tandaiPohon(kind, id) {
    if (!pohonIsi) { return; }
    var target = null;
    pohonIsi.querySelectorAll('.pohon-pilih,.pohon-sub').forEach(function (b) {
      var dipilih = b.dataset.pilihKind === kind && b.dataset.pilihId === id;
      b.classList.toggle('terpilih', dipilih);
      if (dipilih) { target = b; }
    });
    if (target) {
      var det = target.closest('details');
      if (det) { det.open = true; }
    }
  }

  function filterPohon() {
    if (!pohonIsi) { return; }
    pohonIsi.querySelectorAll('.pohon-pilih,.pohon-sub').forEach(function (b) {
      var kind = b.dataset.pilihKind;
      var id = b.dataset.pilihId;
      var ok = true;
      if (kind === 'subfitur') {
        ok = byFeature[id] ? cocokFitur(byFeature[id]) : true;
      } else {
        var node = posisi[id];
        ok = node ? cocok(node) : true;
      }
      b.classList.toggle('redup', !ok);
    });
    angkaGrupRail();
  }

  // Kepala grup rail dan ringkasan sub-fitur di bawahnya membawa angka hasil generate.
  // Keduanya disegerakan dari item yang benar-benar masih lolos penyaring.
  function angkaGrupRail() {
    pohonIsi.querySelectorAll('.pohon-grup').forEach(function (grup) {
      var tombol = grup.querySelector('.pohon-grup-tombol');
      var jumlah = tombol ? tombol.querySelector('b') : null;
      if (jumlah) {
        jumlah.textContent = String(grup.querySelectorAll('.pohon-pilih:not(.redup)').length);
      }
    });
    pohonIsi.querySelectorAll('.pohon-detail').forEach(function (det) {
      var ada = det.querySelectorAll('.pohon-sub:not(.redup)').length;
      var angka = det.querySelector('.pohon-angka');
      var ringkas = det.querySelector('summary');
      if (angka) { angka.textContent = String(ada); }
      if (ringkas) { ringkas.title = ada + ' sub-fitur'; }
    });
  }

  // Legenda status memakai aturan yang sama dengan pilihan penyaring: status hanya ditampilkan
  // kalau datanya memang punya, atau kalau status itu adalah salah satu peran inti. Tidak ada
  // lagi status hantu yang tidak pernah muncul di manifest.
  function statusUntukLegenda() {
    var inti = peta();
    ['utama', 'separuh', 'mati'].forEach(function (peran) {
      var nama = data.peran && data.peran[peran];
      if (nama) { inti[nama] = true; }
    });
    var ada = peta();
    (data.features || []).forEach(function (f) { ada[f.status] = true; });
    return (data.statuses || []).filter(function (s) { return !!ada[s] || !!inti[s]; });
  }

  function bangunLegenda() {
    var legendaEdge = document.getElementById('legenda-edge');
    var legendaStatus = document.getElementById('legenda-status');
    legendaEdge.innerHTML = '';
    legendaStatus.innerHTML = '';
    data.kinds.forEach(function (kind) {
      var item = document.createElement('span');
      item.className = 'legenda-item';
      var garis = document.createElement('span');
      garis.className = 'garis-legenda';
      garis.style.color = kindColor[kind] || '#8a8a86';
      item.appendChild(garis);
      item.appendChild(document.createTextNode(kind));
      legendaEdge.appendChild(item);
    });
    statusUntukLegenda().forEach(function (status) {
      var item = document.createElement('span');
      item.className = 'legenda-item';
      var bar = document.createElement('span');
      bar.className = 'bar-legenda';
      bar.style.background = statusColor[status] || '#8a8a86';
      item.appendChild(bar);
      item.appendChild(document.createTextNode(status));
      legendaStatus.appendChild(item);
    });
  }

  function gantiSkema() {
    var terakhir = detailTerakhir;
    bacaPalet();
    bangunLegenda();
    aktifkanMode(state.mode);
    if (terakhir) { tampilkanDetail(terakhir.kind, terakhir.id); }
  }

  var skema = window.matchMedia('(prefers-color-scheme: dark)');
  if (skema.addEventListener) {
    skema.addEventListener('change', function () {
      // Pilihan eksplisit user mengalahkan OS; kalau belum pernah memilih, ikut OS seperti biasa.
      if (!bacaTema()) { gantiSkema(); }
    });
  }

  var tombolTema = document.getElementById('tombol-tema');
  if (tombolTema) {
    tombolTema.addEventListener('click', function () {
      pasangTema(GELAP ? 'terang' : 'gelap');
      gantiSkema();
    });
  }

  bacaPalet();
  isiIkon();
  bangunLegenda();
  bangunPapan();
  // Boot ditunda satu frame. sesuaikanTampilan() mengukur lebar kanvas, dan sebelum
  // layout tenang hasilnya ~2% terlalu lebar: dasar zoom-nya salah, jadi label persen
  // meleset begitu tampilan tersimpan dipulihkan.
  requestAnimationFrame(function () {
    pulihkanTampilan();
  });
  mulaiPenjagaan();
  // Halaman dari disk tidak pernah bisa membaca manifest; baru server yang menjawab /stamp
  // yang boleh menjanjikan lebih dari sekadar memuat ulang halaman.
  labelMuatData(dilayani);
})();
