package main

// DaftarPengguna menelusuri Baris pada SenaraiPengguna dan mengumpulkan Nama.
func DaftarPengguna(senarai *SenaraiPengguna) []string {
	nama := []string{}
	for _, baris := range senarai.Baris {
		nama = append(nama, baris.Nama)
	}
	return nama
}
