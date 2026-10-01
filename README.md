# Mala matematika

Prva verzija edukativne igre za dete od 4 godine, napravljena u Rustu i Bevyju 0.19. Interfejs je na srpskom (ćirilica), sa velikim dugmadima i kružićima za prebrojavanje. Odrasla osoba može da pročita uputstva detetu.

## Pokretanje

Potreban je Rust 1.95 ili noviji i grafičko okruženje sa podrškom za Bevy/wgpu.

```sh
cargo run
```

Prvo pokretanje preuzima i kompajlira Bevy, pa može potrajati. Na Linuxu su potrebne razvojne biblioteke za Wayland/X11; audio i gamepad nisu uključeni.

## Pravila

- Igra počinje sabiranjem. Roditelj može izabrati sabiranje, oduzimanje ili oba; promena započinje novu igru i resetuje bodove.
- Izbor **Са нивоима / Вежба до 10** nezavisan je od izbora računske operacije. Promena režima započinje novu igru i resetuje bodove; promena operacije zadržava izabrani režim.
- **Вежба до 10** daje neograničeno zadataka sa brojevima, rezultatima i ponuđenim odgovorima do **10**, bez nivoa, povećanja težine ili završnog ekrana. Prikazuje ukupan broj rešenih zadataka i bodove.
- U režimu **Са нивоима**, tri nivoa koriste brojeve i rezultate do **10**, **15** i **20**. Igra podrazumevano počinje u ovom režimu.
- U režimu sa nivoima posle **5 rešenih zadataka** otključava se sledeći nivo. Cela igra ima 15 zadataka.
- Pri prelasku na nivo 2 ili 3 prikazuje se zeleno obaveštenje o otključanom nivou i novoj granici brojeva tokom **4 sekunde**. Igra može da se nastavi dok je poruka prikazana.
- Tačan prvi pokušaj donosi **10 bodova**, a tačan odgovor posle ponovnog pokušaja **5 bodova**. Maksimum u režimu sa nivoima je 150 bodova.
- Nema oduzimanja bodova, vremenskog ograničenja za odgovor ili kraja igre zbog greške.
- Sabiranje pokazuje dve grupe kružića. Oduzimanje pokazuje početnu grupu sa precrtanim kružićima koji odlaze. Rezultat oduzimanja nikada nije negativan.
- Nakon tačnog odgovora prikazuje se velika zelena potvrda sa osvojenim bodovima. Posle **2 sekunde** igra automatski prelazi na sledeći zadatak, nivo ili završnu nagradu.
- Završni ekran prikazuje nagradu i dugme **Играј поново**.

Odgovori se biraju mišem ili tasterima **1 / 2 / 3** (prvi, drugi ili treći ponuđeni odgovor). Prelazak dalje je automatski.

Bodovi se trenutno čuvaju samo tokom igre. Zvuk, trajno čuvanje napretka i animacije mogu se dodati u narednoj verziji.

Interfejs se automatski prilagođava veličini prozora. Font Liberation Sans iz `assets/fonts` ugrađen je u izvršni fajl, tako da `assets` direktorijum nije potreban za pokretanje. Pri distribuciji treba priložiti njegovu licencu.

## Linux izvršni fajl

Paket za Linux x64 nalazi se u `dist/mala-matematika-linux-x64.tar.gz`. Raspakuj ga i pokreni `mala-matematika` dvoklikom ili komandom `./mala-matematika` iz raspakovanog foldera. Rust i dodatni `assets` direktorijum nisu potrebni.

Za pravljenje nove verzije na Linux x64 računaru:

```sh
bash scripts/build-linux.sh
```

Paket sadrži igru, kratko uputstvo i licencu fonta. Koristi sistemske biblioteke računara na kome je napravljen, pa nije univerzalan paket za sve Linux distribucije. Za starije distribucije napravi paket na odgovarajućem sistemu. Potrebni su X11 ili Wayland i odgovarajući grafički drajver.

## Windows izvršni fajl

Paket za Windows x64 nalazi se u `dist/mala-matematika-windows-x64.zip`. Raspakuj paket i dvaput klikni na `mala-matematika.exe`. Za igranje nisu potrebni Rust ili Bevy. Font za ćirilicu je već ugrađen. Release verzija nema dodatni konzolni prozor.

Za pravljenje nove verzije na Windowsu potrebni su Rust i Visual Studio C++ build tools. Iz PowerShell-a pokreni:

```powershell
./scripts/build-windows.ps1
```

Za pravljenje Windows verzije na Linuxu može se koristiti [cargo-xwin](https://github.com/rust-cross/cargo-xwin):

```sh
rustup target add x86_64-pc-windows-msvc
cargo install --locked cargo-xwin
bash scripts/build-windows.sh
```

Za ovu skriptu potrebni su i `clang` i `zip`. Skripta pravi isti Windows paket u `dist` kao PowerShell skripta. `cargo-xwin` pri prvom pokretanju preuzima Windows SDK i CRT.

Izvršni fajl nastaje u `target/x86_64-pc-windows-msvc/release/math-game.exe`. Uz njega priloži `WINDOWS-README.txt` i `assets/fonts/LICENSE-Liberation.txt`. Windows CRT je statički uključen preko `.cargo/config.toml`.

## Razvoj

`src/lib.rs` sadrži pravila, generisanje zadataka i testove. `src/main.rs` sadrži Bevy interfejs i unos.

Zvezdice napretka i završne nagrade koriste ugrađenu sliku `assets/icons/star.png`, pa ne zavise od dostupnih znakova u fontu. Izvor je `assets/icons/star.svg`; posle izmene regeneriši PNG komandom `rsvg-convert -w 128 -h 128 assets/icons/star.svg -o assets/icons/star.png`. Slika se uključuje u izvršni fajl pri kompajliranju.

```sh
cargo test
cargo check
cargo fmt --check
```
