package main

// SapaPengguna menyusun salam pagi untuk satu pengguna.
func SapaPengguna(aksi *AksiSapa) string {
	ucapan := "selamat pagi"
	if aksi == nil {
		return ucapan
	}
	return ucapan + ", " + aksi.Nama
}

// RuteSapa memasang jalur GET /sapa ke SapaPengguna.
func RuteSapa(mux *Mux) {
	mux.Tangkap("GET", "/sapa", SapaPengguna)
}
